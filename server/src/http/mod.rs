pub mod api;
pub mod middleware;
pub mod rate_limit;
pub mod state;

use crate::auth::policy::{Action, Actor, can};
use crate::error::{AppError, AppResult};

/// Gate for handlers: anonymous gets 401, other denied actors get 403.
pub fn require(actor: &Actor, action: Action) -> AppResult<()> {
    if can(actor, action) {
        Ok(())
    } else if matches!(actor, Actor::Anonymous) {
        Err(AppError::Unauthorized)
    } else {
        Err(AppError::Forbidden)
    }
}
