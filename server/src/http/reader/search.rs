mod filter;
mod highlight;

use super::{
    error_page, html, unknown_lang,
    view::{Shell, section_label, section_rank},
};
use filter::LangFilter;
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
    lang: Option<String>,
    section: Option<String>,
}

struct Hit {
    lang: String,
    title: String,
    slug: String,
    snippet: String,
    section: String,
}

struct SectionOption {
    key: String,
    label: String,
}

#[derive(Template)]
#[template(path = "reader/search.html")]
struct SearchPage {
    shell: Shell,
    hits: Vec<Hit>,
    sections: Vec<SectionOption>,
    lang_filter: &'static str,
    section_filter: String,
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
        let lang_filter = LangFilter::parse(q.lang.as_deref());
        let section_filter = q.section.as_deref().unwrap_or("").trim().to_owned();
        let mut batches = Vec::new();
        for l in lang_filter.searches(lang) {
            batches.push(s.pages.search(text, l)?);
        }
        let hits = filter::merge(batches, lang_filter)
            .into_iter()
            .filter_map(|h| {
                let page = titles
                    .iter()
                    .find(|p| p.slug == h.slug && p.lang == h.lang);
                let section = page.map(|p| p.section.clone()).unwrap_or_default();
                if !section_filter.is_empty() && section != section_filter {
                    return None;
                }
                Some(Hit {
                    title: page
                        .map(|p| p.title.clone())
                        .unwrap_or_else(|| h.slug.as_str().into()),
                    lang: h.lang.code().into(),
                    slug: h.slug.as_str().into(),
                    snippet: highlight::snippet(&h.snippet, text),
                    section,
                })
            })
            .collect();
        let mut keys: Vec<&str> = titles.iter().map(|p| p.section.as_str()).collect();
        keys.sort_by_key(|k| (section_rank(k), *k));
        keys.dedup();
        let sections = keys
            .into_iter()
            .filter(|k| !k.is_empty())
            .map(|k| SectionOption {
                key: k.into(),
                label: section_label(lang.code(), k),
            })
            .collect();
        let page = SearchPage {
            shell: Shell::build(&s, lang, "Поиск", "search", text)?.for_viewer(viewer),
            hits,
            sections,
            lang_filter: lang_filter.value(),
            section_filter,
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
