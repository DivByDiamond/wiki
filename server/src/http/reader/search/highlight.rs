//! Highlight plain FTS snippets, escaping every byte of user-controlled text.
fn escape(out: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
}

pub fn snippet(text: &str, query: &str) -> String {
    // Map lowercased UTF-8 bytes back to original character boundaries. Some
    // characters, e.g. İ, expand when lowercased; slicing must remain valid.
    let mut folded = String::new();
    let mut offsets = Vec::new();
    for (at, c) in text.char_indices() {
        let before = folded.len();
        folded.extend(c.to_lowercase());
        offsets.extend(std::iter::repeat_n(
            (at, at + c.len_utf8()),
            folded.len() - before,
        ));
    }
    let mut spans = Vec::new();
    for term in query.split_whitespace().take(32) {
        let term = term.replace('"', "").to_lowercase();
        if term.is_empty() {
            continue;
        }
        for (at, found) in folded.match_indices(&term) {
            spans.push((offsets[at].0, offsets[at + found.len() - 1].1));
        }
    }
    spans.sort_unstable();
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in spans {
        if let Some(last) = merged.last_mut()
            && start <= last.1
        {
            last.1 = last.1.max(end);
        } else {
            merged.push((start, end));
        }
    }
    let mut out = String::new();
    let mut cursor = 0;
    for (start, end) in merged {
        escape(&mut out, &text[cursor..start]);
        out.push_str("<mark>");
        escape(&mut out, &text[start..end]);
        out.push_str("</mark>");
        cursor = end;
    }
    escape(&mut out, &text[cursor..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_unicode_and_overlapping_terms_without_losing_original_case() {
        assert_eq!(
            snippet("OnyxOS: Память İ", "onyx OnyxOS память i"),
            "<mark>OnyxOS</mark>: <mark>Память</mark> <mark>İ</mark>"
        );
    }

    #[test]
    fn stored_html_and_query_are_never_trusted() {
        let html = snippet("<img onerror='x'> & Robot", "robot <img");
        assert_eq!(
            html,
            "<mark>&lt;img</mark> onerror=&#39;x&#39;&gt; &amp; <mark>Robot</mark>"
        );
        assert!(!html.contains("<img"));
        assert_eq!(
            snippet("<script>x</script>", "\""),
            "&lt;script&gt;x&lt;/script&gt;"
        );
    }
}
