//! Shared helpers for the admin tests: sessions without the GitHub round trip.
#![allow(dead_code)]
use crate::support::{save_page, staff_app, text};
use axum::{Router, body::Body, http::Request, response::Response};
use oc2r_wiki::{
    domain::{page::Lang, page::Slug},
    http::state::AppState,
};
use tower::ServiceExt;

pub const ORIGIN: &str = "https://w.example";

pub struct Env {
    pub app: Router,
    pub st: AppState,
    /// `sid=...` cookie of the admin "boss".
    pub admin: String,
    pub admin_id: i64,
}

/// Creates a user and a session; returns (cookie, user id).
pub fn login(
    st: &AppState,
    github_id: i64,
    name: &str,
    admin: bool,
    blocked: bool,
) -> (String, i64) {
    let id = st.access.upsert_user(github_id, name).unwrap().id;
    st.access.set_admin(id, admin).unwrap();
    st.access.set_blocked(id, blocked).unwrap();
    (format!("sid={}", st.sessions.create(id)), id)
}

/// App with admin "boss" (github id 1) and page "a" (en, text "v1").
pub fn env() -> Env {
    let (app, st) = staff_app(Err("unused".into()), &[("PUBLIC_URL", ORIGIN)]);
    let (admin, admin_id) = login(&st, 1, "boss", true, false);
    save_page(&st, "a", Lang::En, "v1", "init");
    Env {
        app,
        st,
        admin,
        admin_id,
    }
}

pub async fn send(
    app: &Router,
    method: &str,
    uri: &str,
    cookie: Option<&str>,
    origin: Option<&str>,
    body: &str,
) -> Response {
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(c) = cookie {
        req = req.header("cookie", c);
    }
    if let Some(o) = origin {
        req = req.header("origin", o);
    }
    let req = req
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body.to_owned()))
        .unwrap();
    app.clone().oneshot(req).await.unwrap()
}

impl Env {
    /// POST as the admin with the site's own origin.
    pub async fn post(&self, uri: &str, body: &str) -> Response {
        send(
            &self.app,
            "POST",
            uri,
            Some(&self.admin),
            Some(ORIGIN),
            body,
        )
        .await
    }

    pub async fn get(&self, uri: &str) -> Response {
        send(&self.app, "GET", uri, Some(&self.admin), None, "").await
    }

    pub async fn get_text(&self, uri: &str) -> String {
        text(self.get(uri).await).await
    }

    /// Open suggestion "proposed" by user "writer" on the current revision of page "a".
    pub fn proposal(&self) -> i64 {
        let writer = self.st.access.upsert_user(2, "writer").unwrap().id;
        let p = self.page("a");
        self.st
            .proposals
            .create(p.id, p.current.id, "proposed", "fix typo", writer)
            .unwrap()
    }

    pub fn page(&self, slug: &str) -> oc2r_wiki::domain::page::Page {
        self.st
            .pages
            .find(&Slug::parse(slug).unwrap(), Lang::En)
            .unwrap()
            .unwrap()
    }
}

/// First `ocw_` followed by 64 hex characters.
pub fn extract_key(html: &str) -> String {
    html.match_indices("ocw_")
        .find_map(|(i, _)| {
            let hex: String = html[i + 4..]
                .chars()
                .take_while(char::is_ascii_hexdigit)
                .collect();
            (hex.len() == 64).then(|| format!("ocw_{hex}"))
        })
        .expect("a full key in the response")
}
