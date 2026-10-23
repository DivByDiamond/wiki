//! Static frontend assets (`web/dist`) embedded into the binary.
//! `build.rs` guarantees the folder exists at compile time (and, in release
//! builds, that the bundle is present). Names are not hashed, so every file is
//! served with `no-cache` and an ETag from its embedded sha256.

use super::state::AppState;
use axum::{
    Router,
    extract::Path,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use rust_embed::RustEmbed;
use sha2::{Digest, Sha256};

#[derive(RustEmbed)]
#[folder = "../web/dist"]
struct Assets;

/// A changed bundle gets a new URL, even when an intermediary retained old CSS.
pub fn version() -> String {
    let mut hash = Sha256::new();
    for name in ["app.css", "app.js", "favicon.svg"] {
        if let Some(file) = Assets::get(name) {
            hash.update(file.metadata.sha256_hash());
        }
    }
    format!("{:x}", hash.finalize())[..16].to_owned()
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/static/{*path}", get(serve))
}

async fn serve(Path(path): Path<String>, headers: HeaderMap) -> Response {
    // Embedded keys never contain `..`, `\` or empty segments, so a lookup of
    // anything else misses; reject explicitly as defence in depth.
    let bad = path.contains('\\')
        || path
            .split('/')
            .any(|s| s.is_empty() || s == "." || s == "..");
    let Some(file) = (!bad).then(|| Assets::get(&path)).flatten() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let etag = format!(
        "\"{}\"",
        file.metadata
            .sha256_hash()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    // File names are not hashed, so browsers must revalidate on every use.
    let common = [
        (header::CACHE_CONTROL, "no-cache"),
        (header::ETAG, etag.as_str()),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
    ];
    if matches(headers.get(header::IF_NONE_MATCH), &etag) {
        return (StatusCode::NOT_MODIFIED, common).into_response();
    }
    (
        [(header::CONTENT_TYPE, mime_for(&path))],
        common,
        file.data.into_owned(),
    )
        .into_response()
}

/// `If-None-Match` hit: `*` or any listed tag, weak prefix ignored.
fn matches(header: Option<&HeaderValue>, etag: &str) -> bool {
    let Some(v) = header.and_then(|v| v.to_str().ok()) else {
        return false;
    };
    v.split(',')
        .map(|t| t.trim().trim_start_matches("W/"))
        .any(|t| t == "*" || t == etag)
}

fn mime_for(path: &str) -> &'static str {
    match path
        .rsplit('.')
        .next()
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        Some("map" | "json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mime_types_by_extension() {
        for (path, mime) in [
            ("app.js", "text/javascript; charset=utf-8"),
            ("app.css", "text/css; charset=utf-8"),
            ("a/index.html", "text/html; charset=utf-8"),
            ("app.js.map", "application/json"),
            ("i.ico", "image/x-icon"),
            ("p.JPG", "image/jpeg"),
            ("p.jpeg", "image/jpeg"),
            ("p.webp", "image/webp"),
            ("f.woff", "font/woff"),
            ("f.woff2", "font/woff2"),
            ("l.svg", "image/svg+xml"),
            ("p.png", "image/png"),
            ("blob.bin", "application/octet-stream"),
            ("noext", "application/octet-stream"),
        ] {
            assert_eq!(mime_for(path), mime, "{path}");
        }
    }

    #[test]
    fn if_none_match_forms() {
        let h = |s: &str| HeaderValue::from_str(s).unwrap();
        assert!(matches(Some(&h("\"a\"")), "\"a\""));
        assert!(matches(Some(&h("W/\"a\"")), "\"a\""));
        assert!(matches(Some(&h("\"b\", \"a\"")), "\"a\""));
        assert!(matches(Some(&h("*")), "\"a\""));
        assert!(!matches(Some(&h("\"b\"")), "\"a\""));
        assert!(!matches(None, "\"a\""));
    }
}
