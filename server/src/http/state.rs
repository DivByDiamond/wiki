use crate::auth::{
    github::{GithubClient, RealGithub},
    tokens::session::Sessions,
};
use crate::config::Config;
use crate::domain::ports::{access::AccessRepo, pages::PageRepo, proposals::ProposalRepo};
use crate::http::limit::rate_limit::RateLimiter;
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
    pub config: Arc<Config>,
    pub github: Arc<dyn GithubClient>,
}

impl AppState {
    /// Wires the SQLite repositories, in-memory sessions and a default limiter.
    pub fn new(db: Db) -> Self {
        Self::with_limiter(db, RateLimiter::new(60, Duration::from_secs(60)))
    }

    /// Same wiring with a custom limiter (tests use small windows).
    pub fn with_limiter(db: Db, limiter: RateLimiter) -> Self {
        Self {
            pages: Arc::new(SqlitePages::new(db.clone())),
            access: Arc::new(SqliteAccess::new(db.clone())),
            proposals: Arc::new(SqliteProposals::new(db)),
            sessions: Arc::new(Sessions::new()),
            limiter: Arc::new(limiter),
            github: Arc::new(RealGithub::new(&Config::default())),
            config: Arc::new(Config::default()),
        }
    }

    /// Replaces the configuration and rebuilds the real GitHub client from it.
    pub fn with_config(mut self, config: Config) -> Self {
        self.github = Arc::new(RealGithub::new(&config));
        self.config = Arc::new(config);
        self
    }

    /// Swaps the GitHub client (tests inject a fake).
    pub fn with_github(mut self, github: Arc<dyn GithubClient>) -> Self {
        self.github = github;
        self
    }
}
