use crate::login::{gh, sign_in};
use crate::support::{save_page, staff_app};
use axum::{Router, body::Body, http::Request, response::Response};
use oc2r_wiki::{
    domain::{page::Lang, page::Slug, proposal::Status},
    http::state::AppState,
};
use tower::ServiceExt;

/// App with page "a" ("v1"), admin "boss" signed in, and one proposal by "writer" on rev 1.
async fn setup() -> (Router, AppState, String, i64) {
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
    let admin = st.access.upsert_user(1, "boss").unwrap().id;
    st.access.set_admin(admin, true).unwrap();
    let writer = st.access.upsert_user(2, "writer").unwrap().id;
    let page = page(&st);
    let pid = st
        .proposals
        .create(page.id, page.current.id, "proposed", "fix typo", writer)
        .unwrap();
    (app, st, cookie, pid)
}

fn page(st: &AppState) -> oc2r_wiki::domain::page::Page {
    st.pages
        .find(&Slug::parse("a").unwrap(), Lang::En)
        .unwrap()
        .unwrap()
}

async fn post(app: &Router, cookie: &str, uri: &str, body: &str) -> Response {
    let req = Request::post(uri)
        .header("cookie", cookie)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body.to_owned()))
        .unwrap();
    app.clone().oneshot(req).await.unwrap()
}

#[tokio::test]
async fn accept_clean_proposal_publishes_with_proposer_as_author() {
    let (app, st, cookie, pid) = setup().await;
    let res = post(
        &app,
        &cookie,
        &format!("/admin/proposals/{pid}/accept"),
        "note=ok",
    )
    .await;
    assert!(res.status().is_redirection(), "{}", res.status());
    let p = page(&st);
    assert_eq!(p.current.body, "proposed");
    assert_eq!(p.current.author.label(), "writer");
    assert_eq!(p.current.summary, "fix typo (принял boss)");
    assert_eq!(
        st.proposals.get(pid).unwrap().unwrap().status,
        Status::Accepted
    );
}

#[tokio::test]
async fn accept_conflicting_proposal_requires_manual_merge() {
    let (app, st, cookie, pid) = setup().await;
    save_page(&st, "a", Lang::En, "v2", "other edit");
    let uri = format!("/admin/proposals/{pid}/accept");
    let res = post(&app, &cookie, &uri, "note=").await;
    assert_eq!(res.status(), 422);
    assert_eq!(page(&st).current.body, "v2");
    assert_eq!(st.proposals.get(pid).unwrap().unwrap().status, Status::Open);
    let detail = app
        .clone()
        .oneshot(
            Request::get(format!("/admin/proposals/{pid}"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail.status(), 200);
    let res = post(&app, &cookie, &uri, "body=merged&note=").await;
    assert!(res.status().is_redirection(), "{}", res.status());
    let p = page(&st);
    assert_eq!(p.current.body, "merged");
    assert_eq!(p.current.author.label(), "writer");
}

#[tokio::test]
async fn reject_stores_note_and_changes_nothing() {
    let (app, st, cookie, pid) = setup().await;
    let before = page(&st).current.id;
    let res = post(
        &app,
        &cookie,
        &format!("/admin/proposals/{pid}/reject"),
        "note=nope",
    )
    .await;
    assert!(res.status().is_redirection(), "{}", res.status());
    assert_eq!(page(&st).current.id, before);
    assert_eq!(
        st.proposals.get(pid).unwrap().unwrap().status,
        Status::Rejected
    );
}
