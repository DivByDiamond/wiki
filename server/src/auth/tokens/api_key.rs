use rand::RngCore;
use sha2::{Digest, Sha256};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn hash(key: &str) -> String {
    hex(&Sha256::digest(key.as_bytes()))
}

pub fn generate() -> (String, String) {
    let mut raw = [0u8; 32];
    rand::rng().fill_bytes(&mut raw);
    let key = format!("ocw_{}", hex(&raw));
    let h = hash(&key);
    (key, h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_key_matches_its_hash() {
        let (key, h) = generate();
        assert!(key.starts_with("ocw_") && key.len() == 4 + 64);
        assert_eq!(hash(&key), h);
        assert_eq!(h.len(), 64);
    }

    #[test]
    fn keys_are_unique() {
        assert_ne!(generate().0, generate().0);
    }

    #[test]
    fn hash_is_deterministic_and_differs_per_input() {
        assert_eq!(hash("a"), hash("a"));
        assert_ne!(hash("a"), hash("b"));
    }
}
