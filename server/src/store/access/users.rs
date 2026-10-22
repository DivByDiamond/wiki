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

pub fn list(conn: &Connection) -> StoreResult<Vec<UserRow>> {
    let mut stmt = conn
        .prepare(&format!("select {COLS} from users order by id"))
        .map_err(err)?;
    stmt.query_map([], row)
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err)
}

pub fn count_active_admins(conn: &Connection) -> StoreResult<u32> {
    conn.query_row(
        "select count(*) from users where admin = 1 and blocked_at is null",
        [],
        |r| r.get(0),
    )
    .map_err(err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::db::open_memory;

    #[test]
    fn list_returns_all_users_in_id_order_with_flags() {
        let db = open_memory().unwrap();
        let c = db.lock().unwrap();
        let a = upsert(&c, 10, "a").unwrap().id;
        let b = upsert(&c, 20, "b").unwrap().id;
        set_blocked(&c, b, true).unwrap();
        let all = list(&c).unwrap();
        assert_eq!(all.iter().map(|u| u.id).collect::<Vec<_>>(), vec![a, b]);
        assert_eq!((all[1].github_id, all[1].blocked), (20, true));
        assert!(!all[0].blocked && !all[0].admin);
    }

    #[test]
    fn active_admins_exclude_blocked_and_plain_users() {
        let db = open_memory().unwrap();
        let c = db.lock().unwrap();
        assert_eq!(count_active_admins(&c).unwrap(), 0);
        let a = upsert(&c, 1, "a").unwrap().id;
        let b = upsert(&c, 2, "b").unwrap().id;
        upsert(&c, 3, "plain").unwrap();
        set_admin(&c, a, true).unwrap();
        set_admin(&c, b, true).unwrap();
        assert_eq!(count_active_admins(&c).unwrap(), 2);
        set_blocked(&c, b, true).unwrap();
        assert_eq!(count_active_admins(&c).unwrap(), 1);
        set_admin(&c, a, false).unwrap();
        assert_eq!(count_active_admins(&c).unwrap(), 0);
    }
}
