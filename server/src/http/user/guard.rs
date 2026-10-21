//! Small helpers shared by the sign-in and form routes: cookies and CSRF.

use crate::config::Config;
use crate::error::{AppError, AppResult};
use axum::http::HeaderMap;

/// Constant-time equality for secrets (lengths are not secret here).
pub fn ct_eq(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .fold(0u8, |acc, (x, y)| acc | (x ^ y))
            == 0
}

/// `Set-Cookie` value: HttpOnly, SameSite=Lax, Path=/, `Secure` on https.
pub fn set_cookie(name: &str, value: &str, max_age: u64, secure: bool) -> String {
    let flag = if secure { "; Secure" } else { "" };
    format!("{name}={value}; Max-Age={max_age}; Path=/; HttpOnly; SameSite=Lax{flag}")
}

/// `scheme://host[:port]` of a URL, lowercased.
fn origin_of(url: &str) -> String {
    let (scheme, rest) = url.split_once("://").unwrap_or(("", url));
    let host = rest.split('/').next().unwrap_or_default();
    format!("{scheme}://{host}").to_lowercase()
}

/// CSRF defence in depth on top of SameSite=Lax: a state-changing request whose
/// `Origin` header is present must come from the site's own origin. A missing
/// header is allowed because some clients omit it on same-site form posts.
pub fn check_origin(headers: &HeaderMap, cfg: &Config) -> AppResult<()> {
    let Some(origin) = headers.get("origin") else {
        return Ok(());
    };
    let own = origin_of(&cfg.public_url);
    match origin.to_str() {
        Ok(o) if o.to_lowercase() == own => Ok(()),
        _ => Err(AppError::Forbidden),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_origin(o: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert("origin", o.parse().unwrap());
        h
    }

    fn cfg(url: &str) -> Config {
        Config::from_lookup(|k| (k == "PUBLIC_URL").then(|| url.to_owned())).unwrap()
    }

    #[test]
    fn origin_rules() {
        let c = cfg("https://wiki.example/");
        assert!(check_origin(&HeaderMap::new(), &c).is_ok());
        assert!(check_origin(&with_origin("https://wiki.example"), &c).is_ok());
        for bad in ["https://evil.example", "http://wiki.example", "null"] {
            assert!(check_origin(&with_origin(bad), &c).is_err(), "{bad}");
        }
    }

    #[test]
    fn ct_eq_and_cookie_flags() {
        assert!(ct_eq("abc", "abc") && !ct_eq("abc", "abd") && !ct_eq("abc", "ab"));
        let c = set_cookie("sid", "x", 5, true);
        assert_eq!(
            c,
            "sid=x; Max-Age=5; Path=/; HttpOnly; SameSite=Lax; Secure"
        );
        assert!(!set_cookie("sid", "x", 5, false).contains("Secure"));
    }
}
