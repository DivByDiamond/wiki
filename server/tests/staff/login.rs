use crate::support::{FakeGithub, staff_app, text};
use axum::{Router, body::Body, http::Request, response::Response};
use oc2r_wiki::{
    auth::github::GithubUser, config::Config, http::state::AppState, router, store::db::open_memory,
};
use tower::ServiceExt;

pub fn gh(id: i64, login: &str) -> Result<GithubUser, String> {
    Ok(GithubUser {
        id,
        login: login.into(),
    })
}

pub async fn get(app: &Router, uri: &str, cookie: Option<&str>) -> Response {
    let mut req = Request::get(uri);
    if let Some(c) = cookie {
        req = req.header("cookie", c);
    }
    app.clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

fn set_cookies(res: &Response) -> Vec<String> {
    res.headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().to_owned())
        .collect()
}

fn cookie_value(res: &Response, name: &str) -> Option<String> {
    set_cookies(res).iter().find_map(|c| {
        c.strip_prefix(&format!("{name}="))?
            .split(';')
            .next()
            .map(str::to_owned)
    })
}

/// Runs start + callback with a valid state; returns the callback response.
pub async fn sign_in(app: &Router) -> Response {
    let start = get(app, "/auth/github", None).await;
    let state = cookie_value(&start, "oauth_state").unwrap();
    get(
        app,
        &format!("/auth/github/callback?code=abc&state={state}"),
        Some(&format!("oauth_state={state}")),
    )
    .await
}

#[tokio::test]
async fn start_redirects_with_state_matching_the_cookie() {
    let (app, _) = staff_app(gh(1, "a"), &[]);
    let res = get(&app, "/auth/github", None).await;
    assert_eq!(res.status(), 303);
    let loc = res.headers()["location"].to_str().unwrap().to_owned();
    let state = cookie_value(&res, "oauth_state").unwrap();
    assert_eq!(state.len(), 48);
    assert!(loc.contains("client_id=cid") && loc.ends_with(&format!("state={state}")));
    let cookie = &set_cookies(&res)[0];
    for flag in ["HttpOnly", "SameSite=Lax", "Path=/", "Max-Age=600"] {
        assert!(cookie.contains(flag), "{cookie}");
    }
    assert!(!cookie.contains("Secure"));
}

#[tokio::test]
async fn https_public_url_marks_cookies_secure() {
    let (app, _) = staff_app(gh(1, "a"), &[("PUBLIC_URL", "https://w.example")]);
    assert!(set_cookies(&get(&app, "/auth/github", None).await)[0].contains("; Secure"));
    assert!(
        set_cookies(&sign_in(&app).await)
            .iter()
            .all(|c| c.contains("; Secure"))
    );
}

#[tokio::test]
async fn callback_with_wrong_or_missing_state_is_400() {
    let (app, st) = staff_app(gh(1, "a"), &[]);
    for (uri, cookie) in [
        (
            "/auth/github/callback?code=c&state=bad",
            Some("oauth_state=good"),
        ),
        ("/auth/github/callback?code=c&state=good", None),
        ("/auth/github/callback?code=c", Some("oauth_state=good")),
        ("/auth/github/callback?code=c&state=", Some("oauth_state=")),
    ] {
        let res = get(&app, uri, cookie).await;
        assert_eq!(res.status(), 400, "{uri}");
        assert!(text(res).await.contains("Sign in again"));
    }
    assert!(st.access.user(1).unwrap().is_none(), "no user is created");
}

#[tokio::test]
async fn callback_success_sets_session_cookie_and_creates_user() {
    let (app, st) = staff_app(gh(10, "alice"), &[]);
    let res = sign_in(&app).await;
    assert_eq!(res.status(), 303);
    assert_eq!(res.headers()["location"], "/");
    let sid = set_cookies(&res)
        .into_iter()
        .find(|c| c.starts_with("sid="))
        .unwrap();
    for flag in ["HttpOnly", "SameSite=Lax", "Path=/", "Max-Age=2592000"] {
        assert!(sid.contains(flag), "{sid}");
    }
    assert_eq!(cookie_value(&res, "oauth_state").as_deref(), Some(""));
    assert!(
        set_cookies(&res)
            .iter()
            .any(|c| c.starts_with("oauth_state=;") && c.contains("Max-Age=0"))
    );
    let user = st.access.upsert_user(10, "alice").unwrap();
    assert!(!user.admin);
    assert_eq!(user.login, "alice");
}

#[tokio::test]
async fn admin_is_promoted_by_id_only() {
    let env = [("ADMIN_GITHUB_IDS", "42")];
    // Listed id becomes admin even though the login changed.
    let (app, st) = staff_app(gh(42, "renamed"), &env);
    st.access.upsert_user(42, "oldname").unwrap();
    sign_in(&app).await;
    assert!(st.access.upsert_user(42, "renamed").unwrap().admin);
    // Someone who now owns the old login but has another id stays a normal user.
    let (app, st) = staff_app(gh(99, "oldname"), &env);
    sign_in(&app).await;
    assert!(!st.access.upsert_user(99, "oldname").unwrap().admin);
}

#[tokio::test]
async fn sign_in_never_demotes_an_admin() {
    let (app, st) = staff_app(gh(5, "boss"), &[]);
    let u = st.access.upsert_user(5, "boss").unwrap();
    st.access.set_admin(u.id, true).unwrap();
    sign_in(&app).await;
    assert!(st.access.user(u.id).unwrap().unwrap().admin);
}

#[tokio::test]
async fn blocked_user_gets_no_session() {
    let (app, st) = staff_app(gh(6, "bad"), &[]);
    let u = st.access.upsert_user(6, "bad").unwrap();
    st.access.set_blocked(u.id, true).unwrap();
    let res = sign_in(&app).await;
    assert_eq!(res.status(), 403);
    assert!(!set_cookies(&res).iter().any(|c| c.starts_with("sid=")));
}

#[tokio::test]
async fn github_failure_is_502_html_with_retry_link() {
    let (app, _) = staff_app(Err("boom secret-detail".into()), &[]);
    let res = sign_in(&app).await;
    assert_eq!(res.status(), 502);
    let body = text(res).await;
    assert!(body.contains("Sign in again") && body.contains("/auth/github"));
    assert!(
        !body.contains("secret-detail"),
        "upstream error text is not shown"
    );
}

#[tokio::test]
async fn unconfigured_sign_in_is_503() {
    let st = AppState::new(open_memory().unwrap())
        .with_github(std::sync::Arc::new(FakeGithub(gh(1, "a"))));
    let app = router(st);
    for uri in ["/auth/github", "/auth/github/callback?code=c&state=s"] {
        let res = get(&app, uri, None).await;
        assert_eq!(res.status(), 503, "{uri}");
        assert!(text(res).await.contains("not configured"));
    }
    assert!(!Config::default().github_configured());
}

#[tokio::test]
async fn login_is_rate_limited_per_client_ip() {
    let (app, _) = staff_app(gh(1, "a"), &[]);
    let mut last = axum::http::StatusCode::OK;
    for _ in 0..61 {
        let req = Request::get("/auth/github").header("x-real-ip", "5.5.5.5");
        last = app
            .clone()
            .oneshot(req.body(Body::empty()).unwrap())
            .await
            .unwrap()
            .status();
    }
    assert_eq!(last.as_u16(), 429);
}

async fn sign_in_with_next(app: &Router, next: &str) -> Response {
    let start = get(app, &format!("/auth/github?next={next}"), None).await;
    let state = cookie_value(&start, "oauth_state").unwrap();
    let mut cookie = format!("oauth_state={state}");
    if let Some(n) = cookie_value(&start, "oauth_next") {
        cookie.push_str(&format!("; oauth_next={n}"));
    }
    get(
        app,
        &format!("/auth/github/callback?code=abc&state={state}"),
        Some(&cookie),
    )
    .await
}

#[tokio::test]
async fn callback_redirects_to_a_valid_next() {
    let (app, _) = staff_app(gh(1, "a"), &[]);
    let start = get(&app, "/auth/github?next=/ru/start", None).await;
    let c = set_cookies(&start)
        .into_iter()
        .find(|c| c.starts_with("oauth_next="))
        .unwrap();
    for flag in ["HttpOnly", "SameSite=Lax", "Max-Age=600"] {
        assert!(c.contains(flag), "{c}");
    }
    let res = sign_in_with_next(&app, "/ru/start").await;
    assert_eq!(res.status(), 303);
    assert_eq!(res.headers()["location"], "/ru/start");
    assert!(
        set_cookies(&res)
            .iter()
            .any(|c| c.starts_with("oauth_next=;") && c.contains("Max-Age=0"))
    );
}

#[tokio::test]
async fn callback_ignores_an_invalid_next() {
    let (app, _) = staff_app(gh(1, "a"), &[]);
    for bad in ["//evil.com", "https://evil.com", "/%5Cevil"] {
        let start = get(&app, &format!("/auth/github?next={bad}"), None).await;
        assert!(cookie_value(&start, "oauth_next").is_none(), "{bad}");
    }
    // A forged cookie is revalidated in the callback.
    let start = get(&app, "/auth/github", None).await;
    let state = cookie_value(&start, "oauth_state").unwrap();
    let res = get(
        &app,
        &format!("/auth/github/callback?code=abc&state={state}"),
        Some(&format!("oauth_state={state}; oauth_next=//evil.com")),
    )
    .await;
    assert_eq!(res.headers()["location"], "/");
}
