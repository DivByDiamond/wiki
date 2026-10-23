use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use oc2r_wiki::{
    http::state::AppState,
    render::document,
    router,
    store::{catalog, db::open_memory},
};
use pulldown_cmark::{Event, Parser, Tag};
use tower::ServiceExt;

#[tokio::test]
async fn all_bundled_articles_and_local_links_are_readable() {
    let db = open_memory().unwrap();
    let bundle = catalog::bundled().unwrap();
    catalog::import(&db, &bundle, true).unwrap();
    let app = router(AppState::new(db));
    let mut targets = std::collections::BTreeSet::new();
    for page in &bundle.pages {
        targets.insert(format!("/{}/{}", page.lang, page.slug));
        for event in Parser::new(&page.body) {
            let dest = match event {
                Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) => dest_url,
                _ => continue,
            };
            if dest.starts_with("https://") || dest.starts_with("http://") {
                continue;
            }
            assert!(
                dest.starts_with('/') || dest.starts_with('#'),
                "unconverted URL in {}: {}",
                page.slug,
                dest
            );
            if dest.starts_with('/') {
                targets.insert(dest.split('#').next().unwrap().to_owned());
            }
        }
    }
    for target in targets {
        let response = app
            .clone()
            .oneshot(Request::get(&target).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), 200, "{target}");
    }
}

#[tokio::test]
async fn reader_has_titles_outline_language_fallback_and_ordered_navigation() {
    let db = open_memory().unwrap();
    catalog::import(&db, &catalog::bundled().unwrap(), true).unwrap();
    let app = router(AppState::new(db));
    let response = app
        .clone()
        .oneshot(
            Request::get("/ru/systems/overview")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(body.contains("<title>Операционные системы - OC2R Wiki</title>"));
    assert!(body.contains("href=\"#section-linux\""));
    assert!(body.contains("id=\"section-linux\""));
    assert!(body.contains("rel=\"prev\"") && body.contains("rel=\"next\""));
    assert!(
        body.find(">Системы</summary>").unwrap() < body.find(">Скрипты и API</summary>").unwrap()
    );
    assert!(
        body.contains("/ru/systems/buildroot"),
        "English-only pages are discoverable"
    );
    let response = app
        .oneshot(
            Request::get("/ru/systems/buildroot")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body = String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(body.contains("Перевода пока нет") && body.contains("lang=\"en\""));
}

#[test]
fn all_bundled_outlines_have_unique_resolvable_anchors() {
    for page in catalog::bundled().unwrap().pages {
        let doc = document::render(&page.body);
        let mut ids = std::collections::HashSet::new();
        for heading in doc.headings {
            assert!(ids.insert(heading.id.clone()), "{}", page.slug);
            assert!(
                doc.html.contains(&format!("id=\"{}\"", heading.id)),
                "{}",
                page.slug
            );
        }
    }
}
