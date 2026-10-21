use crate::auth::policy::Action;
use crate::error::AppResult;
use crate::http::{middleware::Auth, require, state::AppState};
use axum::{Json, extract::State};
use serde_json::{Value, json};

pub async fn export(State(s): State<AppState>, Auth(actor): Auth) -> AppResult<Json<Value>> {
    require(&actor, Action::Export)?;
    let mut pages = Vec::new();
    for p in s.pages.list()? {
        if let Some(page) = s.pages.find(&p.slug, p.lang)? {
            pages.push(json!({
                "slug": p.slug.as_str(),
                "lang": p.lang.code(),
                "body": page.current.body,
            }));
        }
    }
    Ok(Json(json!({ "pages": pages })))
}
