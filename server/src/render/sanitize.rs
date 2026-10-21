use ammonia::Builder;

pub fn clean(html: &str) -> String {
    let mut b = Builder::default();
    b.add_tags(["div"]);
    b.add_allowed_classes(
        "div",
        ["callout", "tip", "warn", "version", "recipe", "slot"],
    );
    b.clean(html).to_string()
}
