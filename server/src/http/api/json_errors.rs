//! Turns plain-text rejections from axum extractors and limits into the API's
//! JSON error shape.
use crate::error::AppError;
use axum::{
    Json,
    body::to_bytes,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde_json::json;

/// Rewrites any non-JSON 4xx/5xx response into `{"error","message"}`.
pub async fn to_json(res: Response) -> Response {
    let status = res.status();
    let is_json = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/json"));
    if is_json || !(status.is_client_error() || status.is_server_error()) {
        return res;
    }
    let (parts, body) = res.into_parts();
    let text = to_bytes(body, 4096).await.unwrap_or_default();
    let mut out = match status {
        StatusCode::PAYLOAD_TOO_LARGE => AppError::TooLarge.into_response(),
        StatusCode::NOT_FOUND => AppError::NotFound.into_response(),
        s if s.is_server_error() => {
            AppError::Internal(String::from_utf8_lossy(&text).into()).into_response()
        }
        s => {
            let message = String::from_utf8_lossy(&text).into_owned();
            (s, Json(json!({ "error": "invalid", "message": message }))).into_response()
        }
    };
    *out.status_mut() = status;
    for (name, value) in &parts.headers {
        if name != header::CONTENT_TYPE && name != header::CONTENT_LENGTH {
            out.headers_mut().insert(name.clone(), value.clone());
        }
    }
    out
}
