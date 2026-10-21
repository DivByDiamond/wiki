use crate::auth::tokens::session::Sessions;
use crate::domain::ports::{access::AccessRepo, pages::PageRepo, proposals::ProposalRepo};
use crate::http::rate_limit::RateLimiter;
use crate::store::{
    access::{SqliteAccess, SqliteProposals},
    content::SqlitePages,
    db::Db,
};
use std::{sync::Arc, time::Duration};

#[derive(Clone)]
pub struct AppState {
    pub pages: Arc<dyn PageRepo>,
    pub access: Arc<dyn AccessRepo>,
    pub proposals: Arc<dyn ProposalRepo>,
    pub sessions: Arc<Sessions>,
    pub limiter: Arc<RateLimiter>,
}

impl AppState {
    /// Wires the SQLite repositories, in-memory sessions and a default limiter.
    pub fn new(db: Db) -> Self {
        Self {
            pages: Arc::new(SqlitePages::new(db.clone())),
            access: Arc::new(SqliteAccess::new(db.clone())),
            proposals: Arc::new(SqliteProposals::new(db)),
            sessions: Arc::new(Sessions::new()),
            limiter: Arc::new(RateLimiter::new(60, Duration::from_secs(60))),
        }
    }
}
