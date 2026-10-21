/// Раскрывает блоки `:::имя [аргумент]` ... `:::` в HTML-контейнеры.
/// Пустые строки вокруг внутреннего Markdown нужны, чтобы он разбирался внутри `<div>`.
pub fn expand(src: &str) -> String {
    let mut out = Vec::new();
    let mut lines = src.lines();
    while let Some(line) = lines.next() {
        let Some(head) = line.trim_end().strip_prefix(":::") else {
            out.push(line.to_owned());
            continue;
        };
        let (name, arg) = head
            .split_once(' ')
            .map_or((head, ""), |(n, a)| (n, a.trim()));
        let body: Vec<&str> = lines
            .by_ref()
            .take_while(|l| l.trim_end() != ":::")
            .collect();
        match name {
            "tip" | "warn" => wrap(&mut out, &format!("callout {name}"), None, &body),
            "version" => wrap(
                &mut out,
                "callout version",
                Some(&format!("**Только для {arg}**")),
                &body,
            ),
            "recipe" => recipe(&mut out, &body),
            _ => out.extend(body.iter().map(|l| (*l).to_owned())),
        }
    }
    out.join(
        "
",
    )
}

fn wrap(out: &mut Vec<String>, class: &str, title: Option<&str>, body: &[&str]) {
    out.push(format!(
        "<div class=\"{class}\">
"
    ));
    if let Some(t) = title {
        out.push(format!(
            "{t}
"
        ));
    }
    out.extend(body.iter().map(|l| (*l).to_owned()));
    out.push(
        "
</div>"
            .into(),
    );
}

fn recipe(out: &mut Vec<String>, body: &[&str]) {
    out.push("<div class=\"recipe\">".into());
    for i in 0..9 {
        let item = escape_html(body.get(i).map_or("", |s| s.trim()));
        out.push(format!("<div class=\"slot\">{item}</div>"));
    }
    out.push("</div>".into());
}

/// Экранирует текст слота, чтобы он оставался текстом и не открывал HTML-теги.
fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tip_becomes_callout_div() {
        let out = expand(":::tip\nТекст\n:::");
        assert!(out.contains("<div class=\"callout tip\">"));
        assert!(out.contains("</div>"));
        assert!(out.contains("Текст"));
    }

    #[test]
    fn version_names_the_version() {
        let out = expand(":::version 26.x\nТолько тут\n:::");
        assert!(out.contains("callout version"));
        assert!(out.contains("26.x"));
    }

    #[test]
    fn recipe_has_nine_slots() {
        let out = expand(":::recipe\niron\n\niron\n:::");
        assert_eq!(out.matches("class=\"slot\"").count(), 9);
        assert!(out.contains(">iron<"));
    }

    #[test]
    fn recipe_slot_cannot_break_out_of_recipe() {
        let html = crate::render::to_html(
            ":::recipe\n</div></div></div><div class=\"callout warn\">spoof\n:::",
        );
        assert_eq!(html.matches("class=\"recipe\"").count(), 1);
        assert_eq!(html.matches("class=\"slot\"").count(), 9);
        assert!(!html.contains("<div class=\"callout"), "{html}");
        assert!(html.contains("&lt;/div&gt;"));
    }

    #[test]
    fn closing_line_tolerates_trailing_space_and_crlf() {
        let html = crate::render::to_html(":::tip\r\ntext\r\n:::  \r\nafter");
        let callout_at = html.find("class=\"callout tip\"").expect("callout");
        let after_at = html.find("after").expect("after");
        let close_at = html[callout_at..].find("</div>").expect("close") + callout_at;
        assert!(callout_at < close_at && close_at < after_at);
    }

    #[test]
    fn opening_line_tolerates_trailing_whitespace() {
        let out = expand(":::tip  \ntext\n:::");
        assert!(out.contains("<div class=\"callout tip\">"));
        assert!(!out.contains(":::"));
    }

    #[test]
    fn plain_text_untouched() {
        assert_eq!(expand("hello\nworld"), "hello\nworld");
    }
}
