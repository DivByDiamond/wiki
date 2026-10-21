use crate::support::{app_with_state, save_page, text};
use axum::{Router, body::Body, http::Request, response::Response};
use oc2r_wiki::domain::page::{Lang, Slug};
use tower::ServiceExt;

async fn get(app: &Router, uri: &str) -> Response {
    app.clone()
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn page(app: &Router, uri: &str) -> (u16, String) {
    let res = get(app, uri).await;
    (res.status().as_u16(), text(res).await)
}

fn rev_ids(st: &oc2r_wiki::http::state::AppState, slug: &str) -> Vec<i64> {
    let p = st
        .pages
        .find(&Slug::parse(slug).unwrap(), Lang::En)
        .unwrap()
        .unwrap();
    let mut ids: Vec<i64> = st
        .pages
        .history(p.id)
        .unwrap()
        .iter()
        .map(|r| r.id)
        .collect();
    ids.sort();
    ids
}

#[tokio::test]
async fn article_renders_markdown_in_layout() {
    let (app, st, ..) = app_with_state();
    save_page(&st, "a", Lang::En, "# Hello\n\n## Part", "init");
    let (status, body) = page(&app, "/en/a").await;
    assert_eq!(status, 200);
    assert!(body.contains("<h1>Hello</h1>"));
    assert!(body.contains(r#"<html lang="en""#));
    assert!(body.contains("oc2r-wiki-settings"));
    assert!(body.contains("/static/app.css") && body.contains("/static/app.js"));
    assert!(body.contains(r#"id="search-form""#) && body.contains(r#"id="search-input""#));
    assert!(body.contains(r#"id="settings-toggle""#) && body.contains(r#"id="search-results""#));
    assert!(body.contains(r#"aria-current="page""#));
    assert!(body.contains("/ru/a") && body.contains("/zh/a"));
    assert!(body.contains("?history"));
}

#[tokio::test]
async fn old_revision_shows_banner_and_old_text() {
    let (app, st, ..) = app_with_state();
    save_page(&st, "a", Lang::En, "one", "s1");
    save_page(&st, "a", Lang::En, "two", "s2");
    let first = rev_ids(&st, "a")[0];
    let (status, body) = page(&app, &format!("/en/a?rev={first}")).await;
    assert_eq!(status, 200);
    assert!(body.contains("one") && body.contains("Старая версия"));
    assert!(body.contains("banner--warn"));
    assert!(!body.contains("two"));
}

#[tokio::test]
async fn foreign_or_missing_revision_is_404() {
    let (app, st, ..) = app_with_state();
    save_page(&st, "a", Lang::En, "one", "s");
    save_page(&st, "b", Lang::En, "other", "s");
    let b = rev_ids(&st, "b")[0];
    assert_eq!(page(&app, &format!("/en/a?rev={b}")).await.0, 404);
    assert_eq!(page(&app, "/en/a?rev=9999").await.0, 404);
}

#[tokio::test]
async fn missing_translation_falls_back_to_english_with_notice() {
    let (app, st, ..) = app_with_state();
    save_page(&st, "a", Lang::En, "english text", "s");
    let (status, body) = page(&app, "/ru/a").await;
    assert_eq!(status, 200);
    assert!(body.contains("english text") && body.contains("Перевода пока нет"));
    assert!(!page(&app, "/en/a").await.1.contains("Перевода пока нет"));
}

#[tokio::test]
async fn unknown_page_is_404_with_search_box() {
    let (app, ..) = app_with_state();
    let (status, body) = page(&app, "/en/nope").await;
    assert_eq!(status, 404);
    assert!(body.contains(r#"action="/en/search""#));
}

#[tokio::test]
async fn deleted_page_is_hidden() {
    let (app, st, ..) = app_with_state();
    save_page(&st, "a", Lang::En, "x", "s");
    let p = st
        .pages
        .find(&Slug::parse("a").unwrap(), Lang::En)
        .unwrap()
        .unwrap();
    st.pages.set_deleted(p.id, true).unwrap();
    assert_eq!(page(&app, "/en/a").await.0, 404);
}

#[tokio::test]
async fn history_lists_revisions_and_diff_marks_changes() {
    let (app, st, ..) = app_with_state();
    save_page(
        &st,
        "a",
        Lang::En,
        "keep\nold line",
        "first <img onerror=x>",
    );
    save_page(&st, "a", Lang::En, "keep\nnew line", "second");
    let ids = rev_ids(&st, "a");
    let (status, body) = page(&app, "/en/a?history").await;
    assert_eq!(status, 200);
    assert!(
        body.contains(&format!("?rev={}", ids[0])) && body.contains(&format!("?rev={}", ids[1]))
    );
    assert!(body.contains("tester") && body.contains("second"));
    assert!(body.contains("first &lt;img onerror=x&gt;") || body.contains("first &#60;img"));
    assert!(!body.contains("<img onerror"));
    let (status, body) = page(&app, &format!("/en/a?diff={}..{}", ids[0], ids[1])).await;
    assert_eq!(status, 200);
    assert!(body.contains("class=\"diff\""));
    assert!(body.contains("<del>old line</del>") && body.contains("<ins>new line</ins>"));
    assert!(body.contains("<span>keep</span>"));
}

#[tokio::test]
async fn bad_diff_is_422_or_404() {
    let (app, st, ..) = app_with_state();
    save_page(&st, "a", Lang::En, "x", "s");
    let id = rev_ids(&st, "a")[0];
    assert_eq!(page(&app, "/en/a?diff=oops").await.0, 422);
    assert_eq!(page(&app, &format!("/en/a?diff={id}..9999")).await.0, 404);
}

#[tokio::test]
async fn script_in_body_is_never_emitted() {
    let (app, st, ..) = app_with_state();
    save_page(&st, "a", Lang::En, "hi <script>alert(1)</script>", "s");
    let (status, body) = page(&app, "/en/a").await;
    assert_eq!(status, 200);
    assert!(!body.contains("<script>alert"));
}

#[tokio::test]
async fn search_full_page_and_fragment() {
    let (app, st, ..) = app_with_state();
    save_page(&st, "a", Lang::En, "the quick zebra jumps", "s");
    let (status, body) = page(&app, "/en/search?q=zebra").await;
    assert_eq!(status, 200);
    assert!(body.contains(r#"href="/en/a""#) && body.contains(r#"id="search-form""#));
    let (status, frag) = page(&app, "/en/search?q=zebra&fragment=1").await;
    assert_eq!(status, 200);
    assert!(frag.starts_with(r#"<ul class="search-results">"#));
    assert!(!frag.contains("<html"));
    let (status, frag) = page(&app, "/en/search?q=&fragment=1").await;
    assert_eq!(status, 200);
    assert!(frag.contains(r#"<ul class="search-results">"#) && !frag.contains("<li"));
    assert!(
        !page(&app, "/ru/search?q=zebra&fragment=1")
            .await
            .1
            .contains("<li")
    );
}

#[tokio::test]
async fn root_redirects_and_other_routes_survive() {
    let (app, ..) = app_with_state();
    let res = get(&app, "/").await;
    assert_eq!(res.status(), 303);
    assert_eq!(res.headers()["location"], "/en/start");
    assert_eq!(page(&app, "/health").await, (200, "ok".into()));
    assert_eq!(get(&app, "/api/pages").await.status(), 200);
}
