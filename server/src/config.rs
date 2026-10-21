pub struct Config {
    pub bind: String,
    pub db_path: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            bind: std::env::var("WIKI_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into()),
            db_path: std::env::var("WIKI_DB").unwrap_or_else(|_| "wiki.db".into()),
        }
    }
}
