//! Data shared by every reader template: the layout shell and date formatting.

mod date;

pub use date::format_utc;

use crate::domain::page::Lang;
use crate::error::AppResult;
use crate::http::{
    middleware::{SignedIn, Viewer},
    state::AppState,
};

pub struct NavItem {
    pub slug: String,
    pub title: String,
    pub current: bool,
}

pub struct Section {
    pub name: String,
    pub items: Vec<NavItem>,
}

/// Everything `layout/base.html` needs; embedded in each page template.
pub struct Shell {
    pub lang: &'static str,
    pub title: String,
    /// Path after the language prefix, used by the language switcher.
    pub path: String,
    pub query: String,
    pub sections: Vec<Section>,
    /// Who is signed in, for the header; `None` when anonymous.
    pub user: Option<SignedIn>,
}

impl Shell {
    /// Shows the signed-in user in the header.
    pub fn for_viewer(mut self, viewer: Viewer) -> Shell {
        self.user = viewer.0;
        self
    }

    /// Shell without a sidebar (error pages).
    pub fn bare(lang: Lang, title: &str, path: &str) -> Shell {
        Shell {
            lang: lang.code(),
            title: title.into(),
            path: path.into(),
            query: String::new(),
            sections: Vec::new(),
            user: None,
        }
    }

    pub fn build(
        s: &AppState,
        lang: Lang,
        title: &str,
        path: &str,
        query: &str,
    ) -> AppResult<Shell> {
        let mut sections: Vec<Section> = Vec::new();
        for p in s.pages.list()?.into_iter().filter(|p| p.lang == lang) {
            let item = NavItem {
                slug: p.slug.as_str().into(),
                title: p
                    .slug
                    .as_str()
                    .rsplit('/')
                    .next()
                    .unwrap_or_default()
                    .into(),
                current: p.slug.as_str() == path,
            };
            match sections.last_mut() {
                Some(sec) if sec.name == p.section => sec.items.push(item),
                _ => sections.push(Section {
                    name: p.section,
                    items: vec![item],
                }),
            }
        }
        Ok(Shell {
            query: query.into(),
            sections,
            ..Shell::bare(lang, title, path)
        })
    }
}
