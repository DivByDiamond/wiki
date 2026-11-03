//! Texts of recent changes, "my proposals" and the admin media page.

pub fn text(lang: &str, key: &str) -> Option<&'static str> {
    let (en, ru) = match key {
        "recent" => ("Recent changes", "Недавние изменения"),
        "recent_empty" => ("No changes yet.", "Изменений пока нет."),
        "recent_by" => ("by", "автор"),
        "recent_feed" => ("Atom feed", "Atom-лента"),
        "recent_no_summary" => ("no comment", "без комментария"),
        "my_proposals" => ("My proposals", "Мои предложения"),
        "mine_empty" => ("You have not suggested any edits yet.", "Вы пока не предлагали правок."),
        "status_open" => ("Pending", "На рассмотрении"),
        "status_accepted" => ("Accepted", "Принято"),
        "status_rejected" => ("Rejected", "Отклонено"),
        "admin_media" => ("Images", "Картинки"),
        "media_upload" => ("Upload an image", "Загрузить картинку"),
        "media_hint" => ("PNG, JPEG, WebP or GIF, up to 5 MB. SVG is not accepted.", "PNG, JPEG, WebP или GIF, до 5 МБ. SVG не принимается."),
        "media_snippet" => ("Markdown snippet", "Код для Markdown"),
        "media_empty" => ("No images yet.", "Картинок пока нет."),
        _ => return None,
    };
    Some(match lang {
        "ru" => ru,
        _ => en,
    })
}
