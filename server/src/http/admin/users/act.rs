use super::guard::{Change, check};
use crate::auth::policy::Actor;
use crate::error::{AppError, AppResult};
use crate::http::{
    admin::{admin_id, fail},
    middleware::Auth,
    state::AppState,
    user::check_origin,
};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};

pub(super) async fn change(
    State(s): State<AppState>,
    Auth(actor): Auth,
    headers: HeaderMap,
    Path((id, action)): Path<(i64, String)>,
) -> Response {
    apply(&s, &actor, &headers, id, &action).unwrap_or_else(fail)
}

fn apply(
    s: &AppState,
    actor: &Actor,
    headers: &HeaderMap,
    id: i64,
    action: &str,
) -> AppResult<Response> {
    check_origin(headers, &s.config)?;
    let me = admin_id(actor)?;
    let change = Change::parse(action).ok_or(AppError::NotFound)?;
    let target = s.access.user(id)?.ok_or(AppError::NotFound)?;
    check(change, me, &target, s.access.count_active_admins()?)?;
    match change {
        Change::Block => s.access.set_blocked(id, true)?,
        Change::Unblock => s.access.set_blocked(id, false)?,
        Change::GrantAdmin => s.access.set_admin(id, true)?,
        Change::RevokeAdmin => s.access.set_admin(id, false)?,
    }
    Ok(Redirect::to("/admin/users").into_response())
}
