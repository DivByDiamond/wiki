use rand::RngCore;
use std::{collections::HashMap, sync::Mutex};

/// In-memory sessions: admins sign in again after a restart, no table needed.
#[derive(Default)]
pub struct Sessions {
    map: Mutex<HashMap<String, i64>>,
}

impl Sessions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&self, user_id: i64) -> String {
        let mut raw = [0u8; 24];
        rand::rng().fill_bytes(&mut raw);
        let token: String = raw.iter().map(|b| format!("{b:02x}")).collect();
        self.map.lock().unwrap().insert(token.clone(), user_id);
        token
    }

    pub fn user_id(&self, token: &str) -> Option<i64> {
        self.map.lock().unwrap().get(token).copied()
    }

    pub fn destroy(&self, token: &str) {
        self.map.lock().unwrap().remove(token);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_maps_to_user_until_destroyed() {
        let s = Sessions::new();
        let t = s.create(7);
        assert_eq!(s.user_id(&t), Some(7));
        s.destroy(&t);
        assert_eq!(s.user_id(&t), None);
        assert_eq!(s.user_id("nope"), None);
    }
}
