use super::*;
use crate::{
    domain::ports::pages::PageRepo,
    store::{content::SqlitePages, db::open_memory},
};

#[test]
fn bundle_is_seeded_searchable_and_idempotent() {
    let db = open_memory().unwrap();
    let catalog = bundled().unwrap();
    assert!(catalog.pages.len() >= 130);
    assert_eq!(
        import(&db, &catalog, true).unwrap().created,
        catalog.pages.len()
    );
    let repo = SqlitePages::new(db.clone());
    for lang in [Lang::Ru, Lang::En, Lang::Zh] {
        assert!(
            repo.find(&Slug::parse("start").unwrap(), lang)
                .unwrap()
                .is_some()
        );
    }
    assert!(!repo.search("OnyxOS", Lang::Ru).unwrap().is_empty());
    assert_eq!(
        import(&db, &catalog, false).unwrap().unchanged,
        catalog.pages.len()
    );
    let revisions: i64 = db
        .lock()
        .unwrap()
        .query_row("select count(*) from revisions", [], |r| r.get(0))
        .unwrap();
    assert_eq!(revisions as usize, catalog.pages.len());
}

#[test]
fn sync_updates_imports_but_preserves_edits_and_deletions() {
    let db = open_memory().unwrap();
    let mut catalog = bundled().unwrap();
    catalog.pages.truncate(3);
    import(&db, &catalog, false).unwrap();
    let repo = SqlitePages::new(db.clone());
    let slug = |i: usize| Slug::parse(&catalog.pages[i].slug).unwrap();
    let lang = |i: usize| Lang::parse(&catalog.pages[i].lang).unwrap();
    repo.save(
        &slug(0),
        lang(0),
        NewRevision {
            body: "editor text".into(),
            summary: "manual edit".into(),
            author: Author::User("writer".into()),
        },
    )
    .unwrap();
    let deleted = repo.find(&slug(1), lang(1)).unwrap().unwrap();
    repo.set_deleted(deleted.id, true).unwrap();
    let changed_slug = slug(2);
    let changed_lang = lang(2);
    catalog.pages[2].body = "new imported text".into();
    let report = import(&db, &catalog, false).unwrap();
    assert_eq!((report.updated, report.protected), (1, 2));
    assert_eq!(
        repo.find(
            &Slug::parse(&catalog.pages[0].slug).unwrap(),
            Lang::parse(&catalog.pages[0].lang).unwrap()
        )
        .unwrap()
        .unwrap()
        .current
        .body,
        "editor text"
    );
    assert!(repo.find_by_id(deleted.id).unwrap().unwrap().deleted);
    let changed = repo.find(&changed_slug, changed_lang).unwrap().unwrap();
    assert_eq!(changed.current.body, "new imported text");
    assert_eq!(repo.history(changed.id).unwrap().len(), 2);
    assert_eq!(import(&db, &catalog, true).unwrap().created, 0);
}

#[test]
fn invalid_bundle_rolls_back_every_page() {
    let db = open_memory().unwrap();
    let mut catalog = bundled().unwrap();
    catalog.pages[1].slug = "../bad".into();
    assert!(import(&db, &catalog, false).is_err());
    assert!(SqlitePages::new(db).list().unwrap().is_empty());
}

#[test]
fn sync_preserves_manual_navigation_only_edits() {
    use crate::domain::ports::pages::Navigation;
    let db = open_memory().unwrap();
    let mut catalog = bundled().unwrap();
    catalog.pages.truncate(1);
    import(&db, &catalog, false).unwrap();
    let repo = SqlitePages::new(db.clone());
    let entry = &catalog.pages[0];
    let slug = Slug::parse(&entry.slug).unwrap();
    let lang = Lang::parse(&entry.lang).unwrap();
    let page = repo.find(&slug, lang).unwrap().unwrap();
    repo.save_with_navigation(
        &slug,
        lang,
        NewRevision {
            body: page.current.body,
            summary: "Move navigation".into(),
            author: Author::User("editor".into()),
        },
        Navigation {
            section: "Custom".into(),
            position: 93,
        },
    )
    .unwrap();
    let report = import(&db, &catalog, false).unwrap();
    assert_eq!(report.protected, 1);
    let page = repo.find(&slug, lang).unwrap().unwrap();
    assert_eq!((page.section.as_str(), page.position), ("Custom", 93));
    assert_eq!(repo.history(page.id).unwrap().len(), 2);
}
