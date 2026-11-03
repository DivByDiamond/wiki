use super::{PAGE_LIMIT, format_utc};
use crate::domain::feed::RecentChange;
use crate::http::state::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};

const FEED_LIMIT: u32 = PAGE_LIMIT;

/// Escapes text for XML and drops characters XML 1.0 forbids.
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            '\t' | '\n' | '\r' => out.push(c),
            c if (c as u32) < 0x20 || matches!(c as u32, 0xFFFE | 0xFFFF) => {}
            c => out.push(c),
        }
    }
    out
}

fn rfc3339(secs: i64) -> String {
    format!("{}:00Z", format_utc(secs).replace(' ', "T"))
}

/// `base` is already escaped; the rest of the URL holds only slug characters and digits.
fn entry(base: &str, c: &RecentChange) -> String {
    let url = format!(
        "{base}/{}/{}?rev={}",
        c.lang.code(),
        c.slug.as_str(),
        c.rev_id
    );
    let summary = if c.summary.is_empty() {
        "-"
    } else {
        &c.summary
    };
    format!(
        "<entry><id>{url}</id><title>{}</title><link rel=\"alternate\" href=\"{url}\"/>\
         <updated>{}</updated><author><name>{}</name></author>\
         <summary>{}</summary></entry>\n",
        esc(&c.title),
        rfc3339(c.created_at),
        esc(&c.author.label()),
        esc(summary),
    )
}

fn document(base: &str, changes: &[RecentChange]) -> String {
    let updated = changes.first().map_or(0, |c| c.created_at);
    let base = esc(base);
    let mut xml = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <feed xmlns=\"http://www.w3.org/2005/Atom\"><id>{base}/feed.atom</id>\
         <title>OC2R Wiki</title><link rel=\"self\" href=\"{base}/feed.atom\"/>\
         <link rel=\"alternate\" href=\"{base}/\"/><updated>{}</updated>\n",
        rfc3339(updated)
    );
    for c in changes {
        xml.push_str(&entry(&base, c));
    }
    xml.push_str("</feed>\n");
    xml
}

pub async fn feed(State(s): State<AppState>, headers: HeaderMap) -> Response {
    let changes = match s.recent.recent(FEED_LIMIT) {
        Ok(c) => c,
        Err(e) => return crate::error::AppError::from(e).into_response(),
    };
    let etag = format!(
        "\"{}-{}\"",
        changes.first().map_or(0, |c| c.rev_id),
        changes.len()
    );
    let common = [
        (header::CACHE_CONTROL, "public, max-age=300"),
        (header::ETAG, etag.as_str()),
    ];
    let fresh = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.split(',').any(|t| t.trim() == etag));
    if fresh {
        return (StatusCode::NOT_MODIFIED, common).into_response();
    }
    let mut res = (common, document(&s.config.public_url, &changes)).into_response();
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/atom+xml; charset=utf-8"),
    );
    res
}
