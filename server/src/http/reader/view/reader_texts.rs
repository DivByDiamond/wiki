//! Labels of reader features: search filters and page export.
pub fn text(lang: &str, key: &str) -> Option<&'static str> {
    let (en, ru, zh) = match key {
        "filter_language" => ("Language", "Язык", "语言"),
        "filter_section" => ("Section", "Раздел", "栏目"),
        "filter_all_languages" => ("All languages", "Все языки", "所有语言"),
        "filter_all_sections" => ("All sections", "Все разделы", "所有栏目"),
        "filter_apply" => ("Search", "Найти", "搜索"),
        "export_markdown" => ("Markdown", "Markdown", "Markdown"),
        "print" => ("Print", "Печать", "打印"),
        _ => return None,
    };
    Some(match lang {
        "ru" => ru,
        "zh" => zh,
        _ => en,
    })
}
