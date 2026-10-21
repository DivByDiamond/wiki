use std::fmt;

#[derive(Clone)]
pub struct Config {
    pub bind: String,
    pub db_path: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    /// Public site URL without a trailing slash.
    pub public_url: String,
    /// Lowercased GitHub logins that become admins on sign-in.
    pub admin_logins: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self::from_lookup(|_| None)
    }
}

/// Manual `Debug` so the client secret can never reach logs.
impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("bind", &self.bind)
            .field("db_path", &self.db_path)
            .field("github_client_id", &self.github_client_id)
            .field("github_client_secret", &"<redacted>")
            .field("public_url", &self.public_url)
            .field("admin_logins", &self.admin_logins)
            .finish()
    }
}

impl Config {
    pub fn from_env() -> Self {
        Self::from_lookup(|k| std::env::var(k).ok())
    }

    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Self {
        let or = |key: &str, default: &str| get(key).unwrap_or_else(|| default.into());
        Self {
            bind: or("WIKI_BIND", "127.0.0.1:8080"),
            db_path: or("WIKI_DB", "wiki.db"),
            github_client_id: or("GITHUB_CLIENT_ID", "").trim().into(),
            github_client_secret: or("GITHUB_CLIENT_SECRET", "").trim().into(),
            public_url: or("PUBLIC_URL", "http://127.0.0.1:8080")
                .trim()
                .trim_end_matches('/')
                .into(),
            admin_logins: or("ADMIN_LOGINS", "")
                .split(',')
                .map(|l| l.trim().to_lowercase())
                .filter(|l| !l.is_empty())
                .collect(),
        }
    }

    pub fn github_configured(&self) -> bool {
        !self.github_client_id.is_empty() && !self.github_client_secret.is_empty()
    }

    pub fn is_admin_login(&self, login: &str) -> bool {
        self.admin_logins.contains(&login.to_lowercase())
    }

    pub fn secure_cookies(&self) -> bool {
        self.public_url.starts_with("https://")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(pairs: &[(&str, &str)]) -> Config {
        Config::from_lookup(|k| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| (*v).to_owned())
        })
    }

    #[test]
    fn defaults_leave_sign_in_unconfigured() {
        let c = cfg(&[]);
        assert_eq!(c.public_url, "http://127.0.0.1:8080");
        assert!(c.admin_logins.is_empty() && !c.github_configured());
        assert!(!c.secure_cookies());
    }

    #[test]
    fn parses_url_and_admin_logins() {
        let c = cfg(&[
            ("PUBLIC_URL", "https://wiki.example/"),
            ("ADMIN_LOGINS", " Alice , bob,,"),
            ("GITHUB_CLIENT_ID", "id"),
            ("GITHUB_CLIENT_SECRET", "s"),
        ]);
        assert_eq!(c.public_url, "https://wiki.example");
        assert_eq!(c.admin_logins, ["alice", "bob"]);
        assert!(c.is_admin_login("ALICE") && !c.is_admin_login("carol"));
        assert!(c.github_configured() && c.secure_cookies());
    }

    #[test]
    fn debug_output_redacts_the_secret() {
        let c = cfg(&[("GITHUB_CLIENT_SECRET", "topsecretvalue")]);
        let shown = format!("{c:?}");
        assert!(!shown.contains("topsecretvalue") && shown.contains("<redacted>"));
    }
}
