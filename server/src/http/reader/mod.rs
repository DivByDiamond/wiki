mod articles;
mod history;
mod search;
mod view;

use super::state::AppState;
use crate::domain::page::{Lang, Slug};
use crate::error::{AppError, AppResult};
use askama::Template;
use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use view::Shell;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(|| async { Redirect::to("/en/start") }))
        .route("/{lang}/search", get(search::search))
        .route("/{lang}/{*slug}", get(articles::article))
}

#[derive(Template)]
#[template(path = "reader/not_found.html")]
struct ErrorPage<'a> {
    shell: Shell,
    heading: &'a str,
    message: &'a str,
}

pub(crate) fn html<T: Template>(t: &T) -> AppResult<Response> {
    t.render()
        .map(|body| Html(body).into_response())
        .map_err(|e| AppError::Internal(e.to_string()))
}

/// HTML error page for readers; JSON errors stay on `/api`.
pub(crate) fn error_page(lang: Lang, err: AppError) -> Response {
    let (status, heading, message) = match &err {
        AppError::NotFound => (
            StatusCode::NOT_FOUND,
            "Страница не найдена",
            "Такой страницы нет. Попробуйте поиск.".to_string(),
        ),
        AppError::Invalid(m) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "Неверный запрос",
            m.clone(),
        ),
        _ => return err.into_response(),
    };
    let page = ErrorPage {
        shell: Shell::bare(lang, heading, "start"),
        heading,
        message: &message,
    };
    match page.render() {
        Ok(body) => (status, Html(body)).into_response(),
        Err(e) => AppError::Internal(e.to_string()).into_response(),
    }
}

/// Unknown language prefix: a plain reader 404.
pub(crate) fn unknown_lang() -> Response {
    error_page(Lang::En, AppError::NotFound)
}

pub(crate) fn parse_slug(s: &str) -> AppResult<Slug> {
    Slug::parse(s).ok_or(AppError::NotFound)
}
