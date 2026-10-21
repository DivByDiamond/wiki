use crate::http::state::AppState;
use axum::Router;
pub fn routes() -> Router<AppState> {
    Router::new()
}
