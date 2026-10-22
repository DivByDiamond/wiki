use super::{DecideForm, MAX_BODY, MAX_NOTE, load, login_of};
use crate::auth::policy::Actor;
use crate::domain::{
    proposal::{Proposal, Status, is_conflict},
    revision::{Author, NewRevision},
};
use crate::error::{AppError, AppResult};
use crate::http::{
    admin::{admin_id, admin_login, fail},
    middleware::Auth,
    state::AppState,
    user::check_origin,
};
use axum::{
    Form,
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};

pub(super) async fn accept(
    State(s): State<AppState>,
    Auth(actor): Auth,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Form(f): Form<DecideForm>,
) -> Response {
    accept_proposal(&s, &actor, &headers, id, f).unwrap_or_else(fail)
}

fn accept_proposal(
    s: &AppState,
    actor: &Actor,
    headers: &HeaderMap,
    id: i64,
    f: DecideForm,
) -> AppResult<Response> {
    check_origin(headers, &s.config)?;
    let admin = admin_id(actor)?;
    let admin_name = admin_login(s, actor)?;
    let p = open_proposal(s, id)?;
    check_note(&f.note)?;
    let page = s.pages.find_by_id(p.page_id)?.ok_or(AppError::NotFound)?;
    let body = if is_conflict(p.base_rev_id, page.current.id) {
        f.body
            .ok_or_else(|| AppError::Invalid("Конфликт: пришлите итоговый текст".into()))?
            .replace("\r\n", "\n")
    } else {
        p.body.clone()
    };
    if body.trim().is_empty() || body.len() > MAX_BODY {
        return Err(AppError::Invalid("The text is empty or too long.".into()));
    }
    s.pages.save(
        &page.slug,
        page.lang,
        NewRevision {
            body,
            summary: format!("{} (принял {})", p.summary, admin_name),
            author: Author::User(login_of(s, p.user_id)?),
        },
    )?;
    s.proposals
        .decide(id, Status::Accepted, admin, f.note.trim())?;
    Ok(Redirect::to("/admin/proposals").into_response())
}

pub(super) async fn reject(
    State(s): State<AppState>,
    Auth(actor): Auth,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Form(f): Form<DecideForm>,
) -> Response {
    reject_proposal(&s, &actor, &headers, id, &f.note).unwrap_or_else(fail)
}

fn reject_proposal(
    s: &AppState,
    actor: &Actor,
    headers: &HeaderMap,
    id: i64,
    note: &str,
) -> AppResult<Response> {
    check_origin(headers, &s.config)?;
    let admin = admin_id(actor)?;
    open_proposal(s, id)?;
    check_note(note)?;
    s.proposals
        .decide(id, Status::Rejected, admin, note.trim())?;
    Ok(Redirect::to("/admin/proposals").into_response())
}

fn open_proposal(s: &AppState, id: i64) -> AppResult<Proposal> {
    let p = load(s, id)?;
    if p.status != Status::Open {
        return Err(AppError::Invalid(
            "The suggestion is already decided.".into(),
        ));
    }
    Ok(p)
}

fn check_note(note: &str) -> AppResult<()> {
    if note.chars().count() > MAX_NOTE {
        return Err(AppError::Invalid("The note is too long.".into()));
    }
    Ok(())
}
