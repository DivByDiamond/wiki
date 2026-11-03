//! Recent changes: the newest revisions across all live pages.

use super::{
    page::{Lang, Slug},
    ports::StoreResult,
    revision::Author,
};

#[derive(Debug, Clone)]
pub struct RecentChange {
    pub rev_id: i64,
    pub slug: Slug,
    pub lang: Lang,
    /// First heading of the revision, falling back to the address.
    pub title: String,
    pub summary: String,
    pub author: Author,
    pub created_at: i64,
}

pub trait RecentRepo: Send + Sync {
    /// Newest first; deleted pages are left out.
    fn recent(&self, limit: u32) -> StoreResult<Vec<RecentChange>>;
}
