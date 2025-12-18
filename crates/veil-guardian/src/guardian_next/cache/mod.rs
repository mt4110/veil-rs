use crate::guardian_next::types::{CacheEntry, CacheKey};
use std::time::SystemTime;

pub mod disk;
pub mod memory;

pub trait CacheStore<T>: Send + Sync {
    fn get(&self, key: &CacheKey) -> std::io::Result<Option<CacheEntry<T>>>;
    fn put(&self, entry: &CacheEntry<T>) -> std::io::Result<()>;
    fn touch(&self, key: &CacheKey, now_unix: u64) -> std::io::Result<()>;

    fn now_unix(now: SystemTime) -> u64 {
        now.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}
