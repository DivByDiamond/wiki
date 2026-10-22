use crate::admin::common::{Env, env};
use crate::support::text;
use oc2r_wiki::domain::proposal::Status;

fn status(e: &Env, pid: i64) -> Status {
    e.st.proposals.get(pid).unwrap().unwrap().status
}

async fn decide(e: &Env, pid: i64, what: &str, body: &str) -> u16 {
    e.post(&format!("/admin/proposals/{pid}/{what}"), body)
        .await
        .status()
        .as_u16()
}

#[tokio::test]
async fn accept_then_accept_again_is_409_and_publishes_once() {
    let e = env();
    let pid = e.proposal();
    assert_eq!(decide(&e, pid, "accept", "note=").await, 303);
    let revs = e.st.pages.history(e.page("a").id).unwrap().len();
    assert_eq!(decide(&e, pid, "accept", "note=").await, 409);
    assert_eq!(e.st.pages.history(e.page("a").id).unwrap().len(), revs);
}

#[tokio::test]
async fn accept_then_reject_and_reject_then_accept_are_409() {
    let e = env();
    let p1 = e.proposal();
    assert_eq!(decide(&e, p1, "accept", "note=").await, 303);
    assert_eq!(decide(&e, p1, "reject", "note=x").await, 409);
    assert_eq!(status(&e, p1), Status::Accepted);

    let p2 = e.proposal();
    assert_eq!(decide(&e, p2, "reject", "note=x").await, 303);
    let before = e.page("a").current.id;
    assert_eq!(decide(&e, p2, "accept", "note=").await, 409);
    assert_eq!(e.page("a").current.id, before);
    assert_eq!(status(&e, p2), Status::Rejected);
}

#[tokio::test]
async fn the_409_is_an_html_page_with_a_message() {
    let e = env();
    let pid = e.proposal();
    decide(&e, pid, "reject", "note=").await;
    let res = e
        .post(&format!("/admin/proposals/{pid}/accept"), "note=")
        .await;
    assert_eq!(res.status(), 409);
    assert!(text(res).await.contains("already decided"));
}

#[tokio::test]
async fn accept_on_a_deleted_page_is_refused_and_detail_still_renders() {
    let e = env();
    let pid = e.proposal();
    let id = e.page("a").id;
    e.st.pages.set_deleted(id, true).unwrap();
    let revs = e.st.pages.history(id).unwrap().len();
    assert_eq!(decide(&e, pid, "accept", "note=").await, 409);
    assert_eq!(status(&e, pid), Status::Open);
    assert_eq!(e.st.pages.history(id).unwrap().len(), revs);
    assert!(
        e.st.pages.list().unwrap().is_empty(),
        "no duplicate live page"
    );
    let html = e.get_text(&format!("/admin/proposals/{pid}")).await;
    assert!(html.contains("page was deleted"), "{html}");
    assert_eq!(decide(&e, pid, "reject", "note=gone").await, 303);
}

#[tokio::test]
async fn accept_after_a_concurrent_save_still_needs_the_merged_text() {
    let e = env();
    let pid = e.proposal();
    crate::support::save_page(&e.st, "a", oc2r_wiki::domain::page::Lang::En, "v2", "other");
    assert_eq!(decide(&e, pid, "accept", "note=").await, 422);
    assert_eq!(status(&e, pid), Status::Open);
    assert_eq!(decide(&e, pid, "accept", "body=merged&note=").await, 303);
    assert_eq!(e.page("a").current.body, "merged");
    assert_eq!(decide(&e, pid, "accept", "body=again&note=").await, 409);
    assert_eq!(e.page("a").current.body, "merged");
}
