use super::{MAX_BODY, MAX_FORM, admin_login, admin_only, fail};
use crate::auth::policy::Actor;
use crate::domain::{
    page::{Lang, Slug},
    revision::{Author, NewRevision, Revision, rollback_to},
};
use crate::error::{AppError, AppResult};
use crate::http::{
    middleware::{Auth, Viewer},
    reader::{self, view::Shell},
    state::AppState,
    user::check_origin,
};
use askama::Template;
use axum::{
    Form, Router,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use serde::Deserialize;

const MAX_SUMMARY: usize = 200;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin", get(dashboard))
        .route(
            "/admin/edit/{lang}/{*slug}",
            get(edit_form)
                .post(edit_save)
                .layer(DefaultBodyLimit::max(MAX_FORM)),
        )
        .route("/admin/rollback/{lang}/{*slug}", post(rollback))
}

struct Row {
    lang: &'static str,
    slug: String,
}

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
struct Dashboard {
    shell: Shell,
    pages: Vec<Row>,
    deleted: Vec<Row>,
}

struct HistoryRow {
    id: i64,
    summary: String,
    author: String,
    current: bool,
}

#[derive(Template)]
#[template(path = "admin/editor.html")]
struct Editor {
    shell: Shell,
    lang: &'static str,
    slug: String,
    body: String,
    is_new: bool,
    same: bool,
    history: Vec<HistoryRow>,
}

#[derive(Deserialize)]
pub struct EditForm {
    #[serde(default)]
    body: String,
    #[serde(default)]
    summary: String,
}

#[derive(Deserialize)]
struct EditQuery {
    same: Option<String>,
}

#[derive(Deserialize)]
pub struct RollbackForm {
    #[serde(default)]
    rev: String,
}

fn rows(list: Vec<crate::domain::ports::pages::PageSummary>) -> Vec<Row> {
    list.into_iter()
        .map(|p| Row {
            lang: p.lang.code(),
            slug: p.slug.as_str().to_owned(),
        })
        .collect()
}

pub(super) fn target(lang: &str, slug: &str) -> AppResult<(Lang, Slug)> {
    let lang = Lang::parse(lang).ok_or(AppError::NotFound)?;
    let slug = Slug::parse(slug).ok_or(AppError::NotFound)?;
    Ok((lang, slug))
}

async fn dashboard(State(s): State<AppState>, Auth(actor): Auth, viewer: Viewer) -> Response {
    show_dashboard(&s, &actor, viewer).unwrap_or_else(fail)
}

fn show_dashboard(s: &AppState, actor: &Actor, viewer: Viewer) -> AppResult<Response> {
    admin_only(actor)?;
    reader::html(&Dashboard {
        shell: Shell::bare(Lang::En, "Admin", "start").for_viewer(viewer),
        pages: rows(s.pages.list()?),
        deleted: rows(s.pages.list_deleted()?),
    })
}

async fn edit_form(
    State(s): State<AppState>,
    Auth(actor): Auth,
    viewer: Viewer,
    Path((lang, slug)): Path<(String, String)>,
    Query(q): Query<EditQuery>,
) -> Response {
    show_editor(&s, &actor, viewer, &lang, &slug, q.same.is_some()).unwrap_or_else(fail)
}

fn show_editor(
    s: &AppState,
    actor: &Actor,
    viewer: Viewer,
    lang: &str,
    slug: &str,
    same: bool,
) -> AppResult<Response> {
    admin_only(actor)?;
    let (lang, slug) = target(lang, slug)?;
    let page = s.pages.find(&slug, lang)?;
    let (body, history) = match &page {
        Some(p) => {
            let history = s.pages.history(p.id)?;
            let rows = history
                .iter()
                .map(|r| HistoryRow {
                    id: r.id,
                    summary: r.summary.clone(),
                    author: r.author.label(),
                    current: r.id == p.current.id,
                })
                .collect();
            (p.current.body.clone(), rows)
        }
        None => (String::new(), Vec::new()),
    };
    reader::html(&Editor {
        shell: Shell::bare(Lang::En, "Edit", "start").for_viewer(viewer),
        lang: lang.code(),
        slug: slug.as_str().to_owned(),
        body,
        is_new: page.is_none(),
        same,
        history,
    })
}

async fn edit_save(
    State(s): State<AppState>,
    Auth(actor): Auth,
    headers: HeaderMap,
    Path((lang, slug)): Path<(String, String)>,
    Form(f): Form<EditForm>,
) -> Response {
    save_edit(&s, &actor, &headers, &lang, &slug, f).unwrap_or_else(fail)
}

fn save_edit(
    s: &AppState,
    actor: &Actor,
    headers: &HeaderMap,
    lang: &str,
    slug: &str,
    f: EditForm,
) -> AppResult<Response> {
    check_origin(headers, &s.config)?;
    let login = admin_login(s, actor)?;
    let (lang, slug) = target(lang, slug)?;
    // Browsers submit textareas with CRLF line breaks.
    let body = f.body.replace("\r\n", "\n");
    let summary = f.summary.trim();
    if body.trim().is_empty() {
        return Err(AppError::Invalid("The text is empty.".into()));
    }
    if body.len() > MAX_BODY {
        return Err(AppError::Invalid("The text is longer than 1 MiB.".into()));
    }
    if summary.chars().count() > MAX_SUMMARY {
        return Err(AppError::Invalid(
            "The summary is longer than 200 characters.".into(),
        ));
    }
    let rev = NewRevision {
        body,
        summary: summary.to_owned(),
        author: Author::User(login),
    };
    s.pages.save(&slug, lang, rev)?;
    Ok(Redirect::to(&format!("/{}/{}", lang.code(), slug.as_str())).into_response())
}

async fn rollback(
    State(s): State<AppState>,
    Auth(actor): Auth,
    headers: HeaderMap,
    Path((lang, slug)): Path<(String, String)>,
    Form(f): Form<RollbackForm>,
) -> Response {
    roll_back(&s, &actor, &headers, &lang, &slug, &f.rev).unwrap_or_else(fail)
}

/// Rollback appends a new revision with the old text; history is never rewritten.
fn roll_back(
    s: &AppState,
    actor: &Actor,
    headers: &HeaderMap,
    lang: &str,
    slug: &str,
    rev: &str,
) -> AppResult<Response> {
    check_origin(headers, &s.config)?;
    let login = admin_login(s, actor)?;
    let (lang, slug) = target(lang, slug)?;
    let page = s.pages.find(&slug, lang)?.ok_or(AppError::NotFound)?;
    let rev_id = rev
        .trim()
        .parse::<i64>()
        .map_err(|_| AppError::Invalid("rev".into()))?;
    let edit = format!("/admin/edit/{}/{}", lang.code(), slug.as_str());
    if rev_id == page.current.id {
        return Ok(Redirect::to(&format!("{edit}?same=1")).into_response());
    }
    let old: Revision = s
        .pages
        .revision(page.id, rev_id)?
        .ok_or(AppError::NotFound)?;
    s.pages
        .save(&slug, lang, rollback_to(&old, Author::User(login)))?;
    Ok(Redirect::to(&edit).into_response())
}
