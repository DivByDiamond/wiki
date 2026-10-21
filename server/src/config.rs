use std::fmt;

#[derive(Clone)]
pub struct Config {
    pub bind: String,
    pub db_path: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    /// Public site URL without a trailing slash.
    pub public_url: String,
    /// Numeric GitHub user ids that become admins on sign-in (ids never change, logins do).
    pub admin_github_ids: Vec<i64>,
}

impl Default for Config {
    fn default() -> Self {
        Self::from_lookup(|_| None).expect("empty environment is valid")
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
            .field("admin_github_ids", &self.admin_github_ids)
            .finish()
    }
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Self::from_lookup(|k| std::env::var(k).ok())
    }

    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let or = |key: &str, default: &str| get(key).unwrap_or_else(|| default.into());
        let admin_github_ids = or("ADMIN_GITHUB_IDS", "")
            .split(',')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(|p| {
                p.parse::<i64>()
                    .map_err(|_| format!("ADMIN_GITHUB_IDS: {p:?} is not a numeric GitHub user id"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            bind: or("WIKI_BIND", "127.0.0.1:8080"),
            db_path: or("WIKI_DB", "wiki.db"),
            github_client_id: or("GITHUB_CLIENT_ID", "").trim().into(),
            github_client_secret: or("GITHUB_CLIENT_SECRET", "").trim().into(),
            public_url: or("PUBLIC_URL", "http://127.0.0.1:8080")
                .trim()
                .trim_end_matches('/')
                .into(),
            admin_github_ids,
        })
    }

    pub fn github_configured(&self) -> bool {
        !self.github_client_id.is_empty() && !self.github_client_secret.is_empty()
    }

    pub fn is_admin_id(&self, github_id: i64) -> bool {
        self.admin_github_ids.contains(&github_id)
    }

    pub fn secure_cookies(&self) -> bool {
        self.public_url.starts_with("https://")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(pairs: &[(&str, &str)]) -> Config {
        try_cfg(pairs).unwrap()
    }

    fn try_cfg(pairs: &[(&str, &str)]) -> Result<Config, String> {
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
        assert!(c.admin_github_ids.is_empty() && !c.github_configured());
        assert!(!c.secure_cookies());
    }

    #[test]
    fn parses_url_and_admin_logins() {
        let c = cfg(&[
            ("PUBLIC_URL", "https://wiki.example/"),
            ("ADMIN_GITHUB_IDS", " 42 , 7,,"),
            ("GITHUB_CLIENT_ID", "id"),
            ("GITHUB_CLIENT_SECRET", "s"),
        ]);
        assert_eq!(c.public_url, "https://wiki.example");
        assert_eq!(c.admin_github_ids, [42, 7]);
        assert!(c.is_admin_id(42) && !c.is_admin_id(43));
        assert!(c.github_configured() && c.secure_cookies());
    }

    #[test]
    fn debug_output_redacts_the_secret() {
        let c = cfg(&[("GITHUB_CLIENT_SECRET", "topsecretvalue")]);
        let shown = format!("{c:?}");
        assert!(!shown.contains("topsecretvalue") && shown.contains("<redacted>"));
    }

    #[test]
    fn non_numeric_admin_id_is_a_startup_error() {
        let err = try_cfg(&[("ADMIN_GITHUB_IDS", "42,alice")]).unwrap_err();
        assert!(err.contains("ADMIN_GITHUB_IDS") && err.contains("alice"));
    }
}
