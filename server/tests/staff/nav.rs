use crate::login::{get, gh, sign_in};
use crate::support::{save_page, staff_app, text};
use oc2r_wiki::domain::page::Lang;

async fn page_as(app: &axum::Router, cookie: Option<&str>) -> String {
    text(get(app, "/en/start", cookie).await).await
}

fn sid_of(res: &axum::response::Response) -> String {
    res.headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().unwrap().split(';').next())
        .find(|c| c.starts_with("sid="))
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn anonymous_sees_sign_in_link_with_next() {
    let (app, st) = staff_app(gh(1, "a"), &[]);
    save_page(&st, "start", Lang::En, "hi", "s");
    let body = page_as(&app, None).await;
    assert!(
        body.contains("href=\"/auth/github?next=/en/start\""),
        "{body}"
    );
    assert!(body.contains("Sign in"));
    assert!(!body.contains("/auth/logout") && !body.contains("href=\"/admin\""));
}

#[tokio::test]
async fn signed_in_user_sees_login_and_logout_form_but_no_admin_link() {
    let (app, st) = staff_app(gh(2, "al<ice"), &[]);
    save_page(&st, "start", Lang::En, "hi", "s");
    let sid = sid_of(&sign_in(&app).await);
    let body = page_as(&app, Some(&sid)).await;
    assert!(body.contains("action=\"/auth/logout\"") && body.contains("method=\"post\""));
    assert!(
        body.contains("al&#60;ice") || body.contains("al&lt;ice"),
        "{body}"
    );
    assert!(!body.contains("al<ice"));
    assert!(!body.contains("Sign in") && !body.contains("href=\"/admin\""));
}

#[tokio::test]
async fn admin_sees_the_admin_link() {
    let (app, st) = staff_app(gh(3, "boss"), &[("ADMIN_GITHUB_IDS", "3")]);
    save_page(&st, "start", Lang::En, "hi", "s");
    let sid = sid_of(&sign_in(&app).await);
    assert!(page_as(&app, Some(&sid)).await.contains("href=\"/admin\""));
}
