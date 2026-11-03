use crate::domain::{
    media::{MediaBlob, MediaFile, MediaRepo, Prepared},
    ports::StoreResult,
};
use crate::store::db::{Db, err, now};
use rusqlite::{OptionalExtension, params};

pub struct SqliteMedia {
    db: Db,
}

impl SqliteMedia {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

impl MediaRepo for SqliteMedia {
    fn add(&self, p: &Prepared, data: &[u8]) -> StoreResult<bool> {
        let changed = self
            .db
            .lock()
            .unwrap()
            .execute(
                "insert or ignore into media (filename, mime, size, sha256, created_at, data) \
                 values (?1,?2,?3,?4,?5,?6)",
                params![
                    p.name,
                    p.kind.mime(),
                    data.len() as i64,
                    p.sha256,
                    now(),
                    data
                ],
            )
            .map_err(err)?;
        Ok(changed == 1)
    }

    fn list(&self) -> StoreResult<Vec<MediaFile>> {
        let db = self.db.lock().unwrap();
        let mut stmt = db
            .prepare("select filename, size, created_at from media order by id desc limit 500")
            .map_err(err)?;
        stmt.query_map([], |r| {
            Ok(MediaFile {
                name: r.get(0)?,
                size: r.get(1)?,
                created_at: r.get(2)?,
            })
        })
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err)
    }

    fn get(&self, name: &str) -> StoreResult<Option<MediaBlob>> {
        self.db
            .lock()
            .unwrap()
            .query_row(
                "select mime, sha256, data from media where filename = ?1",
                [name],
                |r| {
                    Ok(MediaBlob {
                        mime: r.get(0)?,
                        sha256: r.get(1)?,
                        data: r.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::media::prepare;
    use crate::store::db::open_memory;

    #[test]
    fn same_content_is_stored_once() {
        let repo = SqliteMedia::new(open_memory().unwrap());
        let data = b"\x89PNG\r\n\x1a\nbody";
        let p = prepare(data).unwrap();
        assert!(repo.add(&p, data).unwrap());
        assert!(!repo.add(&p, data).unwrap());
        assert_eq!(repo.list().unwrap().len(), 1);
        let blob = repo.get(&p.name).unwrap().unwrap();
        assert_eq!(
            (blob.mime.as_str(), blob.data.as_slice()),
            ("image/png", &data[..])
        );
        assert!(repo.get("nope.png").unwrap().is_none());
    }
}
