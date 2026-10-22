use crate::login::{gh, sign_in};
use crate::support::{staff_app, text};
use axum::{body::Body, http::Request, response::Response};
use tower::ServiceExt;

async fn call(app: &axum::Router, req: Request<Body>) -> Response {
    app.clone().oneshot(req).await.unwrap()
}

fn form(method: &str, uri: &str, cookie: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("cookie", cookie)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body.to_owned()))
        .unwrap()
}

async fn put_status(app: &axum::Router, key: &str) -> u16 {
    let req = Request::put("/api/pages/k?lang=en")
        .header("authorization", format!("Bearer {key}"))
        .body(Body::from("hello"))
        .unwrap();
    call(app, req).await.status().as_u16()
}

#[tokio::test]
async fn key_shown_once_and_revoked_key_stops_working() {
    let (app, st) = staff_app(gh(1, "boss"), &[("PUBLIC_URL", "https://w.example")]);
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
    let id = st.access.upsert_user(1, "boss").unwrap().id;

    // non-admins are refused
    let r = call(&app, form("POST", "/admin/keys", &cookie, "name=x&read=1")).await;
    assert_eq!(r.status(), 403);
    st.access.set_admin(id, true).unwrap();

    let r = call(
        &app,
        form("POST", "/admin/keys", &cookie, "name=ci&read=1&write=1"),
    )
    .await;
    assert_eq!(r.status(), 200);
    let body = text(r).await;
    let start = body.find("ocw_").expect("key shown");
    let key = body[start..start + 68].to_owned();
    assert_eq!(put_status(&app, &key).await, 200);

    let list = text(call(&app, form("GET", "/admin/keys", &cookie, "")).await).await;
    assert!(list.contains("ci") && !list.contains(&key));

    let kid = st.access.list_keys().unwrap()[0].id;
    let r = call(
        &app,
        form("POST", &format!("/admin/keys/{kid}/revoke"), &cookie, ""),
    )
    .await;
    assert_eq!(r.status(), 303);
    assert_eq!(put_status(&app, &key).await, 401);
}
