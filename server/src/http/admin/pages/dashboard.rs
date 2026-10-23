use crate::auth::policy::Actor;
use crate::domain::ports::pages::PageSummary;
use crate::error::{AppError, AppResult};
use crate::http::{
    admin::{admin_only, fail, target},
    middleware::{Auth, Viewer},
    reader::{self, view::Shell},
    state::AppState,
};
use askama::Template;
use axum::{
    Router,
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use serde::Deserialize;

pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin", get(dashboard))
        .route("/admin/create", get(create))
}

#[derive(Deserialize)]
struct Address {
    lang: String,
    slug: String,
}

async fn create(Auth(actor): Auth, viewer: Viewer, Query(f): Query<Address>) -> Response {
    let locale = viewer.lang();
    let run = || -> AppResult<Response> {
        admin_only(&actor)?;
        let (lang, slug) = target(&f.lang, f.slug.trim()).map_err(|_| {
            AppError::Invalid("Choose en, ru or zh and a lowercase page address.".into())
        })?;
        crate::http::ensure_slug_allowed(slug.as_str())?;
        Ok(Redirect::to(&format!("/admin/edit/{}/{}", lang.code(), slug.as_str())).into_response())
    };
    run().unwrap_or_else(|e| fail(locale, e))
}

struct Row {
    lang: &'static str,
    slug: String,
    title: String,
    section: String,
}
struct SectionOption {
    key: String,
    label: String,
}

#[derive(Default, Deserialize)]
struct Filters {
    #[serde(default)]
    q: String,
    #[serde(default)]
    lang: String,
    #[serde(default)]
    section: String,
}

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
struct Dashboard {
    shell: Shell,
    pages: Vec<Row>,
    sections: Vec<SectionOption>,
    filters: Filters,
    total: usize,
    deleted: Vec<Row>,
}

fn row(p: PageSummary, shell: &Shell) -> Row {
    Row {
        lang: p.lang.code(),
        slug: p.slug.as_str().to_owned(),
        title: p.title,
        section: section_label(&p.section, shell),
    }
}

fn section_label(key: &str, shell: &Shell) -> String {
    match key {
        "" => shell.text("admin_uncategorized").into(),
        "start" | "hardware" | "networking" | "robotics" | "systems" | "scripting" | "project" => {
            shell.text(key).into()
        }
        _ => key.into(),
    }
}

async fn dashboard(
    State(s): State<AppState>,
    Auth(actor): Auth,
    viewer: Viewer,
    Query(filters): Query<Filters>,
) -> Response {
    let locale = viewer.lang();
    show(&s, &actor, viewer, filters).unwrap_or_else(|e| fail(locale, e))
}

fn show(s: &AppState, actor: &Actor, viewer: Viewer, filters: Filters) -> AppResult<Response> {
    admin_only(actor)?;
    let shell = Shell::admin(viewer, "admin_pages");
    let all = s.pages.list()?;
    let total = all.len();
    let mut sections: Vec<SectionOption> = Vec::new();
    for p in &all {
        let key = if p.section.is_empty() {
            "_"
        } else {
            &p.section
        };
        if !sections.iter().any(|s| s.key == key) {
            sections.push(SectionOption {
                key: key.into(),
                label: section_label(&p.section, &shell),
            });
        }
    }
    sections.sort_by(|a, b| a.label.cmp(&b.label));
    let query = filters.q.trim().to_lowercase();
    let pages = all
        .into_iter()
        .filter(|p| {
            let section = if p.section.is_empty() {
                "_"
            } else {
                &p.section
            };
            (filters.lang.is_empty() || p.lang.code() == filters.lang)
                && (filters.section.is_empty() || section == filters.section)
                && (query.is_empty()
                    || p.title.to_lowercase().contains(&query)
                    || p.slug.as_str().to_lowercase().contains(&query)
                    || format!("/{}/{}", p.lang.code(), p.slug.as_str()).contains(&query))
        })
        .map(|p| row(p, &shell))
        .collect();
    let deleted = s
        .pages
        .list_deleted()?
        .into_iter()
        .map(|p| row(p, &shell))
        .collect();
    reader::html(&Dashboard {
        shell,
        pages,
        sections,
        filters,
        total,
        deleted,
    })
}
