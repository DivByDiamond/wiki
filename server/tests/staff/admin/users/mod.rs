mod guards;

use crate::admin::common::{env, login, send};
use crate::support::text;

#[tokio::test]
async fn users_page_lists_every_user_with_flags() {
    let e = env();
    login(&e.st, 77, "carol", false, true);
    let html = e.get_text("/admin/users").await;
    for needle in ["boss", "carol", ">77<", "/admin/users/", "Blocked"] {
        assert!(html.contains(needle), "missing {needle}: {html}");
    }
}

#[tokio::test]
async fn dashboard_links_to_users() {
    let e = env();
    assert!(e.get_text("/admin").await.contains("href=\"/admin/users\""));
}

#[tokio::test]
async fn block_unblock_grant_and_revoke_change_the_flags() {
    let e = env();
    let (_, id) = login(&e.st, 5, "dave", false, false);
    for (action, check) in [
        ("block", (false, true)),
        ("unblock", (false, false)),
        ("grant-admin", (true, false)),
        ("revoke-admin", (false, false)),
    ] {
        let res = e.post(&format!("/admin/users/{id}/{action}"), "").await;
        assert_eq!(res.status(), 303, "{action}");
        let u = e.st.access.user(id).unwrap().unwrap();
        assert_eq!((u.admin, u.blocked), check, "{action}");
    }
}

#[tokio::test]
async fn unknown_user_is_404() {
    let e = env();
    assert_eq!(e.post("/admin/users/9999/block", "").await.status(), 404);
}

#[tokio::test]
async fn blocking_takes_effect_on_the_very_next_request() {
    let e = env();
    let (cookie, id) = login(&e.st, 9, "erin", true, false);
    let ok = send(&e.app, "GET", "/admin", Some(&cookie), None, "").await;
    assert_eq!(ok.status(), 200);
    assert_eq!(
        e.post(&format!("/admin/users/{id}/block"), "")
            .await
            .status(),
        303
    );
    let res = send(&e.app, "GET", "/admin", Some(&cookie), None, "").await;
    assert_eq!(res.status(), 403);
    e.post(&format!("/admin/users/{id}/unblock"), "").await;
    let res = send(&e.app, "GET", "/admin", Some(&cookie), None, "").await;
    assert_eq!(res.status(), 200);
    e.post(&format!("/admin/users/{id}/revoke-admin"), "").await;
    let res = send(&e.app, "GET", "/admin", Some(&cookie), None, "").await;
    assert_eq!(res.status(), 403);
    assert!(text(res).await.contains("Not allowed"));
}
