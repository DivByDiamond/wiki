//! Every admin route: 401 anonymous, 403 for non-admins and blocked admins,
//! and 403 for a foreign `Origin` on every POST. `/admin/preview` is gated by
//! "Propose" instead: any unblocked signed-in user may call it.
use crate::admin::common::{ORIGIN, env, login, send};

const GET: &[&str] = &[
    "/admin",
    "/admin/edit/en/a",
    "/admin/proposals",
    "/admin/proposals/1",
    "/admin/keys",
    "/admin/users",
];

const POST: &[&str] = &[
    "/admin/edit/en/a",
    "/admin/rollback/en/a",
    "/admin/delete/en/a",
    "/admin/restore/en/a",
    "/admin/preview",
    "/admin/proposals/1/accept",
    "/admin/proposals/1/reject",
    "/admin/keys",
    "/admin/keys/1/revoke",
    "/admin/users/1/block",
    "/admin/users/1/unblock",
    "/admin/users/1/grant-admin",
    "/admin/users/1/revoke-admin",
];

fn routes() -> Vec<(&'static str, &'static str)> {
    let get = GET.iter().map(|p| ("GET", *p));
    get.chain(POST.iter().map(|p| ("POST", *p))).collect()
}

#[tokio::test]
async fn anonymous_gets_401_everywhere() {
    let e = env();
    e.proposal();
    for (m, p) in routes() {
        let r = send(&e.app, m, p, None, None, "").await;
        assert_eq!(r.status(), 401, "{m} {p}");
    }
}

#[tokio::test]
async fn signed_in_non_admin_and_blocked_admin_get_403_on_admin_routes() {
    let e = env();
    e.proposal();
    let (plain, _) = login(&e.st, 50, "plain", false, false);
    let (blocked, _) = login(&e.st, 51, "ex", true, true);
    for (cookie, preview) in [(&plain, 200), (&blocked, 403)] {
        for (m, p) in routes() {
            let want = if p == "/admin/preview" { preview } else { 403 };
            let r = send(&e.app, m, p, Some(cookie), Some(ORIGIN), "").await;
            assert_eq!(r.status(), want, "{cookie} {m} {p}");
        }
    }
}

#[tokio::test]
async fn api_keys_are_not_admins() {
    let e = env();
    let (key, _) = crate::support::make_key(&e.st, "all", true, true, true);
    for (m, p) in routes() {
        let req = axum::http::Request::builder()
            .method(m)
            .uri(p)
            .header("authorization", format!("Bearer {key}"))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(axum::body::Body::empty())
            .unwrap();
        let r = tower::ServiceExt::oneshot(e.app.clone(), req)
            .await
            .unwrap();
        assert_eq!(r.status(), 403, "{m} {p}");
    }
}

#[tokio::test]
async fn foreign_origin_gets_403_on_every_post() {
    let e = env();
    e.proposal();
    for p in POST {
        let r = send(
            &e.app,
            "POST",
            p,
            Some(&e.admin),
            Some("https://evil.example"),
            "name=x&read=1&rev=1&body=x",
        )
        .await;
        assert_eq!(r.status(), 403, "{p}");
    }
    // nothing happened
    assert_eq!(e.page("a").current.body, "v1");
    assert!(e.st.access.list_keys().unwrap().is_empty());
}
