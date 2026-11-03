//! Admin image uploads: a listing with copyable Markdown and a multipart upload.

use super::{admin_only, fail};
use crate::auth::policy::Actor;
use crate::domain::media::{self, MAX_BYTES, Rejected};
use crate::error::{AppError, AppResult};
use crate::http::{
    middleware::{Auth, Viewer},
    reader::{
        self,
        view::{Shell, format_utc},
    },
    state::AppState,
    user::check_origin,
};
use askama::Template;
use axum::{
    Router,
    extract::{DefaultBodyLimit, Multipart, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};

/// Room for the multipart envelope around the largest image.
const MAX_REQUEST: usize = MAX_BYTES + 64 * 1024;

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/admin/media",
        get(list)
            .post(upload)
            .layer(DefaultBodyLimit::max(MAX_REQUEST)),
    )
}

struct Row {
    name: String,
    snippet: String,
    size_kb: i64,
    date: String,
}

#[derive(Template)]
#[template(path = "admin/media/list.html")]
struct MediaPage {
    shell: Shell,
    rows: Vec<Row>,
}

async fn list(State(s): State<AppState>, Auth(actor): Auth, viewer: Viewer) -> Response {
    let lang = viewer.lang();
    let run = || -> AppResult<Response> {
        admin_only(&actor)?;
        let rows = s
            .media
            .list()?
            .into_iter()
            .map(|m| Row {
                snippet: media::snippet(&m.name),
                name: m.name,
                size_kb: (m.size + 1023) / 1024,
                date: format_utc(m.created_at),
            })
            .collect();
        reader::html(&MediaPage {
            shell: Shell::admin(viewer, "admin_media"),
            rows,
        })
    };
    run().unwrap_or_else(|e| fail(lang, e))
}

async fn upload(
    State(s): State<AppState>,
    Auth(actor): Auth,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    let lang = crate::http::locale::preferred(&headers);
    store(&s, &actor, &headers, multipart)
        .await
        .unwrap_or_else(|e| fail(lang, e))
}

async fn store(
    s: &AppState,
    actor: &Actor,
    headers: &HeaderMap,
    mut multipart: Multipart,
) -> AppResult<Response> {
    // Origin and role are checked before a single byte of the body is read.
    check_origin(headers, &s.config)?;
    admin_only(actor)?;
    let invalid = || AppError::Invalid("Choose an image file to upload.".into());
    while let Some(field) = multipart.next_field().await.map_err(|e| read_error(&e))? {
        if field.name() != Some("file") {
            continue;
        }
        let data = field.bytes().await.map_err(|e| read_error(&e))?;
        let prepared = media::prepare(&data).map_err(|r| match r {
            Rejected::TooLarge => AppError::TooLarge,
            Rejected::Empty => invalid(),
            Rejected::UnsupportedType => {
                AppError::Invalid("Only PNG, JPEG, WebP and GIF images are accepted.".into())
            }
        })?;
        s.media.add(&prepared, &data)?;
        return Ok(Redirect::to("/admin/media").into_response());
    }
    Err(invalid())
}

fn read_error(e: &axum::extract::multipart::MultipartError) -> AppError {
    if e.status() == StatusCode::PAYLOAD_TOO_LARGE {
        AppError::TooLarge
    } else {
        AppError::Invalid("The upload could not be read.".into())
    }
}
