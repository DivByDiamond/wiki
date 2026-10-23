//! Data shared by every reader template: the layout shell and date formatting.

mod admin_texts;
mod date;
mod texts;

pub use date::format_utc;

use crate::domain::page::Lang;
use crate::error::AppResult;
use crate::http::{
    middleware::{SignedIn, Viewer},
    state::AppState,
};

#[derive(Clone)]
pub struct NavItem {
    pub slug: String,
    pub title: String,
    pub current: bool,
}

pub struct Section {
    pub name: String,
    pub open: bool,
    pub items: Vec<NavItem>,
}

/// Everything `layout/base.html` needs; embedded in each page template.
pub struct Shell {
    pub lang: &'static str,
    request_path: Option<String>,
    pub title: String,
    /// Path after the language prefix, used by the language switcher.
    pub path: String,
    pub query: String,
    pub sections: Vec<Section>,
    /// Who is signed in, for the header; `None` when anonymous.
    pub user: Option<SignedIn>,
    pub headings: Vec<crate::render::document::Heading>,
    pub previous: Option<NavItem>,
    pub next: Option<NavItem>,
}

impl Shell {
    pub fn admin(viewer: Viewer, title_key: &str) -> Self {
        let mut shell = Self::bare(viewer.lang(), "", "start").for_viewer(viewer);
        shell.title = shell.text(title_key).into();
        shell
    }

    pub fn asset_version(&self) -> String {
        crate::http::assets::version()
    }

    pub fn text(&self, key: &str) -> &'static str {
        texts::text(self.lang, key)
    }

    /// Shows the signed-in user in the header.
    pub fn for_viewer(mut self, viewer: Viewer) -> Shell {
        if !viewer.1.prefixed {
            self.lang = viewer.lang().code();
            self.request_path = Some(viewer.1.path);
        }
        self.user = viewer.0;
        self
    }

    /// Keep administrative pages in place when selecting another language.
    pub fn language_url(&self, lang: &str) -> String {
        match &self.request_path {
            Some(path) => format!(
                "/language/{lang}?next={}",
                crate::http::locale::encode(path)
            ),
            None if self.path == "search" => format!(
                "/{lang}/search?q={}",
                crate::http::locale::encode(&self.query)
            ),
            None => format!("/{lang}/{}", self.path),
        }
    }

    pub fn current_url(&self) -> String {
        self.request_path
            .clone()
            .unwrap_or_else(|| format!("/{}/{}", self.lang, self.path))
    }

    /// Shell without a sidebar (error pages).
    pub fn bare(lang: Lang, title: &str, path: &str) -> Shell {
        Shell {
            lang: lang.code(),
            request_path: None,
            title: title.into(),
            path: path.into(),
            query: String::new(),
            sections: Vec::new(),
            user: None,
            headings: Vec::new(),
            previous: None,
            next: None,
        }
    }

    pub fn build(
        s: &AppState,
        lang: Lang,
        title: &str,
        path: &str,
        query: &str,
    ) -> AppResult<Shell> {
        let all = s.pages.list()?;
        let translated: std::collections::HashSet<_> = all
            .iter()
            .filter(|p| p.lang == lang)
            .map(|p| p.slug.as_str().to_owned())
            .collect();
        let mut pages: Vec<_> = all
            .into_iter()
            .filter(|p| {
                p.lang == lang || (p.lang == Lang::En && !translated.contains(p.slug.as_str()))
            })
            .collect();
        pages.sort_by(|a, b| {
            (
                texts::section_rank(&a.section),
                &a.section,
                a.position,
                &a.title,
            )
                .cmp(&(
                    texts::section_rank(&b.section),
                    &b.section,
                    b.position,
                    &b.title,
                ))
        });
        let mut sections: Vec<Section> = Vec::new();
        let mut flat = Vec::new();
        for p in pages {
            let item = NavItem {
                slug: p.slug.as_str().into(),
                title: p.title,
                current: p.slug.as_str() == path,
            };
            flat.push(item.clone());
            let name = if texts::section_rank(&p.section) < 7 || p.section.is_empty() {
                texts::text(lang.code(), &p.section).into()
            } else {
                p.section
            };
            match sections.last_mut() {
                Some(sec) if sec.name == name => {
                    sec.open |= item.current;
                    sec.items.push(item);
                }
                _ => sections.push(Section {
                    name,
                    open: item.current || sections.is_empty(),
                    items: vec![item],
                }),
            }
        }
        let index = flat.iter().position(|p| p.current);
        let previous = index
            .and_then(|i| i.checked_sub(1))
            .map(|i| flat[i].clone());
        let next = index.and_then(|i| flat.get(i + 1)).cloned();
        Ok(Shell {
            query: query.into(),
            sections,
            previous,
            next,
            ..Shell::bare(lang, title, path)
        })
    }
}
