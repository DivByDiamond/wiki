use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use oc2r_wiki::{
    http::state::AppState,
    router,
    store::{catalog, db::open_memory},
};
use tower::ServiceExt;

async fn get(uri: &str) -> (u16, String, String) {
    let db = open_memory().unwrap();
    catalog::import(&db, &catalog::bundled().unwrap(), true).unwrap();
    let response = router(AppState::new(db))
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status().as_u16();
    let kind = response
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap().to_owned())
        .unwrap_or_default();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, kind, String::from_utf8(bytes.to_vec()).unwrap())
}

#[tokio::test]
async fn markdown_export_returns_page_source() {
    let (status, kind, body) = get("/en/start?md").await;
    assert_eq!(status, 200);
    assert!(kind.starts_with("text/markdown"));
    assert!(body.starts_with('#'), "source, not HTML: {}", &body[..40.min(body.len())]);
    let (_, _, page) = get("/en/start").await;
    assert!(page.contains("/en/start?md"));
}

fn results(page: &str) -> String {
    let start = page.find("<ul class=\"search-results\">").expect("results list");
    let end = page[start..].find("</ul>").unwrap();
    page[start..start + end].to_owned()
}

#[tokio::test]
async fn search_filters_by_language_and_section() {
    let (_, _, all) = get("/en/search?q=computer&lang=all").await;
    assert!(all.contains("<select name=\"section\">"));
    assert!(results(&all).contains("<li"));
    let (_, _, ru) = get("/en/search?q=computer&lang=ru").await;
    assert!(!results(&ru).contains("href=\"/en/"), "{}", results(&ru));
    let (_, _, none) = get("/en/search?q=computer&section=no-such-section").await;
    assert!(!results(&none).contains("<li"));
    let (_, _, bad) = get("/en/search?q=computer&lang=xx").await;
    assert!(results(&bad).contains("href=\"/en/"));
}

#[tokio::test]
async fn lua_code_blocks_keep_language_class() {
    let html = oc2r_wiki::render::document::render("```lua\nprint(1)\n```\n").html;
    assert!(html.contains("language-lua"), "{html}");
}
