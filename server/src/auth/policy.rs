#[derive(Debug, Clone)]
pub enum Actor {
    Anonymous,
    User {
        id: i64,
        admin: bool,
        blocked: bool,
    },
    Key {
        id: i64,
        read: bool,
        write: bool,
        export: bool,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    ReadPage,
    Propose,
    WritePage,
    DecideProposal,
    ManageKeys,
    ManageUsers,
    Export,
}

pub fn can(actor: &Actor, action: Action) -> bool {
    use Action::*;
    matches!(
        (actor, action),
        (_, ReadPage)
            | (Actor::User { blocked: false, .. }, Propose)
            | (
                Actor::User {
                    admin: true,
                    blocked: false,
                    ..
                },
                _
            )
            | (Actor::Key { write: true, .. }, WritePage)
            | (Actor::Key { export: true, .. }, Export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(admin: bool, blocked: bool) -> Actor {
        Actor::User {
            id: 1,
            admin,
            blocked,
        }
    }

    fn key(read: bool, write: bool, export: bool) -> Actor {
        Actor::Key {
            id: 1,
            read,
            write,
            export,
        }
    }

    #[test]
    fn anyone_can_read() {
        for a in [
            Actor::Anonymous,
            user(false, false),
            key(false, false, false),
        ] {
            assert!(can(&a, Action::ReadPage));
        }
    }

    #[test]
    fn only_signed_in_unblocked_can_propose() {
        assert!(!can(&Actor::Anonymous, Action::Propose));
        assert!(can(&user(false, false), Action::Propose));
        assert!(!can(&user(false, true), Action::Propose));
    }

    #[test]
    fn writing_needs_admin_or_write_key() {
        assert!(!can(&user(false, false), Action::WritePage));
        assert!(can(&user(true, false), Action::WritePage));
        assert!(can(&key(true, true, false), Action::WritePage));
        assert!(!can(&key(true, false, false), Action::WritePage));
    }

    #[test]
    fn admin_only_actions() {
        for act in [
            Action::DecideProposal,
            Action::ManageKeys,
            Action::ManageUsers,
        ] {
            assert!(can(&user(true, false), act));
            assert!(!can(&user(false, false), act));
            assert!(!can(&key(true, true, true), act));
            assert!(!can(&user(true, true), act));
        }
    }

    #[test]
    fn export_for_admin_or_export_key() {
        assert!(can(&user(true, false), Action::Export));
        assert!(can(&key(false, false, true), Action::Export));
        assert!(!can(&key(true, true, false), Action::Export));
    }
}
