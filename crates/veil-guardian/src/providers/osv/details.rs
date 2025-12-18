use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const CACHE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchOutcome {
    CacheHitFresh,
    CacheHitStale,
    CacheHitStaleFallback, // stale used, but refresh failed
    NetworkFetched,
    NetworkNotModified,
    OfflineUsedFreshCache,
    OfflineFallbackUsedStale,
    HitLegacyMigrated, // legacy satisfied request and migrated to v1
    FailedNoUsableCache,
}

impl FetchOutcome {
    /// Generate a consistent display label for this outcome.
    pub fn label(self) -> &'static str {
        match self {
            FetchOutcome::CacheHitFresh => "Hit (Fresh)",
            // Compatibility: Fresh in offline is still just "Hit (Fresh)"
            FetchOutcome::OfflineUsedFreshCache => "Hit (Fresh)",

            FetchOutcome::CacheHitStale => "Hit (Stale)",
            FetchOutcome::CacheHitStaleFallback => "Hit (Stale) [Fallback]",
            FetchOutcome::OfflineFallbackUsedStale => "Hit (Stale) [Offline Fallback]",
            FetchOutcome::HitLegacyMigrated => "Hit (Migrated)",

            FetchOutcome::NetworkFetched => "Fetched",
            FetchOutcome::NetworkNotModified => "Hit (Fresh) [304]",

            FetchOutcome::FailedNoUsableCache => "Miss (No Cache)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheStatus {
    Fresh,
    Stale,
    Expired,
}

#[derive(Debug, Clone)]
pub struct CachePolicy {
    pub fresh_ttl: Duration,
    pub stale_ttl: Duration,
}

impl Default for CachePolicy {
    fn default() -> Self {
        Self {
            // v0.11.3 Constitution: TTL=24h, Grace=7d (stale window = 8d)
            fresh_ttl: Duration::from_secs(24 * 60 * 60),
            stale_ttl: Duration::from_secs((24 + 7 * 24) * 60 * 60),
        }
    }
}

impl CachePolicy {
    pub fn classify_age(&self, age: Duration) -> CacheStatus {
        if age <= self.fresh_ttl {
            CacheStatus::Fresh
        } else if age <= self.stale_ttl {
            CacheStatus::Stale
        } else {
            CacheStatus::Expired
        }
    }

    pub fn classify(&self, fetched_at: SystemTime, now: SystemTime) -> CacheStatus {
        // Clock skew safe: if fetched_at is in the future, treat as fresh.
        let age = now.duration_since(fetched_at).unwrap_or(Duration::ZERO);
        self.classify_age(age)
    }
}

/// OSV details cache entry.
/// Store raw JSON to stay resilient if OSV schema expands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedVuln {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub vuln_id: String,

    /// Seconds since UNIX_EPOCH (UTC).
    #[serde(default)]
    pub fetched_at_unix: u64,

    /// ETag from the server (for conditional requests).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,

    /// Raw OSV /vulns/{id} JSON payload.
    pub vuln: Value,
}

impl CachedVuln {
    pub fn new(
        vuln_id: impl Into<String>,
        fetched_at: SystemTime,
        vuln: Value,
        etag: Option<String>,
    ) -> Self {
        let fetched_at_unix = fetched_at
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();

        Self {
            schema_version: CACHE_SCHEMA_VERSION,
            vuln_id: vuln_id.into(),
            fetched_at_unix,
            etag,
            vuln,
        }
    }

    pub fn fetched_at(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(self.fetched_at_unix)
    }

    pub fn status(&self, policy: &CachePolicy, now: SystemTime) -> CacheStatus {
        policy.classify(self.fetched_at(), now)
    }
}
