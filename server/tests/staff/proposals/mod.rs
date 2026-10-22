use crate::login::{gh, sign_in};
use crate::support::{save_page, staff_app, text};
use axum::{Router, body::Body, http::Request, response::Response};
use oc2r_wiki::{
    domain::{page::Lang, proposal::Status},
    http::state::AppState,
};
use tower::ServiceExt;

pub(super) fn sid(res: &Response) -> String {
    let c = res
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap())
        .find(|c| c.starts_with("sid="))
        .unwrap();
    c.split(';').next().unwrap().to_owned()
}

/// App with page "guide" and a signed-in normal user; returns (app, state, cookie, user id).
pub(super) async fn setup() -> (Router, AppState, String, i64) {
    let (app, st) = staff_app(gh(10, "reader"), &[("PUBLIC_URL", "https://w.example")]);
    save_page(&st, "guide", Lang::En, "original", "init");
    let cookie = sid(&sign_in(&app).await);
    let uid = st.access.upsert_user(10, "reader").unwrap().id;
    (app, st, cookie, uid)
}

pub(super) fn enc(s: &str) -> String {
    s.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

pub(super) async fn post(
    app: &Router,
    cookie: Option<&str>,
    origin: Option<&str>,
    body: &str,
    rev: &str,
    summary: &str,
) -> Response {
    let form = format!("body={}&summary={}&base_rev={rev}", enc(body), enc(summary));
    post_raw(app, cookie, origin, form).await
}

pub(super) async fn post_raw(
    app: &Router,
    cookie: Option<&str>,
    origin: Option<&str>,
    form: String,
) -> Response {
    let mut req = Request::post("/propose/en/guide")
        .header("content-type", "application/x-www-form-urlencoded");
    if let Some(c) = cookie {
        req = req.header("cookie", c);
    }
    if let Some(o) = origin {
        req = req.header("origin", o);
    }
    app.clone()
        .oneshot(req.body(Body::from(form)).unwrap())
        .await
        .unwrap()
}

pub(super) fn current_rev(st: &AppState) -> i64 {
    st.pages
        .find(
            &oc2r_wiki::domain::page::Slug::parse("guide").unwrap(),
            Lang::En,
        )
        .unwrap()
        .unwrap()
        .current
        .id
}

mod checks;
mod create;
