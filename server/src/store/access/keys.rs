use crate::domain::ports::{StoreResult, access::KeyRow};
use crate::store::db::{err, now};
use rusqlite::{Connection, OptionalExtension, params};

pub fn create(
    conn: &Connection,
    name: &str,
    hash: &str,
    r: bool,
    w: bool,
    e: bool,
) -> StoreResult<i64> {
    conn.execute(
        "insert into api_keys (name, key_hash, can_read, can_write, can_export, created_at) \
         values (?1,?2,?3,?4,?5,?6)",
        params![name, hash, r, w, e, now()],
    )
    .map_err(err)?;
    Ok(conn.last_insert_rowid())
}

fn row(r: &rusqlite::Row) -> rusqlite::Result<KeyRow> {
    Ok(KeyRow {
        id: r.get(0)?,
        name: r.get(1)?,
        read: r.get(2)?,
        write: r.get(3)?,
        export: r.get(4)?,
    })
}

pub fn by_hash(conn: &Connection, hash: &str) -> StoreResult<Option<KeyRow>> {
    conn.query_row(
        "select id, name, can_read, can_write, can_export from api_keys \
         where key_hash = ?1 and revoked_at is null",
        [hash],
        row,
    )
    .optional()
    .map_err(err)
}

pub fn touch(conn: &Connection, id: i64) -> StoreResult<()> {
    conn.execute(
        "update api_keys set last_used_at = ?1 where id = ?2",
        params![now(), id],
    )
    .map_err(err)?;
    Ok(())
}

pub fn revoke(conn: &Connection, id: i64) -> StoreResult<()> {
    conn.execute(
        "update api_keys set revoked_at = ?1 where id = ?2",
        params![now(), id],
    )
    .map_err(err)?;
    Ok(())
}

pub fn list(conn: &Connection) -> StoreResult<Vec<KeyRow>> {
    let mut stmt = conn
        .prepare(
            "select id, name, can_read, can_write, can_export from api_keys \
             where revoked_at is null order by id",
        )
        .map_err(err)?;
    stmt.query_map([], row)
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err)
}
