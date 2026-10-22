use crate::admin::common::{env, login};
use crate::support::text;

#[tokio::test]
async fn an_admin_cannot_block_or_demote_themselves() {
    let e = env();
    login(&e.st, 2, "second", true, false);
    for action in ["block", "revoke-admin"] {
        let res = e
            .post(&format!("/admin/users/{}/{action}", e.admin_id), "")
            .await;
        assert_eq!(res.status(), 409, "{action}");
        assert!(text(res).await.contains("yourself"));
    }
    let u = e.st.access.user(e.admin_id).unwrap().unwrap();
    assert!(u.admin && !u.blocked);
}

#[tokio::test]
async fn active_admin_count_follows_grants_and_blocks() {
    let e = env();
    let (_, id) = login(&e.st, 4, "x", false, false);
    assert_eq!(e.st.access.count_active_admins().unwrap(), 1);
    e.post(&format!("/admin/users/{id}/grant-admin"), "").await;
    assert_eq!(e.st.access.count_active_admins().unwrap(), 2);
    e.post(&format!("/admin/users/{id}/block"), "").await;
    assert_eq!(e.st.access.count_active_admins().unwrap(), 1);
}
