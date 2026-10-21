use rand::RngCore;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

/// A session ends 30 days after sign-in no matter what.
pub const ABSOLUTE_SECS: u64 = 30 * 24 * 3600;
/// A session ends after 7 days without a request.
pub const IDLE_SECS: u64 = 7 * 24 * 3600;

/// Source of "now" in unix seconds; tests inject their own.
pub trait Clock: Send + Sync {
    fn now(&self) -> u64;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
    }
}

struct Entry {
    user_id: i64,
    created_at: u64,
    last_seen: u64,
}

impl Entry {
    fn expired(&self, now: u64) -> bool {
        now.saturating_sub(self.created_at) >= ABSOLUTE_SECS
            || now.saturating_sub(self.last_seen) >= IDLE_SECS
    }
}

/// In-memory sessions: everyone signs in again after a restart, no table needed.
pub struct Sessions {
    map: Mutex<HashMap<String, Entry>>,
    clock: Arc<dyn Clock>,
}

impl Default for Sessions {
    fn default() -> Self {
        Self::with_clock(Arc::new(SystemClock))
    }
}

impl Sessions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_clock(clock: Arc<dyn Clock>) -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
            clock,
        }
    }

    pub fn create(&self, user_id: i64) -> String {
        let mut raw = [0u8; 24];
        rand::rng().fill_bytes(&mut raw);
        let token: String = raw.iter().map(|b| format!("{b:02x}")).collect();
        let now = self.clock.now();
        let mut map = self.map.lock().unwrap();
        map.retain(|_, e| !e.expired(now));
        map.insert(
            token.clone(),
            Entry {
                user_id,
                created_at: now,
                last_seen: now,
            },
        );
        token
    }

    /// Owner of a live token; refreshes the idle timer. Expired tokens are dropped.
    pub fn user_id(&self, token: &str) -> Option<i64> {
        let now = self.clock.now();
        let mut map = self.map.lock().unwrap();
        let entry = map.get_mut(token)?;
        if entry.expired(now) {
            map.remove(token);
            return None;
        }
        entry.last_seen = now;
        Some(entry.user_id)
    }

    pub fn destroy(&self, token: &str) {
        self.map.lock().unwrap().remove(token);
    }

    /// Signs the user out everywhere (used when blocking or demoting).
    pub fn destroy_for_user(&self, user_id: i64) {
        self.map.lock().unwrap().retain(|_, e| e.user_id != user_id);
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.map.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Fake(AtomicU64);

    impl Clock for Fake {
        fn now(&self) -> u64 {
            self.0.load(Ordering::SeqCst)
        }
    }

    fn sessions() -> (Sessions, Arc<Fake>) {
        let clock = Arc::new(Fake(AtomicU64::new(1_000)));
        (Sessions::with_clock(clock.clone()), clock)
    }

    fn advance(c: &Fake, secs: u64) {
        c.0.fetch_add(secs, Ordering::SeqCst);
    }

    #[test]
    fn token_maps_to_user_until_destroyed() {
        let s = Sessions::new();
        let t = s.create(7);
        assert_eq!(s.user_id(&t), Some(7));
        s.destroy(&t);
        assert_eq!(s.user_id(&t), None);
        assert_eq!(s.user_id("nope"), None);
    }

    #[test]
    fn idle_session_expires_and_is_removed() {
        let (s, c) = sessions();
        let t = s.create(1);
        advance(&c, IDLE_SECS + 1);
        assert_eq!(s.user_id(&t), None);
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn activity_extends_idle_but_not_absolute_lifetime() {
        let (s, c) = sessions();
        let t = s.create(1);
        for _ in 0..4 {
            advance(&c, IDLE_SECS - 10);
            assert_eq!(s.user_id(&t), Some(1));
        }
        // 4 * (7d - 10s) is about 28d: still inside 30 days.
        advance(&c, ABSOLUTE_SECS - 4 * (IDLE_SECS - 10) - 5);
        assert_eq!(s.user_id(&t), Some(1));
        advance(&c, 10);
        assert_eq!(s.user_id(&t), None, "absolute lifetime is not extended");
    }

    #[test]
    fn creating_a_session_prunes_expired_ones() {
        let (s, c) = sessions();
        let old = s.create(1);
        advance(&c, IDLE_SECS + 1);
        let fresh = s.create(2);
        assert_eq!(s.len(), 1);
        assert_eq!(s.user_id(&old), None);
        assert_eq!(s.user_id(&fresh), Some(2));
    }

    #[test]
    fn destroy_for_user_removes_only_that_users_sessions() {
        let (s, _) = sessions();
        let (a1, a2, b) = (s.create(1), s.create(1), s.create(2));
        s.destroy_for_user(1);
        assert_eq!((s.user_id(&a1), s.user_id(&a2)), (None, None));
        assert_eq!(s.user_id(&b), Some(2));
    }
}
