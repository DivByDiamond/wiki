use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    Unauthorized,
    Forbidden,
    NotFound,
    Invalid(String),
    /// The request clashes with the current state (already decided, name taken).
    Conflict(String),
    TooLarge,
    RateLimited(u64),
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    fn parts(&self) -> (StatusCode, &'static str, String) {
        match self {
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "Нужен вход или ключ".into(),
            ),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "forbidden",
                "Недостаточно прав".into(),
            ),
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found", "Не найдено".into()),
            Self::Invalid(m) => (StatusCode::UNPROCESSABLE_ENTITY, "invalid", m.clone()),
            Self::Conflict(m) => (StatusCode::CONFLICT, "conflict", m.clone()),
            Self::TooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "too_large",
                "Слишком большое тело запроса".into(),
            ),
            Self::RateLimited(_) => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Слишком часто".into(),
            ),
            Self::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                "Внутренняя ошибка".into(),
            ),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if let Self::Internal(m) = &self {
            tracing::error!("internal error: {m}");
        }
        let (status, code, message) = self.parts();
        let mut res = (status, Json(json!({ "error": code, "message": message }))).into_response();
        if let Self::RateLimited(secs) = self {
            res.headers_mut()
                .insert("retry-after", secs.to_string().parse().unwrap());
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use http_body_util::BodyExt;

    async fn body(e: AppError) -> (u16, serde_json::Value) {
        let res = e.into_response();
        let status = res.status().as_u16();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn maps_status_and_code() {
        assert_eq!(body(AppError::Unauthorized).await.0, 401);
        assert_eq!(body(AppError::Forbidden).await.0, 403);
        let (s, j) = body(AppError::NotFound).await;
        assert_eq!((s, j["error"].as_str()), (404, Some("not_found")));
        assert_eq!(body(AppError::Invalid("x".into())).await.0, 422);
        let (s, j) = body(AppError::Conflict("c".into())).await;
        assert_eq!((s, j["error"].as_str()), (409, Some("conflict")));
        let (s, j) = body(AppError::TooLarge).await;
        assert_eq!((s, j["error"].as_str()), (413, Some("too_large")));
    }

    #[test]
    fn rate_limited_sets_retry_after() {
        let res = AppError::RateLimited(30).into_response();
        assert_eq!(res.status(), 429);
        assert_eq!(res.headers()["retry-after"], "30");
    }
}
