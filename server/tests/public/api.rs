use crate::support::{app, app_with_state, json, make_key};
use axum::{Router, body::Body, http::Request};
use tower::ServiceExt;

async fn put(
    app: &Router,
    uri: &str,
    key: Option<&str>,
    body: Vec<u8>,
) -> axum::response::Response {
    let mut req = Request::put(uri);
    if let Some(k) = key {
        req = req.header("authorization", format!("Bearer {k}"));
    }
    app.clone()
        .oneshot(req.body(Body::from(body)).unwrap())
        .await
        .unwrap()
}

async fn get(app: &Router, uri: &str, key: Option<&str>) -> axum::response::Response {
    let mut req = Request::get(uri);
    if let Some(k) = key {
        req = req.header("authorization", format!("Bearer {k}"));
    }
    app.clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn put_then_get_roundtrip() {
    let (app, write, _) = app();
    let res = put(
        &app,
        "/api/pages/blocks/computer?lang=en&summary=init",
        Some(&write),
        b"# Computer".to_vec(),
    )
    .await;
    assert_eq!(res.status(), 200);
    let res = get(&app, "/api/pages/blocks/computer?lang=en", None).await;
    assert_eq!(res.status(), 200);
    assert_eq!(json(res).await["body"], "# Computer");
}

#[tokio::test]
async fn write_without_key_is_401_and_read_key_is_403() {
    let (app, _, read) = app();
    let res = put(&app, "/api/pages/a?lang=en", None, b"x".to_vec()).await;
    assert_eq!(res.status(), 401);
    assert_eq!(json(res).await["error"], "unauthorized");
    let res = put(&app, "/api/pages/a?lang=en", Some(&read), b"x".to_vec()).await;
    assert_eq!(res.status(), 403);
    assert_eq!(get(&app, "/api/pages/a?lang=en", None).await.status(), 404);
}

#[tokio::test]
async fn unknown_key_is_401_on_write() {
    let (app, _, _) = app();
    let res = put(
        &app,
        "/api/pages/a?lang=en",
        Some("ocw_nope"),
        b"x".to_vec(),
    )
    .await;
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn revoked_key_stops_working_immediately() {
    let (app, st, _, _) = app_with_state();
    let (key, id) = make_key(&st, "ci", true, true, false);
    assert_eq!(
        put(&app, "/api/pages/a?lang=en", Some(&key), b"1".to_vec())
            .await
            .status(),
        200
    );
    st.access.revoke_key(id).unwrap();
    assert_eq!(
        put(&app, "/api/pages/a?lang=en", Some(&key), b"2".to_vec())
            .await
            .status(),
        401
    );
    let page = json(get(&app, "/api/pages/a?lang=en", None).await).await;
    assert_eq!(page["body"], "1");
}

#[tokio::test]
async fn repeated_put_does_not_add_revision() {
    let (app, write, _) = app();
    let first = put(&app, "/api/pages/a?lang=en", Some(&write), b"same".to_vec()).await;
    assert!(
        json(first).await["outcome"]
            .as_str()
            .unwrap()
            .starts_with("Created")
    );
    let second = put(&app, "/api/pages/a?lang=en", Some(&write), b"same".to_vec()).await;
    assert_eq!(second.status(), 200);
    assert_eq!(json(second).await["outcome"], "Unchanged");
    let hist = json(get(&app, "/api/revisions/a?lang=en", None).await).await;
    assert_eq!(hist.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn bad_slug_or_lang_is_422() {
    let (app, write, _) = app();
    for uri in [
        "/api/pages/Bad%20Slug?lang=en",
        "/api/pages/a?lang=de",
        "/api/pages/a",
    ] {
        let res = put(&app, uri, Some(&write), b"x".to_vec()).await;
        assert_eq!(res.status(), 422, "{uri}");
        assert_eq!(json(res).await["error"], "invalid");
    }
    assert_eq!(get(&app, "/api/pages/a?lang=de", None).await.status(), 422);
}

#[tokio::test]
async fn bad_body_is_rejected_without_500() {
    let (app, write, _) = app();
    let res = put(
        &app,
        "/api/pages/a?lang=en",
        Some(&write),
        vec![0xff, 0xfe, 0xfd],
    )
    .await;
    assert_eq!(res.status(), 422);
    let big = vec![b'a'; 1024 * 1024 + 1];
    let res = put(&app, "/api/pages/a?lang=en", Some(&write), big).await;
    assert_eq!(res.status(), 413);
    let ok = vec![b'a'; 1024 * 1024];
    assert_eq!(
        put(&app, "/api/pages/a?lang=en", Some(&write), ok)
            .await
            .status(),
        200
    );
}

#[tokio::test]
async fn export_needs_export_scope() {
    let (app, st, write, read) = app_with_state();
    put(&app, "/api/pages/a?lang=en", Some(&write), b"# A".to_vec()).await;
    assert_eq!(get(&app, "/api/export", None).await.status(), 401);
    assert_eq!(get(&app, "/api/export", Some(&read)).await.status(), 403);
    assert_eq!(get(&app, "/api/export", Some(&write)).await.status(), 403);
    let (exp, _) = make_key(&st, "backup", true, false, true);
    let res = get(&app, "/api/export", Some(&exp)).await;
    assert_eq!(res.status(), 200);
    let v = json(res).await;
    assert_eq!(v["pages"][0]["slug"], "a");
    assert_eq!(v["pages"][0]["lang"], "en");
    assert_eq!(v["pages"][0]["body"], "# A");
}

#[tokio::test]
async fn missing_page_is_404_json() {
    let (app, _, _) = app();
    let res = get(&app, "/api/pages/nope?lang=en", None).await;
    assert_eq!(res.status(), 404);
    assert_eq!(json(res).await["error"], "not_found");
    assert_eq!(
        get(&app, "/api/revisions/nope?lang=en", None)
            .await
            .status(),
        404
    );
}

#[tokio::test]
async fn list_shows_saved_pages() {
    let (app, write, _) = app();
    put(&app, "/api/pages/a?lang=en", Some(&write), b"1".to_vec()).await;
    put(&app, "/api/pages/b/c?lang=ru", Some(&write), b"2".to_vec()).await;
    let v = json(get(&app, "/api/pages", None).await).await;
    let items = v.as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert!(
        items
            .iter()
            .any(|i| i["slug"] == "b/c" && i["lang"] == "ru")
    );
    assert!(items.iter().any(|i| i["slug"] == "a" && i["lang"] == "en"));
}

#[tokio::test]
async fn revisions_are_newest_first_with_key_name() {
    let (app, write, _) = app();
    put(
        &app,
        "/api/pages/a/b?lang=en&summary=one",
        Some(&write),
        b"1".to_vec(),
    )
    .await;
    put(
        &app,
        "/api/pages/a/b?lang=en&summary=two",
        Some(&write),
        b"2".to_vec(),
    )
    .await;
    let v = json(get(&app, "/api/revisions/a/b?lang=en", None).await).await;
    let items = v.as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["summary"], "two");
    assert_eq!(items[1]["summary"], "one");
    assert_eq!(items[0]["author"], "ключ: writer");
}
