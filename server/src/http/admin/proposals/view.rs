use super::{load, login_of};
use crate::auth::policy::Actor;
use crate::diff::{Line, MAX_DIFF_CELLS, diff_lines_bounded};
use crate::domain::{
    page::Lang,
    proposal::{Status, is_conflict},
};
use crate::error::{AppError, AppResult};
use crate::http::{
    admin::{admin_only, fail},
    middleware::Auth,
    reader::{self, view::Shell},
    state::AppState,
};
use askama::Template;
use axum::{
    extract::{Path, State},
    response::Response,
};

struct Row {
    id: i64,
    lang: &'static str,
    slug: String,
    summary: String,
    author: String,
    conflict: bool,
}

#[derive(Template)]
#[template(path = "admin/proposals/list.html")]
struct ListPage {
    shell: Shell,
    rows: Vec<Row>,
}

struct DiffLine {
    kind: &'static str,
    text: String,
}

#[derive(Template)]
#[template(path = "admin/proposals/detail.html")]
struct DetailPage {
    shell: Shell,
    id: i64,
    lang: &'static str,
    slug: String,
    summary: String,
    author: String,
    open: bool,
    page_deleted: bool,
    conflict: bool,
    base_rev: i64,
    current_rev: i64,
    base_diff: Option<Vec<DiffLine>>,
    current_diff: Option<Vec<DiffLine>>,
    base_body: String,
    current_body: String,
    body: String,
}

fn diff_view(old: &str, new: &str) -> Option<Vec<DiffLine>> {
    let lines = diff_lines_bounded(old, new, MAX_DIFF_CELLS)?;
    Some(
        lines
            .into_iter()
            .map(|l| match l {
                Line::Added(t) => DiffLine {
                    kind: "add",
                    text: t.into(),
                },
                Line::Removed(t) => DiffLine {
                    kind: "del",
                    text: t.into(),
                },
                Line::Same(t) => DiffLine {
                    kind: "same",
                    text: t.into(),
                },
            })
            .collect(),
    )
}

pub(super) async fn list(State(s): State<AppState>, Auth(actor): Auth) -> Response {
    show_list(&s, &actor).unwrap_or_else(fail)
}

fn show_list(s: &AppState, actor: &Actor) -> AppResult<Response> {
    admin_only(actor)?;
    let mut rows = Vec::new();
    for p in s.proposals.list(Status::Open)? {
        let Some(page) = s.pages.find_by_id(p.page_id)? else {
            continue;
        };
        rows.push(Row {
            id: p.id,
            lang: page.lang.code(),
            slug: page.slug.as_str().to_owned(),
            summary: p.summary.clone(),
            author: login_of(s, p.user_id)?,
            conflict: is_conflict(p.base_rev_id, page.current.id),
        });
    }
    reader::html(&ListPage {
        shell: Shell::bare(Lang::En, "Suggestions", "start"),
        rows,
    })
}

pub(super) async fn detail(
    State(s): State<AppState>,
    Auth(actor): Auth,
    Path(id): Path<i64>,
) -> Response {
    show_detail(&s, &actor, id).unwrap_or_else(fail)
}

fn show_detail(s: &AppState, actor: &Actor, id: i64) -> AppResult<Response> {
    admin_only(actor)?;
    let p = load(s, id)?;
    let page = s.pages.find_by_id(p.page_id)?.ok_or(AppError::NotFound)?;
    let base = s
        .pages
        .revision(page.id, p.base_rev_id)?
        .ok_or(AppError::NotFound)?;
    let conflict = is_conflict(p.base_rev_id, page.current.id);
    reader::html(&DetailPage {
        shell: Shell::bare(Lang::En, "Suggestion", "start"),
        id: p.id,
        lang: page.lang.code(),
        slug: page.slug.as_str().to_owned(),
        summary: p.summary.clone(),
        author: login_of(s, p.user_id)?,
        open: p.status == Status::Open,
        page_deleted: page.deleted,
        conflict,
        base_rev: base.id,
        current_rev: page.current.id,
        base_diff: diff_view(&base.body, &p.body),
        current_diff: diff_view(&page.current.body, &p.body),
        base_body: base.body,
        current_body: page.current.body.clone(),
        body: p.body,
    })
}
