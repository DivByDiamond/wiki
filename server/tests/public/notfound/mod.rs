use crate::support::{app_with_state, save_page, text};
use axum::{Router, body::Body, http::Request};
use oc2r_wiki::domain::page::Lang;
use tower::ServiceExt;

async fn page(app: &Router, uri: &str) -> (u16, String) {
    let res = app
        .clone()
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    (res.status().as_u16(), text(res).await)
}

#[tokio::test]
async fn status_is_404_and_heading_is_localized() {
    let (app, ..) = app_with_state();
    for (lang, heading) in [
        ("en", "Page not found"),
        ("ru", "Страница не найдена"),
        ("zh", "页面未找到"),
    ] {
        let (status, body) = page(&app, &format!("/{lang}/missing")).await;
        assert_eq!(status, 404, "{lang}");
        assert!(body.contains(heading), "{lang}");
        assert!(
            body.contains(&format!(r#"action="/{lang}/search""#)),
            "{lang}"
        );
        assert!(body.contains(&format!(r#"href="/{lang}/start""#)), "{lang}");
    }
}

#[tokio::test]
async fn requested_path_is_shown_and_escaped() {
    let (app, ..) = app_with_state();
    let (status, body) = page(&app, "/en/%3Cscript%3Ealert(1)").await;
    assert_eq!(status, 404);
    assert!(!body.contains("<script>alert"));
    let (_, plain) = page(&app, "/en/missing/path").await;
    assert!(plain.contains("<code class=\"nf__path\">/en/missing/path</code>"));
}

#[tokio::test]
async fn similar_pages_are_listed_only_when_close() {
    let (app, st, ..) = app_with_state();
    save_page(&st, "blocks/computer", Lang::En, "x", "s");
    save_page(&st, "items/chip", Lang::En, "x", "s");
    save_page(&st, "blocks/other", Lang::Ru, "x", "s");
    let (status, body) = page(&app, "/en/blocks/comp").await;
    assert_eq!(status, 404);
    assert!(body.contains("Similar pages"));
    let list = body.split("nf__similar").nth(1).unwrap();
    assert!(list.contains(r#"href="/en/blocks/computer""#));
    assert!(!list.contains("items/chip") && !list.contains("blocks/other"));
    let (_, none) = page(&app, "/en/zzz/qqq").await;
    assert!(!none.contains("Similar pages"));
}
