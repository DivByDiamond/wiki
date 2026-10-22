use super::StoreResult;
use crate::domain::{
    page::{Lang, Page, Slug},
    revision::{NewRevision, Revision},
};

#[derive(Debug, PartialEq, Eq)]
pub enum SaveOutcome {
    Created(i64),
    Updated(i64),
    Unchanged,
}

pub struct PageSummary {
    pub id: i64,
    pub slug: Slug,
    pub lang: Lang,
    pub section: String,
    pub position: i64,
    pub title_rev_id: i64,
}

pub struct SearchHit {
    pub slug: Slug,
    pub lang: Lang,
    pub snippet: String,
}

pub trait PageRepo: Send + Sync {
    fn find(&self, slug: &Slug, lang: Lang) -> StoreResult<Option<Page>>;
    /// Like `find` but by id, and it also returns deleted pages.
    fn find_by_id(&self, page_id: i64) -> StoreResult<Option<Page>>;
    fn revision(&self, page_id: i64, rev_id: i64) -> StoreResult<Option<Revision>>;
    fn history(&self, page_id: i64) -> StoreResult<Vec<Revision>>;
    fn save(&self, slug: &Slug, lang: Lang, rev: NewRevision) -> StoreResult<SaveOutcome>;
    fn list(&self) -> StoreResult<Vec<PageSummary>>;
    fn list_deleted(&self) -> StoreResult<Vec<PageSummary>>;
    fn set_deleted(&self, page_id: i64, deleted: bool) -> StoreResult<()>;
    fn search(&self, query: &str, lang: Lang) -> StoreResult<Vec<SearchHit>>;
}
