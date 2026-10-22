#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Author {
    User(String),
    ApiKey(String),
    Import,
}

impl Author {
    pub fn label(&self) -> String {
        match self {
            Author::User(login) => login.clone(),
            Author::ApiKey(name) => format!("ключ: {name}"),
            Author::Import => "импорт".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Revision {
    pub id: i64,
    pub page_id: i64,
    pub body: String,
    pub summary: String,
    pub author: Author,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct NewRevision {
    pub body: String,
    pub summary: String,
    pub author: Author,
}

/// A history row without the body: the reader history and the admin editor only
/// need the size, so large pages are not loaded just to list their revisions.
#[derive(Debug, Clone)]
pub struct RevisionMeta {
    pub id: i64,
    pub summary: String,
    pub author: Author,
    pub created_at: i64,
    /// Body length in characters, the same unit as the history delta.
    pub size: i64,
}

pub fn rollback_to(target: &Revision, author: Author) -> NewRevision {
    NewRevision {
        body: target.body.clone(),
        summary: format!("Откат к №{}", target.id),
        author,
    }
}

pub fn is_noop(current: Option<&Revision>, body: &str) -> bool {
    current.is_some_and(|c| c.body == body)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rev(id: i64, body: &str) -> Revision {
        Revision {
            id,
            page_id: 1,
            body: body.into(),
            summary: String::new(),
            author: Author::Import,
            created_at: 0,
        }
    }

    #[test]
    fn rollback_copies_body_and_names_target() {
        let n = rollback_to(&rev(12, "old"), Author::User("loki".into()));
        assert_eq!(n.body, "old");
        assert_eq!(n.summary, "Откат к №12");
    }

    #[test]
    fn noop_only_when_body_equal() {
        assert!(is_noop(Some(&rev(1, "a")), "a"));
        assert!(!is_noop(Some(&rev(1, "a")), "b"));
        assert!(!is_noop(None, "a"));
    }

    #[test]
    fn author_label() {
        assert_eq!(Author::ApiKey("ci".into()).label(), "ключ: ci");
        assert_eq!(Author::User("loki".into()).label(), "loki");
    }
}
