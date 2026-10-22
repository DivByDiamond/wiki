use super::{
    error_page, history, html, notfound, parse_slug, unknown_lang,
    view::{Shell, format_utc},
};
use crate::domain::page::Lang;
use crate::error::{AppError, AppResult};
use crate::http::{
    limit::{client_ip::ClientIp, enforce},
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
    Path((lang, slug)): Path<(String, String)>,
    Query(q): Query<ArticleQuery>,
) -> Response {
    let Some(lang) = Lang::parse(&lang) else {
        return unknown_lang();
    };
    handle(&s, &ip, lang, &slug, &q).unwrap_or_else(|e| match e {
        AppError::NotFound => {
            notfound::render(&s, lang, &slug).unwrap_or_else(|e| error_page(lang, e))
        }
        e => error_page(lang, e),
    })
}

fn handle(s: &AppState, ip: &str, lang: Lang, slug: &str, q: &ArticleQuery) -> AppResult<Response> {
    let slug = parse_slug(slug)?;
    let (page, missing_translation) = match s.pages.find(&slug, lang)? {
        Some(p) => (p, false),
        None if lang != Lang::En => (
            s.pages.find(&slug, Lang::En)?.ok_or(AppError::NotFound)?,
            true,
        ),
        None => return Err(AppError::NotFound),
    };
    let shell = Shell::build(s, lang, slug.as_str(), slug.as_str(), "")?;
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
    html(&ArticlePage {
        shell,
        html: render::to_html(&revision.body),
        old_rev,
        missing_translation,
        proposed: q.proposed.is_some(),
    })
}
