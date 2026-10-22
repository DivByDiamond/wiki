use super::{fail, guard::check_origin};
use crate::auth::policy::{Action, Actor};
use crate::domain::page::{Lang, Slug};
use crate::error::{AppError, AppResult};
use crate::http::{
    limit::enforce,
    middleware::{Auth, Viewer},
    reader::{self, view::Shell},
    require,
    state::AppState,
};
use askama::Template;
use axum::{
    Form, Router,
    extract::{DefaultBodyLimit, Path, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use serde::Deserialize;

pub const MAX_OPEN: u32 = 5;
pub const MAX_PER_DAY: u32 = 20;
const MAX_BODY: usize = 1024 * 1024;
const MAX_SUMMARY: usize = 200;
const DAY_SECS: i64 = 86_400;
/// Transport cap for the url-encoded form (encoding can triple the size).
const MAX_FORM: usize = 4 * MAX_BODY;

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/propose/{lang}/{*slug}",
        get(form)
            .post(submit)
            .layer(DefaultBodyLimit::max(MAX_FORM)),
    )
}

/// Open and daily limits for one reader.
pub fn check_quota(open: u32, today: u32) -> Result<(), AppError> {
    if open >= MAX_OPEN || today >= MAX_PER_DAY {
        return Err(AppError::RateLimited(3600));
    }
    Ok(())
}

#[derive(Template)]
#[template(path = "user/propose.html")]
struct ProposePage {
    shell: Shell,
    base_rev: i64,
    body: String,
}

#[derive(Deserialize)]
pub struct ProposeForm {
    #[serde(default)]
    body: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    base_rev: String,
}

fn user_id(actor: &Actor) -> AppResult<i64> {
    match actor {
        Actor::User { id, .. } => Ok(*id),
        _ => Err(AppError::Forbidden),
    }
}

fn parse_target(lang: &str, slug: &str) -> Result<(Lang, Slug), Box<Response>> {
    let lang = Lang::parse(lang).ok_or_else(|| Box::new(reader::unknown_lang()))?;
    let slug = Slug::parse(slug).ok_or_else(|| Box::new(fail(lang, AppError::NotFound)))?;
    Ok((lang, slug))
}

async fn form(
    State(s): State<AppState>,
    Auth(actor): Auth,
    viewer: Viewer,
    Path((lang, slug)): Path<(String, String)>,
) -> Response {
    let (lang, slug) = match parse_target(&lang, &slug) {
        Ok(t) => t,
        Err(res) => return *res,
    };
    show(&s, &actor, viewer, lang, &slug).unwrap_or_else(|e| fail(lang, e))
}

fn show(
    s: &AppState,
    actor: &Actor,
    viewer: Viewer,
    lang: Lang,
    slug: &Slug,
) -> AppResult<Response> {
    require(actor, Action::Propose)?;
    let page = s.pages.find(slug, lang)?.ok_or(AppError::NotFound)?;
    let shell = Shell::build(s, lang, slug.as_str(), slug.as_str(), "")?.for_viewer(viewer);
    reader::html(&ProposePage {
        shell,
        base_rev: page.current.id,
        body: page.current.body,
    })
}

async fn submit(
    State(s): State<AppState>,
    Auth(actor): Auth,
    headers: HeaderMap,
    Path((lang, slug)): Path<(String, String)>,
    Form(f): Form<ProposeForm>,
) -> Response {
    let (lang, slug) = match parse_target(&lang, &slug) {
        Ok(t) => t,
        Err(res) => return *res,
    };
    store(&s, &actor, &headers, lang, &slug, f).unwrap_or_else(|e| fail(lang, e))
}

fn store(
    s: &AppState,
    actor: &Actor,
    headers: &HeaderMap,
    lang: Lang,
    slug: &Slug,
    f: ProposeForm,
) -> AppResult<Response> {
    check_origin(headers, &s.config)?;
    require(actor, Action::Propose)?;
    let uid = user_id(actor)?;
    enforce(s, "propose", &uid.to_string())?;
    let page = s.pages.find(slug, lang)?.ok_or(AppError::NotFound)?;
    // Browsers submit textareas with CRLF line breaks.
    let body = f.body.replace("\r\n", "\n");
    let summary = f.summary.trim();
    if body.trim().is_empty() {
        return Err(AppError::Invalid("The suggested text is empty.".into()));
    }
    if body.len() > MAX_BODY {
        return Err(AppError::Invalid("The text is longer than 1 MiB.".into()));
    }
    if summary.chars().count() > MAX_SUMMARY {
        return Err(AppError::Invalid(
            "The summary is longer than 200 characters.".into(),
        ));
    }
    // A base revision of another page (or garbage) falls back to the current one.
    let base = match f.base_rev.trim().parse::<i64>() {
        Ok(id) => s.pages.revision(page.id, id)?,
        Err(_) => None,
    }
    .unwrap_or_else(|| page.current.clone());
    if body == base.body {
        return Err(AppError::Invalid("The text is unchanged.".into()));
    }
    let since = crate::store::db::now() - DAY_SECS;
    check_quota(
        s.proposals.count_open_by_user(uid)?,
        s.proposals.count_since_by_user(uid, since)?,
    )?;
    s.proposals.create(page.id, base.id, &body, summary, uid)?;
    let to = format!("/{}/{}?proposed=1", lang.code(), slug.as_str());
    Ok(Redirect::to(&to).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quota_limits() {
        assert!(check_quota(0, 0).is_ok());
        assert!(check_quota(MAX_OPEN - 1, MAX_PER_DAY - 1).is_ok());
        assert!(matches!(
            check_quota(MAX_OPEN, 0),
            Err(AppError::RateLimited(_))
        ));
        assert!(matches!(
            check_quota(0, MAX_PER_DAY),
            Err(AppError::RateLimited(_))
        ));
    }
}
