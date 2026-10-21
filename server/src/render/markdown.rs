use super::{extensions, sanitize};
use pulldown_cmark::{Options, Parser, html};

pub fn to_html(src: &str) -> String {
    let expanded = extensions::expand(src);
    let opts =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_HEADING_ATTRIBUTES;
    let mut raw = String::new();
    html::push_html(&mut raw, Parser::new_ext(&expanded, opts));
    sanitize::clean(&raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_basic_markdown() {
        let html = to_html("# Title\n\n**bold**");
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn strips_script_and_handlers() {
        let html = to_html("<script>alert(1)</script><img src=x onerror=alert(1)>hi");
        assert!(!html.contains("script"));
        assert!(!html.contains("onerror"));
        assert!(html.contains("hi"));
    }

    #[test]
    fn keeps_callout_classes_but_not_arbitrary_ones() {
        let html = to_html(":::tip\nok\n:::");
        assert!(html.contains("class=\"callout tip\""));
        let evil = to_html("<div class=\"evil\" style=\"x\">a</div>");
        assert!(!evil.contains("evil") && !evil.contains("style"));
    }

    #[test]
    fn javascript_markdown_link_has_no_href() {
        let html = to_html("[x](javascript:alert(1))");
        assert!(!html.contains("javascript:"));
    }

    #[test]
    fn javascript_anchor_and_onclick_removed() {
        let html = to_html("<a href=\"javascript:alert(1)\" onclick=\"x()\">t</a>");
        assert!(!html.contains("javascript:"));
        assert!(!html.contains("onclick"));
        assert!(html.contains('t'));
    }

    #[test]
    fn iframe_removed() {
        let html = to_html("a\n\n<iframe src=\"https://evil.example\"></iframe>\n\nb");
        assert!(!html.contains("iframe"));
        assert!(!html.contains("evil.example"));
    }

    #[test]
    fn recipe_slot_script_is_neutralised() {
        let html = to_html(":::recipe\n<script>alert(1)</script>\n:::");
        assert!(!html.contains("<script"));
        assert!(!html.contains("onerror"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn every_link_is_noopener_and_has_no_target() {
        let html = to_html("<a href=\"https://x.test\" target=\"_blank\" rel=\"opener\">t</a>");
        assert!(html.contains("<a "));
        assert!(html.contains("rel=\"noopener noreferrer\""));
        assert!(!html.contains("target"));
    }

    #[test]
    fn svg_with_onload_is_removed() {
        let html = to_html("<svg onload=alert(1)><circle/></svg>");
        assert!(!html.contains("<svg"));
        assert!(!html.contains("onload"));
    }

    #[test]
    fn data_uri_image_has_no_data_src() {
        let html = to_html("<img src=\"data:text/html,x\">");
        assert!(!html.contains("data:"));
    }

    #[test]
    fn mixed_case_script_is_removed() {
        let html = to_html("<ScRiPt>alert(1)</ScRiPt>");
        assert!(!html.to_lowercase().contains("script"));
    }

    #[test]
    fn heading_attribute_syntax_has_no_handler() {
        let html = to_html("# h {onclick=alert(1)}");
        assert!(!html.contains("onclick"));
    }

    #[test]
    fn nested_tip_in_text_renders() {
        let html = to_html("before\n:::tip\ninner\n:::\nafter");
        assert!(html.contains("class=\"callout tip\""));
        assert!(html.contains("inner"));
        assert!(html.contains("before") && html.contains("after"));
    }

    #[test]
    fn unknown_block_keeps_inner_text() {
        let html = to_html(":::foo\nkept text\n:::");
        assert!(html.contains("kept text"));
        assert!(!html.contains(":::"));
    }

    #[test]
    fn unterminated_tip_is_balanced() {
        let html = to_html(":::tip\nno end");
        assert!(html.contains("no end"));
        assert_eq!(html.matches("<div").count(), html.matches("</div>").count());
    }
}
