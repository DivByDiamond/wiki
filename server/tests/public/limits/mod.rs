use crate::support::{make_key, save_page};
use axum::{Router, body::Body, http::Request, response::Response};
use oc2r_wiki::{
    domain::page::Lang,
    http::{limit::rate_limit::RateLimiter, state::AppState},
    router,
    store::db::open_memory,
};
use std::time::Duration;
use tower::ServiceExt;

fn limited(max: u32) -> (Router, AppState) {
    let st = AppState::with_limiter(
        open_memory().unwrap(),
        RateLimiter::new(max, Duration::from_secs(60)),
    );
    (router(st.clone()), st)
}

async fn send(app: &Router, method: &str, uri: &str, ip: &str, auth: Option<&str>) -> Response {
    let mut req = Request::builder().method(method).uri(uri);
    if !ip.is_empty() {
        req = req.header("x-real-ip", ip);
    }
    if let Some(key) = auth {
        req = req.header("authorization", format!("Bearer {key}"));
    }
    app.clone()
        .oneshot(req.body(Body::from("body")).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn diff_is_limited_per_client_ip() {
    let (app, st) = limited(2);
    save_page(&st, "a", Lang::En, "one", "s1");
    save_page(&st, "a", Lang::En, "two", "s2");
    let uri = "/en/a?diff=1..2";
    for _ in 0..2 {
        assert_eq!(send(&app, "GET", uri, "1.1.1.1", None).await.status(), 200);
    }
    let res = send(&app, "GET", uri, "1.1.1.1", None).await;
    assert_eq!(res.status(), 429);
    assert!(res.headers().contains_key("retry-after"));
    assert_eq!(send(&app, "GET", uri, "2.2.2.2", None).await.status(), 200);
}

#[tokio::test]
async fn search_and_history_are_limited() {
    let (app, st) = limited(1);
    save_page(&st, "a", Lang::En, "one", "s1");
    assert_eq!(
        send(&app, "GET", "/en/search?q=one", "3.3.3.3", None)
            .await
            .status(),
        200
    );
    let res = send(&app, "GET", "/en/search?q=one", "3.3.3.3", None).await;
    assert_eq!(res.status(), 429);
    assert!(res.headers().contains_key("retry-after"));
    assert_eq!(
        send(&app, "GET", "/en/a?history", "3.3.3.3", None)
            .await
            .status(),
        200
    );
    assert_eq!(
        send(&app, "GET", "/en/a?history", "3.3.3.3", None)
            .await
            .status(),
        429
    );
}

#[tokio::test]
async fn api_put_is_limited_per_key() {
    let (app, st) = limited(2);
    let (key, _) = make_key(&st, "w", true, true, false);
    let uri = "/api/pages/a?lang=en";
    for _ in 0..2 {
        assert_eq!(send(&app, "PUT", uri, "", Some(&key)).await.status(), 200);
    }
    let res = send(&app, "PUT", uri, "", Some(&key)).await;
    assert_eq!(res.status(), 429);
    assert!(res.headers().contains_key("retry-after"));
    let (other, _) = make_key(&st, "w2", true, true, false);
    assert_eq!(send(&app, "PUT", uri, "", Some(&other)).await.status(), 200);
}

#[tokio::test]
async fn plain_article_views_are_not_limited() {
    let (app, st) = limited(1);
    save_page(&st, "a", Lang::En, "one", "s1");
    for _ in 0..5 {
        assert_eq!(
            send(&app, "GET", "/en/a", "4.4.4.4", None).await.status(),
            200
        );
    }
}
