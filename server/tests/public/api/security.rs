use super::{get, put};
use crate::support::{app_with_state, json, make_key, save_page};
use axum::{body::Body, http::Request};
use oc2r_wiki::domain::page::{Lang, Slug};
use tower::ServiceExt;

async fn put_cookie(app: &axum::Router, sid: &str) -> u16 {
    let req = Request::put("/api/pages/a?lang=en")
        .header("cookie", format!("sid={sid}"))
        .body(Body::from("x"))
        .unwrap();
    app.clone().oneshot(req).await.unwrap().status().as_u16()
}

#[tokio::test]
async fn hostile_slugs_are_rejected_over_http() {
    let (app, _, write, _) = app_with_state();
    // Observed: the handler answers 422 for every one of these (routing never
    // rejects first); the check is kept tight so a regression to 200/500 or
    // a changed status is noticed.
    for uri in [
        "/api/pages/..%2f..%2fx?lang=en",
        "/api/pages/%2e%2e?lang=en",
        "/api/pages/%2e%2e/x?lang=en",
        "/api/pages/a/?lang=en",
        "/api/pages/a//b?lang=en",
        "/api/pages/Upper?lang=en",
    ] {
        let res = put(&app, uri, Some(&write), b"x".to_vec()).await;
        assert_eq!(res.status(), 422, "put {uri}");
        let res = get(&app, uri, None).await;
        assert_eq!(res.status(), 422, "get {uri}");
    }
}

#[tokio::test]
async fn the_search_address_is_reserved_for_new_pages() {
    let (app, st, write, _) = app_with_state();
    let res = put(
        &app,
        "/api/pages/search?lang=en",
        Some(&write),
        b"x".to_vec(),
    )
    .await;
    assert_eq!(res.status(), 422);
    assert_eq!(json(res).await["error"], "invalid");
    assert_eq!(
        get(&app, "/api/pages/search?lang=en", None).await.status(),
        404
    );
    // A page that predates the reservation can still be updated.
    save_page(&st, "search", Lang::En, "old", "legacy");
    let res = put(
        &app,
        "/api/pages/search?lang=en",
        Some(&write),
        b"new".to_vec(),
    )
    .await;
    assert_eq!(res.status(), 200);
    assert_eq!(json(res).await["outcome"], "updated");
}

#[tokio::test]
async fn soft_deleted_page_disappears_from_api() {
    let (app, st, _, _) = app_with_state();
    save_page(&st, "gone", Lang::En, "# Gone", "s");
    save_page(&st, "kept", Lang::En, "# Kept", "s");
    let (exp, _) = make_key(&st, "backup", true, false, true);
    let id = st
        .pages
        .find(&Slug::parse("gone").unwrap(), Lang::En)
        .unwrap()
        .unwrap()
        .id;
    st.pages.set_deleted(id, true).unwrap();
    assert_eq!(
        get(&app, "/api/pages/gone?lang=en", None).await.status(),
        404
    );
    let res = get(&app, "/api/revisions/gone?lang=en", None).await;
    assert_eq!(res.status(), 404);
    let list = json(get(&app, "/api/pages", None).await).await;
    assert!(list.as_array().unwrap().iter().all(|i| i["slug"] != "gone"));
    let export = json(get(&app, "/api/export", Some(&exp)).await).await;
    let pages = export["pages"].as_array().unwrap();
    assert!(pages.iter().all(|i| i["slug"] != "gone"));
    assert!(pages.iter().any(|i| i["slug"] == "kept"));
}

#[tokio::test]
async fn session_users_write_only_when_admin_and_not_blocked() {
    let (app, st, _, _) = app_with_state();
    let user = st.access.upsert_user(1, "plain").unwrap();
    assert_eq!(put_cookie(&app, &st.sessions.create(user.id)).await, 403);

    let adm = st.access.upsert_user(2, "adm").unwrap();
    st.access.set_admin(adm.id, true).unwrap();
    let sid = st.sessions.create(adm.id);
    st.access.set_blocked(adm.id, true).unwrap();
    assert_eq!(put_cookie(&app, &sid).await, 403);
    st.access.set_blocked(adm.id, false).unwrap();
    assert_eq!(put_cookie(&app, &sid).await, 200);
}

#[tokio::test]
async fn oversize_put_is_413_json() {
    let (app, _, write, _) = app_with_state();
    let big = vec![b'a'; 1024 * 1024 + 1];
    let res = put(&app, "/api/pages/a?lang=en", Some(&write), big).await;
    assert_eq!(res.status(), 413);
    assert_eq!(json(res).await["error"], "too_large");
}

#[tokio::test]
async fn extractor_rejections_are_json() {
    let (app, _, _, _) = app_with_state();
    let res = get(&app, "/api/pages/x", None).await;
    assert_eq!(res.status(), 422);
    assert_eq!(json(res).await["error"], "invalid");
    for uri in [
        "/api/pages/%ff?lang=en",
        "/api/pages/x?lang=en&lang=ru",
        "/api/pages/x?lang=%zz",
    ] {
        let res = get(&app, uri, None).await;
        let status = res.status().as_u16();
        assert!((400..500).contains(&status), "{uri}");
        assert!(json(res).await["error"].is_string(), "{uri}");
    }
}

#[tokio::test]
async fn unknown_api_path_is_json_404_but_reader_stays_html() {
    let (app, _, _, _) = app_with_state();
    let res = get(&app, "/api/unknown", None).await;
    assert_eq!(res.status(), 404);
    assert_eq!(json(res).await["error"], "not_found");
    let res = get(&app, "/en/missing", None).await;
    assert_eq!(res.status(), 404);
    let ct = res.headers()["content-type"].to_str().unwrap().to_string();
    assert!(ct.starts_with("text/html"), "{ct}");
}
