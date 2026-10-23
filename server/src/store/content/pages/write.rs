use super::super::{revisions, search};
use crate::domain::{
    page::{Lang, Slug},
    ports::{
        StoreResult,
        pages::{Navigation, SaveOutcome},
    },
    revision::{self, NewRevision},
};
use crate::store::db::{Db, err, now};
use rusqlite::{Connection, OptionalExtension, params};

/// Live (not deleted) page id and its current revision id.
fn live_page(
    conn: &Connection,
    slug: &Slug,
    lang: Lang,
) -> StoreResult<Option<(i64, Option<i64>)>> {
    conn.query_row(
        "select id, current_rev_id from pages where slug = ?1 and lang = ?2 and deleted_at is null",
        params![slug.as_str(), lang.code()],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()
    .map_err(err)
}

pub(super) fn save(
    db: &Db,
    slug: &Slug,
    lang: Lang,
    rev: NewRevision,
    navigation: Option<Navigation>,
) -> StoreResult<SaveOutcome> {
    let mut conn = db.lock().unwrap();
    let tx = conn.transaction().map_err(err)?;
    let (id, created) = match live_page(&tx, slug, lang)? {
        Some((id, cur)) => {
            let current = match cur {
                Some(c) => revisions::get(&tx, id, c)?,
                None => None,
            };
            let navigation_changed = if let Some(n) = &navigation {
                let stored: (String, i64) = tx
                    .query_row(
                        "select section, position from pages where id = ?1",
                        [id],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                    .map_err(err)?;
                stored != (n.section.clone(), n.position)
            } else {
                false
            };
            if !navigation_changed && revision::is_noop(current.as_ref(), &rev.body) {
                // nothing was written; dropping the transaction rolls it back
                return Ok(SaveOutcome::Unchanged);
            }
            (id, false)
        }
        None => {
            tx.execute(
                "insert into pages (slug, lang, created_at) values (?1, ?2, ?3)",
                params![slug.as_str(), lang.code(), now()],
            )
            .map_err(err)?;
            (tx.last_insert_rowid(), true)
        }
    };
    if let Some(n) = navigation {
        tx.execute(
            "update pages set section = ?1, position = ?2 where id = ?3",
            params![n.section, n.position, id],
        )
        .map_err(err)?;
    }
    let rev_id = revisions::insert(&tx, id, &rev)?;
    tx.execute(
        "update pages set current_rev_id = ?1 where id = ?2",
        params![rev_id, id],
    )
    .map_err(err)?;
    search::index(&tx, id, slug, &rev.body)?;
    tx.commit().map_err(err)?;
    Ok(if created {
        SaveOutcome::Created(rev_id)
    } else {
        SaveOutcome::Updated(rev_id)
    })
}
