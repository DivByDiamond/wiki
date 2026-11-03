//! Public serving of uploaded images. Names are content hashes, so a file never changes.

use crate::error::AppError;
use crate::http::state::AppState;
use axum::{
    Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/media/{name}", get(serve))
}

async fn serve(
    State(s): State<AppState>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> Response {
    let blob = match s.media.get(&name) {
        Ok(Some(b)) => b,
        Ok(None) => return AppError::NotFound.into_response(),
        Err(e) => return AppError::from(e).into_response(),
    };
    let etag = format!("\"{}\"", blob.sha256);
    let common = [
        (
            header::CACHE_CONTROL,
            "public, max-age=31536000, immutable".to_owned(),
        ),
        (header::ETAG, etag.clone()),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_owned()),
        // Even a hostile file opened directly can run nothing.
        (
            header::CONTENT_SECURITY_POLICY,
            "default-src 'none'; sandbox".to_owned(),
        ),
    ];
    let fresh = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.split(',').any(|t| t.trim() == etag));
    if fresh {
        return (StatusCode::NOT_MODIFIED, common).into_response();
    }
    (common, [(header::CONTENT_TYPE, blob.mime)], blob.data).into_response()
}
