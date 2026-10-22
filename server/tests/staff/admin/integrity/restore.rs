use crate::admin::common::env;
use crate::support::{json, save_page, text};
use axum::{body::Body, http::Request};
use oc2r_wiki::domain::page::{Lang, Slug};
use tower::ServiceExt;

async fn status(e: &crate::admin::common::Env, uri: &str) -> u16 {
    let req = Request::get(uri).body(Body::empty()).unwrap();
    e.app.clone().oneshot(req).await.unwrap().status().as_u16()
}

#[tokio::test]
async fn delete_hides_the_page_everywhere_and_restore_brings_it_back() {
    let e = env();
    let (key, _) = crate::support::make_key(&e.st, "r", true, false, false);
    let api = |k: &str| {
        Request::get("/api/pages/a?lang=en")
            .header("authorization", format!("Bearer {k}"))
            .body(Body::empty())
            .unwrap()
    };
    assert_eq!(status(&e, "/en/a").await, 200);
    let res = e.post("/admin/delete/en/a", "").await;
    assert_eq!(res.status(), 303);
    assert_eq!(status(&e, "/en/a").await, 404);
    assert_eq!(
        e.app.clone().oneshot(api(&key)).await.unwrap().status(),
        404
    );
    assert!(e.get_text("/admin").await.contains("Restore"));

    let res = e.post("/admin/restore/en/a", "").await;
    assert_eq!(res.status(), 303);
    assert_eq!(status(&e, "/en/a").await, 200);
    let res = e.app.clone().oneshot(api(&key)).await.unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(json(res).await["slug"], "a");
}

#[tokio::test]
async fn deleting_or_restoring_a_missing_page_is_404() {
    let e = env();
    assert_eq!(e.post("/admin/delete/en/nope", "").await.status(), 404);
    assert_eq!(e.post("/admin/restore/en/a", "").await.status(), 404);
}

#[tokio::test]
async fn restore_with_a_live_page_on_the_same_slug_is_a_clean_409() {
    let e = env();
    let id = e.page("a").id;
    e.st.pages.set_deleted(id, true).unwrap();
    save_page(&e.st, "a", Lang::En, "new live page", "again");
    let res = e.post("/admin/restore/en/a", "").await;
    assert_eq!(res.status(), 409);
    let html = text(res).await;
    assert!(html.contains("already exists"), "{html}");
    assert!(
        e.st.pages
            .list_deleted()
            .unwrap()
            .iter()
            .any(|p| p.id == id && p.slug == Slug::parse("a").unwrap())
    );
    assert_eq!(e.page("a").current.body, "new live page");
}
