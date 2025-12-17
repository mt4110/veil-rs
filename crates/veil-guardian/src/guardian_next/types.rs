use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheFreshness {
    Fresh,
    StaleUsable,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMeta {
    pub schema: u32,
    pub key: String,
    pub fetched_at_unix: u64, // epoch seconds
    pub ttl_seconds: u64,
    pub etag: Option<String>,
}

impl CacheMeta {
    pub fn new(key: CacheKey, now: SystemTime, ttl_seconds: u64, etag: Option<String>) -> Self {
        let fetched_at_unix = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        Self {
            schema: 1,
            key: key.0,
            fetched_at_unix,
            ttl_seconds,
            etag,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub meta: CacheMeta,
    pub payload: T,
}

impl<T> CacheEntry<T> {
    pub fn fetched_at(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(self.meta.fetched_at_unix)
    }

    pub fn ttl(&self) -> Duration {
        Duration::from_secs(self.meta.ttl_seconds)
    }

    pub fn age(&self, now: SystemTime) -> Duration {
        now.duration_since(self.fetched_at())
            .unwrap_or(Duration::ZERO)
    }

    pub fn freshness(&self, now: SystemTime, grace: Duration) -> CacheFreshness {
        let age = self.age(now);
        if age <= self.ttl() {
            CacheFreshness::Fresh
        } else if age <= self.ttl() + grace {
            CacheFreshness::StaleUsable
        } else {
            CacheFreshness::Expired
        }
    }
}
