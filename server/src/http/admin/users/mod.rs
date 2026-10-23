//! User management: list, block and unblock, grant and revoke admin.

mod act;
mod guard;

use super::{admin_only, fail};
use crate::auth::policy::Actor;
use crate::domain::ports::access::UserRow;
use crate::error::AppResult;
use crate::http::{
    middleware::{Auth, Viewer},
    reader::{self, view::Shell},
    state::AppState,
};
use askama::Template;
use axum::{
    Router,
    extract::State,
    response::Response,
    routing::{get, post},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin/users", get(list))
        .route("/admin/users/{id}/{action}", post(act::change))
}

#[derive(Template)]
#[template(path = "admin/users.html")]
struct Users {
    shell: Shell,
    me: i64,
    users: Vec<UserRow>,
}

async fn list(State(s): State<AppState>, Auth(actor): Auth, viewer: Viewer) -> Response {
    let language = viewer.lang();
    show(&s, &actor, viewer).unwrap_or_else(|e| fail(language, e))
}

fn show(s: &AppState, actor: &Actor, viewer: Viewer) -> AppResult<Response> {
    admin_only(actor)?;
    let me = match actor {
        Actor::User { id, .. } => *id,
        _ => 0,
    };
    reader::html(&Users {
        shell: Shell::admin(viewer, "admin_users"),
        me,
        users: s.access.list_users()?,
    })
}
