//! Versioned content bundle. All writes use the same revision and FTS helpers as edits.
use crate::domain::{
    page::{Lang, Slug},
    ports::{StoreError, StoreResult},
    revision::{Author, NewRevision},
};
use crate::store::{
    content::{revisions, search},
    db::{Db, err, now},
};
use rusqlite::{OptionalExtension, params};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Catalog {
    pub source_revision: String,
    pub pages: Vec<Entry>,
}

#[derive(Deserialize)]
pub struct Entry {
    pub slug: String,
    pub lang: String,
    pub section: String,
    pub position: i64,
    pub body: String,
}

#[derive(Debug, Default)]
pub struct Report {
    pub created: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub protected: usize,
}

pub fn bundled() -> StoreResult<Catalog> {
    serde_json::from_str(include_str!("../../../../content/catalog.json"))
        .map_err(|e| StoreError(e.to_string()))
}

/// Startup only initializes a completely empty database. Explicit sync updates
/// bundle-owned pages; editor changes and deleted pages are always protected.
pub fn import(db: &Db, catalog: &Catalog, only_empty: bool) -> StoreResult<Report> {
    let mut conn = db.lock().unwrap();
    let tx = conn.transaction().map_err(err)?;
    let mut report = Report::default();
    let count: i64 = tx
        .query_row("select count(*) from pages", [], |r| r.get(0))
        .map_err(err)?;
    if only_empty && count != 0 {
        return Ok(report);
    }
    for entry in &catalog.pages {
        let slug =
            Slug::parse(&entry.slug).ok_or_else(|| StoreError("invalid bundle slug".into()))?;
        let lang =
            Lang::parse(&entry.lang).ok_or_else(|| StoreError("invalid bundle language".into()))?;
        let existing: Option<(i64, bool, Option<i64>)> = tx
            .query_row(
                "select id, deleted_at is not null, current_rev_id from pages
             where slug = ?1 and lang = ?2 order by deleted_at is null desc, id desc limit 1",
                params![slug.as_str(), lang.code()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(err)?;
        let id = if let Some((id, deleted, rev)) = existing {
            let current = rev
                .map(|rev| revisions::get(&tx, id, rev))
                .transpose()?
                .flatten();
            if deleted
                || current.as_ref().is_none_or(|r| {
                    r.author != Author::Import || !r.summary.starts_with("OC2R bundle ")
                })
            {
                report.protected += 1;
                continue;
            }
            if current.is_some_and(|r| r.body == entry.body) {
                report.unchanged += 1;
                tx.execute(
                    "update pages set section = ?1, position = ?2 where id = ?3",
                    params![entry.section, entry.position, id],
                )
                .map_err(err)?;
                continue;
            }
            report.updated += 1;
            id
        } else {
            tx.execute(
                "insert into pages (slug, lang, created_at) values (?1, ?2, ?3)",
                params![slug.as_str(), lang.code(), now()],
            )
            .map_err(err)?;
            report.created += 1;
            tx.last_insert_rowid()
        };
        let rev = revisions::insert(
            &tx,
            id,
            &NewRevision {
                body: entry.body.clone(),
                summary: format!("OC2R bundle {}", catalog.source_revision),
                author: Author::Import,
            },
        )?;
        tx.execute(
            "update pages set current_rev_id = ?1, section = ?2, position = ?3 where id = ?4",
            params![rev, entry.section, entry.position, id],
        )
        .map_err(err)?;
        search::index(&tx, id, &slug, &entry.body)?;
    }
    tx.commit().map_err(err)?;
    Ok(report)
}

#[cfg(test)]
mod tests;
