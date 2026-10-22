mod pages;
mod revisions;
mod search;

pub use pages::SqlitePages;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ports::pages::{PageRepo, SaveOutcome};
    use crate::domain::{
        page::{Lang, Slug},
        revision::{Author, NewRevision, rollback_to},
    };
    use crate::store::db::{Db, open_memory};

    fn setup() -> (SqlitePages, Db) {
        let db = open_memory().unwrap();
        (SqlitePages::new(db.clone()), db)
    }

    fn repo() -> SqlitePages {
        setup().0
    }

    fn rev(body: &str) -> NewRevision {
        NewRevision {
            body: body.into(),
            summary: "s".into(),
            author: Author::User("loki".into()),
        }
    }

    fn slug(s: &str) -> Slug {
        Slug::parse(s).unwrap()
    }

    fn count(db: &Db, table: &str) -> i64 {
        db.lock()
            .unwrap()
            .query_row(&format!("select count(*) from {table}"), [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn save_creates_then_updates_then_noops() {
        let r = repo();
        assert!(matches!(
            r.save(&slug("a"), Lang::En, rev("one")).unwrap(),
            SaveOutcome::Created(_)
        ));
        assert!(matches!(
            r.save(&slug("a"), Lang::En, rev("two")).unwrap(),
            SaveOutcome::Updated(_)
        ));
        assert_eq!(
            r.save(&slug("a"), Lang::En, rev("two")).unwrap(),
            SaveOutcome::Unchanged
        );
        let page = r.find(&slug("a"), Lang::En).unwrap().unwrap();
        assert_eq!(page.current.body, "two");
        assert_eq!(r.history(page.id).unwrap().len(), 2);
    }

    #[test]
    fn unchanged_save_writes_nothing() {
        let (r, db) = setup();
        r.save(&slug("a"), Lang::En, rev("one")).unwrap();
        let before = (
            count(&db, "pages"),
            count(&db, "revisions"),
            count(&db, "revisions_fts"),
        );
        assert_eq!(
            r.save(&slug("a"), Lang::En, rev("one")).unwrap(),
            SaveOutcome::Unchanged
        );
        let after = (
            count(&db, "pages"),
            count(&db, "revisions"),
            count(&db, "revisions_fts"),
        );
        assert_eq!(before, after);
        assert_eq!(before, (1, 1, 1));
        // the connection is usable again, so the dropped transaction was rolled back
        assert!(matches!(
            r.save(&slug("a"), Lang::En, rev("two")).unwrap(),
            SaveOutcome::Updated(_)
        ));
    }

    #[test]
    fn old_revision_stays_readable() {
        let r = repo();
        r.save(&slug("a"), Lang::En, rev("one")).unwrap();
        r.save(&slug("a"), Lang::En, rev("two")).unwrap();
        let page = r.find(&slug("a"), Lang::En).unwrap().unwrap();
        let first = r.history(page.id).unwrap().last().unwrap().clone();
        assert_eq!(r.revision(page.id, first.id).unwrap().unwrap().body, "one");
    }

    #[test]
    fn rollback_adds_a_new_revision() {
        let r = repo();
        r.save(&slug("a"), Lang::En, rev("one")).unwrap();
        r.save(&slug("a"), Lang::En, rev("two")).unwrap();
        let page = r.find(&slug("a"), Lang::En).unwrap().unwrap();
        let first = r.history(page.id).unwrap().last().unwrap().clone();
        r.save(
            &slug("a"),
            Lang::En,
            rollback_to(&first, Author::User("loki".into())),
        )
        .unwrap();
        let now = r.find(&slug("a"), Lang::En).unwrap().unwrap();
        assert_eq!(now.current.body, "one");
        assert_eq!(r.history(page.id).unwrap().len(), 3);
    }

    #[test]
    fn languages_are_separate_pages() {
        let r = repo();
        r.save(&slug("a"), Lang::En, rev("en")).unwrap();
        assert!(r.find(&slug("a"), Lang::Ru).unwrap().is_none());
        r.save(&slug("a"), Lang::Ru, rev("ru")).unwrap();
        assert_eq!(
            r.find(&slug("a"), Lang::En).unwrap().unwrap().current.body,
            "en"
        );
        assert_eq!(
            r.find(&slug("a"), Lang::Ru).unwrap().unwrap().current.body,
            "ru"
        );
    }

    #[test]
    fn find_by_id_returns_the_same_page_as_find() {
        let r = repo();
        r.save(&slug("a"), Lang::En, rev("x")).unwrap();
        let a = r.find(&slug("a"), Lang::En).unwrap().unwrap();
        let b = r.find_by_id(a.id).unwrap().unwrap();
        assert_eq!((b.id, b.slug, b.lang), (a.id, a.slug, a.lang));
        assert_eq!(b.current.id, a.current.id);
        assert!(!b.deleted);
        r.set_deleted(a.id, true).unwrap();
        assert!(r.find_by_id(a.id).unwrap().unwrap().deleted);
        assert_eq!(r.list_deleted().unwrap()[0].id, a.id);
        assert!(r.find_by_id(a.id + 1).unwrap().is_none());
    }

    #[test]
    fn deleted_pages_are_hidden_and_restorable() {
        let r = repo();
        r.save(&slug("a"), Lang::En, rev("x")).unwrap();
        let id = r.find(&slug("a"), Lang::En).unwrap().unwrap().id;
        r.set_deleted(id, true).unwrap();
        assert!(r.find(&slug("a"), Lang::En).unwrap().is_none());
        assert!(r.list().unwrap().is_empty());
        assert!(r.search("x", Lang::En).unwrap().is_empty());
        r.set_deleted(id, false).unwrap();
        assert!(r.find(&slug("a"), Lang::En).unwrap().is_some());
        assert_eq!(r.list().unwrap().len(), 1);
        assert_eq!(r.search("x", Lang::En).unwrap().len(), 1);
    }

    #[test]
    fn save_on_deleted_slug_creates_a_fresh_page() {
        let r = repo();
        r.save(&slug("a"), Lang::En, rev("old")).unwrap();
        let old = r.find(&slug("a"), Lang::En).unwrap().unwrap().id;
        r.set_deleted(old, true).unwrap();
        let out = r.save(&slug("a"), Lang::En, rev("new")).unwrap();
        assert!(matches!(out, SaveOutcome::Created(_)));
        let page = r.find(&slug("a"), Lang::En).unwrap().unwrap();
        assert_ne!(page.id, old);
        assert_eq!(page.current.body, "new");
        assert_eq!(r.history(page.id).unwrap().len(), 1);
    }

    #[test]
    fn search_finds_current_text_only() {
        let r = repo();
        r.save(&slug("a"), Lang::En, rev("alpha beta")).unwrap();
        r.save(&slug("a"), Lang::En, rev("gamma")).unwrap();
        assert!(r.search("alpha", Lang::En).unwrap().is_empty());
        assert_eq!(r.search("gamma", Lang::En).unwrap().len(), 1);
        assert!(r.search("gamma", Lang::Ru).unwrap().is_empty());
        assert!(r.search("   ", Lang::En).unwrap().is_empty());
    }

    #[test]
    fn search_matches_slug_words() {
        let r = repo();
        r.save(&slug("blocks/robot-arm"), Lang::En, rev("text"))
            .unwrap();
        assert_eq!(r.search("robot", Lang::En).unwrap().len(), 1);
    }

    #[test]
    fn search_survives_fts_syntax_characters() {
        let r = repo();
        r.save(&slug("a"), Lang::En, rev("alpha beta")).unwrap();
        for q in [
            "\"",
            "\"alpha",
            "alpha*",
            "AND",
            "alpha AND",
            "(",
            "(alpha",
            "a OR",
            "NEAR(",
            "-x",
            "col:alpha",
            "'",
            "^",
        ] {
            assert!(r.search(q, Lang::En).is_ok(), "{q}");
        }
        // operators are plain words: every word must be present
        assert_eq!(r.search("beta alpha", Lang::En).unwrap().len(), 1);
        assert!(r.search("alpha AND beta", Lang::En).unwrap().is_empty());
    }
}
