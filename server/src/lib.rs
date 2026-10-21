pub mod auth;
pub mod config;
pub mod diff;
pub mod domain;
pub mod error;
pub mod render;

use axum::{Router, routing::get};

pub fn router() -> Router {
    Router::new().route("/health", get(|| async { "ok" }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_returns_ok() {
        let res = router()
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
    }
}
