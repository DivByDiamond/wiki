use super::{get, put};
use crate::support::{app, json};

#[tokio::test]
async fn key_can_set_navigation_without_losing_it_on_body_only_updates() {
    let (app, key, _) = app();
    let uri = "/api/pages/recipes?lang=ru&section=recipes&position=5";
    let response = put(&app, uri, Some(&key), b"# Recipes".to_vec()).await;
    assert_eq!(response.status(), 200);
    let first = json(get(&app, "/api/pages/recipes?lang=ru", None).await).await;
    assert_eq!(first["section"], "recipes");
    assert_eq!(first["position"], 5);
    let response = put(
        &app,
        "/api/pages/recipes?lang=ru",
        Some(&key),
        b"# Updated".to_vec(),
    )
    .await;
    assert_eq!(response.status(), 200);
    let updated = json(get(&app, "/api/pages/recipes?lang=ru", None).await).await;
    assert_eq!(updated["section"], "recipes");
    assert_eq!(updated["position"], 5);
    let moved = put(
        &app,
        "/api/pages/recipes?lang=ru&section=hardware",
        Some(&key),
        b"# Updated".to_vec(),
    )
    .await;
    assert_eq!(json(moved).await["outcome"], "updated");
    let final_page = json(get(&app, "/api/pages/recipes?lang=ru", None).await).await;
    assert_eq!(final_page["section"], "hardware");
    assert_eq!(final_page["position"], 5);
}

#[tokio::test]
async fn invalid_navigation_is_rejected_without_creating_a_page() {
    let (app, key, _) = app();
    for query in ["position=-1", "position=1000001", "section=bad%0Aname"] {
        let uri = format!("/api/pages/new?lang=en&{query}");
        assert_eq!(
            put(&app, &uri, Some(&key), b"# New".to_vec())
                .await
                .status(),
            422
        );
        assert_eq!(
            get(&app, "/api/pages/new?lang=en", None).await.status(),
            404
        );
    }
}
