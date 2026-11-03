//! Labels of reader features: search filters and page export.
pub fn text(lang: &str, key: &str) -> Option<&'static str> {
    let (en, ru) = match key {
        "filter_language" => ("Language", "Язык"),
        "filter_section" => ("Section", "Раздел"),
        "filter_all_languages" => ("All languages", "Все языки"),
        "filter_all_sections" => ("All sections", "Все разделы"),
        "filter_apply" => ("Search", "Найти"),
        "export_markdown" => ("Markdown", "Markdown"),
        "print" => ("Print", "Печать"),
        _ => return None,
    };
    Some(match lang {
        "ru" => ru,
        _ => en,
    })
}
