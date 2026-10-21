#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Open,
    Accepted,
    Rejected,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Open => "open",
            Status::Accepted => "accepted",
            Status::Rejected => "rejected",
        }
    }

    pub fn parse(s: &str) -> Option<Status> {
        match s {
            "open" => Some(Status::Open),
            "accepted" => Some(Status::Accepted),
            "rejected" => Some(Status::Rejected),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Proposal {
    pub id: i64,
    pub page_id: i64,
    pub base_rev_id: i64,
    pub body: String,
    pub summary: String,
    pub user_id: i64,
    pub status: Status,
    pub created_at: i64,
}

pub fn is_conflict(base_rev_id: i64, current_rev_id: i64) -> bool {
    base_rev_id != current_rev_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conflict_when_current_is_newer() {
        assert!(!is_conflict(5, 5));
        assert!(is_conflict(5, 7));
    }

    #[test]
    fn status_roundtrip() {
        for s in [Status::Open, Status::Accepted, Status::Rejected] {
            assert_eq!(Status::parse(s.as_str()), Some(s));
        }
    }
}
