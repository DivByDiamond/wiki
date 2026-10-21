#![allow(dead_code)] // each test binary uses a different subset of helpers

use axum::{Router, body::Body, http::Response};
use http_body_util::BodyExt;
use oc2r_wiki::domain::{
    page::{Lang, Slug},
    revision::{Author, NewRevision},
};
use oc2r_wiki::{auth::tokens::api_key, http::state::AppState, router, store::db::open_memory};
use serde_json::Value;

/// Creates a key with the given scopes; returns (plain key, key id).
pub fn make_key(st: &AppState, name: &str, read: bool, write: bool, export: bool) -> (String, i64) {
    let (key, hash) = api_key::generate();
    let id = st
        .access
        .create_key(name, &hash, read, write, export)
        .unwrap();
    (key, id)
}

/// App plus its state, a write key ("writer") and a read-only key ("reader").
pub fn app_with_state() -> (Router, AppState, String, String) {
    let st = AppState::new(open_memory().unwrap());
    let (write, _) = make_key(&st, "writer", true, true, false);
    let (read, _) = make_key(&st, "reader", true, false, false);
    (router(st.clone()), st, write, read)
}

/// App, write key and read-only key.
pub fn app() -> (Router, String, String) {
    let (app, _, write, read) = app_with_state();
    (app, write, read)
}

pub async fn json(res: Response<Body>) -> Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

/// Saves a revision of `slug` in `lang` through the page repository.
pub fn save_page(st: &AppState, slug: &str, lang: Lang, body: &str, summary: &str) {
    let rev = NewRevision {
        body: body.into(),
        summary: summary.into(),
        author: Author::User("tester".into()),
    };
    st.pages
        .save(&Slug::parse(slug).unwrap(), lang, rev)
        .unwrap();
}

pub async fn text(res: Response<Body>) -> String {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

/// Fake GitHub: `exchange` returns the canned result, so no network is touched.
pub struct FakeGithub(pub Result<oc2r_wiki::auth::github::GithubUser, String>);

impl oc2r_wiki::auth::github::GithubClient for FakeGithub {
    fn authorize_url(&self, state: &str) -> String {
        format!("https://github.example/authorize?client_id=cid&state={state}")
    }

    fn exchange<'a>(&'a self, _code: &'a str) -> oc2r_wiki::auth::github::ExchangeFuture<'a> {
        let r = self.0.clone();
        Box::pin(async move { r })
    }
}

/// App with sign-in configured (client id "cid", secret "sec") and the given extra env.
pub fn staff_app(
    github: Result<oc2r_wiki::auth::github::GithubUser, String>,
    env: &[(&str, &str)],
) -> (Router, AppState) {
    let cfg = oc2r_wiki::config::Config::from_lookup(|k| match k {
        "GITHUB_CLIENT_ID" => Some("cid".into()),
        "GITHUB_CLIENT_SECRET" => Some("sec".into()),
        _ => env
            .iter()
            .find(|(n, _)| *n == k)
            .map(|(_, v)| (*v).to_owned()),
    })
    .unwrap();
    let st = AppState::new(open_memory().unwrap())
        .with_config(cfg)
        .with_github(std::sync::Arc::new(FakeGithub(github)));
    (router(st.clone()), st)
}
