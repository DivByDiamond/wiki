use crate::admin::common::{env, login, send};
use oc2r_wiki::domain::page::{Lang, Slug};

#[tokio::test]
async fn dashboard_creates_nested_pages_without_javascript() {
    let e = env();
    let dashboard = e.get_text("/admin").await;
    assert!(dashboard.contains("action=\"/admin/create\""));
    assert!(dashboard.contains("name=\"lang\""));
    assert!(dashboard.contains("name=\"slug\""));
    let res = e.get("/admin/create?lang=ru&slug=blocks/new-page").await;
    assert_eq!(res.status(), 303);
    assert_eq!(res.headers()["location"], "/admin/edit/ru/blocks/new-page");
    let form = e.get_text("/admin/edit/ru/blocks/new-page").await;
    assert!(form.contains("name=\"section\""));
    assert!(form.contains("name=\"position\""));
    assert_eq!(
        e.post(
            "/admin/edit/ru/blocks/new-page",
            "body=%23+New&section=Blocks&position=4"
        )
        .await
        .status(),
        303
    );
    let page =
        e.st.pages
            .find(&Slug::parse("blocks/new-page").unwrap(), Lang::Ru)
            .unwrap()
            .unwrap();
    assert_eq!((page.section.as_str(), page.position), ("Blocks", 4));
    assert!(e.get_text("/admin").await.contains("<h3>Blocks</h3>"));
}

#[tokio::test]
async fn create_address_is_validated_and_admin_only() {
    let e = env();
    for query in [
        "lang=en&slug=Bad",
        "lang=de&slug=a",
        "lang=en&slug=search",
        "lang=en&slug=../x",
    ] {
        assert_eq!(e.get(&format!("/admin/create?{query}")).await.status(), 422);
    }
    assert_eq!(
        send(
            &e.app,
            "GET",
            "/admin/create?lang=en&slug=new",
            None,
            None,
            ""
        )
        .await
        .status(),
        401
    );
    let (reader, _) = login(&e.st, 2, "reader", false, false);
    assert_eq!(
        send(
            &e.app,
            "GET",
            "/admin/create?lang=en&slug=new",
            Some(&reader),
            None,
            ""
        )
        .await
        .status(),
        403
    );
}

#[tokio::test]
async fn navigation_changes_are_recorded_and_rollback_keeps_navigation() {
    let e = env();
    let first = e.page("a").current.id;
    let edit = "body=v1&section=Systems&position=12";
    assert_eq!(e.post("/admin/edit/en/a", edit).await.status(), 303);
    let page = e.page("a");
    assert_eq!((page.section.as_str(), page.position), ("Systems", 12));
    assert_ne!(page.current.id, first);
    assert_eq!(page.current.author.label(), "boss");
    assert_eq!(e.st.pages.history(page.id).unwrap().len(), 2);
    // An identical resubmission is still idempotent.
    assert_eq!(e.post("/admin/edit/en/a", edit).await.status(), 303);
    assert_eq!(e.st.pages.history(page.id).unwrap().len(), 2);
    assert!(
        e.get_text("/admin/edit/en/a")
            .await
            .contains("value=\"Systems\"")
    );
    assert_eq!(e.post("/admin/edit/en/a", "body=v2").await.status(), 303);
    assert_eq!(
        e.post("/admin/rollback/en/a", &format!("rev={first}"))
            .await
            .status(),
        303
    );
    let page = e.page("a");
    assert_eq!(page.current.body, "v1");
    assert_eq!((page.section.as_str(), page.position), ("Systems", 12));
}

#[tokio::test]
async fn invalid_navigation_never_partially_saves_text() {
    let e = env();
    for field in [
        "position=-1".into(),
        "position=1000001".into(),
        "position=oops".into(),
        "section=bad%0Asection".into(),
        format!("section={}", "x".repeat(101)),
    ] {
        assert_eq!(
            e.post("/admin/edit/en/a", &format!("body=changed&{field}"))
                .await
                .status(),
            422
        );
        assert_eq!(e.page("a").current.body, "v1");
        assert_eq!(e.st.pages.history(e.page("a").id).unwrap().len(), 1);
    }
    assert_eq!(
        send(
            &e.app,
            "POST",
            "/admin/edit/en/a",
            Some(&e.admin),
            Some("https://evil.example"),
            "body=changed&section=Systems&position=4"
        )
        .await
        .status(),
        403
    );
    assert_eq!(e.page("a").section, "");
}
