#[path = "support/mod.rs"]
mod support;

use axum::{Router, body::Body, http::Request, response::Response};
use oc2r_wiki::{
    auth::github::GithubUser,
    domain::{
        page::{Lang, Slug},
        proposal::Status,
    },
    http::state::AppState,
    router,
    store::db::open_memory,
};
use std::sync::Arc;
use support::{FakeGithub, make_key, text};
use tower::ServiceExt;

const ORIGIN: &str = "https://w.example";

async fn call(app: &Router, req: Request<Body>) -> Response {
    app.clone().oneshot(req).await.unwrap()
}

async fn get(app: &Router, uri: &str, cookie: Option<&str>) -> Response {
    let mut req = Request::get(uri);
    if let Some(c) = cookie {
        req = req.header("cookie", c);
    }
    call(app, req.body(Body::empty()).unwrap()).await
}

async fn put_api(app: &Router, key: &str, body: &str) -> Response {
    let req = Request::put("/api/pages/a?lang=en&summary=sync")
        .header("authorization", format!("Bearer {key}"))
        .body(Body::from(body.to_owned()))
        .unwrap();
    call(app, req).await
}

async fn post_form(app: &Router, cookie: &str, uri: &str, form: &str) -> Response {
    let req = Request::post(uri)
        .header("cookie", cookie)
        .header("origin", ORIGIN)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form.to_owned()))
        .unwrap();
    call(app, req).await
}

fn cookie_of(res: &Response, name: &str) -> String {
    res.headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().unwrap().split(';').next())
        .find(|c| c.starts_with(&format!("{name}=")))
        .unwrap()
        .to_owned()
}

/// Full GitHub sign-in against the fake client; returns the session cookie.
async fn sign_in(app: &Router) -> String {
    let start = get(app, "/auth/github", None).await;
    let state = cookie_of(&start, "oauth_state");
    let uri = format!(
        "/auth/github/callback?code=abc&state={}",
        state.strip_prefix("oauth_state=").unwrap()
    );
    let res = get(app, &uri, Some(&state)).await;
    cookie_of(&res, "sid")
}

fn app_as(st: &AppState, id: i64, login: &str) -> Router {
    let gh = GithubUser {
        id,
        login: login.into(),
    };
    router(st.clone().with_github(Arc::new(FakeGithub(Ok(gh)))))
}

fn history(st: &AppState) -> Vec<(i64, String, String)> {
    let page = st
        .pages
        .find(&Slug::parse("a").unwrap(), Lang::En)
        .unwrap()
        .unwrap();
    st.pages
        .history(page.id)
        .unwrap()
        .into_iter()
        .map(|r| (r.id, r.body, r.author.label()))
        .collect()
}

#[tokio::test]
async fn propose_accept_history_rollback() {
    let cfg = oc2r_wiki::config::Config::from_lookup(|k| match k {
        "GITHUB_CLIENT_ID" => Some("cid".into()),
        "GITHUB_CLIENT_SECRET" => Some("sec".into()),
        "PUBLIC_URL" => Some(ORIGIN.into()),
        _ => None,
    })
    .unwrap();
    let st = AppState::new(open_memory().unwrap()).with_config(cfg);
    let (key, _) = make_key(&st, "ci", true, true, false);
    let anon = router(st.clone());

    // 1. A write key uploads v1, then v2.
    assert_eq!(put_api(&anon, &key, "v1").await.status(), 200);
    assert_eq!(put_api(&anon, &key, "v2").await.status(), 200);
    assert_eq!(history(&st).len(), 2);
    let first = history(&st)[1].0;
    let v2 = history(&st)[0].0;

    // 2. A signed-in reader proposes a change based on v2.
    let reader_app = app_as(&st, 2, "writer");
    let reader = sign_in(&reader_app).await;
    let form = format!("body=proposed&summary=fix&base_rev={v2}");
    let res = post_form(&reader_app, &reader, "/propose/en/a", &form).await;
    assert!(res.status().is_redirection(), "{}", res.status());
    assert_eq!(history(&st).len(), 2, "a proposal must not publish");

    // 3. The admin accepts it: new text, author is the proposer.
    let boss_app = app_as(&st, 1, "boss");
    let boss = sign_in(&boss_app).await;
    let admin = st.access.upsert_user(1, "boss").unwrap().id;
    st.access.set_admin(admin, true).unwrap();
    let pid = st
        .proposals
        .list(Status::Open)
        .unwrap()
        .first()
        .expect("one open proposal")
        .id;
    let res = post_form(
        &boss_app,
        &boss,
        &format!("/admin/proposals/{pid}/accept"),
        "note=ok",
    )
    .await;
    assert!(res.status().is_redirection(), "{}", res.status());
    let page = get(&anon, "/en/a", None).await;
    assert_eq!(page.status(), 200);
    assert!(text(page).await.contains("proposed"));
    assert_eq!(history(&st)[0].2, "writer");

    // 4. History has 3 revisions; ?rev= shows v1 with the old-version banner.
    assert_eq!(history(&st).len(), 3);
    let res = get(&anon, "/en/a?history", None).await;
    assert_eq!(res.status(), 200);
    let res = get(&anon, &format!("/en/a?rev={first}"), None).await;
    assert_eq!(res.status(), 200);
    let html = text(res).await;
    assert!(
        html.contains("v1") && html.contains("banner--warn"),
        "{html}"
    );

    // 5. Rollback to the first revision adds a 4th entry with v1 as current text.
    let res = post_form(
        &boss_app,
        &boss,
        "/admin/rollback/en/a",
        &format!("rev={first}"),
    )
    .await;
    assert!(res.status().is_redirection(), "{}", res.status());
    let h = history(&st);
    assert_eq!(h.len(), 4);
    assert_eq!(h[0].1, "v1");
    let html = text(get(&anon, "/en/a", None).await).await;
    assert!(html.contains("v1") && !html.contains("proposed"));
}
