use super::{
    html,
    view::{Shell, format_utc},
};
use crate::diff::{Line, diff_lines};
use crate::domain::{page::Page, revision::size_delta};
use crate::error::{AppError, AppResult};
use crate::http::state::AppState;
use askama::Template;
use axum::response::Response;

struct Row {
    id: i64,
    date: String,
    author: String,
    summary: String,
    delta: String,
    prev_id: Option<i64>,
}

struct DiffLine {
    kind: &'static str,
    text: String,
}

#[derive(Template)]
#[template(path = "reader/history.html")]
struct HistoryPage {
    shell: Shell,
    rows: Vec<Row>,
    diff: Option<Vec<DiffLine>>,
    diff_label: String,
}

pub fn history(s: &AppState, shell: Shell, page: &Page) -> AppResult<Response> {
    let revs = s.pages.history(page.id)?;
    // Newest first: the previous revision is the next element.
    let rows = revs
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let older = revs.get(i + 1);
            let d = size_delta(older.map(|o| o.body.as_str()), &r.body);
            Row {
                id: r.id,
                date: format_utc(r.created_at),
                author: r.author.label(),
                summary: r.summary.clone(),
                delta: format!("{d:+}"),
                prev_id: older.map(|o| o.id),
            }
        })
        .collect();
    html(&HistoryPage {
        shell,
        rows,
        diff: None,
        diff_label: String::new(),
    })
}

fn parse_range(range: &str) -> AppResult<(i64, i64)> {
    let bad = || AppError::Invalid("diff: ожидается A..B".into());
    let (a, b) = range.split_once("..").ok_or_else(bad)?;
    Ok((a.parse().map_err(|_| bad())?, b.parse().map_err(|_| bad())?))
}

pub fn diff(s: &AppState, shell: Shell, page: &Page, range: &str) -> AppResult<Response> {
    let (a, b) = parse_range(range)?;
    let old = s.pages.revision(page.id, a)?.ok_or(AppError::NotFound)?;
    let new = s.pages.revision(page.id, b)?.ok_or(AppError::NotFound)?;
    let lines = diff_lines(&old.body, &new.body)
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
        .collect();
    html(&HistoryPage {
        shell,
        rows: Vec::new(),
        diff: Some(lines),
        diff_label: format!("№{a} и №{b}"),
    })
}
