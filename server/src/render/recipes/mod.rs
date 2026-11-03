use serde::Deserialize;

#[derive(Deserialize)]
struct Item {
    #[serde(default)]
    id: String,
    name: String,
    tag: Option<String>,
    icon: Option<String>,
    #[serde(default = "one")]
    count: u32,
}
fn one() -> u32 {
    1
}
#[derive(Deserialize)]
struct Recipe {
    kind: String,
    lang: Option<String>,
    slots: Vec<Option<Item>>,
    output: Item,
    seconds: Option<f64>,
    experience: Option<f64>,
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn icon(id: &str) -> Option<String> {
    let (namespace, path) = id.split_once(':')?;
    let valid = |s: &str| {
        !s.is_empty()
            && s.bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"_-".contains(&c))
    };
    if !valid(namespace) || !path.split('/').all(valid) {
        return None;
    }
    Some(format!("/static/items/{namespace}/{path}.png"))
}
fn image(item: &Item) -> String {
    let id = item.icon.as_deref().unwrap_or(&item.id);
    icon(id).map_or(String::new(), |src| {
        format!("<img src=\"{src}\" alt=\"\" loading=\"lazy\" width=\"32\" height=\"32\">")
    })
}
fn label(item: &Item, ru: bool) -> String {
    let mut text = item.name.clone();
    if let Some(tag) = &item.tag {
        let any = if ru {
            "Любой из группы"
        } else {
            "Any item in group"
        };
        text.push_str(&format!(": {any}: #{tag}"));
    }
    text
}
fn legend(slots: &[Option<Item>], ru: bool) -> String {
    let title = if ru {
        "Ингредиенты"
    } else {
        "Ingredients"
    };
    let mut html = format!("<div class=\"recipe-legend\"><p><strong>{title}</strong></p>");
    let mut seen = std::collections::HashSet::new();
    for item in slots.iter().flatten() {
        let key = (&item.id, &item.tag, &item.name);
        if seen.insert(key) {
            html.push_str(&format!("<p>{}</p>", escape(&label(item, ru))));
        }
    }
    html.push_str("</div>");
    html
}

pub(super) fn render(body: &str) -> Option<String> {
    let r: Recipe = serde_json::from_str(body).ok()?;
    let furnace = r.kind == "furnace";
    if !matches!(r.kind.as_str(), "shaped" | "shapeless" | "furnace")
        || r.slots.len() != if furnace { 1 } else { 9 }
    {
        return None;
    }
    let ru = r.lang.as_deref() == Some("ru");
    let title = match (r.kind.as_str(), ru) {
        ("furnace", true) => "Печь",
        ("furnace", false) => "Furnace",
        ("shapeless", true) => "Бесформенный рецепт",
        ("shapeless", false) => "Shapeless crafting",
        (_, true) => "Верстак · 3 × 3",
        _ => "Crafting table · 3 × 3",
    };
    let mut html = format!(
        "<div class=\"recipe-card\"><p><strong>{title}</strong></p><div class=\"recipe-flow\"><div class=\"{}\">",
        if furnace {
            "recipe-furnace"
        } else {
            "recipe-grid"
        }
    );
    for slot in &r.slots {
        if let Some(value) = slot {
            let label = escape(&label(value, ru));
            html.push_str(&format!("<div class=\"recipe-slot\" role=\"img\" title=\"{label}\" aria-label=\"{label}\">{}", image(value)));
            if value.count > 1 {
                html.push_str(&format!("<strong>× {}</strong>", value.count));
            }
        } else {
            html.push_str("<div class=\"recipe-slot\">");
        }
        html.push_str("</div>");
    }
    html.push_str("</div><div class=\"recipe-arrow\" aria-hidden=\"true\">→</div><div class=\"recipe-result\">");
    html.push_str(&image(&r.output));
    html.push_str(&format!(
        "<span>{}</span><strong>× {}</strong>",
        escape(&r.output.name),
        r.output.count
    ));
    html.push_str("</div></div>");
    html.push_str(&legend(&r.slots, ru));
    if furnace {
        let (time, xp) = if ru {
            ("Время (с при 20 TPS)", "Опыт")
        } else {
            ("Time (s at 20 TPS)", "Experience")
        };
        if let (Some(seconds), Some(experience)) = (r.seconds, r.experience) {
            html.push_str(&format!("<p>{time}: {seconds} · {xp}: {experience}</p>"));
        }
    }
    html.push_str("</div>\n");
    Some(html)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paths_are_local_and_cannot_traverse() {
        assert_eq!(
            icon("c:ingots/iron").as_deref(),
            Some("/static/items/c/ingots/iron.png")
        );
        for bad in [
            "https://evil/x",
            "minecraft:../evil",
            "minecraft:x?y",
            "minecraft:/x",
        ] {
            assert!(icon(bad).is_none());
        }
    }
    #[test]
    fn external_icons_and_malformed_directives_cannot_create_fetches() {
        for src in [
            "https://evil.test/icon.png",
            "//evil.test/icon.png",
            "minecraft:../secret",
        ] {
            let json = serde_json::json!({"kind":"furnace", "slots":[{"name":"Input", "icon":src}],
                "output":{"name":"Output", "id":src}});
            let html = crate::render::to_html(&format!(":::recipe\n{json}\n:::"));
            assert!(!html.contains("<img"), "{html}");
        }
        let html = crate::render::to_html(
            ":::recipe\n{bad json <img src=https://evil.test/icon.png>}\n:::",
        );
        assert!(!html.contains("<img"), "{html}");
        assert!(html.contains("&lt;img"), "{html}");
    }
    #[test]
    fn repeated_ingredients_have_one_legend_entry() {
        let slot = r#"{"id":"minecraft:iron_ingot","name":"Iron"}"#;
        let slots = std::iter::repeat_n(slot, 9).collect::<Vec<_>>().join(",");
        let html = render(&format!(r#"{{"kind":"shaped","slots":[{slots}],"output":{{"id":"minecraft:iron_block","name":"Block"}}}}"#)).unwrap();
        assert_eq!(html.matches("class=\"recipe-slot\"").count(), 9);
        assert_eq!(html.matches("<p>Iron</p>").count(), 1);
        assert_eq!(html.matches("aria-label=\"Iron\"").count(), 9);
    }
    #[test]
    fn shared_pipeline_keeps_recipe_icons_and_escapes_labels() {
        let html = crate::render::to_html(
            r#":::recipe
{"kind":"furnace","slots":[{"name":"<script>x</script>","tag":"c:ingots/iron","icon":"minecraft:iron_ingot"}],"output":{"id":"oc2r:silicon","name":"Silicon","count":4},"seconds":10,"experience":0.1}
:::"#,
        );
        assert!(html.contains("recipe-card"), "{html}");
        assert!(
            html.contains("/static/items/minecraft/iron_ingot.png"),
            "{html}"
        );
        assert!(html.contains("Any item in group"));
        assert!(
            html.contains(
                "aria-label=\"&lt;script&gt;x&lt;/script&gt;: Any item in group: #c:ingots/iron\""
            ),
            "{html}"
        );
        assert!(html.contains("class=\"recipe-legend\""));
        assert!(html.contains("× 4"));
        assert!(!html.contains("<script>"));
    }
}
