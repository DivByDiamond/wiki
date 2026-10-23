//! Admin panel: pages, edit suggestions and API keys. Every route is admin-only.

mod keys;
mod pages;
mod proposals;
mod users;

use super::{
    middleware::Auth,
    state::AppState,
    user::{self, check_origin},
};
use crate::auth::policy::{Action, Actor};
use crate::domain::page::{Lang, Slug};
use crate::error::{AppError, AppResult};
use crate::render;
use axum::{
    Router,
    extract::{DefaultBodyLimit, Path, State},
    http::HeaderMap,
    response::{Html, IntoResponse, Redirect, Response},
    routing::post,
};

/// Largest Markdown text an admin may save, preview or merge.
pub(super) const MAX_BODY: usize = 1024 * 1024;
/// Transport cap for the url-encoded form (encoding can triple the size).
pub(super) const MAX_FORM: usize = 4 * MAX_BODY;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(pages::routes())
        .route("/admin/delete/{lang}/{*slug}", post(delete))
        .route("/admin/restore/{lang}/{*slug}", post(restore))
        .route(
            "/admin/preview",
            post(preview).layer(DefaultBodyLimit::max(MAX_BODY)),
        )
        .merge(proposals::routes())
        .merge(keys::routes())
        .merge(users::routes())
}

/// The action every admin route is gated on (only admins pass it).
pub(super) const ADMIN: Action = Action::ManageUsers;

/// Admin gate: anonymous gets 401, everyone else without rights 403.
pub(super) fn admin_only(actor: &Actor) -> AppResult<()> {
    super::require(actor, ADMIN)
}

/// Id of the signed-in admin; API keys never act as an admin user.
pub(super) fn admin_id(actor: &Actor) -> AppResult<i64> {
    admin_only(actor)?;
    match actor {
        Actor::User { id, .. } => Ok(*id),
        _ => Err(AppError::Forbidden),
    }
}

/// GitHub login of the signed-in admin, used as the revision author.
pub(super) fn admin_login(s: &AppState, actor: &Actor) -> AppResult<String> {
    let id = admin_id(actor)?;
    Ok(s.access.user(id)?.ok_or(AppError::Forbidden)?.login)
}

/// Parses `{lang}/{slug}` path segments shared by every page route.
pub(super) fn target(lang: &str, slug: &str) -> AppResult<(Lang, Slug)> {
    let lang = Lang::parse(lang).ok_or(AppError::NotFound)?;
    let slug = Slug::parse(slug).ok_or(AppError::NotFound)?;
    Ok((lang, slug))
}

/// HTML error page for admin routes.
pub(super) fn fail(lang: Lang, e: AppError) -> Response {
    user::fail(lang, e)
}

async fn delete(
    State(s): State<AppState>,
    Auth(actor): Auth,
    headers: HeaderMap,
    Path((lang, slug)): Path<(String, String)>,
) -> Response {
    set_deleted(&s, &actor, &headers, &lang, &slug, true)
        .unwrap_or_else(|e| fail(super::locale::preferred(&headers), e))
}

async fn restore(
    State(s): State<AppState>,
    Auth(actor): Auth,
    headers: HeaderMap,
    Path((lang, slug)): Path<(String, String)>,
) -> Response {
    set_deleted(&s, &actor, &headers, &lang, &slug, false)
        .unwrap_or_else(|e| fail(super::locale::preferred(&headers), e))
}

fn set_deleted(
    s: &AppState,
    actor: &Actor,
    headers: &HeaderMap,
    lang: &str,
    slug: &str,
    deleted: bool,
) -> AppResult<Response> {
    check_origin(headers, &s.config)?;
    admin_id(actor)?;
    let (lang, slug) = target(lang, slug)?;
    let id = if deleted {
        s.pages.find(&slug, lang)?.map(|p| p.id)
    } else {
        s.pages
            .list_deleted()?
            .into_iter()
            .find(|p| p.slug == slug && p.lang == lang)
            .map(|p| p.id)
    };
    let id = id.ok_or(AppError::NotFound)?;
    if !deleted && s.pages.find(&slug, lang)?.is_some() {
        return Err(taken());
    }
    s.pages.set_deleted(id, deleted).map_err(|e| {
        // Defensive: the unique index on live (slug, lang) caught a race.
        if !deleted && e.0.contains("UNIQUE") {
            taken()
        } else {
            e.into()
        }
    })?;
    Ok(Redirect::to("/admin").into_response())
}

fn taken() -> AppError {
    AppError::Conflict(
        "A live page with this address already exists. Delete or rename it first.".into(),
    )
}

/// Markdown in the request body, rendered HTML out (admin editor and reader
/// suggest page both use it, so any signed-in reader may call it).
async fn preview(
    State(s): State<AppState>,
    Auth(actor): Auth,
    headers: HeaderMap,
    body: String,
) -> Response {
    let run = || -> AppResult<Response> {
        check_origin(&headers, &s.config)?;
        super::require(&actor, Action::Propose)?;
        Ok(Html(render::to_html(&body)).into_response())
    };
    run().unwrap_or_else(|e| fail(super::locale::preferred(&headers), e))
}
