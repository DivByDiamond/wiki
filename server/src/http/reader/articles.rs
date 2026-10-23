use super::{
    error_page, history, html, notfound, parse_slug, unknown_lang,
    view::{Shell, format_utc},
};
use crate::domain::page::Lang;
use crate::error::{AppError, AppResult};
use crate::http::{
    limit::{client_ip::ClientIp, enforce},
    middleware::Viewer,
    state::AppState,
};
use crate::render;
use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::Response,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ArticleQuery {
    pub rev: Option<String>,
    pub history: Option<String>,
    pub diff: Option<String>,
    pub proposed: Option<String>,
}

pub struct OldRev {
    pub date: String,
    pub author: String,
}

#[derive(Template)]
#[template(path = "reader/article.html")]
struct ArticlePage {
    shell: Shell,
    html: String,
    old_rev: Option<OldRev>,
    missing_translation: bool,
    proposed: bool,
}

pub async fn article(
    State(s): State<AppState>,
    ClientIp(ip): ClientIp,
    viewer: Viewer,
    Path((lang, slug)): Path<(String, String)>,
    Query(q): Query<ArticleQuery>,
) -> Response {
    let Some(lang) = Lang::parse(&lang) else {
        return unknown_lang();
    };
    let signed_in = viewer.0.clone();
    handle(&s, &ip, lang, &slug, &q, viewer).unwrap_or_else(|e| match e {
        AppError::NotFound => notfound::render(&s, lang, &slug, Viewer(signed_in))
            .unwrap_or_else(|e| error_page(lang, e)),
        e => error_page(lang, e),
    })
}

fn handle(
    s: &AppState,
    ip: &str,
    lang: Lang,
    slug: &str,
    q: &ArticleQuery,
    viewer: Viewer,
) -> AppResult<Response> {
    let slug = parse_slug(slug)?;
    let (page, missing_translation) = match s.pages.find(&slug, lang)? {
        Some(p) => (p, false),
        None if lang != Lang::En => (
            s.pages.find(&slug, Lang::En)?.ok_or(AppError::NotFound)?,
            true,
        ),
        None => return Err(AppError::NotFound),
    };
    let mut shell = Shell::build(s, lang, slug.as_str(), slug.as_str(), "")?.for_viewer(viewer);
    if q.history.is_some() {
        enforce(s, "history", ip)?;
        return history::history(s, shell, &page);
    }
    if let Some(range) = &q.diff {
        enforce(s, "diff", ip)?;
        return history::diff(s, shell, &page, range);
    }
    let (revision, old_rev) = match &q.rev {
        Some(raw) => {
            let id: i64 = raw.parse().map_err(|_| AppError::Invalid("rev".into()))?;
            let r = s.pages.revision(page.id, id)?.ok_or(AppError::NotFound)?;
            let old = OldRev {
                date: format_utc(r.created_at),
                author: r.author.label(),
            };
            (r, Some(old))
        }
        None => (page.current.clone(), None),
    };
    let document = render::document::render(&revision.body);
    shell.title = render::document::title(&revision.body).unwrap_or_else(|| slug.as_str().into());
    shell.headings = document.headings;
    html(&ArticlePage {
        shell,
        html: document.html,
        old_rev,
        missing_translation,
        proposed: q.proposed.is_some(),
    })
}
