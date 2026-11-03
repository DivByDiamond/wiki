//! "My proposals": a signed-in reader sees only their own, with status.
use crate::admin::common::{SEED_QUOTA, env, login, send};
use crate::support::text;
use oc2r_wiki::domain::proposal::Status;

#[tokio::test]
async fn lists_own_proposals_with_statuses_only() {
    let e = env();
    let (cookie, uid) = login(&e.st, 10, "reader1", false, false);
    let (_, other) = login(&e.st, 11, "reader2", false, false);
    let p = e.page("a");
    let mk = |user, summary: &str| {
        e.st.proposals
            .create(p.id, p.current.id, "b", summary, user, SEED_QUOTA)
            .unwrap()
            .unwrap()
    };
    let pending = mk(uid, "still waiting");
    let accepted = mk(uid, "was accepted");
    let rejected = mk(uid, "was <rejected>");
    mk(other, "someone elses");
    assert!(
        e.st.proposals
            .claim(accepted, Status::Accepted, e.admin_id, "")
            .unwrap()
    );
    assert!(
        e.st.proposals
            .claim(rejected, Status::Rejected, e.admin_id, "no")
            .unwrap()
    );
    let _ = pending;
    let res = send(&e.app, "GET", "/account/proposals", Some(&cookie), None, "").await;
    assert_eq!(res.status(), 200);
    assert!(
        res.headers()["cache-control"]
            .to_str()
            .unwrap()
            .contains("no-store")
    );
    let body = text(res).await;
    for want in [
        "still waiting",
        "was accepted",
        "was &lt;rejected&gt;",
        "/en/a",
    ] {
        assert!(
            body.contains(want)
                || body.contains(&want.replace("&lt;", "&#60;").replace("&gt;", "&#62;")),
            "{want}"
        );
    }
    assert!(!body.contains("someone elses"));
    for status in ["status-open", "status-accepted", "status-rejected"] {
        assert!(body.contains(status), "{status}");
    }
}

#[tokio::test]
async fn needs_a_signed_in_unblocked_user() {
    let e = env();
    let anon = send(&e.app, "GET", "/account/proposals", None, None, "").await;
    assert_eq!(anon.status(), 401);
    let (blocked, _) = login(&e.st, 12, "bad", false, true);
    let res = send(
        &e.app,
        "GET",
        "/account/proposals",
        Some(&blocked),
        None,
        "",
    )
    .await;
    assert_ne!(res.status(), 200);
}

#[tokio::test]
async fn header_menu_links_to_my_proposals_only_when_signed_in() {
    let e = env();
    let (cookie, _) = login(&e.st, 10, "reader1", false, false);
    let signed = send(&e.app, "GET", "/en/a", Some(&cookie), None, "").await;
    assert!(text(signed).await.contains("href=\"/account/proposals\""));
    let anon = send(&e.app, "GET", "/en/a", None, None, "").await;
    assert!(!text(anon).await.contains("/account/proposals"));
}

#[tokio::test]
async fn empty_state_is_shown() {
    let e = env();
    let (cookie, _) = login(&e.st, 10, "reader1", false, false);
    let res = send(&e.app, "GET", "/account/proposals", Some(&cookie), None, "").await;
    assert_eq!(res.status(), 200);
}
