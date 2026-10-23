use crate::domain::{
    page::{Lang, Slug},
    ports::{StoreResult, pages::SearchHit},
};
use crate::store::db::err;
use rusqlite::{Connection, params};

pub fn index(conn: &Connection, page_id: i64, slug: &Slug, body: &str) -> StoreResult<()> {
    remove(conn, page_id)?;
    conn.execute(
        "insert into revisions_fts (slug, body, page_id) values (?1, ?2, ?3)",
        params![slug.as_str().replace(['/', '-', '_'], " "), body, page_id],
    )
    .map_err(err)?;
    Ok(())
}

pub fn remove(conn: &Connection, page_id: i64) -> StoreResult<()> {
    conn.execute("delete from revisions_fts where page_id = ?1", [page_id])
        .map_err(err)?;
    Ok(())
}

/// Wraps every word in quotes so user input cannot break the FTS5 syntax.
fn quote(query: &str) -> String {
    query
        .split_whitespace()
        .map(|w| format!("\"{}\"", w.replace('"', "")))
        .filter(|w| w != "\"\"")
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn query(conn: &Connection, q: &str, lang: Lang) -> StoreResult<Vec<SearchHit>> {
    let q = quote(q);
    if q.is_empty() {
        return Ok(Vec::new());
    }
    let mut stmt = conn
        .prepare(
            "select p.slug, p.lang, snippet(revisions_fts, 1, '', '', '…', 12)
             from revisions_fts f join pages p on p.id = f.page_id
             where revisions_fts match ?1 and (p.lang = ?2 or p.lang = 'en') and p.deleted_at is null
             order by (p.lang = ?2) desc, rank limit 40",
        )
        .map_err(err)?;
    let rows = stmt
        .query_map(params![q, lang.code()], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .map_err(err)?;
    let mut hits = Vec::new();
    for row in rows {
        let (slug, lang, snippet) = row.map_err(err)?;
        if let (Some(slug), Some(lang)) = (Slug::parse(&slug), Lang::parse(&lang)) {
            if hits.iter().any(|h: &SearchHit| h.slug == slug) || hits.len() == 20 {
                continue;
            }
            hits.push(SearchHit {
                slug,
                lang,
                snippet,
            });
        }
    }
    Ok(hits)
}
