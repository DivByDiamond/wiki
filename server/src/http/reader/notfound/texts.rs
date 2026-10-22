//! Localized strings of the reader 404 page.

use crate::domain::page::Lang;

pub struct Texts {
    pub heading: &'static str,
    pub message: &'static str,
    pub placeholder: &'static str,
    pub search: &'static str,
    pub home: &'static str,
    pub similar: &'static str,
}

pub fn texts(lang: Lang) -> Texts {
    match lang {
        Lang::En => Texts {
            heading: "Page not found",
            message: "This page does not exist. It may have been moved or deleted.",
            placeholder: "Search the wiki",
            search: "Search",
            home: "Home",
            similar: "Similar pages",
        },
        Lang::Ru => Texts {
            heading: "Страница не найдена",
            message: "Такой страницы нет. Возможно, она перемещена или удалена.",
            placeholder: "Поиск по вики",
            search: "Найти",
            home: "На главную",
            similar: "Похожие страницы",
        },
        Lang::Zh => Texts {
            heading: "页面未找到",
            message: "此页面不存在，可能已被移动或删除。",
            placeholder: "搜索维基",
            search: "搜索",
            home: "返回首页",
            similar: "相似页面",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_has_all_texts() {
        for lang in [Lang::En, Lang::Ru, Lang::Zh] {
            let t = texts(lang);
            for s in [
                t.heading,
                t.message,
                t.placeholder,
                t.search,
                t.home,
                t.similar,
            ] {
                assert!(!s.trim().is_empty(), "{lang:?}");
            }
        }
    }
}
