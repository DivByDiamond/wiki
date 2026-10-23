use crate::auth::policy::Actor;
use crate::domain::{page::Lang, ports::pages::PageSummary};
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

async fn create(Auth(actor): Auth, Query(f): Query<Address>) -> Response {
    let run = || -> AppResult<Response> {
        admin_only(&actor)?;
        let (lang, slug) = target(&f.lang, f.slug.trim()).map_err(|_| {
            AppError::Invalid("Choose en, ru or zh and a lowercase page address.".into())
        })?;
        crate::http::ensure_slug_allowed(slug.as_str())?;
        Ok(Redirect::to(&format!("/admin/edit/{}/{}", lang.code(), slug.as_str())).into_response())
    };
    run().unwrap_or_else(fail)
}

struct Row {
    lang: &'static str,
    slug: String,
    title: String,
}
struct Group {
    section: String,
    pages: Vec<Row>,
}

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
struct Dashboard {
    shell: Shell,
    groups: Vec<Group>,
    deleted: Vec<Row>,
}

fn row(p: PageSummary) -> Row {
    Row {
        lang: p.lang.code(),
        slug: p.slug.as_str().to_owned(),
        title: p.title,
    }
}

async fn dashboard(State(s): State<AppState>, Auth(actor): Auth, viewer: Viewer) -> Response {
    show(&s, &actor, viewer).unwrap_or_else(fail)
}

fn show(s: &AppState, actor: &Actor, viewer: Viewer) -> AppResult<Response> {
    admin_only(actor)?;
    let mut groups: Vec<Group> = Vec::new();
    for p in s.pages.list()? {
        if let Some(group) = groups.iter_mut().find(|g| g.section == p.section) {
            group.pages.push(row(p));
        } else {
            groups.push(Group {
                section: p.section.clone(),
                pages: vec![row(p)],
            });
        }
    }
    reader::html(&Dashboard {
        shell: Shell::bare(Lang::En, "Admin", "start").for_viewer(viewer),
        groups,
        deleted: s.pages.list_deleted()?.into_iter().map(row).collect(),
    })
}
