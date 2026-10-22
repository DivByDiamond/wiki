//! Suggestions for a missing page: other slugs of the same language that look alike.

use std::collections::HashSet;

pub const LIMIT: usize = 5;

fn tokens(slug: &str) -> HashSet<String> {
    slug.split(['/', '-', '_'])
        .filter(|t| !t.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn last(slug: &str) -> String {
    slug.rsplit('/').next().unwrap_or_default().to_lowercase()
}

/// One point per shared token, plus one when either last segment contains the other.
fn score(wanted: &str, candidate: &str) -> usize {
    let shared = tokens(wanted).intersection(&tokens(candidate)).count();
    let (a, b) = (last(wanted), last(candidate));
    let bonus = !a.is_empty() && !b.is_empty() && (a.contains(&b) || b.contains(&a));
    shared + usize::from(bonus)
}

/// Up to `LIMIT` `(slug, title)` pairs, best score first, ties by slug. Title is the last segment.
pub fn similar<'a>(
    wanted: &str,
    slugs: impl IntoIterator<Item = &'a str>,
) -> Vec<(String, String)> {
    let mut scored: Vec<(usize, &str)> = slugs
        .into_iter()
        .filter(|s| *s != wanted)
        .map(|s| (score(wanted, s), s))
        .filter(|(n, _)| *n > 0)
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(b.1)));
    scored
        .into_iter()
        .take(LIMIT)
        .map(|(_, s)| {
            (
                s.to_string(),
                s.rsplit('/').next().unwrap_or_default().to_string(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slugs(v: &[(String, String)]) -> Vec<&str> {
        v.iter().map(|(s, _)| s.as_str()).collect()
    }

    #[test]
    fn shared_token_plus_containing_last_segment_ranks_first() {
        let all = ["blocks/computer", "blocks/monitor", "items/chip"];
        let got = similar("blocks/comp", all);
        assert_eq!(slugs(&got), ["blocks/computer", "blocks/monitor"]);
        assert_eq!(got[0].1, "computer");
    }

    #[test]
    fn tokens_split_on_dash_and_underscore_and_ignore_case() {
        let got = similar("Hard_Drive", ["items/hard-drive", "items/cpu"]);
        assert_eq!(slugs(&got), ["items/hard-drive"]);
    }

    #[test]
    fn no_match_gives_nothing() {
        assert!(similar("zzz/qqq", ["blocks/computer", "items/chip"]).is_empty());
    }

    #[test]
    fn at_most_five_ties_sorted_by_slug() {
        let all = ["a/x1", "a/x2", "a/x3", "a/x4", "a/x5", "a/x6", "a/x7"];
        let got = similar("a/missing", all);
        assert_eq!(slugs(&got), ["a/x1", "a/x2", "a/x3", "a/x4", "a/x5"]);
    }

    #[test]
    fn the_requested_slug_itself_is_skipped() {
        assert!(similar("a/b", ["a/b"]).is_empty());
    }
}
