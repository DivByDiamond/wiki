use super::{SqliteAccess, SqliteProposals};
use crate::domain::ports::{
    access::AccessRepo,
    proposals::{ProposalRepo, Quota},
};
use crate::domain::proposal::Status;
use crate::store::db::{Db, open_memory};

fn quota() -> Quota {
    Quota {
        open: 100,
        per_day: 1000,
        since: 0,
    }
}

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
    let id1 = p
        .create(page, rev, "one", "s1", u1, quota())
        .unwrap()
        .unwrap();
    let id2 = p
        .create(page, rev, "two", "s2", u2, quota())
        .unwrap()
        .unwrap();
    let got = p.get(id1).unwrap().unwrap();
    assert_eq!((got.body.as_str(), got.summary.as_str()), ("one", "s1"));
    assert_eq!((got.user_id, got.status), (u1, Status::Open));
    assert!(p.get(999).unwrap().is_none());
    assert_eq!(p.list(Status::Open).unwrap().len(), 2);
    p.claim(id2, Status::Accepted, u1, "ok").unwrap();
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
    let a = p.create(page, rev, "a", "", u1, quota()).unwrap().unwrap();
    p.create(page, rev, "b", "", u1, quota()).unwrap();
    p.create(page, rev, "c", "", u2, quota()).unwrap();
    assert_eq!(p.count_open_by_user(u1).unwrap(), 2);
    p.claim(a, Status::Rejected, u2, "no").unwrap();
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
fn claiming_twice_fails() {
    let (db, page, rev, u1, u2) = seeded();
    let p = SqliteProposals::new(db);
    let id = p.create(page, rev, "a", "", u1, quota()).unwrap().unwrap();
    assert!(p.claim(id, Status::Accepted, u2, "fine").unwrap());
    assert!(!p.claim(id, Status::Rejected, u2, "again").unwrap());
    assert!(!p.claim(999, Status::Accepted, u2, "").unwrap());
    assert_eq!(p.get(id).unwrap().unwrap().status, Status::Accepted);
}

#[test]
fn quota_is_enforced_inside_the_insert() {
    let (db, page, rev, u1, u2) = seeded();
    let p = SqliteProposals::new(db);
    let one = |per_day| Quota {
        open: 1,
        per_day,
        since: 0,
    };
    let first = p.create(page, rev, "a", "", u1, one(10)).unwrap().unwrap();
    // The open limit blocks the second proposal while the first is still open.
    assert!(p.create(page, rev, "b", "", u1, one(10)).unwrap().is_none());
    p.claim(first, Status::Rejected, u2, "").unwrap();
    // Now no proposal is open, but the daily budget is already used up.
    assert!(p.create(page, rev, "c", "", u1, one(1)).unwrap().is_none());
}

#[test]
fn claim_wins_once_and_reopen_releases_it() {
    let (db, page, rev, u1, u2) = seeded();
    let p = SqliteProposals::new(db);
    let id = p.create(page, rev, "a", "", u1, quota()).unwrap().unwrap();
    assert!(p.claim(id, Status::Accepted, u2, "ok").unwrap());
    assert!(!p.claim(id, Status::Rejected, u2, "late").unwrap());
    assert!(!p.claim(999, Status::Accepted, u2, "").unwrap());
    assert_eq!(p.get(id).unwrap().unwrap().status, Status::Accepted);
    p.reopen(id).unwrap();
    assert_eq!(p.get(id).unwrap().unwrap().status, Status::Open);
    assert!(p.claim(id, Status::Rejected, u2, "again").unwrap());
}
