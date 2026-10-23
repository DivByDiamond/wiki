use super::{
    fail,
    guard::{check_origin, ct_eq, safe_next, set_cookie},
    notice,
};
use crate::auth::tokens::session::{ABSOLUTE_SECS, random_token};
use crate::domain::page::Lang;
use crate::error::{AppError, AppResult};
use crate::http::{
    limit::{client_ip::ClientIp, enforce},
    middleware::cookie,
    state::AppState,
};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::{AppendHeaders, IntoResponse, Redirect, Response},
};
use serde::Deserialize;

const STATE_COOKIE: &str = "oauth_state";
const NEXT_COOKIE: &str = "oauth_next";
const STATE_TTL_SECS: u64 = 600;

#[derive(Deserialize)]
pub struct StartQuery {
    next: Option<String>,
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
}

fn not_configured(lang: Lang) -> Response {
    notice(
        StatusCode::SERVICE_UNAVAILABLE,
        lang,
        "Sign-in is not configured",
        "This site has no GitHub credentials yet, so sign-in is unavailable.",
        None,
    )
}

fn guard(s: &AppState, ip: &str) -> AppResult<()> {
    enforce(s, "login", ip)
}

/// `GET /auth/github`: remember a random `state` in a cookie, go to GitHub.
pub async fn start(
    State(s): State<AppState>,
    ClientIp(ip): ClientIp,
    Query(q): Query<StartQuery>,
    headers: HeaderMap,
) -> Response {
    let lang = crate::http::locale::preferred(&headers);
    if !s.config.github_configured() {
        return not_configured(lang);
    }
    if let Err(e) = guard(&s, &ip) {
        return fail(lang, e);
    }
    let state = random_token();
    let secure = s.config.secure_cookies();
    let mut cookies = vec![set_cookie(STATE_COOKIE, &state, STATE_TTL_SECS, secure)];
    if let Some(next) = q.next.as_deref().and_then(safe_next) {
        cookies.push(set_cookie(NEXT_COOKIE, next, STATE_TTL_SECS, secure));
    }
    (
        AppendHeaders(cookies.into_iter().map(|c| (SET_COOKIE, c))),
        Redirect::to(&s.github.authorize_url(&state)),
    )
        .into_response()
}

fn bad_state(lang: Lang) -> Response {
    notice(
        StatusCode::BAD_REQUEST,
        lang,
        "Sign-in failed",
        "The sign-in request is invalid or expired.",
        Some(("/auth/github", "Sign in again")),
    )
}

/// `GET /auth/github/callback`: verify `state`, exchange the code, open a session.
pub async fn callback(
    State(s): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Query(q): Query<CallbackQuery>,
) -> Response {
    let lang = crate::http::locale::preferred(&headers);
    if !s.config.github_configured() {
        return not_configured(lang);
    }
    if let Err(e) = guard(&s, &ip) {
        return fail(lang, e);
    }
    let expected = cookie(&headers, STATE_COOKIE).unwrap_or_default();
    let given = q.state.as_deref().unwrap_or_default();
    let code = q.code.as_deref().unwrap_or_default();
    if expected.is_empty() || code.is_empty() || !ct_eq(expected, given) {
        return bad_state(lang);
    }
    let user = match s.github.exchange(code).await {
        Ok(u) => u,
        Err(e) => {
            tracing::warn!("github sign-in failed: {e}");
            return notice(
                StatusCode::BAD_GATEWAY,
                lang,
                "GitHub did not answer",
                "Could not complete the sign-in with GitHub.",
                Some(("/auth/github", "Sign in again")),
            );
        }
    };
    finish(&s, &headers, user.id, &user.login).unwrap_or_else(|e| fail(lang, e))
}

fn finish(s: &AppState, headers: &HeaderMap, github_id: i64, login: &str) -> AppResult<Response> {
    let user = s.access.upsert_user(github_id, login)?;
    if user.blocked {
        return Err(AppError::Forbidden);
    }
    // Promote by the stable numeric id only (logins can be renamed and re-claimed);
    // removing the admin flag is a deliberate admin action.
    if s.config.is_admin_id(github_id) && !user.admin {
        s.access.set_admin(user.id, true)?;
    }
    if let Some(old) = cookie(headers, "sid") {
        s.sessions.destroy(old);
    }
    let secure = s.config.secure_cookies();
    let sid = set_cookie("sid", &s.sessions.create(user.id), ABSOLUTE_SECS, secure);
    let clear = set_cookie(STATE_COOKIE, "", 0, secure);
    let clear_next = set_cookie(NEXT_COOKIE, "", 0, secure);
    let target = cookie(headers, NEXT_COOKIE)
        .and_then(safe_next)
        .unwrap_or("/");
    Ok((
        AppendHeaders([
            (SET_COOKIE, sid),
            (SET_COOKIE, clear),
            (SET_COOKIE, clear_next),
        ]),
        Redirect::to(target),
    )
        .into_response())
}

/// `POST /auth/logout`: destroy the session, clear the cookie, go home.
pub async fn logout(State(s): State<AppState>, headers: HeaderMap) -> Response {
    let lang = crate::http::locale::preferred(&headers);
    if let Err(e) = check_origin(&headers, &s.config) {
        return fail(lang, e);
    }
    if let Some(token) = cookie(&headers, "sid") {
        s.sessions.destroy(token);
    }
    let clear = set_cookie("sid", "", 0, s.config.secure_cookies());
    (AppendHeaders([(SET_COOKIE, clear)]), Redirect::to("/")).into_response()
}
