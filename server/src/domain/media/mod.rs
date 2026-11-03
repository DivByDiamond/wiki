//! Uploaded images: type detection by magic bytes, content-hash names, the storage port.

mod sniff;

pub use sniff::ImageKind;

use super::ports::StoreResult;
use sha2::{Digest, Sha256};

/// Largest accepted image.
pub const MAX_BYTES: usize = 5 * 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub enum Rejected {
    Empty,
    TooLarge,
    /// Not png, jpeg, webp or gif (SVG included, it can carry scripts).
    UnsupportedType,
}

/// A validated upload: the name comes from the content, never from the client.
#[derive(Debug)]
pub struct Prepared {
    pub kind: ImageKind,
    pub sha256: String,
    pub name: String,
}

pub fn prepare(bytes: &[u8]) -> Result<Prepared, Rejected> {
    if bytes.is_empty() {
        return Err(Rejected::Empty);
    }
    if bytes.len() > MAX_BYTES {
        return Err(Rejected::TooLarge);
    }
    let kind = ImageKind::sniff(bytes).ok_or(Rejected::UnsupportedType)?;
    let sha256: String = Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let name = format!("{}.{}", &sha256[..32], kind.ext());
    Ok(Prepared { kind, sha256, name })
}

/// Markdown that embeds an uploaded image.
pub fn snippet(name: &str) -> String {
    format!("![](/media/{name})")
}

#[derive(Debug, Clone)]
pub struct MediaFile {
    pub name: String,
    pub size: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct MediaBlob {
    pub mime: String,
    pub sha256: String,
    pub data: Vec<u8>,
}

pub trait MediaRepo: Send + Sync {
    /// Stores the image; `false` when the same content is already there.
    fn add(&self, p: &Prepared, data: &[u8]) -> StoreResult<bool>;
    /// Newest first, without the bytes.
    fn list(&self) -> StoreResult<Vec<MediaFile>>;
    fn get(&self, name: &str) -> StoreResult<Option<MediaBlob>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\nrest";

    #[test]
    fn name_is_hash_and_extension() {
        let a = prepare(PNG).unwrap();
        assert_eq!(a.sha256.len(), 64);
        assert_eq!(a.name, format!("{}.png", &a.sha256[..32]));
        assert_eq!(a.name, prepare(PNG).unwrap().name);
    }

    #[test]
    fn rejects_empty_big_and_unknown() {
        assert_eq!(prepare(b"").unwrap_err(), Rejected::Empty);
        let big = vec![0u8; MAX_BYTES + 1];
        assert_eq!(prepare(&big).unwrap_err(), Rejected::TooLarge);
        for bad in [
            &b"<svg xmlns='x'></svg>"[..],
            b"<html>",
            b"GIF",
            b"RIFF1234WAVE",
        ] {
            assert_eq!(prepare(bad).unwrap_err(), Rejected::UnsupportedType);
        }
    }
}
