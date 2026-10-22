use crate::support::app;
use axum::{Router, body::Body, http::Request, response::Response};
use tower::ServiceExt;

async fn get(app: &Router, uri: &str) -> Response {
    app.clone()
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

fn built() -> bool {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../web/dist/app.js")
        .exists()
}

#[tokio::test]
async fn serves_app_js_with_type_and_cache() {
    if !built() {
        eprintln!("web/dist not built; skipping");
        return;
    }
    let (app, ..) = app();
    let res = get(&app, "/static/app.js").await;
    assert_eq!(res.status(), 200);
    let h = res.headers();
    assert!(
        h["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/javascript")
    );
    assert!(h["cache-control"].to_str().unwrap().contains("max-age"));
}

#[tokio::test]
async fn unknown_asset_is_404() {
    let (app, ..) = app();
    assert_eq!(get(&app, "/static/nope.js").await.status(), 404);
}

#[tokio::test]
async fn traversal_is_rejected() {
    let (app, ..) = app();
    for uri in [
        "/static/../Cargo.toml",
        "/static/%2e%2e/Cargo.toml",
        "/static/..%2fCargo.toml",
        "/static/a/../../Cargo.toml",
        "/static/..%5cCargo.toml",
    ] {
        let s = get(&app, uri).await.status();
        assert!(s == 404 || s == 400, "{uri} -> {s}");
    }
}
