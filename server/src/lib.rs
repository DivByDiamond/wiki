pub mod auth;
pub mod config;
pub mod diff;
pub mod domain;
pub mod error;
pub mod http;
pub mod render;
pub mod store;

use axum::{Router, routing::get};
use http::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(http::api::routes())
        .merge(http::assets::routes())
        .merge(http::admin::routes())
        .merge(http::user::routes())
        .merge(http::reader::routes())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            http::locale::remember,
        ))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::db::open_memory;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_returns_ok() {
        let res = router(AppState::new(open_memory().unwrap()))
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
    }
}
