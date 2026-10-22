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
mod tests {
    use super::*;
    use crate::domain::ports::{access::AccessRepo, proposals::ProposalRepo};
    use crate::domain::proposal::Status;
    use crate::store::db::{Db, open_memory};

    #[test]
    fn upsert_user_is_stable_and_updates_login() {
        let a = SqliteAccess::new(open_memory().unwrap());
        let u1 = a.upsert_user(42, "old").unwrap();
        let u2 = a.upsert_user(42, "new").unwrap();
        assert_eq!(u1.id, u2.id);
        assert_eq!(u2.login, "new");
        assert_eq!(u2.github_id, 42);
        assert!(!u2.admin && !u2.blocked);
    }

    #[test]
    fn keys_found_by_hash_until_revoked() {
        let a = SqliteAccess::new(open_memory().unwrap());
        let id = a.create_key("ci", "h1", true, true, false).unwrap();
        let k = a.key_by_hash("h1").unwrap().unwrap();
        assert!(k.read && k.write && !k.export);
        assert_eq!(a.list_keys().unwrap().len(), 1);
        a.touch_key(id).unwrap();
        a.revoke_key(id).unwrap();
        assert!(a.key_by_hash("h1").unwrap().is_none());
        assert!(a.list_keys().unwrap().is_empty());
    }

    #[test]
    fn block_and_admin_flags() {
        let a = SqliteAccess::new(open_memory().unwrap());
        let u = a.upsert_user(1, "x").unwrap();
        a.set_blocked(u.id, true).unwrap();
        a.set_admin(u.id, true).unwrap();
        let u = a.user(u.id).unwrap().unwrap();
        assert!(u.blocked && u.admin);
        a.set_blocked(u.id, false).unwrap();
        assert!(!a.user(u.id).unwrap().unwrap().blocked);
        assert!(a.user(999).unwrap().is_none());
    }

    /// Db with one page, one revision and two users; returns (db, page, rev, u1, u2).
    fn seeded() -> (Db, i64, i64, i64, i64) {
        let db = open_memory().unwrap();
        let (page, rev) = {
            let c = db.lock().unwrap();
            c.execute(
                "insert into pages (slug, lang, created_at) values ('a','en',1)",
                [],
            )
            .unwrap();
            let page = c.last_insert_rowid();
            c.execute(
                "insert into revisions (page_id, body, author_kind, created_at) values (?1,'b','import',1)",
                [page],
            )
            .unwrap();
            (page, c.last_insert_rowid())
        };
        let a = SqliteAccess::new(db.clone());
        let u1 = a.upsert_user(1, "u1").unwrap().id;
        let u2 = a.upsert_user(2, "u2").unwrap().id;
        (db, page, rev, u1, u2)
    }

    #[test]
    fn proposals_create_get_and_list_by_status() {
        let (db, page, rev, u1, u2) = seeded();
        let p = SqliteProposals::new(db);
        let id1 = p.create(page, rev, "one", "s1", u1).unwrap();
        let id2 = p.create(page, rev, "two", "s2", u2).unwrap();
        let got = p.get(id1).unwrap().unwrap();
        assert_eq!((got.body.as_str(), got.summary.as_str()), ("one", "s1"));
        assert_eq!((got.user_id, got.status), (u1, Status::Open));
        assert!(p.get(999).unwrap().is_none());
        assert_eq!(p.list(Status::Open).unwrap().len(), 2);
        p.decide(id2, Status::Accepted, u1, "ok").unwrap();
        let open = p.list(Status::Open).unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].id, id1);
        let acc = p.list(Status::Accepted).unwrap();
        assert_eq!(acc.len(), 1);
        assert_eq!(acc[0].id, id2);
        assert!(p.list(Status::Rejected).unwrap().is_empty());
    }

    #[test]
    fn proposals_counts() {
        let (db, page, rev, u1, u2) = seeded();
        let p = SqliteProposals::new(db.clone());
        let a = p.create(page, rev, "a", "", u1).unwrap();
        p.create(page, rev, "b", "", u1).unwrap();
        p.create(page, rev, "c", "", u2).unwrap();
        assert_eq!(p.count_open_by_user(u1).unwrap(), 2);
        p.decide(a, Status::Rejected, u2, "no").unwrap();
        assert_eq!(p.count_open_by_user(u1).unwrap(), 1);
        db.lock()
            .unwrap()
            .execute("update proposals set created_at = 100 where id = ?1", [a])
            .unwrap();
        assert_eq!(p.count_since_by_user(u1, 0).unwrap(), 2);
        assert_eq!(p.count_since_by_user(u1, 101).unwrap(), 1);
        assert_eq!(p.count_since_by_user(u2, 101).unwrap(), 1);
    }

    #[test]
    fn deciding_twice_is_an_error() {
        let (db, page, rev, u1, u2) = seeded();
        let p = SqliteProposals::new(db);
        let id = p.create(page, rev, "a", "", u1).unwrap();
        p.decide(id, Status::Accepted, u2, "fine").unwrap();
        assert!(p.decide(id, Status::Rejected, u2, "again").is_err());
        assert!(p.decide(999, Status::Accepted, u2, "").is_err());
        assert_eq!(p.get(id).unwrap().unwrap().status, Status::Accepted);
    }

    #[test]
    fn claim_wins_once_and_reopen_releases_it() {
        let (db, page, rev, u1, u2) = seeded();
        let p = SqliteProposals::new(db);
        let id = p.create(page, rev, "a", "", u1).unwrap();
        assert!(p.claim(id, Status::Accepted, u2, "ok").unwrap());
        assert!(!p.claim(id, Status::Rejected, u2, "late").unwrap());
        assert!(!p.claim(999, Status::Accepted, u2, "").unwrap());
        assert_eq!(p.get(id).unwrap().unwrap().status, Status::Accepted);
        p.reopen(id).unwrap();
        assert_eq!(p.get(id).unwrap().unwrap().status, Status::Open);
        assert!(p.claim(id, Status::Rejected, u2, "again").unwrap());
    }
}
