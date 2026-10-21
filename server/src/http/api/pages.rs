use crate::auth::policy::{Action, Actor};
use crate::domain::{
    page::{Lang, Slug},
    ports::pages::SaveOutcome,
    revision::{Author, NewRevision},
};
use crate::error::{AppError, AppResult};
use crate::http::{middleware::Auth, require, state::AppState};
use axum::{
    Json,
    body::Bytes,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub struct PageQuery {
    pub lang: Option<String>,
    pub summary: Option<String>,
}

fn parse(slug: &str, lang: Option<&str>) -> AppResult<(Slug, Lang)> {
    let slug = Slug::parse(slug).ok_or_else(|| AppError::Invalid("slug".into()))?;
    let lang = lang
        .and_then(Lang::parse)
        .ok_or_else(|| AppError::Invalid("lang".into()))?;
    Ok((slug, lang))
}

/// Revision author for a writing actor; keys are labelled by name.
fn author(s: &AppState, actor: &Actor) -> Author {
    match actor {
        Actor::Key { id, .. } => {
            let name = s
                .access
                .list_keys()
                .ok()
                .and_then(|keys| keys.into_iter().find(|k| k.id == *id))
                .map_or_else(|| format!("#{id}"), |k| k.name);
            Author::ApiKey(name)
        }
        Actor::User { id, .. } => Author::User(format!("#{id}")),
        Actor::Anonymous => Author::Import,
    }
}

pub async fn get_page(
    State(s): State<AppState>,
    Path(slug): Path<String>,
    Query(q): Query<PageQuery>,
) -> AppResult<Json<Value>> {
    let (slug, lang) = parse(&slug, q.lang.as_deref())?;
    let page = s.pages.find(&slug, lang)?.ok_or(AppError::NotFound)?;
    Ok(Json(json!({
        "slug": slug.as_str(),
        "lang": lang.code(),
        "body": page.current.body,
        "revision": page.current.id,
    })))
}

pub async fn put_page(
    State(s): State<AppState>,
    Auth(actor): Auth,
    Path(slug): Path<String>,
    Query(q): Query<PageQuery>,
    body: Bytes,
) -> AppResult<Json<Value>> {
    require(&actor, Action::WritePage)?;
    let (slug, lang) = parse(&slug, q.lang.as_deref())?;
    let body = String::from_utf8(body.to_vec())
        .map_err(|_| AppError::Invalid("body must be UTF-8".into()))?;
    let rev = NewRevision {
        body,
        summary: q.summary.unwrap_or_default(),
        author: author(&s, &actor),
    };
    let outcome = s.pages.save(&slug, lang, rev)?;
    Ok(Json(match outcome {
        SaveOutcome::Created(id) => json!({ "outcome": "created", "revision": id }),
        SaveOutcome::Updated(id) => json!({ "outcome": "updated", "revision": id }),
        SaveOutcome::Unchanged => json!({ "outcome": "unchanged" }),
    }))
}

pub async fn revisions(
    State(s): State<AppState>,
    Path(slug): Path<String>,
    Query(q): Query<PageQuery>,
) -> AppResult<Json<Value>> {
    let (slug, lang) = parse(&slug, q.lang.as_deref())?;
    let page = s.pages.find(&slug, lang)?.ok_or(AppError::NotFound)?;
    let items: Vec<Value> = s
        .pages
        .history(page.id)?
        .iter()
        .map(|r| {
            json!({
                "id": r.id,
                "summary": r.summary,
                "author": r.author.label(),
                "created_at": r.created_at,
            })
        })
        .collect();
    Ok(Json(json!(items)))
}

pub async fn list(State(s): State<AppState>) -> AppResult<Json<Value>> {
    let items: Vec<Value> = s
        .pages
        .list()?
        .iter()
        .map(|p| json!({ "slug": p.slug.as_str(), "lang": p.lang.code(), "section": p.section }))
        .collect();
    Ok(Json(json!(items)))
}
