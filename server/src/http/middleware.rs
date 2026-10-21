use super::state::AppState;
use crate::auth::{policy::Actor, tokens::api_key};
use axum::{extract::FromRequestParts, http::request::Parts};
use std::convert::Infallible;

pub struct Auth(pub Actor);

fn cookie<'a>(parts: &'a Parts, name: &str) -> Option<&'a str> {
    let header = parts.headers.get("cookie")?.to_str().ok()?;
    header
        .split(';')
        .filter_map(|p| p.trim().split_once('='))
        .find(|(k, _)| *k == name)
        .map(|(_, v)| v)
}

impl FromRequestParts<AppState> for Auth {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Infallible> {
        if let Some(key) = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
        {
            return Ok(Auth(match state.access.key_by_hash(&api_key::hash(key)) {
                Ok(Some(k)) => {
                    let _ = state.access.touch_key(k.id);
                    Actor::Key {
                        id: k.id,
                        read: k.read,
                        write: k.write,
                        export: k.export,
                    }
                }
                _ => Actor::Anonymous,
            }));
        }
        let user = cookie(parts, "sid")
            .and_then(|t| state.sessions.user_id(t))
            .and_then(|id| state.access.user(id).ok().flatten());
        Ok(Auth(match user {
            Some(u) => Actor::User {
                id: u.id,
                admin: u.admin,
                blocked: u.blocked,
            },
            None => Actor::Anonymous,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::tokens::api_key;
    use crate::store::db::open_memory;
    use axum::{Router, body::Body, http::Request, routing::get};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn whoami(Auth(actor): Auth) -> String {
        format!("{actor:?}")
    }

    async fn call(state: AppState, header: Option<(&str, &str)>) -> String {
        let app = Router::new().route("/", get(whoami)).with_state(state);
        let mut req = Request::get("/");
        if let Some((k, v)) = header {
            req = req.header(k, v);
        }
        let res = app.oneshot(req.body(Body::empty()).unwrap()).await.unwrap();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    fn state() -> AppState {
        AppState::new(open_memory().unwrap())
    }

    #[tokio::test]
    async fn valid_key_gives_key_actor() {
        let st = state();
        let (key, hash) = api_key::generate();
        let id = st
            .access
            .create_key("ci", &hash, true, true, false)
            .unwrap();
        let out = call(st, Some(("authorization", &format!("Bearer {key}")))).await;
        assert_eq!(
            out,
            format!("Key {{ id: {id}, read: true, write: true, export: false }}")
        );
    }

    #[tokio::test]
    async fn unknown_or_malformed_key_is_anonymous() {
        let st = state();
        assert_eq!(
            call(st.clone(), Some(("authorization", "Bearer nope"))).await,
            "Anonymous"
        );
        assert_eq!(
            call(st.clone(), Some(("authorization", "Basic abc"))).await,
            "Anonymous"
        );
        assert_eq!(call(st, None).await, "Anonymous");
    }

    #[tokio::test]
    async fn session_cookie_gives_user_with_flags() {
        let st = state();
        let u = st.access.upsert_user(5, "adm").unwrap();
        st.access.set_admin(u.id, true).unwrap();
        let t = st.sessions.create(u.id);
        let cookie = format!("theme=dark; sid={t}");
        let out = call(st.clone(), Some(("cookie", &cookie))).await;
        assert_eq!(
            out,
            format!("User {{ id: {}, admin: true, blocked: false }}", u.id)
        );
        st.access.set_blocked(u.id, true).unwrap();
        let out = call(st.clone(), Some(("cookie", &cookie))).await;
        assert_eq!(
            out,
            format!("User {{ id: {}, admin: true, blocked: true }}", u.id)
        );
        assert_eq!(call(st, Some(("cookie", "sid=unknown"))).await, "Anonymous");
    }
}
