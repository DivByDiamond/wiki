use super::*;
use crate::{router, store::db::open_memory};
use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use tower::ServiceExt;

fn state() -> AppState {
    AppState::new(open_memory().unwrap())
}

async fn request(state: AppState, path: &str, cookie: &str, browser: &str) -> Response {
    router(state)
        .oneshot(
            Request::get(path)
                .header("cookie", cookie)
                .header("accept-language", browser)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn body(response: Response) -> String {
    String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap()
}

#[tokio::test]
async fn root_uses_browser_cookie_and_english_fallback() {
    for (cookie, browser, expected) in [
        ("", "", "/en/start"),
        ("", "ru-RU,en;q=0.5", "/ru/start"),
        ("", "zh-Hans-CN", "/zh/start"),
        ("wiki_language=en", "ru", "/en/start"),
        ("wiki_language=zh", "en", "/zh/start"),
        ("wiki_language=invalid", "de", "/en/start"),
    ] {
        let response = request(state(), "/", cookie, browser).await;
        assert_eq!(response.headers()["location"], expected);
    }
}

#[tokio::test]
async fn explicit_public_language_overrides_and_remembers_choice() {
    let response = request(state(), "/ru/start", "wiki_language=zh", "en").await;
    let cookies: Vec<_> = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|value| value.to_str().unwrap())
        .collect();
    assert!(
        cookies
            .iter()
            .any(|value| value.starts_with("wiki_language=ru;")
                && value.contains("Path=/")
                && value.contains("SameSite=Lax"))
    );
    assert!(body(response).await.contains("<html lang=\"ru\">"));
}

#[tokio::test]
async fn admin_uses_remembered_language_and_switches_without_leaving_page() {
    let state = state();
    let user = state.access.upsert_user(17, "admin").unwrap();
    state.access.set_admin(user.id, true).unwrap();
    let session = state.sessions.create(user.id);
    let cookie = format!("sid={session}; wiki_language=ru");
    let response = request(state.clone(), "/admin?q=machine&lang=en", &cookie, "en").await;
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "private, no-store");
    let html = body(response).await;
    assert!(html.contains("<html lang=\"ru\">"));
    assert!(html.contains("/language/zh?next=%2Fadmin%3Fq%3Dmachine%26lang%3Den"));
    let response = request(state, "/language/zh?next=%2Fadmin", &cookie, "en").await;
    assert_eq!(response.headers()["location"], "/admin");
    assert!(
        response.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .starts_with("wiki_language=zh;")
    );
}

#[tokio::test]
async fn language_switch_preserves_long_multilingual_filters() {
    let target = format!(
        "/admin?q={}&lang=ru&section=systems",
        super::encode("Операционные системы и программирование 中文界面")
    );
    assert!(target.len() > 200);
    let url = format!("/language/zh?next={}", super::encode(&target));
    let response = request(state(), &url, "wiki_language=ru", "en").await;
    assert_eq!(response.headers()["location"].to_str().unwrap(), target);
}

#[tokio::test]
async fn language_switch_rejects_external_redirect() {
    let response = request(
        state(),
        "/language/ru?next=https%3A%2F%2Fevil.example",
        "",
        "",
    )
    .await;
    assert_eq!(response.headers()["location"], "/ru/start");
}

#[tokio::test]
async fn login_and_logout_keep_language_preference() {
    let state = state();
    let response = request(state.clone(), "/auth/github", "wiki_language=zh", "en").await;
    assert!(body(response).await.contains("<html lang=\"zh\">"));
    let response = router(state)
        .oneshot(
            Request::post("/auth/logout")
                .header("cookie", "wiki_language=ru")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.headers()["location"], "/");
    assert!(
        response
            .headers()
            .get_all("set-cookie")
            .iter()
            .all(|value| !value.to_str().unwrap().starts_with("wiki_language="))
    );
}

#[tokio::test]
async fn admin_denial_and_failed_save_keep_selected_language() {
    let state = state();
    let response = request(state.clone(), "/admin", "wiki_language=zh", "en").await;
    assert_eq!(response.status(), 401);
    assert!(body(response).await.contains("<html lang=\"zh\">"));
    let user = state.access.upsert_user(18, "admin").unwrap();
    state.access.set_admin(user.id, true).unwrap();
    let session = state.sessions.create(user.id);
    let response = router(state)
        .oneshot(
            Request::post("/admin/edit/en/start")
                .header("cookie", format!("sid={session}; wiki_language=ru"))
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("body=&summary=empty"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 422);
    assert!(body(response).await.contains("<html lang=\"ru\">"));
}

struct Github;
impl crate::auth::github::GithubClient for Github {
    fn authorize_url(&self, state: &str) -> String {
        format!("https://github.example/?state={state}")
    }
    fn exchange<'a>(&'a self, _: &'a str) -> crate::auth::github::ExchangeFuture<'a> {
        Box::pin(async {
            Ok(crate::auth::github::GithubUser {
                id: 19,
                login: "editor".into(),
            })
        })
    }
}

#[tokio::test]
async fn oauth_round_trip_preserves_language_and_return_destination() {
    let config = crate::config::Config::from_lookup(|key| match key {
        "GITHUB_CLIENT_ID" | "GITHUB_CLIENT_SECRET" => Some("test".into()),
        _ => None,
    })
    .unwrap();
    let state = state()
        .with_config(config)
        .with_github(std::sync::Arc::new(Github));
    let response = request(
        state.clone(),
        "/auth/github?next=%2Fadmin",
        "wiki_language=ru",
        "en",
    )
    .await;
    let mut cookies = vec!["wiki_language=ru".to_string()];
    for value in response.headers().get_all("set-cookie") {
        cookies.push(value.to_str().unwrap().split(';').next().unwrap().into());
    }
    let nonce = cookies
        .iter()
        .find_map(|value| value.strip_prefix("oauth_state="))
        .unwrap();
    let response = request(
        state,
        &format!("/auth/github/callback?code=test&state={nonce}"),
        &cookies.join("; "),
        "en",
    )
    .await;
    assert_eq!(response.headers()["location"], "/admin");
    assert!(
        response
            .headers()
            .get_all("set-cookie")
            .iter()
            .any(|value| value.to_str().unwrap().starts_with("sid="))
    );
    assert!(
        response
            .headers()
            .get_all("set-cookie")
            .iter()
            .all(|value| !value.to_str().unwrap().starts_with("wiki_language="))
    );
}
