use super::{revisions, search};
use crate::domain::{
    page::{Lang, Page, Slug},
    ports::{
        StoreError, StoreResult,
        pages::{PageRepo, PageSummary, SaveOutcome, SearchHit},
    },
    revision::{self, NewRevision, Revision},
};
use crate::store::db::{Db, err, now};
use rusqlite::{Connection, OptionalExtension, params};

pub struct SqlitePages {
    db: Db,
}

impl SqlitePages {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

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

/// Slug and current body of a page, used to rebuild its search entry.
fn indexed_text(conn: &Connection, page_id: i64) -> StoreResult<Option<(String, String)>> {
    conn.query_row(
        "select p.slug, r.body from pages p join revisions r on r.id = p.current_rev_id
         where p.id = ?1",
        [page_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()
    .map_err(err)
}

fn reindex(conn: &Connection, page_id: i64) -> StoreResult<()> {
    let Some((slug, body)) = indexed_text(conn, page_id)? else {
        return Ok(());
    };
    let slug = Slug::parse(&slug).ok_or_else(|| StoreError(format!("bad stored slug: {slug}")))?;
    search::index(conn, page_id, &slug, &body)
}

fn summaries(db: &Db, deleted: bool) -> StoreResult<Vec<PageSummary>> {
    let conn = db.lock().unwrap();
    let mut stmt = conn
        .prepare(if deleted {
            "select slug, lang, section, position, current_rev_id, id from pages
             where deleted_at is not null order by section, position, slug"
        } else {
            "select slug, lang, section, position, current_rev_id, id from pages
             where deleted_at is null order by section, position, slug"
        })
        .map_err(err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, i64>(5)?,
            ))
        })
        .map_err(err)?;
    let mut out = Vec::new();
    for row in rows {
        let (slug, lang, section, position, title_rev_id, id) = row.map_err(err)?;
        if let (Some(slug), Some(lang)) = (Slug::parse(&slug), Lang::parse(&lang)) {
            out.push(PageSummary {
                id,
                slug,
                lang,
                section,
                position,
                title_rev_id,
            });
        }
    }
    Ok(out)
}

impl PageRepo for SqlitePages {
    fn find(&self, slug: &Slug, lang: Lang) -> StoreResult<Option<Page>> {
        let conn = self.db.lock().unwrap();
        let row = conn
            .query_row(
                "select id, section, position, current_rev_id from pages
                 where slug = ?1 and lang = ?2 and deleted_at is null",
                params![slug.as_str(), lang.code()],
                |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get::<_, i64>(3)?,
                    ))
                },
            )
            .optional()
            .map_err(err)?;
        let Some((id, section, position, rev_id)) = row else {
            return Ok(None);
        };
        let current = revisions::get(&conn, id, rev_id)?
            .ok_or_else(|| StoreError("missing current revision".into()))?;
        Ok(Some(Page {
            id,
            slug: slug.clone(),
            lang,
            section,
            position,
            deleted: false,
            current,
        }))
    }

    fn find_by_id(&self, page_id: i64) -> StoreResult<Option<Page>> {
        let conn = self.db.lock().unwrap();
        let row = conn
            .query_row(
                "select slug, lang, section, position, current_rev_id, deleted_at is not null
                 from pages where id = ?1",
                [page_id],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get::<_, i64>(4)?,
                        r.get::<_, bool>(5)?,
                    ))
                },
            )
            .optional()
            .map_err(err)?;
        let Some((slug, lang, section, position, rev_id, deleted)) = row else {
            return Ok(None);
        };
        let (Some(slug), Some(lang)) = (Slug::parse(&slug), Lang::parse(&lang)) else {
            return Err(StoreError(format!("bad stored page: {slug}/{lang}")));
        };
        let current = revisions::get(&conn, page_id, rev_id)?
            .ok_or_else(|| StoreError("missing current revision".into()))?;
        Ok(Some(Page {
            id: page_id,
            slug,
            lang,
            section,
            position,
            deleted,
            current,
        }))
    }

    fn revision(&self, page_id: i64, rev_id: i64) -> StoreResult<Option<Revision>> {
        revisions::get(&self.db.lock().unwrap(), page_id, rev_id)
    }

    fn history(&self, page_id: i64) -> StoreResult<Vec<Revision>> {
        revisions::history(&self.db.lock().unwrap(), page_id)
    }

    fn save(&self, slug: &Slug, lang: Lang, rev: NewRevision) -> StoreResult<SaveOutcome> {
        let mut conn = self.db.lock().unwrap();
        let tx = conn.transaction().map_err(err)?;
        let (id, created) = match live_page(&tx, slug, lang)? {
            Some((id, cur)) => {
                let current = match cur {
                    Some(c) => revisions::get(&tx, id, c)?,
                    None => None,
                };
                if revision::is_noop(current.as_ref(), &rev.body) {
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

    fn list(&self) -> StoreResult<Vec<PageSummary>> {
        summaries(&self.db, false)
    }

    fn list_deleted(&self) -> StoreResult<Vec<PageSummary>> {
        summaries(&self.db, true)
    }

    fn set_deleted(&self, page_id: i64, deleted: bool) -> StoreResult<()> {
        let conn = self.db.lock().unwrap();
        let at: Option<i64> = deleted.then(now);
        conn.execute(
            "update pages set deleted_at = ?1 where id = ?2",
            params![at, page_id],
        )
        .map_err(err)?;
        if deleted {
            search::remove(&conn, page_id)
        } else {
            reindex(&conn, page_id)
        }
    }

    fn search(&self, query: &str, lang: Lang) -> StoreResult<Vec<SearchHit>> {
        search::query(&self.db.lock().unwrap(), query, lang)
    }
}
