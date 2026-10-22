pub mod page;
pub mod ports;
pub mod proposal;
pub mod revision;

/// Current unix time; domain and HTTP code use it instead of reaching into the store.
pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}
