mod keys;
mod proposals;
mod users;

use crate::domain::{
    ports::{
        StoreResult,
        access::{AccessRepo, KeyRow, UserRow},
        proposals::ProposalRepo,
    },
    proposal::{Proposal, Status},
};
use crate::store::db::Db;

pub struct SqliteAccess {
    db: Db,
}

impl SqliteAccess {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

impl AccessRepo for SqliteAccess {
    fn upsert_user(&self, github_id: i64, login: &str) -> StoreResult<UserRow> {
        users::upsert(&self.db.lock().unwrap(), github_id, login)
    }
    fn user(&self, id: i64) -> StoreResult<Option<UserRow>> {
        users::get(&self.db.lock().unwrap(), id)
    }
    fn list_users(&self) -> StoreResult<Vec<UserRow>> {
        users::list(&self.db.lock().unwrap())
    }
    fn count_active_admins(&self) -> StoreResult<u32> {
        users::count_active_admins(&self.db.lock().unwrap())
    }
    fn block_keeping_an_admin(&self, id: i64) -> StoreResult<bool> {
        users::block_keeping_an_admin(&self.db.lock().unwrap(), id)
    }
    fn revoke_admin_keeping_one(&self, id: i64) -> StoreResult<bool> {
        users::revoke_admin_keeping_one(&self.db.lock().unwrap(), id)
    }
    fn set_blocked(&self, id: i64, blocked: bool) -> StoreResult<()> {
        users::set_blocked(&self.db.lock().unwrap(), id, blocked)
    }
    fn set_admin(&self, id: i64, admin: bool) -> StoreResult<()> {
        users::set_admin(&self.db.lock().unwrap(), id, admin)
    }
    fn create_key(
        &self,
        name: &str,
        hash: &str,
        read: bool,
        write: bool,
        export: bool,
    ) -> StoreResult<i64> {
        keys::create(&self.db.lock().unwrap(), name, hash, read, write, export)
    }
    fn key_by_hash(&self, hash: &str) -> StoreResult<Option<KeyRow>> {
        keys::by_hash(&self.db.lock().unwrap(), hash)
    }
    fn touch_key(&self, id: i64) -> StoreResult<()> {
        keys::touch(&self.db.lock().unwrap(), id)
    }
    fn revoke_key(&self, id: i64) -> StoreResult<()> {
        keys::revoke(&self.db.lock().unwrap(), id)
    }
    fn list_keys(&self) -> StoreResult<Vec<KeyRow>> {
        keys::list(&self.db.lock().unwrap())
    }
}

pub struct SqliteProposals {
    db: Db,
}

impl SqliteProposals {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

impl ProposalRepo for SqliteProposals {
    fn create(
        &self,
        page_id: i64,
        base_rev_id: i64,
        body: &str,
        summary: &str,
        user_id: i64,
    ) -> StoreResult<i64> {
        proposals::create(
            &self.db.lock().unwrap(),
            page_id,
            base_rev_id,
            body,
            summary,
            user_id,
        )
    }
    fn get(&self, id: i64) -> StoreResult<Option<Proposal>> {
        proposals::get(&self.db.lock().unwrap(), id)
    }
    fn list(&self, status: Status) -> StoreResult<Vec<Proposal>> {
        proposals::list(&self.db.lock().unwrap(), status)
    }
    fn count_open_by_user(&self, user_id: i64) -> StoreResult<u32> {
        proposals::count_open_by_user(&self.db.lock().unwrap(), user_id)
    }
    fn count_since_by_user(&self, user_id: i64, since: i64) -> StoreResult<u32> {
        proposals::count_since_by_user(&self.db.lock().unwrap(), user_id, since)
    }
    fn claim(&self, id: i64, status: Status, admin_id: i64, note: &str) -> StoreResult<bool> {
        proposals::claim(&self.db.lock().unwrap(), id, status, admin_id, note)
    }
    fn reopen(&self, id: i64) -> StoreResult<()> {
        proposals::reopen(&self.db.lock().unwrap(), id)
    }
    fn decide(&self, id: i64, status: Status, admin_id: i64, note: &str) -> StoreResult<()> {
        proposals::decide(&self.db.lock().unwrap(), id, status, admin_id, note)
    }
}

#[cfg(test)]
mod tests;
