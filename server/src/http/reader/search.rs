mod highlight;

use super::{error_page, html, unknown_lang, view::Shell};
use crate::domain::page::Lang;
use crate::error::{AppError, AppResult};
use crate::http::{
    limit::{client_ip::ClientIp, enforce},
    middleware::Viewer,
    state::AppState,
};
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
    lang: String,
    title: String,
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
    hits: &'a [Hit],
}

pub async fn search(
    State(s): State<AppState>,
    ClientIp(ip): ClientIp,
    viewer: Viewer,
    Path(lang): Path<String>,
    Query(q): Query<SearchQuery>,
) -> Response {
    let Some(lang) = Lang::parse(&lang) else {
        return unknown_lang();
    };
    let run = || -> AppResult<Response> {
        enforce(&s, "search", &ip)?;
        let text = q.q.as_deref().unwrap_or("").trim();
        let titles = s.pages.list()?;
        let hits = s
            .pages
            .search(text, lang)?
            .into_iter()
            .map(|h| Hit {
                title: titles
                    .iter()
                    .find(|p| p.slug == h.slug && p.lang == h.lang)
                    .map(|p| p.title.clone())
                    .unwrap_or_else(|| h.slug.as_str().into()),
                lang: h.lang.code().into(),
                slug: h.slug.as_str().into(),
                snippet: highlight::snippet(&h.snippet, text),
            })
            .collect();
        let page = SearchPage {
            shell: Shell::build(&s, lang, "Поиск", "search", text)?.for_viewer(viewer),
            hits,
        };
        if q.fragment.is_some() {
            let body = Fragment { hits: &page.hits }
                .render()
                .map_err(|e| AppError::Internal(e.to_string()))?;
            return Ok(Html(body).into_response());
        }
        html(&page)
    };
    run().unwrap_or_else(|e| error_page(lang, e))
}
