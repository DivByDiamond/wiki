use crate::domain::ports::{StoreResult, access::UserRow};
use crate::store::db::{err, now};
use rusqlite::{Connection, OptionalExtension, params};

const COLS: &str = "id, github_id, login, admin, blocked_at is not null";

fn row(r: &rusqlite::Row) -> rusqlite::Result<UserRow> {
    Ok(UserRow {
        id: r.get(0)?,
        github_id: r.get(1)?,
        login: r.get(2)?,
        admin: r.get(3)?,
        blocked: r.get(4)?,
    })
}

pub fn upsert(conn: &Connection, github_id: i64, login: &str) -> StoreResult<UserRow> {
    conn.query_row(
        &format!(
            "insert into users (github_id, login, created_at) values (?1,?2,?3) \
             on conflict(github_id) do update set login = excluded.login \
             returning {COLS}"
        ),
        params![github_id, login, now()],
        row,
    )
    .map_err(err)
}

pub fn get(conn: &Connection, id: i64) -> StoreResult<Option<UserRow>> {
    conn.query_row(
        &format!("select {COLS} from users where id = ?1"),
        [id],
        row,
    )
    .optional()
    .map_err(err)
}

pub fn set_blocked(conn: &Connection, id: i64, blocked: bool) -> StoreResult<()> {
    let at = blocked.then(now);
    conn.execute(
        "update users set blocked_at = ?1 where id = ?2",
        params![at, id],
    )
    .map_err(err)?;
    Ok(())
}

pub fn set_admin(conn: &Connection, id: i64, admin: bool) -> StoreResult<()> {
    conn.execute(
        "update users set admin = ?1 where id = ?2",
        params![admin, id],
    )
    .map_err(err)?;
    Ok(())
}
