//! The reader 404 page: localized texts, a search box and similar pages.

mod similar;
mod texts;

use super::{html, view::Shell};
use crate::domain::page::Lang;
use crate::error::AppResult;
use crate::http::state::AppState;
use askama::Template;
use axum::{http::StatusCode, response::Response};
use texts::{Texts, texts};

#[derive(Template)]
#[template(path = "reader/notfound/page.html")]
struct NotFoundPage {
    shell: Shell,
    t: Texts,
    requested: String,
    similar: Vec<(String, String)>,
}

fn build(s: &AppState, lang: Lang, slug: &str) -> AppResult<NotFoundPage> {
    let pages = s.pages.list()?;
    let same_lang = pages.iter().filter(|p| p.lang == lang);
    let t = texts(lang);
    Ok(NotFoundPage {
        shell: Shell::build(s, lang, t.heading, "start", "")?,
        similar: similar::similar(slug, same_lang.map(|p| p.slug.as_str())),
        requested: format!("/{}/{slug}", lang.code()),
        t,
    })
}

/// Renders the 404 page for `slug` (the raw path after the language prefix).
pub(crate) fn render(s: &AppState, lang: Lang, slug: &str) -> AppResult<Response> {
    let mut res = html(&build(s, lang, slug)?)?;
    *res.status_mut() = StatusCode::NOT_FOUND;
    Ok(res)
}
