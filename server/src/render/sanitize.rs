use ammonia::Builder;

pub fn clean(html: &str) -> String {
    let mut b = Builder::default();
    b.add_tags(["div", "details", "summary"]);
    b.add_allowed_classes(
        "div",
        [
            "callout",
            "note",
            "tip",
            "warn",
            "version",
            "recipe",
            "slot",
            "recipe-card",
            "recipe-flow",
            "recipe-grid",
            "recipe-furnace",
            "recipe-slot",
            "recipe-arrow",
            "recipe-result",
            "recipe-legend",
        ],
    );
    b.add_tag_attributes("div", ["aria-label", "aria-hidden", "role", "title"]);
    b.add_allowed_classes("code", ["language-lua"]);
    for tag in ["h2", "h3", "h4", "h5", "h6"] {
        b.add_tag_attributes(tag, ["id"]);
    }
    b.add_tag_attributes("img", ["loading", "decoding"]);
    // Article IDs live outside the shell namespace, including raw HTML headings.
    b.id_prefix(Some("section-"));
    b.attribute_filter(|_, attribute, value| {
        let local = value.starts_with('#') || (value.starts_with('/') && !value.starts_with("//"));
        if attribute == "href"
            && local
            && let Some((path, fragment)) = value.split_once('#')
            && !fragment.starts_with("section-")
        {
            Some(std::borrow::Cow::Owned(format!(
                "{path}#section-{fragment}"
            )))
        } else {
            Some(std::borrow::Cow::Borrowed(value))
        }
    });
    b.clean(html).to_string()
}
