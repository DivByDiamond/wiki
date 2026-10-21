use crate::domain::{
    ports::StoreResult,
    revision::{Author, NewRevision, Revision},
};
use crate::store::db::{err, now};
use rusqlite::{Connection, Row, params};

fn author_parts(a: &Author) -> (&'static str, String) {
    match a {
        Author::User(s) => ("user", s.clone()),
        Author::ApiKey(s) => ("api_key", s.clone()),
        Author::Import => ("import", String::new()),
    }
}

fn author_from(kind: &str, r: String) -> Author {
    match kind {
        "user" => Author::User(r),
        "api_key" => Author::ApiKey(r),
        _ => Author::Import,
    }
}

const COLUMNS: &str = "id, page_id, body, summary, author_kind, author_ref, created_at";

fn from_row(row: &Row) -> rusqlite::Result<Revision> {
    Ok(Revision {
        id: row.get(0)?,
        page_id: row.get(1)?,
        body: row.get(2)?,
        summary: row.get(3)?,
        author: author_from(&row.get::<_, String>(4)?, row.get(5)?),
        created_at: row.get(6)?,
    })
}

pub fn insert(conn: &Connection, page_id: i64, rev: &NewRevision) -> StoreResult<i64> {
    let (kind, who) = author_parts(&rev.author);
    conn.execute(
        "insert into revisions (page_id, body, summary, author_kind, author_ref, created_at) \
         values (?1,?2,?3,?4,?5,?6)",
        params![page_id, rev.body, rev.summary, kind, who, now()],
    )
    .map_err(err)?;
    Ok(conn.last_insert_rowid())
}

pub fn get(conn: &Connection, page_id: i64, rev_id: i64) -> StoreResult<Option<Revision>> {
    let sql = format!("select {COLUMNS} from revisions where page_id = ?1 and id = ?2");
    match conn.query_row(&sql, params![page_id, rev_id], from_row) {
        Ok(r) => Ok(Some(r)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(err(e)),
    }
}

pub fn history(conn: &Connection, page_id: i64) -> StoreResult<Vec<Revision>> {
    let sql = format!("select {COLUMNS} from revisions where page_id = ?1 order by id desc");
    let mut stmt = conn.prepare(&sql).map_err(err)?;
    stmt.query_map([page_id], from_row)
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err)
}
