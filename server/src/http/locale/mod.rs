//! Request language: explicit URL, remembered choice, browser preference, English.

mod preference;
#[cfg(test)]
mod tests;

use super::{middleware::cookie, state::AppState, user::guard::safe_local_redirect};
use crate::domain::page::Lang;
use axum::{
    extract::{Path, Query, Request, State},
    http::{HeaderMap, Uri, header},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

const COOKIE: &str = "wiki_language";

#[derive(Clone)]
pub struct Locale {
    pub lang: Lang,
    pub prefixed: bool,
    pub path: String,
}

impl Locale {
    pub fn resolve(headers: &HeaderMap, uri: &Uri) -> Self {
        let explicit = uri.path().split('/').nth(1).and_then(Lang::parse);
        Self {
            lang: explicit.unwrap_or_else(|| preferred(headers)),
            prefixed: explicit.is_some(),
            path: uri.path_and_query().map_or("/", |p| p.as_str()).into(),
        }
    }
}

pub fn preferred(headers: &HeaderMap) -> Lang {
    cookie(headers, COOKIE)
        .and_then(Lang::parse)
        .unwrap_or_else(|| preference::browser(headers))
}

/// Explicit language URLs persist across public pages, admin, OAuth and logout.
pub async fn remember(State(s): State<AppState>, req: Request, next: Next) -> Response {
    let locale = Locale::resolve(req.headers(), req.uri());
    let needs_cookie = locale.prefixed && cookie(req.headers(), COOKIE) != Some(locale.lang.code());
    let mut response = next.run(req).await;
    let private = locale.path.starts_with("/admin") || locale.path.starts_with("/auth/");
    let html = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("text/html"));
    if private || html {
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            if private {
                "private, no-store"
            } else {
                "no-cache"
            }
            .parse()
            .unwrap(),
        );
    }
    if needs_cookie {
        set_preference(&mut response, locale.lang, s.config.secure_cookies());
    }
    response
        .headers_mut()
        .append(header::VARY, "Accept-Language, Cookie".parse().unwrap());
    response
}

fn set_preference(response: &mut Response, lang: Lang, secure: bool) {
    let value = super::user::guard::set_cookie(COOKIE, lang.code(), 31_536_000, secure);
    response
        .headers_mut()
        .append(header::SET_COOKIE, value.parse().unwrap());
}

pub async fn home(headers: HeaderMap) -> Redirect {
    Redirect::to(&format!("/{}/start", preferred(&headers).code()))
}

#[derive(Deserialize)]
pub struct SwitchQuery {
    next: Option<String>,
}

pub async fn switch(
    State(s): State<AppState>,
    Path(lang): Path<String>,
    Query(q): Query<SwitchQuery>,
) -> Response {
    let Some(lang) = Lang::parse(&lang) else {
        return super::reader::unknown_lang();
    };
    let fallback = format!("/{}/start", lang.code());
    let target = q
        .next
        .as_deref()
        .and_then(|next| safe_local_redirect(next, 4096))
        .unwrap_or(&fallback);
    let mut response = Redirect::to(target).into_response();
    set_preference(&mut response, lang, s.config.secure_cookies());
    response
}

/// Percent-encode a complete query value; never treat an embedded `&` as a parameter.
pub fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}
