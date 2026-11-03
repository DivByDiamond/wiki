//! Recent changes page and Atom feed: public, localized, escaped, cacheable.
use crate::support::{app_with_state, save_page, text};
use axum::{Router, body::Body, http::Request, response::Response};
use oc2r_wiki::domain::page::{Lang, Slug};
use tower::ServiceExt;

async fn get(app: &Router, uri: &str, extra: Option<(&str, &str)>) -> Response {
    let mut req = Request::get(uri);
    if let Some((k, v)) = extra {
        req = req.header(k, v);
    }
    app.clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn recent_page_lists_revisions_newest_first() {
    let (app, st, ..) = app_with_state();
    save_page(&st, "a", Lang::En, "# Alpha\n\nx", "first change");
    save_page(&st, "b", Lang::Ru, "# Beta\n\ny", "second change");
    save_page(&st, "a", Lang::En, "# Alpha\n\nz", "third change");
    let res = get(&app, "/en/recent", None).await;
    assert_eq!(res.status(), 200);
    let body = text(res).await;
    for want in [
        "Alpha",
        "Beta",
        "tester",
        "first change",
        "href=\"/en/a?rev=",
        "href=\"/ru/b?rev=",
    ] {
        assert!(body.contains(want), "{want} in {body}");
    }
    let (third, second, first) = (
        body.find("third change").unwrap(),
        body.find("second change").unwrap(),
        body.find("first change").unwrap(),
    );
    assert!(third < second && second < first);
}

#[tokio::test]
async fn recent_page_is_localized() {
    let (app, ..) = app_with_state();
    let mut headings = Vec::new();
    for lang in ["en", "ru", "zh"] {
        let body = text(get(&app, &format!("/{lang}/recent"), None).await).await;
        assert!(
            body.contains(&format!("<html lang=\"{lang}\""))
                || body.contains(&format!("lang=\"{lang}\""))
        );
        headings.push(
            body.split("<h1")
                .nth(1)
                .unwrap()
                .split("</h1>")
                .next()
                .unwrap()
                .to_owned(),
        );
    }
    assert!(
        headings[0] != headings[1] && headings[1] != headings[2],
        "{headings:?}"
    );
    assert_eq!(get(&app, "/xx/recent", None).await.status(), 404);
}

#[tokio::test]
async fn recent_hides_deleted_pages_and_escapes_text() {
    let (app, st, ..) = app_with_state();
    save_page(&st, "gone", Lang::En, "# Gone", "secret-removed");
    save_page(
        &st,
        "a",
        Lang::En,
        "# <b>T</b>",
        "<script>alert(1)</script> & \"q\"",
    );
    let id = st
        .pages
        .find(&Slug::parse("gone").unwrap(), Lang::En)
        .unwrap()
        .unwrap()
        .id;
    st.pages.set_deleted(id, true).unwrap();
    let body = text(get(&app, "/en/recent", None).await).await;
    assert!(!body.contains("secret-removed"));
    assert!(!body.contains("<script>alert(1)</script>"));
    assert!(
        body.contains("&lt;script&gt;alert(1)&lt;/script&gt;") || body.contains("&#60;script&#62;")
    );
}

#[tokio::test]
async fn atom_feed_is_escaped_well_formed_and_cacheable() {
    let (app, st, ..) = app_with_state();
    save_page(
        &st,
        "a",
        Lang::En,
        "# T & x",
        "x </summary><script>1</script> & \u{1}ok",
    );
    let res = get(&app, "/feed.atom", None).await;
    assert_eq!(res.status(), 200);
    let h = res.headers().clone();
    assert!(
        h["content-type"]
            .to_str()
            .unwrap()
            .starts_with("application/atom+xml")
    );
    assert!(h["cache-control"].to_str().unwrap().contains("max-age"));
    let etag = h["etag"].to_str().unwrap().to_owned();
    let body = text(res).await;
    assert!(body.starts_with("<?xml"));
    assert!(body.contains("<feed xmlns=\"http://www.w3.org/2005/Atom\""));
    assert!(body.contains("<entry>") && body.contains("/en/a?rev="));
    assert!(!body.contains("<script>") && !body.contains('\u{1}'));
    assert!(
        body.contains("&lt;script&gt;1&lt;/script&gt; &amp; ok"),
        "{body}"
    );
    assert!(body.contains("T &amp; x"));
    let again = get(&app, "/feed.atom", Some(("if-none-match", &etag))).await;
    assert_eq!(again.status(), 304);
    save_page(&st, "a", Lang::En, "# T2", "new");
    let changed = get(&app, "/feed.atom", Some(("if-none-match", &etag))).await;
    assert_eq!(changed.status(), 200);
}

#[tokio::test]
async fn empty_feed_is_valid() {
    let (app, ..) = app_with_state();
    let res = get(&app, "/feed.atom", None).await;
    assert_eq!(res.status(), 200);
    let body = text(res).await;
    assert!(body.contains("<updated>") && !body.contains("<entry>"));
}
