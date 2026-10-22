use crate::admin::common::env;

#[tokio::test]
async fn editor_page_has_the_ids_the_frontend_looks_up() {
    let e = env();
    let html = e.get_text("/admin/edit/en/a").await;
    for id in ["editor", "editor-toolbar", "editor-preview"] {
        assert!(html.contains(&format!("id=\"{id}\"")), "missing #{id}");
    }
    assert!(!html.contains("data-preview-url"));
    assert!(!html.contains("id=\"preview\""));
}

#[tokio::test]
async fn rollback_to_the_current_revision_says_so() {
    let e = env();
    let cur = e.page("a").current.id;
    let res = e.post("/admin/rollback/en/a", &format!("rev={cur}")).await;
    assert_eq!(res.status(), 303);
    let loc = res.headers()["location"].to_str().unwrap().to_owned();
    assert!(loc.contains("same=1"), "{loc}");
    let html = e.get_text(&loc).await;
    assert!(html.contains("already the current revision"), "{html}");
    assert_eq!(e.st.pages.history(e.page("a").id).unwrap().len(), 1);
}
