mod articles;
mod history;
mod notfound;
mod search;
pub(crate) mod view;

use super::state::AppState;
use crate::domain::page::{Lang, Slug};
use crate::error::{AppError, AppResult};
use crate::http::middleware::Viewer;
use askama::Template;
use axum::{
    Router,
    extract::{OriginalUri, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use view::Shell;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(|| async { Redirect::to("/en/start") }))
        .route("/{lang}", get(lang_root))
        .route("/{lang}/search", get(search::search))
        .route("/{lang}/{*slug}", get(articles::article))
        .fallback(fallback)
}

/// `/en` on its own: send the reader to that language's start page.
async fn lang_root(Path(lang): Path<String>) -> Response {
    match Lang::parse(&lang) {
        Some(lang) => Redirect::to(&format!("/{}/start", lang.code())).into_response(),
        None => unknown_lang(),
    }
}

/// Unmatched paths (`/favicon.ico`, typos) get the reader 404 page, not a bare body.
async fn fallback(State(s): State<AppState>, viewer: Viewer, uri: OriginalUri) -> Response {
    let path = uri.path().trim_matches('/');
    let mut parts = path.splitn(2, '/');
    let lang = parts.next().and_then(Lang::parse).unwrap_or(Lang::En);
    let slug = parts.next().unwrap_or_default();
    notfound::render(&s, lang, slug, viewer).unwrap_or_else(|e| error_page(lang, e))
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
        AppError::Conflict(m) => (StatusCode::CONFLICT, "Conflict", m.clone()),
        AppError::Unauthorized => (
            StatusCode::UNAUTHORIZED,
            "Нужен вход",
            "Войдите и попробуйте снова.".to_string(),
        ),
        AppError::Forbidden => (
            StatusCode::FORBIDDEN,
            "Недостаточно прав",
            "У вас нет доступа к этой странице.".to_string(),
        ),
        AppError::TooLarge => (
            StatusCode::PAYLOAD_TOO_LARGE,
            "Запрос слишком большой",
            "Тело запроса больше допустимого.".to_string(),
        ),
        AppError::RateLimited(_) => (
            StatusCode::TOO_MANY_REQUESTS,
            "Слишком много запросов",
            "Подождите немного и повторите.".to_string(),
        ),
        // Reader pages always answer with HTML, even when the store fails.
        AppError::Internal(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Ошибка сервера",
            "Что-то пошло не так. Попробуйте позже.".to_string(),
        ),
    };
    let page = ErrorPage {
        shell: Shell::bare(lang, heading, "start"),
        heading,
        message: &message,
    };
    match page.render() {
        Ok(body) => {
            let mut res = (status, Html(body)).into_response();
            if let AppError::RateLimited(secs) = err {
                res.headers_mut().insert("retry-after", secs.into());
            }
            res
        }
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
