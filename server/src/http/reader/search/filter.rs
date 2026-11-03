//! Search filters: which languages and which section the hits may come from.
use crate::domain::{page::Lang, ports::pages::SearchHit};

/// What the `lang` query parameter asks for.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LangFilter {
    /// Reader's language with English fallback (no parameter).
    Default,
    All,
    Only(Lang),
}

impl LangFilter {
    pub fn parse(raw: Option<&str>) -> Self {
        match raw.map(str::trim) {
            Some("all") => Self::All,
            Some(code) => Lang::parse(code).map_or(Self::Default, Self::Only),
            None => Self::Default,
        }
    }

    /// Languages to ask the store about, in priority order.
    pub fn searches(self, current: Lang) -> Vec<Lang> {
        match self {
            Self::Default => vec![current],
            Self::Only(l) => vec![l],
            Self::All => {
                let mut v = vec![current];
                v.extend([Lang::En, Lang::Ru, Lang::Zh].into_iter().filter(|l| *l != current));
                v
            }
        }
    }

    /// Value for the form select.
    pub fn value(self) -> &'static str {
        match self {
            Self::Default => "",
            Self::All => "all",
            Self::Only(l) => l.code(),
        }
    }
}

/// Merges batches in priority order, dropping repeats and wrong languages.
pub fn merge(batches: Vec<Vec<SearchHit>>, filter: LangFilter) -> Vec<SearchHit> {
    let mut out: Vec<SearchHit> = Vec::new();
    for hit in batches.into_iter().flatten() {
        if let LangFilter::Only(l) = filter
            && hit.lang != l
        {
            continue;
        }
        if out.iter().any(|h| h.slug == hit.slug && h.lang == hit.lang) {
            continue;
        }
        out.push(hit);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::page::Slug;

    fn hit(slug: &str, lang: Lang) -> SearchHit {
        SearchHit { slug: Slug::parse(slug).unwrap(), lang, snippet: String::new() }
    }

    #[test]
    fn parses_filter_values() {
        assert_eq!(LangFilter::parse(None), LangFilter::Default);
        assert_eq!(LangFilter::parse(Some("")), LangFilter::Default);
        assert_eq!(LangFilter::parse(Some("all")), LangFilter::All);
        assert_eq!(LangFilter::parse(Some("ru")), LangFilter::Only(Lang::Ru));
        assert_eq!(LangFilter::parse(Some("xx")), LangFilter::Default);
    }

    #[test]
    fn all_searches_current_language_first() {
        assert_eq!(LangFilter::All.searches(Lang::Ru), vec![Lang::Ru, Lang::En, Lang::Zh]);
    }

    #[test]
    fn only_keeps_requested_language_and_drops_repeats() {
        let batches = vec![
            vec![hit("a", Lang::En), hit("a", Lang::Ru)],
            vec![hit("a", Lang::Ru), hit("b", Lang::Ru)],
        ];
        let out = merge(batches, LangFilter::Only(Lang::Ru));
        let got: Vec<_> = out.iter().map(|h| h.slug.as_str().to_owned()).collect();
        assert_eq!(got, ["a", "b"]);
    }

    #[test]
    fn all_keeps_translations_of_one_page() {
        let out = merge(vec![vec![hit("a", Lang::En), hit("a", Lang::Ru)]], LangFilter::All);
        assert_eq!(out.len(), 2);
    }
}
