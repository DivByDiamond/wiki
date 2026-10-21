//! Sign-in with GitHub and edit suggestions from readers.

mod guard;
mod login;
mod propose;

pub use guard::check_origin;

use super::{reader, state::AppState};
use crate::domain::page::Lang;
use crate::error::AppError;
use askama::Template;
use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use reader::view::Shell;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/github", get(login::start))
        .route("/auth/github/callback", get(login::callback))
        .route("/auth/logout", post(login::logout))
        .merge(propose::routes())
}

#[derive(Template)]
#[template(path = "user/notice.html")]
struct Notice<'a> {
    shell: Shell,
    heading: &'a str,
    message: &'a str,
    link: Option<(&'a str, &'a str)>,
}

/// Small HTML page with an optional (href, label) button.
pub(super) fn notice(
    status: StatusCode,
    lang: Lang,
    heading: &str,
    message: &str,
    link: Option<(&str, &str)>,
) -> Response {
    let page = Notice {
        shell: Shell::bare(lang, heading, "start"),
        heading,
        message,
        link,
    };
    match page.render() {
        Ok(body) => (status, Html(body)).into_response(),
        Err(e) => AppError::Internal(e.to_string()).into_response(),
    }
}

pub(super) fn sign_in_needed(lang: Lang) -> Response {
    notice(
        StatusCode::UNAUTHORIZED,
        lang,
        "Sign in required",
        "Sign in with your GitHub account to suggest an edit.",
        Some(("/auth/github", "Sign in with GitHub")),
    )
}

/// HTML error page for the sign-in and suggestion routes.
pub(super) fn fail(lang: Lang, err: AppError) -> Response {
    match err {
        AppError::Unauthorized => sign_in_needed(lang),
        AppError::Forbidden => notice(
            StatusCode::FORBIDDEN,
            lang,
            "Not allowed",
            "This request was refused.",
            None,
        ),
        other => reader::error_page(lang, other),
    }
}
