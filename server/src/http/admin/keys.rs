use super::{admin_only, fail};
use crate::auth::{policy::Actor, tokens::api_key};
use crate::domain::page::Lang;
use crate::error::{AppError, AppResult};
use crate::http::{
    middleware::{Auth, Viewer},
    reader::{self, view::Shell},
    state::AppState,
    user::check_origin,
};
use askama::Template;
use axum::{
    Form, Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, header},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use serde::Deserialize;

const MAX_NAME: usize = 100;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin/keys", get(list).post(create))
        .route("/admin/keys/{id}/revoke", post(revoke))
}

struct Row {
    id: i64,
    name: String,
    perms: String,
}

#[derive(Template)]
#[template(path = "admin/keys.html")]
struct Keys {
    shell: Shell,
    keys: Vec<Row>,
    /// Freshly created key, shown on this response only.
    new_key: Option<String>,
}

#[derive(Deserialize)]
pub struct KeyForm {
    #[serde(default)]
    name: String,
    read: Option<String>,
    write: Option<String>,
    export: Option<String>,
}

fn page(s: &AppState, viewer: Viewer, new_key: Option<String>) -> AppResult<Response> {
    let fresh = new_key.is_some();
    let keys = s
        .access
        .list_keys()?
        .into_iter()
        .map(|k| {
            let perms: Vec<&str> = [(k.read, "read"), (k.write, "write"), (k.export, "export")]
                .into_iter()
                .filter_map(|(on, n)| on.then_some(n))
                .collect();
            Row {
                id: k.id,
                name: k.name,
                perms: perms.join(", "),
            }
        })
        .collect();
    reader::html(&Keys {
        shell: Shell::bare(Lang::En, "API keys", "start").for_viewer(viewer),
        keys,
        new_key,
    })
    .map(|mut res| {
        // The plain key is in this response only; keep it out of caches and history.
        if fresh {
            res.headers_mut()
                .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        }
        res
    })
}

async fn list(State(s): State<AppState>, Auth(actor): Auth, viewer: Viewer) -> Response {
    let run = || -> AppResult<Response> {
        admin_only(&actor)?;
        page(&s, viewer, None)
    };
    run().unwrap_or_else(fail)
}

async fn create(
    State(s): State<AppState>,
    Auth(actor): Auth,
    viewer: Viewer,
    headers: HeaderMap,
    Form(f): Form<KeyForm>,
) -> Response {
    make(&s, &actor, viewer, &headers, f).unwrap_or_else(fail)
}

fn make(
    s: &AppState,
    actor: &Actor,
    viewer: Viewer,
    headers: &HeaderMap,
    f: KeyForm,
) -> AppResult<Response> {
    check_origin(headers, &s.config)?;
    admin_only(actor)?;
    let name = f.name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME {
        return Err(AppError::Invalid(
            "The key name must be 1-100 characters.".into(),
        ));
    }
    let (read, write, export) = (f.read.is_some(), f.write.is_some(), f.export.is_some());
    if !(read || write || export) {
        return Err(AppError::Invalid("Pick at least one permission.".into()));
    }
    let (key, hash) = api_key::generate();
    s.access.create_key(name, &hash, read, write, export)?;
    page(s, viewer, Some(key))
}

async fn revoke(
    State(s): State<AppState>,
    Auth(actor): Auth,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    let run = || -> AppResult<Response> {
        check_origin(&headers, &s.config)?;
        admin_only(&actor)?;
        s.access.revoke_key(id)?;
        Ok(Redirect::to("/admin/keys").into_response())
    };
    run().unwrap_or_else(fail)
}
