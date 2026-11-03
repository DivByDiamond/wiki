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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_has_all_texts() {
        for lang in [Lang::En, Lang::Ru] {
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
