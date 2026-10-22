use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

/// Map size that triggers a sweep of expired windows.
const SWEEP_AT: usize = 1024;

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
        if hits.len() >= SWEEP_AT {
            let per = self.per;
            hits.retain(|_, (start, _)| now.duration_since(*start) < per);
        }
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

    #[cfg(test)]
    fn len(&self) -> usize {
        self.hits.lock().unwrap().len()
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

    #[test]
    fn expired_windows_are_evicted_when_the_map_grows() {
        let l = RateLimiter::new(1, Duration::from_millis(20));
        for i in 0..SWEEP_AT {
            l.check(&format!("ip{i}")).unwrap();
        }
        assert_eq!(l.len(), SWEEP_AT);
        std::thread::sleep(Duration::from_millis(30));
        l.check("fresh").unwrap();
        assert_eq!(l.len(), 1, "only the live window is kept");
    }
}
