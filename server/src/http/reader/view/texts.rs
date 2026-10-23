//! Reader chrome translated independently from article content.
pub fn text(lang: &str, key: &str) -> &'static str {
    let (en, ru, zh) = match key {
        "start" => ("Getting started", "Начало", "入门"),
        "hardware" => ("Blocks and items", "Блоки и предметы", "方块与物品"),
        "networking" => ("Networking", "Сети", "网络"),
        "robotics" => ("Robotics", "Роботы", "机器人"),
        "systems" => ("Operating systems", "Системы", "操作系统"),
        "scripting" => ("Scripts and API", "Скрипты и API", "脚本与 API"),
        "project" => ("Project and versions", "Проект и версии", "项目与版本"),
        "navigation" => ("Browse documentation", "Разделы документации", "浏览文档"),
        "toc" => ("On this page", "На этой странице", "本页目录"),
        "search" => ("Search documentation", "Поиск по документации", "搜索文档"),
        "language" => ("Language", "Язык", "语言"),
        "settings" => ("Reading settings", "Настройки чтения", "阅读设置"),
        "skip" => ("Skip to article", "Перейти к статье", "跳转到正文"),
        "signin" => ("Sign in", "Войти", "登录"),
        "signout" => ("Sign out", "Выйти", "退出"),
        "admin" => ("Admin", "Админка", "管理"),
        "history" => ("History", "История", "历史"),
        "edit" => ("Suggest an edit", "Предложить правку", "建议修改"),
        "previous" => ("Previous", "Назад", "上一篇"),
        "next" => ("Next", "Дальше", "下一篇"),
        "missing" => (
            "This translation is not available yet. Showing English.",
            "Перевода пока нет. Показана английская версия.",
            "暂无此语言的翻译，正在显示英文版本。",
        ),
        "submitted" => (
            "Your suggestion has been sent for review.",
            "Предложение отправлено. Редакторы скоро его рассмотрят.",
            "您的修改建议已提交审核。",
        ),
        "old" => ("Old revision", "Старая версия", "旧版本"),
        "latest" => (
            "Read the current version",
            "Перейти к актуальной",
            "查看当前版本",
        ),
        "empty" => (
            "No results. Try a device name or a shorter query.",
            "Ничего не найдено. Попробуйте название устройства или более короткий запрос.",
            "未找到结果。请尝试设备名称或更简短的关键词。",
        ),
        _ => ("Documentation", "Документация", "文档"),
    };
    match lang {
        "ru" => ru,
        "zh" => zh,
        _ => en,
    }
}

pub fn section_rank(section: &str) -> usize {
    [
        "start",
        "hardware",
        "networking",
        "robotics",
        "systems",
        "scripting",
        "project",
    ]
    .iter()
    .position(|s| *s == section)
    .unwrap_or(7)
}
