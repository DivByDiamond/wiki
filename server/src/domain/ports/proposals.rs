use super::StoreResult;
use crate::domain::proposal::{Proposal, Status};

pub trait ProposalRepo: Send + Sync {
    fn create(
        &self,
        page_id: i64,
        base_rev_id: i64,
        body: &str,
        summary: &str,
        user_id: i64,
    ) -> StoreResult<i64>;
    fn get(&self, id: i64) -> StoreResult<Option<Proposal>>;
    fn list(&self, status: Status) -> StoreResult<Vec<Proposal>>;
    fn count_open_by_user(&self, user_id: i64) -> StoreResult<u32>;
    fn count_since_by_user(&self, user_id: i64, since: i64) -> StoreResult<u32>;
    /// Atomically closes an open proposal; `false` when it was not open (someone else decided).
    fn claim(&self, id: i64, status: Status, admin_id: i64, note: &str) -> StoreResult<bool>;
    /// Puts a claimed proposal back to open (the publish after the claim failed).
    fn reopen(&self, id: i64) -> StoreResult<()>;
    fn decide(&self, id: i64, status: Status, admin_id: i64, note: &str) -> StoreResult<()>;
}
