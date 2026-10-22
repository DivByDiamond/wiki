pub mod admin;
pub mod api;
pub mod assets;
pub mod limit;
pub mod middleware;
pub mod reader;
pub mod state;
pub mod user;

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

/// New pages must not take a reader route address; such a page would be unreachable.
pub fn ensure_slug_allowed(slug: &str) -> AppResult<()> {
    if slug == "search" {
        return Err(AppError::Invalid(
            "The address `search` is reserved for the search page.".into(),
        ));
    }
    Ok(())
}
