//! "My proposals": the signed-in reader's own edit suggestions and their status.

use super::fail;
use crate::auth::policy::{Action, Actor};
use crate::domain::proposal::Status;
use crate::error::{AppError, AppResult};
use crate::http::{
    middleware::{Auth, Viewer},
    reader::{
        self,
        view::{Shell, format_utc},
    },
    require,
    state::AppState,
};
use askama::Template;
use axum::{
    Router,
    extract::State,
    http::{HeaderValue, header},
    response::Response,
    routing::get,
};

const LIMIT: u32 = 100;

pub fn routes() -> Router<AppState> {
    Router::new().route("/account/proposals", get(list))
}

struct Row {
    href: String,
    title: String,
    summary: String,
    date: String,
    status: &'static str,
}

#[derive(Template)]
#[template(path = "user/mine.html")]
struct Mine {
    shell: Shell,
    rows: Vec<Row>,
}

async fn list(State(s): State<AppState>, Auth(actor): Auth, viewer: Viewer) -> Response {
    let lang = viewer.lang();
    page(&s, &actor, viewer).unwrap_or_else(|e| fail(lang, e))
}

fn page(s: &AppState, actor: &Actor, viewer: Viewer) -> AppResult<Response> {
    require(actor, Action::Propose)?;
    let Actor::User { id, .. } = actor else {
        return Err(AppError::Forbidden);
    };
    let mut rows = Vec::new();
    for p in s.proposals.list_by_user(*id, LIMIT)? {
        // The page may be deleted by now; the suggestion is still listed.
        let (href, title) = match s.pages.find_by_id(p.page_id)? {
            Some(page) => (
                format!("/{}/{}", page.lang.code(), page.slug.as_str()),
                crate::render::document::title(&page.current.body)
                    .filter(|t| !t.is_empty())
                    .unwrap_or_else(|| page.slug.as_str().to_owned()),
            ),
            None => (String::new(), String::new()),
        };
        rows.push(Row {
            href,
            title,
            summary: p.summary,
            date: format_utc(p.created_at),
            status: match p.status {
                Status::Open => "open",
                Status::Accepted => "accepted",
                Status::Rejected => "rejected",
            },
        });
    }
    let mut res = reader::html(&Mine {
        shell: Shell::admin(viewer, "my_proposals"),
        rows,
    })?;
    res.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    Ok(res)
}
