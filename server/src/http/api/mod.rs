mod export;
mod json_errors;
mod pages;

use super::state::AppState;
use crate::error::AppError;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware::map_response,
    routing::{any, get},
};

/// Largest accepted page body (1 MiB).
const MAX_BODY: usize = 1024 * 1024;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/pages", get(pages::list))
        .route(
            "/api/pages/{*slug}",
            get(pages::get_page)
                .put(pages::put_page)
                .layer(DefaultBodyLimit::max(MAX_BODY)),
        )
        .route("/api/revisions/{*slug}", get(pages::revisions))
        .route("/api/export", get(export::export))
        .route("/api/{*rest}", any(|| async { AppError::NotFound }))
        .layer(map_response(json_errors::to_json))
}
