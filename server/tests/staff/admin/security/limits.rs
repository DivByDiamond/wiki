use crate::admin::common::{ORIGIN, env, extract_key, send};
use crate::support::text;

#[tokio::test]
async fn preview_renders_markdown_for_admins() {
    let e = env();
    let r = e.post("/admin/preview", "# Hello").await;
    assert_eq!(r.status(), 200);
    assert!(text(r).await.contains("<h1"));
}

#[tokio::test]
async fn preview_rejects_bodies_over_one_mib_with_a_clean_error() {
    let e = env();
    let big = "a".repeat(1024 * 1024 + 1);
    let r = send(
        &e.app,
        "POST",
        "/admin/preview",
        Some(&e.admin),
        Some(ORIGIN),
        &big,
    )
    .await;
    assert_eq!(r.status(), 413);
    let html = text(r).await;
    assert!(!html.is_empty() && !html.contains("panicked"));
    let ok = "a".repeat(1024 * 1024);
    let r = send(
        &e.app,
        "POST",
        "/admin/preview",
        Some(&e.admin),
        Some(ORIGIN),
        &ok,
    )
    .await;
    assert_eq!(r.status(), 200);
}

#[tokio::test]
async fn editor_save_rejects_over_long_text_and_summary() {
    let e = env();
    let big = "a".repeat(1024 * 1024 + 1);
    let r = e.post("/admin/edit/en/a", &format!("body={big}")).await;
    assert_eq!(r.status(), 422);
    let long = "s".repeat(201);
    let r = e
        .post("/admin/edit/en/a", &format!("body=x&summary={long}"))
        .await;
    assert_eq!(r.status(), 422);
    assert_eq!(e.page("a").current.body, "v1");
}

#[tokio::test]
async fn plaintext_key_appears_only_in_the_creation_response() {
    let e = env();
    let r = e.post("/admin/keys", "name=ci&read=1").await;
    let created = text(r).await;
    let key = extract_key(&created);
    assert!(e.get_text("/admin/keys").await.contains("ci"));
    assert!(!e.get_text("/admin/keys").await.contains(&key));
    assert!(!e.get_text("/admin").await.contains(&key));
}
