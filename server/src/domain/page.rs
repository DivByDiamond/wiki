use super::revision::Revision;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    En,
    Ru,
    Zh,
}

impl Lang {
    pub fn parse(s: &str) -> Option<Lang> {
        match s {
            "en" => Some(Lang::En),
            "ru" => Some(Lang::Ru),
            "zh" => Some(Lang::Zh),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ru => "ru",
            Lang::Zh => "zh",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Slug(String);

impl Slug {
    pub fn parse(s: &str) -> Option<Slug> {
        let segment_ok = |seg: &str| {
            !seg.is_empty()
                && seg
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
        };
        (s.len() <= 100 && s.split('/').all(segment_ok)).then(|| Slug(s.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Page {
    pub id: i64,
    pub slug: Slug,
    pub lang: Lang,
    pub section: String,
    pub position: i64,
    pub deleted: bool,
    pub current: Revision,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_accepts_nested_lowercase() {
        assert_eq!(
            Slug::parse("blocks/computer").unwrap().as_str(),
            "blocks/computer"
        );
        assert!(Slug::parse("quick_start-2").is_some());
    }

    #[test]
    fn slug_rejects_bad_input() {
        for bad in ["", "Bad", "a//b", "../x", "a b", "/a", "a/"] {
            assert!(Slug::parse(bad).is_none(), "{bad}");
        }
        assert!(Slug::parse(&"a".repeat(101)).is_none());
    }

    #[test]
    fn lang_roundtrip() {
        for c in ["en", "ru", "zh"] {
            assert_eq!(Lang::parse(c).unwrap().code(), c);
        }
        assert!(Lang::parse("de").is_none());
    }
}
