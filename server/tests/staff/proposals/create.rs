use super::*;

#[tokio::test]
async fn anonymous_cannot_propose() {
    let (app, st, _, _) = setup().await;
    let res = post(&app, None, None, "new", "1", "").await;
    assert_eq!(res.status(), 401);
    assert!(text(res).await.contains("Sign in with GitHub"));
    assert!(st.proposals.list(Status::Open).unwrap().is_empty());
}

#[tokio::test]
async fn anonymous_form_page_links_to_sign_in() {
    let (app, _, _, _) = setup().await;
    let res = app
        .clone()
        .oneshot(
            Request::get("/propose/en/guide")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
    assert!(text(res).await.contains("href=\"/auth/github\""));
}

#[tokio::test]
async fn blocked_user_cannot_propose() {
    let (app, st, cookie, uid) = setup().await;
    st.access.set_blocked(uid, true).unwrap();
    assert_eq!(
        post(&app, Some(&cookie), None, "new", "1", "")
            .await
            .status(),
        403
    );
    assert!(st.proposals.list(Status::Open).unwrap().is_empty());
}

#[tokio::test]
async fn proposal_is_stored_with_base_revision() {
    let (app, st, cookie, uid) = setup().await;
    let rev = current_rev(&st);
    let res = post(
        &app,
        Some(&cookie),
        Some("https://w.example"),
        "line1\r\nline2",
        &rev.to_string(),
        "  fix  ",
    )
    .await;
    assert_eq!(res.status(), 303);
    assert_eq!(res.headers()["location"], "/en/guide?proposed=1");
    let open = st.proposals.list(Status::Open).unwrap();
    assert_eq!(open.len(), 1);
    assert_eq!((open[0].base_rev_id, open[0].user_id), (rev, uid));
    assert_eq!(
        (open[0].body.as_str(), open[0].summary.as_str()),
        ("line1\nline2", "fix")
    );
}

#[tokio::test]
async fn base_rev_of_another_page_falls_back_to_current() {
    let (app, st, cookie, _) = setup().await;
    save_page(&st, "other", Lang::En, "x", "init");
    let other = st
        .pages
        .find(
            &oc2r_wiki::domain::page::Slug::parse("other").unwrap(),
            Lang::En,
        )
        .unwrap()
        .unwrap()
        .current
        .id;
    assert_eq!(
        post(&app, Some(&cookie), None, "new", &other.to_string(), "")
            .await
            .status(),
        303
    );
    assert_eq!(
        post(&app, Some(&cookie), None, "new2", "junk", "")
            .await
            .status(),
        303
    );
    let open = st.proposals.list(Status::Open).unwrap();
    assert!(open.iter().all(|p| p.base_rev_id == current_rev(&st)));
}

#[tokio::test]
async fn sixth_open_proposal_is_429() {
    let (app, st, cookie, _) = setup().await;
    for i in 0..5 {
        assert_eq!(
            post(&app, Some(&cookie), None, &format!("v{i}"), "1", "")
                .await
                .status(),
            303
        );
    }
    let res = post(&app, Some(&cookie), None, "v6", "1", "").await;
    assert_eq!(res.status(), 429);
    assert!(res.headers().contains_key("retry-after"));
    assert_eq!(st.proposals.list(Status::Open).unwrap().len(), 5);
}

#[tokio::test]
async fn twenty_first_proposal_of_the_day_is_429() {
    let (app, st, cookie, uid) = setup().await;
    let page_rev = current_rev(&st);
    let page_id = st
        .pages
        .find(
            &oc2r_wiki::domain::page::Slug::parse("guide").unwrap(),
            Lang::En,
        )
        .unwrap()
        .unwrap()
        .id;
    for i in 0..20 {
        let id = st
            .proposals
            .create(page_id, page_rev, &format!("old{i}"), "", uid)
            .unwrap();
        st.proposals
            .decide(id, Status::Rejected, uid, "no")
            .unwrap();
    }
    assert_eq!(
        post(&app, Some(&cookie), None, "fresh", "1", "")
            .await
            .status(),
        429
    );
}

#[tokio::test]
async fn invalid_submissions_are_rejected() {
    let (app, st, cookie, _) = setup().await;
    let c = Some(cookie.as_str());
    assert_eq!(
        post(&app, c, None, "original", "1", "").await.status(),
        422,
        "unchanged"
    );
    assert_eq!(
        post(&app, c, None, "  \n ", "1", "").await.status(),
        422,
        "blank"
    );
    assert_eq!(
        post(&app, c, None, "new", "1", &"s".repeat(201))
            .await
            .status(),
        422,
        "summary"
    );
    assert_eq!(
        post(&app, c, None, &"a".repeat(1024 * 1024 + 1), "1", "")
            .await
            .status(),
        422,
        "over 1 MiB"
    );
    let huge = format!("body={}", "a".repeat(5 * 1024 * 1024));
    assert_eq!(
        post_raw(&app, c, None, huge).await.status(),
        413,
        "transport cap"
    );
    assert!(st.proposals.list(Status::Open).unwrap().is_empty());
}

