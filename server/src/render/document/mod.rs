//! One Markdown pipeline for articles, previews and table-of-contents anchors.
use super::{extensions, sanitize};
use pulldown_cmark::{CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd, html};
use std::collections::HashSet;

pub struct Heading {
    pub id: String,
    pub text: String,
    pub nested: bool,
}

pub struct Document {
    pub html: String,
    pub headings: Vec<Heading>,
}

pub fn title(src: &str) -> Option<String> {
    let mut active = false;
    let mut title = String::new();
    for event in Parser::new(src) {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            }) => active = true,
            Event::End(TagEnd::Heading(HeadingLevel::H1)) if active => return Some(title),
            Event::Text(t) | Event::Code(t) if active => title.push_str(&t),
            _ => {}
        }
    }
    None
}

pub fn render(src: &str) -> Document {
    let expanded = extensions::expand(src);
    let opts =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_HEADING_ATTRIBUTES;
    let mut events: Vec<_> = Parser::new_ext(&expanded, opts).collect();
    let mut headings = Vec::new();
    let mut used = HashSet::new();
    for i in 0..events.len() {
        let Event::Start(Tag::Heading { level, id, .. }) = &events[i] else {
            continue;
        };
        // H1 is the page title, outside the article outline.
        if *level == HeadingLevel::H1 {
            continue;
        }
        let level = *level;
        let text: String = events[i + 1..]
            .iter()
            .take_while(|e| !matches!(e, Event::End(TagEnd::Heading(_))))
            .filter_map(|e| match e {
                Event::Text(t) | Event::Code(t) => Some(t.as_ref()),
                Event::SoftBreak | Event::HardBreak => Some(" "),
                _ => None,
            })
            .collect();
        let base: String = id
            .as_deref()
            .unwrap_or(&text)
            .to_lowercase()
            .chars()
            .filter_map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    Some(c)
                } else if c.is_whitespace() {
                    Some('-')
                } else {
                    None
                }
            })
            .collect();
        let base = if base.is_empty() {
            "section".into()
        } else {
            base
        };
        let base = format!("section-{}", base.trim_start_matches("section-"));
        let mut id = base.clone();
        let mut n = 1;
        while !used.insert(id.clone()) {
            id = format!("{base}-{n}");
            n += 1;
        }
        if let Event::Start(Tag::Heading { id: dest, .. }) = &mut events[i] {
            *dest = Some(CowStr::from(id.clone()));
        }
        headings.push(Heading {
            id,
            text,
            nested: level != HeadingLevel::H2,
        });
    }
    let mut raw = String::new();
    html::push_html(&mut raw, events.into_iter());
    Document {
        html: sanitize::clean(&raw),
        headings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headings_are_unicode_stable_and_unique() {
        let doc = render("# Title\n\n## Привет `мир`\n\n## Привет мир\n\n### 中文\n\n## !!!");
        let ids: Vec<_> = doc.headings.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "section-привет-мир",
                "section-привет-мир-1",
                "section-中文",
                "section-section"
            ]
        );
        for h in &doc.headings {
            assert!(doc.html.contains(&format!("id=\"{}\"", h.id)));
        }
        assert!(doc.headings[2].nested);
    }

    #[test]
    fn outline_ids_cannot_shadow_the_shell_and_fragment_links_follow() {
        let doc = render("## Article\n\n[Jump](#article)\n\n<h2 id=\"search-input\">Raw</h2>");
        assert_eq!(doc.headings[0].id, "section-article");
        assert!(doc.html.contains("href=\"#section-article\""));
        assert!(doc.html.contains("id=\"section-search-input\""));
        assert!(!doc.html.contains("id=\"article\""));
    }

    #[test]
    fn screenshot_dimensions_and_loading_survive_sanitizing() {
        let doc = render(
            "<a href=\"/static/photos/example.webp\"><img src=\"/static/photos/example.webp\" alt=\"Real display\" width=\"1018\" height=\"255\" loading=\"lazy\" decoding=\"async\" onerror=\"evil()\"></a>",
        );
        for attribute in [
            "width=\"1018\"",
            "height=\"255\"",
            "loading=\"lazy\"",
            "decoding=\"async\"",
        ] {
            assert!(doc.html.contains(attribute), "{attribute}");
        }
        assert!(!doc.html.contains("onerror"));
    }

    #[test]
    fn titles_strip_formatting_and_code_fences_do_not_create_headings() {
        assert_eq!(title("# A **computer**"), Some("A computer".into()));
        assert!(render("```md\n## Example\n```").headings.is_empty());
    }
}
