use crate::domain::proposal::Proposal;
use crate::error::{AppError, AppResult};
use crate::http::{
    admin::{MAX_BODY, MAX_FORM},
    state::AppState,
};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use serde::Deserialize;

mod decide;
mod view;

pub(super) const MAX_NOTE: usize = 1000;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin/proposals", get(view::list))
        .route("/admin/proposals/{id}", get(view::detail))
        .route(
            "/admin/proposals/{id}/accept",
            post(decide::accept).layer(DefaultBodyLimit::max(MAX_FORM)),
        )
        .route("/admin/proposals/{id}/reject", post(decide::reject))
}

#[derive(Deserialize)]
pub(super) struct DecideForm {
    pub(super) body: Option<String>,
    #[serde(default)]
    pub(super) note: String,
}

pub(super) fn login_of(s: &AppState, user_id: i64) -> AppResult<String> {
    Ok(s.access.user(user_id)?.map(|u| u.login).unwrap_or_default())
}

pub(super) fn load(s: &AppState, id: i64) -> AppResult<Proposal> {
    s.proposals.get(id)?.ok_or(AppError::NotFound)
}
