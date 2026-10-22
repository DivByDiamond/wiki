//! Static frontend assets (`web/dist`) embedded into the binary.
//! `build.rs` guarantees the folder exists at compile time.

use super::state::AppState;
use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../web/dist"]
struct Assets;

pub fn routes() -> Router<AppState> {
    Router::new().route("/static/{*path}", get(serve))
}

async fn serve(Path(path): Path<String>) -> Response {
    // Embedded keys never contain `..`, `\` or empty segments, so a lookup of
    // anything else misses; reject explicitly as defence in depth.
    let bad = path.contains('\\')
        || path
            .split('/')
            .any(|s| s.is_empty() || s == "." || s == "..");
    let Some(file) = (!bad).then(|| Assets::get(&path)).flatten() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let mime = match path.rsplit('.').next() {
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("map") | Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("woff2") => "font/woff2",
        Some("png") => "image/png",
        _ => "application/octet-stream",
    };
    (
        [
            (header::CONTENT_TYPE, mime),
            (header::CACHE_CONTROL, "public, max-age=3600"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        file.data.into_owned(),
    )
        .into_response()
}
