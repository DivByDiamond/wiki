pub mod client_ip;
pub mod rate_limit;

use crate::error::{AppError, AppResult};
use crate::http::state::AppState;

/// Counts one request of `key` under `scope`; Err(RateLimited) when exhausted.
pub fn enforce(s: &AppState, scope: &str, key: &str) -> AppResult<()> {
    s.limiter
        .check(&format!("{scope}:{key}"))
        .map_err(AppError::RateLimited)
}
