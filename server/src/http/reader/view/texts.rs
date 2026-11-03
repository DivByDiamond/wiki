//! Reader chrome translated independently from article content.
mod extra;

pub fn text(lang: &str, key: &str) -> &'static str {
    if let Some(text) = super::admin_texts::text(lang, key) {
        return text;
    }
    if let Some(text) = super::reader_texts::text(lang, key) {
        return text;
    }
    if let Some(text) = extra::text(lang, key) {
        return text;
    }
    let (en, ru) = match key {
        "start" => ("Getting started", "Начало"),
        "hardware" => ("Blocks and items", "Блоки и предметы"),
        "recipes" => ("Crafting", "Крафты"),
        "networking" => ("Networking", "Сети"),
        "robotics" => ("Robotics", "Роботы"),
        "systems" => ("Operating systems", "Системы"),
        "scripting" => ("Scripts and API", "Скрипты и API"),
        "project" => ("Project and versions", "Проект и версии"),
        "navigation" => ("Browse documentation", "Разделы документации"),
        "toc" => ("On this page", "На этой странице"),
        "search" => ("Search documentation", "Поиск по документации"),
        "language" => ("Language", "Язык"),
        "settings" => ("Reading settings", "Настройки чтения"),
        "skip" => ("Skip to article", "Перейти к статье"),
        "account" => ("Account", "Аккаунт"),
        "signin" => ("Sign in", "Войти"),
        "signout" => ("Sign out", "Выйти"),
        "admin" => ("Admin", "Админка"),
        "history" => ("History", "История"),
        "edit" => ("Suggest an edit", "Предложить правку"),
        "previous" => ("Previous", "Назад"),
        "next" => ("Next", "Дальше"),
        "missing" => ("This translation is not available yet. Showing English.", "Перевода пока нет. Показана английская версия."),
        "submitted" => ("Your suggestion has been sent for review.", "Предложение отправлено. Редакторы скоро его рассмотрят."),
        "old" => ("Old revision", "Старая версия"),
        "latest" => ("Read the current version", "Перейти к актуальной"),
        "empty" => ("No results. Try a device name or a shorter query.", "Ничего не найдено. Попробуйте название устройства или более короткий запрос."),
        _ => ("Documentation", "Документация"),
    };
    match lang {
        "ru" => ru,
        _ => en,
    }
}

const SECTIONS: [&str; 8] = [
    "start",
    "hardware",
    "recipes",
    "networking",
    "robotics",
    "systems",
    "scripting",
    "project",
];

pub fn is_known_section(section: &str) -> bool {
    SECTIONS.contains(&section)
}

pub fn section_rank(section: &str) -> usize {
    SECTIONS
        .iter()
        .position(|s| *s == section)
        .unwrap_or(SECTIONS.len())
}
