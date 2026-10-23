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
    accept_proposal(&s, &actor, &headers, id, f)
        .unwrap_or_else(|e| fail(crate::http::locale::preferred(&headers), e))
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
    if page.deleted {
        return Err(AppError::Conflict(
            "The page was deleted. Restore it first or reject the suggestion.".into(),
        ));
    }
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
    let author = Author::User(login_of(s, p.user_id)?);
    // Claim first: only one admin can win, so a revision is published at most once.
    claim(s, id, Status::Accepted, admin, f.note.trim())?;
    let rev = NewRevision {
        body,
        summary: format!("{} (принял {})", p.summary, admin_name),
        author,
    };
    if let Err(e) = s.pages.save(&page.slug, page.lang, rev) {
        s.proposals.reopen(id)?;
        return Err(e.into());
    }
    Ok(Redirect::to("/admin/proposals").into_response())
}

pub(super) async fn reject(
    State(s): State<AppState>,
    Auth(actor): Auth,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Form(f): Form<DecideForm>,
) -> Response {
    reject_proposal(&s, &actor, &headers, id, &f.note)
        .unwrap_or_else(|e| fail(crate::http::locale::preferred(&headers), e))
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
    claim(s, id, Status::Rejected, admin, note.trim())?;
    Ok(Redirect::to("/admin/proposals").into_response())
}

const DECIDED: &str = "The suggestion is already decided.";

fn open_proposal(s: &AppState, id: i64) -> AppResult<Proposal> {
    let p = load(s, id)?;
    if p.status != Status::Open {
        return Err(AppError::Conflict(DECIDED.into()));
    }
    Ok(p)
}

/// Closes the proposal; losing the race against another admin is a 409.
fn claim(s: &AppState, id: i64, status: Status, admin: i64, note: &str) -> AppResult<()> {
    if s.proposals.claim(id, status, admin, note)? {
        Ok(())
    } else {
        Err(AppError::Conflict(DECIDED.into()))
    }
}

fn check_note(note: &str) -> AppResult<()> {
    if note.chars().count() > MAX_NOTE {
        return Err(AppError::Invalid("The note is too long.".into()));
    }
    Ok(())
}
