use super::StoreResult;
use crate::domain::proposal::{Proposal, Status};

/// Limits applied inside the insert, so two concurrent requests cannot both pass
/// the check and exceed the quota.
pub struct Quota {
    pub open: u32,
    pub per_day: u32,
    pub since: i64,
}

pub trait ProposalRepo: Send + Sync {
    /// `None` when the user is at the open or daily limit; the insert is atomic.
    fn create(
        &self,
        page_id: i64,
        base_rev_id: i64,
        body: &str,
        summary: &str,
        user_id: i64,
        quota: Quota,
    ) -> StoreResult<Option<i64>>;
    fn get(&self, id: i64) -> StoreResult<Option<Proposal>>;
    fn list(&self, status: Status) -> StoreResult<Vec<Proposal>>;
    fn count_open_by_user(&self, user_id: i64) -> StoreResult<u32>;
    fn count_since_by_user(&self, user_id: i64, since: i64) -> StoreResult<u32>;
    /// Atomically closes an open proposal; `false` when it was not open (someone else decided).
    fn claim(&self, id: i64, status: Status, admin_id: i64, note: &str) -> StoreResult<bool>;
    /// Puts a claimed proposal back to open (the publish after the claim failed).
    fn reopen(&self, id: i64) -> StoreResult<()>;
}
