use crate::domain::page::Lang;
use axum::http::HeaderMap;

/// Honor quality ordering and region/script subtags; unsupported defaults use English.
pub fn browser(headers: &HeaderMap) -> Lang {
    let Some(value) = headers.get("accept-language").and_then(|h| h.to_str().ok()) else {
        return Lang::En;
    };
    let mut best = None;
    for range in value.split(',') {
        let mut fields = range.trim().split(';');
        let tag = fields
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        let quality = fields
            .try_fold(1.0_f32, |q, field| match field.trim().split_once('=') {
                Some(("q", value)) => value
                    .parse::<f32>()
                    .ok()
                    .filter(|v| (0.0..=1.0).contains(v)),
                _ => Some(q),
            })
            .unwrap_or(0.0);
        if quality <= 0.0 {
            continue;
        }
        let Some(lang) = Lang::parse(tag.split('-').next().unwrap_or_default()) else {
            continue;
        };
        if best.is_none_or(|(_, q)| quality > q) {
            best = Some((lang, quality));
        }
    }
    best.map_or(Lang::En, |(lang, _)| lang)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn browser_quality_regions_and_fallback() {
        for (value, expected) in [
            ("ru-RU, en;q=0.8", Lang::Ru),
            ("zh-Hant-TW,en;q=0.8", Lang::Zh),
            ("ru;q=0.2, en-US;q=0.9", Lang::En),
            ("de-DE,zh-CN;q=0.6", Lang::Zh),
            ("ru;q=0,zh;q=0", Lang::En),
            ("ru;q=NaN, en", Lang::En),
            ("de-DE,*;q=0.5", Lang::En),
        ] {
            let mut headers = HeaderMap::new();
            headers.insert("accept-language", value.parse().unwrap());
            assert_eq!(browser(&headers), expected, "{value}");
        }
    }
}
