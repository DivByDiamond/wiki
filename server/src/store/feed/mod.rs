use crate::domain::{
    feed::{RecentChange, RecentRepo},
    page::{Lang, Slug},
    ports::StoreResult,
    revision::Author,
};
use crate::store::{
    content::revisions::author_from,
    db::{Db, err},
};

pub struct SqliteRecent {
    db: Db,
}

impl SqliteRecent {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

impl RecentRepo for SqliteRecent {
    fn recent(&self, limit: u32) -> StoreResult<Vec<RecentChange>> {
        let db = self.db.lock().unwrap();
        // Only the head of the body: the title is its first heading.
        let mut stmt = db
            .prepare(
                "select r.id, p.slug, p.lang, substr(r.body, 1, 2000), r.summary, \
                 r.author_kind, r.author_ref, r.created_at \
                 from revisions r join pages p on p.id = r.page_id \
                 where p.deleted_at is null order by r.id desc limit ?1",
            )
            .map_err(err)?;
        let rows = stmt
            .query_map([i64::from(limit)], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    author_from(&r.get::<_, String>(5)?, r.get(6)?),
                    r.get::<_, i64>(7)?,
                ))
            })
            .map_err(err)?;
        let mut out = Vec::new();
        for row in rows {
            let (rev_id, slug, lang, head, summary, author, created_at): (
                i64,
                String,
                String,
                String,
                String,
                Author,
                i64,
            ) = row.map_err(err)?;
            let (Some(slug), Some(lang)) = (Slug::parse(&slug), Lang::parse(&lang)) else {
                continue;
            };
            let title = crate::render::document::title(&head)
                .filter(|t| !t.is_empty())
                .unwrap_or_else(|| slug.as_str().to_owned());
            out.push(RecentChange {
                rev_id,
                slug,
                lang,
                title,
                summary,
                author,
                created_at,
            });
        }
        Ok(out)
    }
}
