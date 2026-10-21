use super::{error_page, html, unknown_lang, view::Shell};
use crate::domain::page::Lang;
use crate::error::{AppError, AppResult};
use crate::http::state::AppState;
use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
    fragment: Option<String>,
}

struct Hit {
    slug: String,
    snippet: String,
}

#[derive(Template)]
#[template(path = "reader/search.html")]
struct SearchPage {
    shell: Shell,
    hits: Vec<Hit>,
}

#[derive(Template)]
#[template(path = "reader/search.html", block = "results")]
struct Fragment<'a> {
    shell: &'a Shell,
    hits: &'a [Hit],
}

pub async fn search(
    State(s): State<AppState>,
    Path(lang): Path<String>,
    Query(q): Query<SearchQuery>,
) -> Response {
    let Some(lang) = Lang::parse(&lang) else {
        return unknown_lang();
    };
    let run = || -> AppResult<Response> {
        let text = q.q.as_deref().unwrap_or("").trim();
        let hits = s
            .pages
            .search(text, lang)?
            .into_iter()
            .map(|h| Hit {
                slug: h.slug.as_str().into(),
                snippet: h.snippet,
            })
            .collect();
        let page = SearchPage {
            shell: Shell::build(&s, lang, "Поиск", "search", text)?,
            hits,
        };
        if q.fragment.is_some() {
            let body = Fragment {
                shell: &page.shell,
                hits: &page.hits,
            }
            .render()
            .map_err(|e| AppError::Internal(e.to_string()))?;
            return Ok(Html(body).into_response());
        }
        html(&page)
    };
    run().unwrap_or_else(|e| error_page(lang, e))
}
