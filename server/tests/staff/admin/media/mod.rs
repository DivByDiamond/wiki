//! Image upload: admin only, origin-checked, type sniffed, size limited, content-addressed.
use super::common::{Env, ORIGIN, env, login};
use crate::support::text;
use axum::{body::Body, http::Request, response::Response};
use tower::ServiceExt;

const PNG: &[u8] = b"\x89PNG\r\n\x1a\nIHDR-data";
const BOUNDARY: &str = "XBOUNDARYX";

fn multipart(filename: &str, claimed: &str, data: &[u8]) -> Vec<u8> {
    let mut b = format!(
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: {claimed}\r\n\r\n"
    )
    .into_bytes();
    b.extend_from_slice(data);
    b.extend_from_slice(format!("\r\n--{BOUNDARY}--\r\n").as_bytes());
    b
}

async fn upload(
    e: &Env,
    cookie: Option<&str>,
    origin: Option<&str>,
    filename: &str,
    claimed: &str,
    data: &[u8],
) -> Response {
    let mut req = Request::builder()
        .method("POST")
        .uri("/admin/media")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={BOUNDARY}"),
        );
    if let Some(c) = cookie {
        req = req.header("cookie", c);
    }
    if let Some(o) = origin {
        req = req.header("origin", o);
    }
    let body = Body::from(multipart(filename, claimed, data));
    e.app
        .clone()
        .oneshot(req.body(body).unwrap())
        .await
        .unwrap()
}

async fn admin_upload(e: &Env, filename: &str, claimed: &str, data: &[u8]) -> Response {
    upload(e, Some(&e.admin), Some(ORIGIN), filename, claimed, data).await
}

fn names(e: &Env) -> Vec<String> {
    e.st.media
        .list()
        .unwrap()
        .into_iter()
        .map(|m| m.name)
        .collect()
}

#[tokio::test]
async fn upload_then_list_snippet_and_serve() {
    let e = env();
    let res = admin_upload(&e, "../../evil name.png", "image/png", PNG).await;
    assert_eq!(res.status(), 303);
    let list = names(&e);
    assert_eq!(list.len(), 1);
    let name = &list[0];
    assert!(name.ends_with(".png") && !name.contains("evil") && name.len() == 36);
    let page = e.get_text("/admin/media").await;
    assert!(page.contains(&format!("![](/media/{name})")), "{page}");

    let served = e
        .app
        .clone()
        .oneshot(
            Request::get(format!("/media/{name}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(served.status(), 200);
    let h = served.headers().clone();
    assert_eq!(h["content-type"], "image/png");
    assert!(h["cache-control"].to_str().unwrap().contains("immutable"));
    assert_eq!(h["x-content-type-options"], "nosniff");
    assert!(
        h["content-security-policy"]
            .to_str()
            .unwrap()
            .contains("default-src 'none'")
    );
    let etag = h["etag"].to_str().unwrap().to_owned();
    let bytes = axum::body::to_bytes(served.into_body(), 1 << 20)
        .await
        .unwrap();
    assert_eq!(&bytes[..], PNG);
    let cached = e
        .app
        .clone()
        .oneshot(
            Request::get(format!("/media/{name}"))
                .header("if-none-match", etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cached.status(), 304);
}

#[tokio::test]
async fn identical_content_is_deduplicated_and_all_formats_work() {
    let e = env();
    admin_upload(&e, "a.png", "image/png", PNG).await;
    admin_upload(&e, "b.png", "image/png", PNG).await;
    assert_eq!(names(&e).len(), 1);
    for (data, ext) in [
        (&b"\xFF\xD8\xFF\xE0jpeg"[..], "jpg"),
        (b"GIF89a-gif", "gif"),
        (b"RIFF\x04\0\0\0WEBPVP8 ", "webp"),
    ] {
        assert_eq!(
            admin_upload(&e, "x.bin", "application/octet-stream", data)
                .await
                .status(),
            303
        );
        assert!(names(&e).iter().any(|n| n.ends_with(ext)), "{ext}");
    }
}

#[tokio::test]
async fn svg_html_and_disguised_files_are_refused() {
    let e = env();
    let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><script>alert(1)</script></svg>";
    for (name, claimed, data) in [
        ("x.svg", "image/svg+xml", &svg[..]),
        ("x.png", "image/png", &svg[..]),
        ("x.png", "image/png", b"<html><script>1</script></html>"),
        ("x.png", "image/png", b""),
        ("x.png", "image/png", b"GIF"),
    ] {
        let res = admin_upload(&e, name, claimed, data).await;
        assert_eq!(res.status(), 422, "{name} {claimed}");
    }
    assert!(names(&e).is_empty());
}

#[tokio::test]
async fn oversized_upload_is_refused() {
    let e = env();
    let mut big = PNG.to_vec();
    big.resize(5 * 1024 * 1024 + 1, 0);
    let res = admin_upload(&e, "big.png", "image/png", &big).await;
    assert_eq!(res.status(), 413);
    assert!(names(&e).is_empty());
}

#[tokio::test]
async fn upload_requires_admin_and_own_origin() {
    let e = env();
    let (user, _) = login(&e.st, 20, "plain", false, false);
    assert_eq!(
        upload(&e, None, Some(ORIGIN), "a.png", "image/png", PNG)
            .await
            .status(),
        401
    );
    assert_eq!(
        upload(&e, Some(&user), Some(ORIGIN), "a.png", "image/png", PNG)
            .await
            .status(),
        403
    );
    assert_eq!(
        upload(
            &e,
            Some(&e.admin),
            Some("https://evil.example"),
            "a.png",
            "image/png",
            PNG
        )
        .await
        .status(),
        403
    );
    assert!(names(&e).is_empty());
}

#[tokio::test]
async fn media_page_is_admin_only_and_missing_files_are_404() {
    let e = env();
    let (user, _) = login(&e.st, 20, "plain", false, false);
    let get = |cookie: Option<&str>, uri: &str| {
        let mut r = Request::get(uri.to_owned());
        if let Some(c) = cookie {
            r = r.header("cookie", c.to_owned());
        }
        e.app.clone().oneshot(r.body(Body::empty()).unwrap())
    };
    assert_eq!(get(None, "/admin/media").await.unwrap().status(), 401);
    assert_eq!(
        get(Some(&user), "/admin/media").await.unwrap().status(),
        403
    );
    assert_eq!(get(None, "/media/nothing.png").await.unwrap().status(), 404);
    assert_eq!(
        get(None, "/media/..%2F..%2Fetc%2Fpasswd")
            .await
            .unwrap()
            .status(),
        404
    );
    let _ = text;
}

#[tokio::test]
async fn admin_nav_links_to_media() {
    let e = env();
    assert!(e.get_text("/admin").await.contains("href=\"/admin/media\""));
}
