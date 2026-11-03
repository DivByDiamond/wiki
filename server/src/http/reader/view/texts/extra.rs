//! Texts of recent changes, "my proposals" and the admin media page.

pub fn text(lang: &str, key: &str) -> Option<&'static str> {
    let (en, ru, zh) = match key {
        "recent" => ("Recent changes", "Недавние изменения", "最近更改"),
        "recent_empty" => ("No changes yet.", "Изменений пока нет.", "暂无更改。"),
        "recent_by" => ("by", "автор", "作者"),
        "recent_feed" => ("Atom feed", "Atom-лента", "Atom 订阅"),
        "recent_no_summary" => ("no comment", "без комментария", "无备注"),
        "my_proposals" => ("My proposals", "Мои предложения", "我的建议"),
        "mine_empty" => (
            "You have not suggested any edits yet.",
            "Вы пока не предлагали правок.",
            "您还没有提交过修改建议。",
        ),
        "status_open" => ("Pending", "На рассмотрении", "待审核"),
        "status_accepted" => ("Accepted", "Принято", "已接受"),
        "status_rejected" => ("Rejected", "Отклонено", "已拒绝"),
        "admin_media" => ("Images", "Картинки", "图片"),
        "media_upload" => ("Upload an image", "Загрузить картинку", "上传图片"),
        "media_hint" => (
            "PNG, JPEG, WebP or GIF, up to 5 MB. SVG is not accepted.",
            "PNG, JPEG, WebP или GIF, до 5 МБ. SVG не принимается.",
            "支持 PNG、JPEG、WebP 或 GIF，最大 5 MB，不接受 SVG。",
        ),
        "media_snippet" => ("Markdown snippet", "Код для Markdown", "Markdown 代码"),
        "media_empty" => ("No images yet.", "Картинок пока нет.", "暂无图片。"),
        _ => return None,
    };
    Some(match lang {
        "ru" => ru,
        "zh" => zh,
        _ => en,
    })
}
