use super::*;

#[tokio::test]
async fn deleted_or_missing_page_is_404() {
    let (app, _, cookie, _) = setup().await;
    let req = Request::post("/propose/en/nope")
        .header("cookie", &cookie)
        .header("content-type", "application/x-www-form-urlencoded");
    let res = app
        .clone()
        .oneshot(req.body(Body::from("body=x")).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn mismatching_origin_is_403_and_matching_is_accepted() {
    let (app, st, cookie, _) = setup().await;
    assert_eq!(
        post(
            &app,
            Some(&cookie),
            Some("https://evil.example"),
            "new",
            "1",
            ""
        )
        .await
        .status(),
        403
    );
    assert!(st.proposals.list(Status::Open).unwrap().is_empty());
    assert_eq!(
        post(
            &app,
            Some(&cookie),
            Some("https://w.example"),
            "new",
            "1",
            ""
        )
        .await
        .status(),
        303
    );
}

#[tokio::test]
async fn form_shows_current_text_escaped_and_base_rev() {
    let (app, st, cookie, _) = setup().await;
    save_page(
        &st,
        "guide",
        Lang::En,
        "</textarea><script>alert(1)</script>",
        "evil",
    );
    let rev = current_rev(&st);
    let res = app
        .clone()
        .oneshot(
            Request::get("/propose/en/guide")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let html = text(res).await;
    assert!(
        html.contains("id=\"editor\"")
            && html.contains(&format!("name=\"base_rev\" value=\"{rev}\""))
    );
    assert!(
        !html.contains("<script>alert(1)"),
        "page text must be escaped"
    );
    assert!(html.contains("&lt;script&gt;") || html.contains("&#60;script&#62;"));
}

#[tokio::test]
async fn article_links_to_the_form_and_shows_the_sent_banner() {
    let (app, _, _, _) = setup().await;
    let get = |uri: &'static str| {
        app.clone()
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
    };
    let plain = text(get("/en/guide").await.unwrap()).await;
    assert!(
        plain.contains("href=\"/propose/en/guide\"") && !plain.contains("Предложение отправлено")
    );
    assert!(
        text(get("/en/guide?proposed=1").await.unwrap())
            .await
            .contains("Предложение отправлено")
    );
}
