use ammonia::Builder;

pub fn clean(html: &str) -> String {
    let mut b = Builder::default();
    b.add_tags(["div"]);
    b.add_allowed_classes(
        "div",
        ["callout", "tip", "warn", "version", "recipe", "slot"],
    );
    for tag in ["h2", "h3", "h4", "h5", "h6"] {
        b.add_tag_attributes(tag, ["id"]);
    }
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
