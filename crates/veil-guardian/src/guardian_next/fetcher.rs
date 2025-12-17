use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::guardian_next::cache::disk::DiskCache;
use crate::guardian_next::cache::CacheStore;
use crate::guardian_next::error::GuardianNextError;
use crate::guardian_next::net::concurrency::{ConcurrencyGate, ConcurrencyPolicy};
use crate::guardian_next::net::http_client::{FetchJsonResult, HttpClient, ReqwestHttpClient};
use crate::guardian_next::net::retry::RetryRunner;
use crate::guardian_next::net::SingleFlight;
use crate::guardian_next::outcome::FetchOutcome;
use crate::guardian_next::types::{CacheEntry, CacheFreshness, CacheKey, CacheMeta};

#[derive(Clone)]
pub struct GuardianNextConfig {
    pub offline: bool,
    pub ttl: Duration,
    pub grace: Duration,
    pub http_timeout_ms: u64,
    pub retry: crate::guardian_next::net::retry::RetryPolicy,
    pub concurrency: usize,
    pub cache_dir: std::path::PathBuf,
}

impl Default for GuardianNextConfig {
    fn default() -> Self {
        Self {
            offline: false,
            ttl: Duration::from_secs(24 * 60 * 60),
            grace: Duration::from_secs(7 * 24 * 60 * 60),
            http_timeout_ms: 10000,
            retry: crate::guardian_next::net::retry::RetryPolicy::default(),
            concurrency: 10,
            cache_dir: std::env::temp_dir().join("veil_guardian_next"),
        }
    }
}

pub struct GuardianNext {
    config: GuardianNextConfig,
    cache: DiskCache,
    http: Arc<ReqwestHttpClient>,
    retry: RetryRunner,
    concurrency: ConcurrencyGate,
    single_flight:
        SingleFlight<CacheKey, Result<(serde_json::Value, FetchOutcome), Arc<GuardianNextError>>>,
}

impl GuardianNext {
    pub fn new(
        config: GuardianNextConfig,
        http: Arc<ReqwestHttpClient>,
    ) -> Result<Self, GuardianNextError> {
        let cache = DiskCache::new(config.cache_dir.clone()).map_err(GuardianNextError::Io)?;
        let retry = RetryRunner::new(config.retry.clone());
        let concurrency = ConcurrencyGate::new(ConcurrencyPolicy {
            max_in_flight: config.concurrency,
        });

        Ok(Self {
            config,
            cache,
            http,
            retry,
            concurrency,
            single_flight: SingleFlight::new(),
        })
    }

    pub async fn get_osv_details_json(
        &self,
        id: &str,
    ) -> Result<(serde_json::Value, FetchOutcome), Arc<GuardianNextError>> {
        let url = format!("https://api.osv.dev/v1/vulns/{}", id);
        let key = CacheKey(format!("osv:vuln:{}", id));
        self.get_json_with_policy(&key, &url).await
    }

    pub async fn get_json_with_policy(
        &self,
        key: &CacheKey,
        url: &str,
    ) -> Result<(serde_json::Value, FetchOutcome), Arc<GuardianNextError>> {
        let http = self.http.clone();
        let cache = self.cache.clone();
        let retry = self.retry.clone();
        let gate = self.concurrency.clone();

        let key_owned = key.clone();
        let url_owned = url.to_string();
        let offline = self.config.offline;
        let ttl = self.config.ttl;
        let grace = self.config.grace;
        let timeout_ms = self.config.http_timeout_ms;

        self.single_flight
            .do_call(key.clone(), move || async move {
                execute_fetch_logic(
                    key_owned, url_owned, http, cache, retry, gate, offline, ttl, grace, timeout_ms,
                )
                .await
                .map_err(Arc::new)
            })
            .await
    }
}

#[allow(clippy::too_many_arguments)]
async fn execute_fetch_logic(
    key: CacheKey,
    url: String,
    http: Arc<ReqwestHttpClient>,
    cache: DiskCache,
    retry: RetryRunner,
    gate: ConcurrencyGate,
    offline: bool,
    ttl: Duration,
    grace: Duration,
    timeout_ms: u64,
) -> Result<(serde_json::Value, FetchOutcome), GuardianNextError> {
    let now = SystemTime::now();
    let now_unix = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();

    // 1. Check Cache
    // Implicitly defines T = serde_json::Value via assignment
    let cache_val: Option<CacheEntry<serde_json::Value>> =
        cache.get(&key).map_err(GuardianNextError::Io)?; // Explicit check? get returns Result

    // Actually cache.get returns std::io::Result.
    // If we use helper var, types are inferred.

    if let Some(entry) = cache_val {
        let freshness = entry.freshness(now, grace);

        match freshness {
            CacheFreshness::Fresh => {
                let outcome = if offline {
                    FetchOutcome::OfflineUsedFreshCache
                } else {
                    FetchOutcome::CacheHitFresh
                };
                return Ok((entry.payload, outcome));
            }
            CacheFreshness::StaleUsable => {
                if offline {
                    return Ok((entry.payload, FetchOutcome::OfflineFallbackUsedStale));
                }
            }
            CacheFreshness::Expired => {
                if offline {
                    return Err(GuardianNextError::NoUsableCache(
                        "Offline and cache expired".into(),
                    ));
                }
            }
        }

        let etag = entry.meta.etag.clone();

        // Refresh with Concurrency Rate Limit
        let result = retry
            .run(|_| {
                let u = url.clone();
                let h = http.clone();
                let et = etag.clone();
                let g = gate.clone();
                async move {
                    let _permit = g.acquire().await;
                    h.fetch_json::<serde_json::Value>(&u, et.as_deref(), timeout_ms)
                        .await
                }
            })
            .await;

        match result {
            Ok(FetchJsonResult::Fetched {
                payload: val,
                etag: new_etag,
            }) => {
                let meta = CacheMeta::new(key.clone(), now, ttl.as_secs(), new_etag);
                let new_entry = CacheEntry {
                    meta,
                    payload: val.clone(),
                };
                cache.put(&new_entry)?;
                Ok((val, FetchOutcome::NetworkFetched))
            }
            Ok(FetchJsonResult::NotModified) => {
                // Explicitly specify T=Value for touch
                CacheStore::<serde_json::Value>::touch(&cache, &key, now_unix)?;

                // Re-read payload
                if let Some(e) = CacheStore::<serde_json::Value>::get(&cache, &key)? {
                    Ok((e.payload, FetchOutcome::NetworkNotModified))
                } else {
                    // Should not happen as we just touched
                    Ok((entry.payload, FetchOutcome::NetworkNotModified))
                }
            }
            Err(e) => {
                if let CacheFreshness::StaleUsable = freshness {
                    Ok((entry.payload, FetchOutcome::CacheHitStaleFallback))
                } else {
                    Err(e)
                }
            }
        }
    } else {
        if offline {
            return Err(GuardianNextError::NoUsableCache(
                "Offline and no cache".into(),
            ));
        }

        let result = retry
            .run(|_| {
                let u = url.clone();
                let h = http.clone();
                let g = gate.clone();
                async move {
                    let _permit = g.acquire().await;
                    h.fetch_json::<serde_json::Value>(&u, None, timeout_ms)
                        .await
                }
            })
            .await;

        match result {
            Ok(FetchJsonResult::Fetched {
                payload: val,
                etag: new_etag,
            }) => {
                let meta = CacheMeta::new(key.clone(), now, ttl.as_secs(), new_etag);
                let new_entry = CacheEntry {
                    meta,
                    payload: val.clone(),
                };
                cache.put(&new_entry)?;
                Ok((val, FetchOutcome::NetworkFetched))
            }
            Ok(FetchJsonResult::NotModified) => Err(GuardianNextError::Network(
                "Unexpected 304 without cache".into(),
            )),
            Err(e) => Err(e),
        }
    }
}
