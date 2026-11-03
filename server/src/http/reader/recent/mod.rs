//! Recent changes page and Atom feed, both built from the newest revisions.

mod atom;

pub use atom::feed;

use super::{
    error_page, html, unknown_lang,
    view::{Shell, format_utc},
};
use crate::domain::page::Lang;
use crate::error::AppResult;
use crate::http::{middleware::Viewer, state::AppState};
use askama::Template;
use axum::{
    extract::{Path, State},
    response::Response,
};

const PAGE_LIMIT: u32 = 50;

struct Row {
    href: String,
    lang: &'static str,
    title: String,
    summary: String,
    author: String,
    date: String,
}

#[derive(Template)]
#[template(path = "reader/recent/list.html")]
struct RecentPage {
    shell: Shell,
    rows: Vec<Row>,
}

pub async fn recent(
    State(s): State<AppState>,
    viewer: Viewer,
    Path(lang): Path<String>,
) -> Response {
    let Some(lang) = Lang::parse(&lang) else {
        return unknown_lang();
    };
    render(&s, lang, viewer).unwrap_or_else(|e| error_page(lang, e))
}

fn render(s: &AppState, lang: Lang, viewer: Viewer) -> AppResult<Response> {
    let rows = s
        .recent
        .recent(PAGE_LIMIT)?
        .into_iter()
        .map(|c| Row {
            href: format!("/{}/{}?rev={}", c.lang.code(), c.slug.as_str(), c.rev_id),
            lang: c.lang.code(),
            title: c.title,
            summary: c.summary,
            author: c.author.label(),
            date: format_utc(c.created_at),
        })
        .collect();
    let mut shell = Shell::build(s, lang, "", "recent", "")?.for_viewer(viewer);
    shell.title = shell.text("recent").into();
    html(&RecentPage { shell, rows })
}
