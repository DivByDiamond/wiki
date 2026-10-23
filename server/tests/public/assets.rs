use crate::support::app;
use axum::{Router, body::Body, http::Request, response::Response};
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn get(app: &Router, uri: &str) -> Response {
    app.clone()
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn get_with(app: &Router, uri: &str, inm: Option<&str>) -> Response {
    let mut req = Request::get(uri);
    if let Some(v) = inm {
        req = req.header("if-none-match", v);
    }
    app.clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

/// The frontend must be built first (`cd web && bun run build`); never skip silently.
fn require_built() {
    let dist = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../web/dist");
    for f in ["app.js", "app.css"] {
        assert!(
            dist.join(f).exists(),
            "web/dist/{f} is missing: run `bun run build` in web/ before `cargo test`"
        );
    }
}

#[tokio::test]
async fn serves_app_js_and_css_with_type_and_no_cache() {
    require_built();
    let (app, ..) = app();
    for (uri, ty) in [
        ("/static/app.js", "text/javascript"),
        ("/static/app.css", "text/css"),
        ("/static/favicon.svg", "image/svg+xml"),
        ("/static/favicon-32.png", "image/png"),
        ("/static/apple-touch-icon.png", "image/png"),
    ] {
        let res = get(&app, uri).await;
        assert_eq!(res.status(), 200, "{uri}");
        let h = res.headers();
        assert!(h["content-type"].to_str().unwrap().starts_with(ty));
        assert_eq!(h["cache-control"], "no-cache");
        assert_eq!(h["x-content-type-options"], "nosniff");
        let etag = h["etag"].to_str().unwrap();
        assert!(
            etag.starts_with('"') && etag.ends_with('"') && etag.len() == 66,
            "{etag}"
        );
    }
}

#[tokio::test]
async fn matching_etag_gets_304_without_a_body() {
    require_built();
    let (app, ..) = app();
    let first = get(&app, "/static/app.js").await;
    let etag = first.headers()["etag"].to_str().unwrap().to_owned();
    for inm in [
        etag.clone(),
        format!("W/{etag}"),
        format!("\"x\", {etag}"),
        "*".into(),
    ] {
        let res = get_with(&app, "/static/app.js", Some(&inm)).await;
        assert_eq!(res.status(), 304, "{inm}");
        assert_eq!(res.headers()["etag"].to_str().unwrap(), etag);
        assert_eq!(res.headers()["cache-control"], "no-cache");
        let body = res.into_body().collect().await.unwrap().to_bytes();
        assert!(body.is_empty());
    }
    let res = get_with(&app, "/static/app.js", Some("\"stale\"")).await;
    assert_eq!(res.status(), 200);
    assert_ne!(
        get(&app, "/static/app.css").await.headers()["etag"]
            .to_str()
            .unwrap(),
        etag
    );
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
