//! GitHub OAuth behind a trait so tests can inject a fake.

use crate::config::Config;
use serde::Deserialize;
use std::{future::Future, pin::Pin, time::Duration};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubUser {
    pub id: i64,
    pub login: String,
}

pub type ExchangeFuture<'a> = Pin<Box<dyn Future<Output = Result<GithubUser, String>> + Send + 'a>>;

pub trait GithubClient: Send + Sync {
    fn authorize_url(&self, state: &str) -> String;
    /// Trades the callback `code` for the signed-in GitHub identity.
    fn exchange<'a>(&'a self, code: &'a str) -> ExchangeFuture<'a>;
}

fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// No `scope`: identity only, GitHub's default public-profile access.
pub fn authorize_url(client_id: &str, redirect: &str, state: &str) -> String {
    format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&state={}",
        encode(client_id),
        encode(redirect),
        encode(state)
    )
}

#[derive(Deserialize)]
struct Token {
    access_token: String,
}

#[derive(Deserialize)]
struct Me {
    id: i64,
    login: String,
}

/// The real client; holds the secret, so it has no `Debug`.
pub struct RealGithub {
    client_id: String,
    client_secret: String,
    redirect: String,
    http: reqwest::Client,
}

impl RealGithub {
    pub fn new(cfg: &Config) -> Self {
        Self {
            client_id: cfg.github_client_id.clone(),
            client_secret: cfg.github_client_secret.clone(),
            redirect: format!("{}/auth/github/callback", cfg.public_url),
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    async fn fetch(&self, code: &str) -> Result<GithubUser, reqwest::Error> {
        let token: Token = self
            .http
            .post("https://github.com/login/oauth/access_token")
            .header("accept", "application/json")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("code", code),
                ("redirect_uri", self.redirect.as_str()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let me: Me = self
            .http
            .get("https://api.github.com/user")
            .header("user-agent", "oc2r-wiki")
            .header("accept", "application/json")
            .bearer_auth(token.access_token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(GithubUser {
            id: me.id,
            login: me.login,
        })
    }
}

impl GithubClient for RealGithub {
    fn authorize_url(&self, state: &str) -> String {
        authorize_url(&self.client_id, &self.redirect, state)
    }

    fn exchange<'a>(&'a self, code: &'a str) -> ExchangeFuture<'a> {
        // Error text comes from reqwest and never contains the secret (it is in the body only).
        Box::pin(async move {
            self.fetch(code)
                .await
                .map_err(|e| e.without_url().to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorize_url_has_client_state_and_encoded_redirect() {
        let u = authorize_url("cid", "https://wiki.example/auth/github/callback", "st");
        assert!(u.starts_with("https://github.com/login/oauth/authorize?"));
        assert!(u.contains("client_id=cid") && u.contains("state=st"));
        assert!(u.contains("redirect_uri=https%3A%2F%2Fwiki.example%2Fauth%2Fgithub%2Fcallback"));
        assert!(!u.contains("scope"));
    }

    #[test]
    fn real_client_builds_url_from_config() {
        let cfg = Config::from_lookup(|k| match k {
            "GITHUB_CLIENT_ID" => Some("abc".into()),
            "PUBLIC_URL" => Some("https://w.example/".into()),
            _ => None,
        });
        let u = RealGithub::new(&cfg).authorize_url("x y");
        assert!(u.contains("client_id=abc") && u.contains("state=x%20y"));
        assert!(u.contains("https%3A%2F%2Fw.example%2Fauth%2Fgithub%2Fcallback"));
    }
}
