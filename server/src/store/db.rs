use crate::domain::ports::{StoreError, StoreResult};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<Connection>>;

const MIGRATIONS: [&str; 3] = [
    include_str!("sql/0001_content.sql"),
    include_str!("sql/0002_access.sql"),
    include_str!("sql/0003_search.sql"),
];

pub fn err(e: rusqlite::Error) -> StoreError {
    StoreError(e.to_string())
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

pub fn migrate(conn: &Connection) -> StoreResult<()> {
    let applied: usize = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .map_err(err)?;
    for (i, sql) in MIGRATIONS.iter().enumerate().skip(applied) {
        conn.execute_batch(sql).map_err(err)?;
        conn.pragma_update(None, "user_version", i + 1)
            .map_err(err)?;
    }
    Ok(())
}

fn setup(conn: Connection) -> StoreResult<Db> {
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(err)?;
    migrate(&conn)?;
    Ok(Arc::new(Mutex::new(conn)))
}

pub fn open(path: &str) -> StoreResult<Db> {
    let conn = Connection::open(path).map_err(err)?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(err)?;
    setup(conn)
}

pub fn open_memory() -> StoreResult<Db> {
    setup(Connection::open_in_memory().map_err(err)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_create_all_tables() {
        let db = open_memory().unwrap();
        let conn = db.lock().unwrap();
        for table in [
            "pages",
            "revisions",
            "proposals",
            "users",
            "api_keys",
            "media",
            "revisions_fts",
        ] {
            let n: i64 = conn
                .query_row(
                    "select count(*) from sqlite_master where name = ?1",
                    [table],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(n, 1, "{table}");
        }
    }

    #[test]
    fn migrations_are_idempotent() {
        let db = open_memory().unwrap();
        let conn = db.lock().unwrap();
        migrate(&conn).unwrap();
        let v: i64 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(v, 3);
    }
}
