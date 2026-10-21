/// Раскрывает блоки `:::имя [аргумент]` ... `:::` в HTML-контейнеры.
/// Пустые строки вокруг внутреннего Markdown нужны, чтобы он разбирался внутри `<div>`.
pub fn expand(src: &str) -> String {
    let mut out = Vec::new();
    let mut lines = src.lines();
    while let Some(line) = lines.next() {
        let Some(head) = line.strip_prefix(":::") else {
            out.push(line.to_owned());
            continue;
        };
        let (name, arg) = head
            .split_once(' ')
            .map_or((head, ""), |(n, a)| (n, a.trim()));
        let body: Vec<&str> = lines.by_ref().take_while(|l| *l != ":::").collect();
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
        let item = body.get(i).map_or("", |s| s.trim());
        out.push(format!("<div class=\"slot\">{item}</div>"));
    }
    out.push("</div>".into());
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
    fn plain_text_untouched() {
        assert_eq!(expand("hello\nworld"), "hello\nworld");
    }
}
