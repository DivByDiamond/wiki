//! Rules that keep the site manageable: nobody locks themselves out and the
//! last active admin always stays.
use crate::domain::ports::access::UserRow;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Change {
    Block,
    Unblock,
    GrantAdmin,
    RevokeAdmin,
}

impl Change {
    pub(super) fn parse(s: &str) -> Option<Self> {
        match s {
            "block" => Some(Self::Block),
            "unblock" => Some(Self::Unblock),
            "grant-admin" => Some(Self::GrantAdmin),
            "revoke-admin" => Some(Self::RevokeAdmin),
            _ => None,
        }
    }

    fn removes_access(self) -> bool {
        matches!(self, Self::Block | Self::RevokeAdmin)
    }
}

/// `active_admins` counts admins that are not blocked, `target` included.
pub(super) fn check(
    change: Change,
    actor_id: i64,
    target: &UserRow,
    active_admins: u32,
) -> AppResult<()> {
    if !change.removes_access() {
        return Ok(());
    }
    if target.id == actor_id {
        return Err(AppError::Conflict(
            "You cannot block or demote yourself.".into(),
        ));
    }
    if target.admin && !target.blocked && active_admins <= 1 {
        return Err(last_admin());
    }
    Ok(())
}

/// Also the answer when the store's atomic guard refuses (a concurrent change won).
pub(super) fn last_admin() -> AppError {
    AppError::Conflict("The last active admin cannot be blocked or demoted.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(id: i64, admin: bool, blocked: bool) -> UserRow {
        UserRow {
            id,
            github_id: id,
            login: format!("u{id}"),
            admin,
            blocked,
        }
    }

    #[test]
    fn parses_the_four_actions_only() {
        assert_eq!(Change::parse("grant-admin"), Some(Change::GrantAdmin));
        assert_eq!(Change::parse("delete"), None);
    }

    #[test]
    fn self_block_and_self_demote_are_refused() {
        for c in [Change::Block, Change::RevokeAdmin] {
            assert!(check(c, 1, &user(1, true, false), 3).is_err());
        }
        assert!(check(Change::Block, 1, &user(2, false, false), 1).is_ok());
    }

    #[test]
    fn last_active_admin_is_protected() {
        for c in [Change::Block, Change::RevokeAdmin] {
            assert!(check(c, 9, &user(2, true, false), 1).is_err());
            assert!(check(c, 9, &user(2, true, false), 2).is_ok());
        }
        // a blocked admin is not counted, so touching them is harmless
        assert!(check(Change::RevokeAdmin, 9, &user(2, true, true), 1).is_ok());
    }

    #[test]
    fn granting_and_unblocking_are_never_guarded() {
        assert!(check(Change::Unblock, 1, &user(1, true, true), 0).is_ok());
        assert!(check(Change::GrantAdmin, 1, &user(1, true, false), 1).is_ok());
    }
}
