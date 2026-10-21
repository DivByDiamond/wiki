use crate::login::{get, gh, sign_in};
use crate::support::staff_app;
use axum::{body::Body, http::Request, response::Response};
use tower::ServiceExt;

async fn put(app: &axum::Router, cookie: &str, origin: Option<&str>) -> Response {
    let mut req = Request::put("/api/pages/sess-test?lang=en").header("cookie", cookie);
    if let Some(o) = origin {
        req = req.header("origin", o);
    }
    app.clone()
        .oneshot(req.body(Body::from("text")).unwrap())
        .await
        .unwrap()
}

fn sid(res: &Response) -> String {
    let c = res
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().to_owned())
        .find(|c| c.starts_with("sid="))
        .unwrap();
    c.split(';').next().unwrap().to_owned()
}

#[tokio::test]
async fn demoting_an_admin_takes_effect_on_the_next_request() {
    let (app, st) = staff_app(gh(42, "boss"), &[("ADMIN_GITHUB_IDS", "42")]);
    let cookie = sid(&sign_in(&app).await);
    assert_eq!(put(&app, &cookie, None).await.status(), 200);
    let id = st.access.upsert_user(42, "boss").unwrap().id;
    st.access.set_admin(id, false).unwrap();
    assert_eq!(put(&app, &cookie, None).await.status(), 403);
}

#[tokio::test]
async fn blocking_a_user_takes_effect_on_the_next_request() {
    let (app, st) = staff_app(gh(42, "boss"), &[("ADMIN_GITHUB_IDS", "42")]);
    let cookie = sid(&sign_in(&app).await);
    assert_eq!(put(&app, &cookie, None).await.status(), 200);
    let id = st.access.upsert_user(42, "boss").unwrap().id;
    st.access.set_blocked(id, true).unwrap();
    assert_eq!(put(&app, &cookie, None).await.status(), 403);
}

#[tokio::test]
async fn logout_destroys_the_session() {
    let (app, _) = staff_app(gh(42, "boss"), &[("ADMIN_GITHUB_IDS", "42")]);
    let cookie = sid(&sign_in(&app).await);
    assert_eq!(put(&app, &cookie, None).await.status(), 200);
    let req = Request::post("/auth/logout").header("cookie", &cookie);
    let res = app
        .clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), 303);
    assert_eq!(res.headers()["location"], "/");
    let cleared = res.headers()["set-cookie"].to_str().unwrap();
    assert!(cleared.starts_with("sid=;") && cleared.contains("Max-Age=0"));
    assert_eq!(
        put(&app, &cookie, None).await.status(),
        401,
        "old cookie is dead"
    );
}

#[tokio::test]
async fn logout_checks_the_origin_header() {
    let (app, _) = staff_app(gh(42, "boss"), &[("PUBLIC_URL", "https://w.example")]);
    let cookie = sid(&sign_in(&app).await);
    let post = |origin: Option<&str>| {
        let mut req = Request::post("/auth/logout").header("cookie", &cookie);
        if let Some(o) = origin {
            req = req.header("origin", o);
        }
        app.clone().oneshot(req.body(Body::empty()).unwrap())
    };
    assert_eq!(
        post(Some("https://evil.example")).await.unwrap().status(),
        403
    );
    assert_eq!(get(&app, "/health", Some(&cookie)).await.status(), 200);
    assert_eq!(post(Some("https://w.example")).await.unwrap().status(), 303);
}
