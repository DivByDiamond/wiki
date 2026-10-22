use crate::domain::ports::{StoreError, StoreResult};
use crate::domain::proposal::{Proposal, Status};
use crate::store::db::{err, now};
use rusqlite::{Connection, OptionalExtension, params};

const COLS: &str = "id, page_id, base_rev_id, body, summary, user_id, status, created_at";

fn row(r: &rusqlite::Row) -> rusqlite::Result<Proposal> {
    let status: String = r.get(6)?;
    Ok(Proposal {
        id: r.get(0)?,
        page_id: r.get(1)?,
        base_rev_id: r.get(2)?,
        body: r.get(3)?,
        summary: r.get(4)?,
        user_id: r.get(5)?,
        status: Status::parse(&status).unwrap_or(Status::Open),
        created_at: r.get(7)?,
    })
}

pub fn create(
    conn: &Connection,
    page_id: i64,
    base_rev_id: i64,
    body: &str,
    summary: &str,
    user_id: i64,
) -> StoreResult<i64> {
    conn.execute(
        "insert into proposals (page_id, base_rev_id, body, summary, user_id, created_at) \
         values (?1,?2,?3,?4,?5,?6)",
        params![page_id, base_rev_id, body, summary, user_id, now()],
    )
    .map_err(err)?;
    Ok(conn.last_insert_rowid())
}

pub fn get(conn: &Connection, id: i64) -> StoreResult<Option<Proposal>> {
    conn.query_row(
        &format!("select {COLS} from proposals where id = ?1"),
        [id],
        row,
    )
    .optional()
    .map_err(err)
}

pub fn list(conn: &Connection, status: Status) -> StoreResult<Vec<Proposal>> {
    let mut stmt = conn
        .prepare(&format!(
            "select {COLS} from proposals where status = ?1 order by id"
        ))
        .map_err(err)?;
    stmt.query_map([status.as_str()], row)
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err)
}

fn count(conn: &Connection, sql: &str, args: impl rusqlite::Params) -> StoreResult<u32> {
    conn.query_row(sql, args, |r| r.get(0)).map_err(err)
}

pub fn count_open_by_user(conn: &Connection, user_id: i64) -> StoreResult<u32> {
    count(
        conn,
        "select count(*) from proposals where user_id = ?1 and status = 'open'",
        [user_id],
    )
}

pub fn count_since_by_user(conn: &Connection, user_id: i64, since: i64) -> StoreResult<u32> {
    count(
        conn,
        "select count(*) from proposals where user_id = ?1 and created_at >= ?2",
        params![user_id, since],
    )
}

/// Closes an open proposal in one guarded UPDATE; `false` when it was not open.
pub fn claim(
    conn: &Connection,
    id: i64,
    status: Status,
    admin_id: i64,
    note: &str,
) -> StoreResult<bool> {
    let changed = conn
        .execute(
            "update proposals set status = ?1, decided_by = ?2, decided_at = ?3, \
             decision_note = ?4 where id = ?5 and status = 'open'",
            params![status.as_str(), admin_id, now(), note, id],
        )
        .map_err(err)?;
    Ok(changed == 1)
}

/// Undoes a claim: back to open with the decision fields cleared.
pub fn reopen(conn: &Connection, id: i64) -> StoreResult<()> {
    conn.execute(
        "update proposals set status = 'open', decided_by = null, decided_at = null, \
         decision_note = '' where id = ?1 and status <> 'open'",
        [id],
    )
    .map_err(err)?;
    Ok(())
}

/// Closes an open proposal; deciding a missing or already decided one is an error.
pub fn decide(
    conn: &Connection,
    id: i64,
    status: Status,
    admin_id: i64,
    note: &str,
) -> StoreResult<()> {
    if !claim(conn, id, status, admin_id, note)? {
        return Err(StoreError(format!("proposal {id} is not open")));
    }
    Ok(())
}
