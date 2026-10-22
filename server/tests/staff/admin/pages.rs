use crate::login::{gh, sign_in};
use crate::support::{save_page, staff_app, text};
use axum::{Router, body::Body, http::Request, response::Response};
use oc2r_wiki::{
    domain::page::{Lang, Slug},
    http::state::AppState,
};
use tower::ServiceExt;

/// App with page "a" (one revision, "v1") and a signed-in admin "boss"; returns (app, state, cookie).
async fn setup() -> (Router, AppState, String) {
    let (app, st) = staff_app(gh(1, "boss"), &[("PUBLIC_URL", "https://w.example")]);
    save_page(&st, "a", Lang::En, "v1", "init");
    let res = sign_in(&app).await;
    let cookie = res
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap())
        .find(|c| c.starts_with("sid="))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let id = st.access.upsert_user(1, "boss").unwrap().id;
    st.access.set_admin(id, true).unwrap();
    (app, st, cookie)
}

async fn send(app: &Router, method: &str, uri: &str, cookie: Option<&str>, body: &str) -> Response {
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(c) = cookie {
        req = req.header("cookie", c);
    }
    req = req.header("content-type", "application/x-www-form-urlencoded");
    app.clone()
        .oneshot(req.body(Body::from(body.to_owned())).unwrap())
        .await
        .unwrap()
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
        .map(|r| {
            let body = st.pages.revision(page.id, r.id).unwrap().unwrap().body;
            (r.id, body, r.author.label())
        })
        .collect()
}

#[tokio::test]
async fn non_admin_is_403_and_anonymous_is_401() {
    let (app, st, _) = setup().await;
    assert_eq!(send(&app, "GET", "/admin", None, "").await.status(), 401);
    // a signed-in reader without the admin flag
    let id = st.access.upsert_user(1, "boss").unwrap().id;
    st.access.set_admin(id, false).unwrap();
    let cookie = {
        let res = sign_in(&app).await;
        let c = res.headers().get_all("set-cookie").iter().find_map(|v| {
            v.to_str()
                .ok()?
                .strip_prefix("sid=")?
                .split(';')
                .next()
                .map(str::to_owned)
        });
        format!("sid={}", c.unwrap())
    };
    for (m, uri) in [
        ("GET", "/admin"),
        ("GET", "/admin/edit/en/a"),
        ("POST", "/admin/edit/en/a"),
    ] {
        assert_eq!(
            send(&app, m, uri, Some(&cookie), "body=x").await.status(),
            403,
            "{uri}"
        );
    }
    // preview is a proposer tool, available to any unblocked signed-in user
    let res = send(&app, "POST", "/admin/preview", Some(&cookie), "body=x").await;
    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn admin_edit_creates_revision_authored_by_admin() {
    let (app, st, cookie) = setup().await;
    let dash = send(&app, "GET", "/admin", Some(&cookie), "").await;
    assert_eq!(dash.status(), 200);
    assert!(text(dash).await.contains("/admin/edit/en/a"));
    let res = send(
        &app,
        "POST",
        "/admin/edit/en/a",
        Some(&cookie),
        "body=v2&summary=fix",
    )
    .await;
    assert_eq!(res.status(), 303);
    let h = history(&st);
    assert_eq!(h.len(), 2);
    assert_eq!((h[0].1.as_str(), h[0].2.as_str()), ("v2", "boss"));
    // a new page can be created the same way
    send(
        &app,
        "POST",
        "/admin/edit/en/fresh",
        Some(&cookie),
        "body=hi",
    )
    .await;
    assert!(
        st.pages
            .find(&Slug::parse("fresh").unwrap(), Lang::En)
            .unwrap()
            .is_some()
    );
    // a foreign origin is refused
    let req = Request::post("/admin/edit/en/a")
        .header("cookie", &cookie)
        .header("origin", "https://evil.example")
        .header("content-type", "application/x-www-form-urlencoded");
    let res = app
        .clone()
        .oneshot(req.body(Body::from("body=v3")).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), 403);
    assert_eq!(history(&st).len(), 2);
}

#[tokio::test]
async fn rollback_creates_new_revision_not_rewrite() {
    let (app, st, cookie) = setup().await;
    let first = history(&st)[0].0;
    send(&app, "POST", "/admin/edit/en/a", Some(&cookie), "body=v2").await;
    let res = send(
        &app,
        "POST",
        "/admin/rollback/en/a",
        Some(&cookie),
        &format!("rev={first}"),
    )
    .await;
    assert_eq!(res.status(), 303);
    let h = history(&st);
    assert_eq!(h.len(), 3);
    assert_eq!(h[0].1, "v1");
    assert_eq!(h[0].2, "boss");
    assert_eq!((h[1].1.as_str(), h[2].1.as_str()), ("v2", "v1"));
    assert_eq!(h[2].0, first);
}

#[tokio::test]
async fn preview_uses_the_shared_renderer() {
    let (app, _, cookie) = setup().await;
    let res = send(&app, "POST", "/admin/preview", Some(&cookie), "**x**").await;
    assert_eq!(res.status(), 200);
    assert!(text(res).await.contains("<strong>x</strong>"));
}
