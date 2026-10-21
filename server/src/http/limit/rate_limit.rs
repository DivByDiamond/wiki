use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

pub struct RateLimiter {
    max: u32,
    per: Duration,
    hits: Mutex<HashMap<String, (Instant, u32)>>,
}

impl RateLimiter {
    pub fn new(max: u32, per: Duration) -> Self {
        Self {
            max,
            per,
            hits: Mutex::new(HashMap::new()),
        }
    }

    /// Ok if the limit is not exhausted; Err holds the seconds to wait.
    pub fn check(&self, key: &str) -> Result<(), u64> {
        let mut hits = self.hits.lock().unwrap();
        let now = Instant::now();
        let entry = hits.entry(key.to_owned()).or_insert((now, 0));
        if now.duration_since(entry.0) >= self.per {
            *entry = (now, 0);
        }
        if entry.1 >= self.max {
            return Err((self.per - now.duration_since(entry.0)).as_secs().max(1));
        }
        entry.1 += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn blocks_after_max_and_reports_retry() {
        let l = RateLimiter::new(2, Duration::from_secs(60));
        assert!(l.check("ip").is_ok());
        assert!(l.check("ip").is_ok());
        let retry = l.check("ip").unwrap_err();
        assert!((1..=60).contains(&retry));
        assert!(l.check("other").is_ok());
    }
}
