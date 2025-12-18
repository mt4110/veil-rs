use super::cache::Cache;
use super::details::{CachePolicy, CacheStatus, CachedVuln, FetchOutcome};
use super::details_store::{DetailsStore, StoreLoad};
use super::net::{
    backoff_delay, clamp_timeout, classify_error, classify_response, ConcurrencyGate, NetConfig,
    RetryClass, Sleeper, TimeBudget, TokioSleeper,
};
use crate::models::{Advisory, Ecosystem, PackageRef};
use crate::report::Vulnerability;
use crate::GuardianError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Metrics;
use futures::future::{BoxFuture, FutureExt, Shared};
use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

// Type aliases for Coalescing
type DetailsResult = Result<(Value, String, SystemTime), Arc<GuardianError>>;
type QueryBatchResult = Result<Vec<Option<Vec<OsvVuln>>>, Arc<GuardianError>>;

// Internal Helpers
// (CacheStatus is defined in details.rs)

#[derive(Debug)]
enum NetworkResult {
    Fetched(Value, Option<String>),
    NotModified,
}

// Key for Batch Query
type BatchKey = String;

const DEFAULT_OSV_URL: &str = "https://api.osv.dev/v1/querybatch";
const CHUNK_SIZE: usize = 1000;

#[derive(Serialize)]
struct BatchQuery<'a> {
    queries: Vec<Query<'a>>,
}

#[derive(Serialize)]
struct Query<'a> {
    package: OsvPackage<'a>,
    version: &'a str,
}

#[derive(Serialize)]
struct OsvPackage<'a> {
    name: &'a str,
    ecosystem: &'a str,
}

#[derive(Deserialize)]
pub struct BatchResponse {
    pub results: Vec<QueryResult>,
}

#[derive(Deserialize)]
pub struct QueryResult {
    pub vulns: Option<Vec<OsvVuln>>,
}

#[derive(Deserialize, Clone)]
pub struct OsvVuln {
    pub id: String,
    // OSV batch response doesn't always contain details/summary unless requested?
    // Actually standard OSV response includes 'summary' or 'details'.
    pub summary: Option<String>,
    pub details: Option<String>,
}

pub struct OsvClient {
    client: Client, // Async Client
    cache: Option<Cache>,
    details_store: Option<DetailsStore>,
    offline: bool,
    api_url: String,
    metrics: Option<Arc<Metrics>>,

    // Internal Runtime for Sync Bridge
    rt: Arc<tokio::runtime::Runtime>,

    // Coalescing States
    in_flight_details: Arc<Mutex<HashMap<String, Shared<BoxFuture<'static, DetailsResult>>>>>,
    in_flight_query: Arc<Mutex<HashMap<BatchKey, Shared<BoxFuture<'static, QueryBatchResult>>>>>,

    // Network & Sleeper (v0.11.4)
    net: NetConfig,
    sleeper: Arc<dyn Sleeper>,

    // v0.11.5 Concurrency
    gate: ConcurrencyGate,
    net_in_flight: Arc<AtomicU64>,
}

impl OsvClient {
    pub fn new(
        offline: bool,
        api_url: Option<String>,
        metrics: Option<Arc<Metrics>>,
        cache_dir: Option<std::path::PathBuf>,
    ) -> Self {
        // Priority: Argument -> Env Var -> Default
        let api_url = api_url
            .or_else(|| env::var("OSV_API_URL").ok())
            .unwrap_or_else(|| DEFAULT_OSV_URL.to_string());

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime for OsvClient");

        // Use NetConfig defaults for connect_timeout in prod
        let net = NetConfig::default();
        let client = Client::builder()
            .connect_timeout(net.connect_timeout)
            .build()
            .expect("Failed to build reqwest client");

        // Split cache_dir into sub-caches if provided, or pass None to use defaults/env
        // Point D: Unified Path Strategy.
        // Query Cache uses `<root>/osv` (which internally does /query?) No, Cache implementation uses root directly?
        // Wait, Cache::new(path) takes a directory.
        // User wants: <root>/osv/query and <root>/osv/vulns.
        // DetailsStore now expects <root>/osv and manages `vulns` inside.
        // So we pass `base.join("osv")` to DetailsStore.

        let (query_cache, details_cache) = if let Some(base) = cache_dir {
            // If base provided (e.g. /tmp/test), we want /tmp/test/osv as root.
            let osv_root = base.join("osv");

            // Query Cache: historically <root>/osv/query? or just <root>/osv?
            // Existing Cache::new uses the path provided as the directory.
            // Let's explicitly segregate: `osv_root.join("query")`.
            (Some(osv_root.join("query")), Some(osv_root))
        } else {
            (None, None)
        };

        Self {
            client,
            cache: if !offline {
                Cache::new(query_cache)
            } else {
                None
            },
            details_store: DetailsStore::new(details_cache),
            offline,
            api_url,
            metrics,
            rt: Arc::new(rt),
            in_flight_details: Arc::new(Mutex::new(HashMap::new())),
            in_flight_query: Arc::new(Mutex::new(HashMap::new())),
            net: net.clone(),
            sleeper: Arc::new(TokioSleeper),
            gate: ConcurrencyGate::new(net.concurrency.clone()),
            net_in_flight: Arc::new(AtomicU64::new(0)),
        }
    }

    /// For testing: inject a custom store and/or URL
    pub fn new_custom(offline: bool, store: Option<DetailsStore>, api_url: Option<String>) -> Self {
        let api_url = api_url
            .or_else(|| env::var("OSV_API_URL").ok())
            .unwrap_or_else(|| DEFAULT_OSV_URL.to_string());

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime for OsvClient");

        let net = NetConfig::default();
        let client = Client::builder()
            .connect_timeout(net.connect_timeout)
            .build()
            .expect("Failed to build reqwest client");

        Self {
            client,
            cache: if !offline { Cache::new(None) } else { None },
            details_store: store,
            offline,
            api_url,
            metrics: None,
            rt: Arc::new(rt),
            in_flight_details: Arc::new(Mutex::new(HashMap::new())),
            in_flight_query: Arc::new(Mutex::new(HashMap::new())),
            net: net.clone(),
            sleeper: Arc::new(TokioSleeper),
            gate: ConcurrencyGate::new(net.concurrency.clone()),
            net_in_flight: Arc::new(AtomicU64::new(0)),
        }
    }

    // v0.11.5: test helper (keeps old signature intact)
    pub fn new_custom_with_net_and_metrics(
        offline: bool,
        store: Option<DetailsStore>,
        api_url: Option<String>,
        net: NetConfig,
        sleeper: Arc<dyn Sleeper>,
        metrics: Option<Arc<Metrics>>,
    ) -> Self {
        let api_url = api_url
            .or_else(|| env::var("OSV_API_URL").ok())
            .unwrap_or_else(|| DEFAULT_OSV_URL.to_string());

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime for OsvClient");

        let client = Client::builder()
            .connect_timeout(net.connect_timeout)
            .build()
            .expect("Failed to build reqwest client");

        Self {
            client,
            cache: if !offline { Cache::new(None) } else { None },
            details_store: store,
            offline,
            api_url,
            metrics,
            rt: Arc::new(rt),
            in_flight_details: Arc::new(Mutex::new(HashMap::new())),
            in_flight_query: Arc::new(Mutex::new(HashMap::new())),
            gate: ConcurrencyGate::new(net.concurrency.clone()),
            net_in_flight: Arc::new(AtomicU64::new(0)),
            net,
            sleeper,
        }
    }

    pub fn new_custom_with_net(
        offline: bool,
        store: Option<DetailsStore>,
        api_url: Option<String>,
        net: NetConfig,
        sleeper: Arc<dyn Sleeper>,
    ) -> Self {
        Self::new_custom_with_net_and_metrics(offline, store, api_url, net, sleeper, None)
    }

    fn env_flag(name: &str) -> bool {
        matches!(
            std::env::var(name).as_deref(),
            Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("on")
        )
    }

    /// Fetch vulnerability details (Sync Wrapper)
    pub fn fetch_vuln_details(
        &self,
        id: &str,
    ) -> Result<(Value, String, SystemTime), GuardianError> {
        let id_owned = id.to_string();
        self.rt
            .block_on(async { self.fetch_vuln_details_async(&id_owned).await })
            .map_err(|arc_err| match Arc::try_unwrap(arc_err) {
                Ok(e) => e,
                Err(arc) => GuardianError::NetworkError(arc.to_string()), // Fallback clone
            })
    }

    /// Async Fetch with Coalescing (Delegates to Internal)
    async fn fetch_vuln_details_async(&self, id: &str) -> DetailsResult {
        let this = self.clone_for_task();
        this.fetch_vuln_details_with_coalescing(id).await
    }

    // Helper to clone state for tasks
    fn clone_for_task(&self) -> OsvClientInternal {
        OsvClientInternal {
            client: self.client.clone(),
            cache: self.cache.clone(),
            details_store: self.details_store.clone(),
            offline: self.offline,
            api_url: self.api_url.clone(),
            metrics: self.metrics.clone(),
            in_flight_details: self.in_flight_details.clone(),
            in_flight_query: self.in_flight_query.clone(),
            net: self.net.clone(),
            sleeper: self.sleeper.clone(),
            gate: self.gate.clone(),
            net_in_flight: self.net_in_flight.clone(),
        }
    }

    pub fn check_packages(
        &self,
        packages: &[PackageRef],
        show_details: bool,
    ) -> Result<Vec<Vulnerability>, GuardianError> {
        // Run Async
        let this = self.clone_for_task();
        let pkgs = packages.to_vec();

        self.rt
            .block_on(async { this.check_packages_async_inner(pkgs, show_details).await })
            .map_err(|arc| match Arc::try_unwrap(arc) {
                Ok(e) => e,
                Err(arc) => GuardianError::NetworkError(arc.to_string()),
            })
    }
}

// v0.11.5: keep permit + concurrency counter together, and guarantee drop-before-await
struct NetConcurrencyGuard {
    _permit: tokio::sync::OwnedSemaphorePermit,
    in_flight: Arc<AtomicU64>,
}
impl NetConcurrencyGuard {
    fn new(
        permit: tokio::sync::OwnedSemaphorePermit,
        in_flight: Arc<AtomicU64>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Self {
        let cur = in_flight.fetch_add(1, Ordering::Relaxed) + 1;
        if let Some(m) = metrics {
            m.observe_concurrency(cur);
        }
        Self {
            _permit: permit,
            in_flight,
        }
    }
}
impl Drop for NetConcurrencyGuard {
    fn drop(&mut self) {
        self.in_flight.fetch_sub(1, Ordering::Relaxed);
    }
}

#[derive(Clone)]
struct OsvClientInternal {
    client: Client,
    cache: Option<Cache>,
    details_store: Option<DetailsStore>,
    offline: bool,
    api_url: String,
    metrics: Option<Arc<Metrics>>,
    in_flight_details: Arc<Mutex<HashMap<String, Shared<BoxFuture<'static, DetailsResult>>>>>,
    in_flight_query: Arc<Mutex<HashMap<BatchKey, Shared<BoxFuture<'static, QueryBatchResult>>>>>,
    net: NetConfig,
    sleeper: Arc<dyn Sleeper>,

    // v0.11.5 Concurrency
    gate: ConcurrencyGate,
    net_in_flight: Arc<AtomicU64>,
}

#[derive(Debug, Clone, Copy)]
enum NetOp {
    QueryBatch,
    Details,
}

impl OsvClientInternal {
    async fn send_with_retry<F>(
        &self,
        op: NetOp,
        mut make_req: F,
    ) -> Result<reqwest::Response, GuardianError>
    where
        F: FnMut(std::time::Duration) -> reqwest::RequestBuilder,
    {
        let budget = TimeBudget::new(self.net.total_budget);
        let mut attempt = 1usize;
        let mut retry_index = 0usize;

        loop {
            // v0.11.5: budget-aware acquire (avoid waiting forever)
            let Some(rem_for_gate) = budget.remaining() else {
                if let Some(m) = &self.metrics {
                    m.net_budget_exceeded.fetch_add(1, Ordering::Relaxed);
                }
                return Err(GuardianError::NetworkError(
                    "OSV budget exceeded".to_string(),
                ));
            };

            let permit = match tokio::time::timeout(
                rem_for_gate,
                self.gate.acquire_with_metrics(self.metrics.as_deref()),
            )
            .await
            {
                Ok(p) => p,
                Err(_) => {
                    if let Some(m) = &self.metrics {
                        m.net_budget_exceeded.fetch_add(1, Ordering::Relaxed);
                    }
                    return Err(GuardianError::NetworkError(
                        "OSV budget exceeded".to_string(),
                    ));
                }
            };
            let net_guard =
                NetConcurrencyGuard::new(permit, self.net_in_flight.clone(), &self.metrics);

            // clamp per-request timeout AFTER waiting for permit
            let Some(t) = clamp_timeout(&budget, self.net.per_request_timeout) else {
                drop(net_guard);
                if let Some(m) = &self.metrics {
                    m.net_budget_exceeded.fetch_add(1, Ordering::Relaxed);
                }
                return Err(GuardianError::NetworkError(
                    "OSV budget exceeded".to_string(),
                ));
            };

            let start = std::time::Instant::now();
            let resp = make_req(t).send().await;

            // metrics
            if let Some(m) = &self.metrics {
                match op {
                    NetOp::Details => {
                        m.req_details.fetch_add(1, Ordering::Relaxed);
                        m.time_osv_details_ms
                            .fetch_add(start.elapsed().as_millis() as u64, Ordering::Relaxed);
                    }
                    NetOp::QueryBatch => {
                        m.req_querybatch.fetch_add(1, Ordering::Relaxed);
                        m.time_osv_query_ms
                            .fetch_add(start.elapsed().as_millis() as u64, Ordering::Relaxed);
                    }
                }
            }

            match resp {
                Ok(r) => {
                    let class = classify_response(r.status().as_u16(), r.headers());

                    if r.status() == 429 {
                        if let Some(m) = &self.metrics {
                            m.net_limit_exceeded.fetch_add(1, Ordering::Relaxed);
                        }
                    }

                    match class {
                        RetryClass::Success => return Ok(r),
                        RetryClass::Fatal => {
                            return Err(GuardianError::NetworkError(format!(
                                "OSV API error: {}",
                                r.status()
                            )));
                        }
                        RetryClass::RetryAfter(_) | RetryClass::Backoff => {
                            let status = r.status();
                            // Drop response to release connection/socket before sleeping
                            drop(r);
                            // v0.11.5: release permit BEFORE sleeping
                            drop(net_guard);

                            if attempt >= self.net.retry.max_attempts {
                                return Err(GuardianError::NetworkError(format!(
                                    "OSV API failed after {} attempts (last status: {})",
                                    attempt, status
                                )));
                            }

                            let delay = match class {
                                RetryClass::RetryAfter(d) => d,
                                _ => {
                                    retry_index += 1;
                                    backoff_delay(&self.net.retry, retry_index)
                                }
                            };

                            let Some(rem) = budget.remaining() else {
                                if let Some(m) = &self.metrics {
                                    m.net_budget_exceeded.fetch_add(1, Ordering::Relaxed);
                                }
                                return Err(GuardianError::NetworkError(
                                    "OSV budget exceeded".to_string(),
                                ));
                            };
                            if delay >= rem {
                                if let Some(m) = &self.metrics {
                                    m.net_budget_exceeded.fetch_add(1, Ordering::Relaxed);
                                }
                                return Err(GuardianError::NetworkError(
                                    "OSV budget exceeded".to_string(),
                                ));
                            }

                            if let Some(m) = &self.metrics {
                                m.net_retry_attempts.fetch_add(1, Ordering::Relaxed);
                                m.net_retry_sleep_ms
                                    .fetch_add(delay.as_millis() as u64, Ordering::Relaxed);
                            }
                            self.sleeper.sleep(delay).await;
                            attempt += 1;
                            continue;
                        }
                    }
                }
                Err(e) => {
                    let class = classify_error(&e);
                    if class == RetryClass::Fatal {
                        return Err(GuardianError::NetworkError(e.to_string()));
                    }

                    if attempt >= self.net.retry.max_attempts {
                        return Err(GuardianError::NetworkError(format!(
                            "OSV request failed after {} attempts: {}",
                            attempt, e
                        )));
                    }

                    retry_index += 1;
                    let delay = backoff_delay(&self.net.retry, retry_index);

                    // release permit BEFORE sleeping
                    drop(net_guard);

                    let Some(rem) = budget.remaining() else {
                        if let Some(m) = &self.metrics {
                            m.net_budget_exceeded.fetch_add(1, Ordering::Relaxed);
                        }
                        return Err(GuardianError::NetworkError(
                            "OSV budget exceeded".to_string(),
                        ));
                    };
                    if delay >= rem {
                        if let Some(m) = &self.metrics {
                            m.net_budget_exceeded.fetch_add(1, Ordering::Relaxed);
                        }
                        return Err(GuardianError::NetworkError(
                            "OSV budget exceeded".to_string(),
                        ));
                    }

                    if let Some(m) = &self.metrics {
                        m.net_retry_attempts.fetch_add(1, Ordering::Relaxed);
                        m.net_retry_sleep_ms
                            .fetch_add(delay.as_millis() as u64, Ordering::Relaxed);
                    }
                    self.sleeper.sleep(delay).await;
                    attempt += 1;
                }
            }
        }
    }
    async fn fetch_vuln_details_with_coalescing(&self, id: &str) -> DetailsResult {
        let future = {
            let mut map = self.in_flight_details.lock().unwrap();

            if let Some(fut) = map.get(id) {
                if let Some(m) = &self.metrics {
                    m.coalesced_waiters.fetch_add(1, Ordering::Relaxed);
                }
                fut.clone()
            } else {
                // Create Future (fast, safe to do under lock)
                let id_owned = id.to_string();
                let this = self.clone();

                let future =
                    async move { this.fetch_vuln_details_network_or_cache(&id_owned).await }
                        .boxed()
                        .shared();

                map.insert(id.to_string(), future.clone());
                future
            }
        };

        // Await (Lock is dropped)
        let result = future.await;

        // Cleanup
        {
            let mut map = self.in_flight_details.lock().unwrap();
            map.remove(id);
        }

        result
    }

    async fn fetch_vuln_details_network_or_cache(&self, id: &str) -> DetailsResult {
        let now = SystemTime::now();
        let policy = CachePolicy::default();
        let force_refresh = OsvClient::env_flag("VEIL_OSV_FORCE_REFRESH");

        // 1. Check Cache
        // Using StoreLoad to detect migrations
        let cache_result = if let Some(store) = &self.details_store {
            store.load(id)
        } else {
            StoreLoad::Miss {
                quarantined: Default::default(),
            }
            // Note: If no store, we synthesize empty miss.
            // Ideally should just be Miss. But StoreLoad requires fields.
            // Wait, we defined StoreLoad in details_store.
            // We can't construct it easily if we don't import QuarantineFlags or if they are private?
            // They are pub.
            // But actually simpler: make `store.load` handle the logic.
            // If store is None, we just treat as simple miss without flags.
        };

        // Handle Metrics for Quarantine
        if let Some(m) = &self.metrics {
            let q_flags = match &cache_result {
                StoreLoad::Hit { quarantined, .. } => quarantined,
                StoreLoad::Miss { quarantined } => quarantined,
            };
            if q_flags.corrupt {
                m.cache_quarantine_corrupt.fetch_add(1, Ordering::Relaxed);
            }
            if q_flags.unsupported {
                m.cache_quarantine_unsupported
                    .fetch_add(1, Ordering::Relaxed);
            }
            if q_flags.conflict {
                m.cache_quarantine_conflict.fetch_add(1, Ordering::Relaxed);
            }
        }

        if let StoreLoad::Hit {
            entry,
            source: _,
            migrated,
            quarantined: _,
        } = cache_result
        {
            // Count Legacy Migration
            if migrated {
                if let Some(m) = &self.metrics {
                    m.cache_hit_legacy_total.fetch_add(1, Ordering::Relaxed);
                }
            }

            let mut status = entry.status(&policy, now);
            // Force refresh should NOT violate "Fresh never fetch".
            // Also, offline stays strict: do not relax Expired -> usable.
            if force_refresh && !self.offline && status != CacheStatus::Fresh {
                status = CacheStatus::Expired;
            }

            match status {
                CacheStatus::Fresh => {
                    if let Some(m) = &self.metrics {
                        m.cache_fresh.fetch_add(1, Ordering::Relaxed);
                    }
                    let outcome = if self.offline {
                        FetchOutcome::OfflineUsedFreshCache
                    } else if migrated {
                        FetchOutcome::HitLegacyMigrated
                    } else {
                        FetchOutcome::CacheHitFresh
                    };
                    return Ok((
                        entry.vuln.clone(),
                        outcome.label().to_string(),
                        entry.fetched_at(),
                    ));
                }
                CacheStatus::Stale => {
                    if let Some(m) = &self.metrics {
                        m.cache_stale.fetch_add(1, Ordering::Relaxed);
                    }
                    if self.offline {
                        let outcome = if migrated {
                            FetchOutcome::HitLegacyMigrated
                        } else {
                            FetchOutcome::OfflineFallbackUsedStale
                        };
                        return Ok((
                            entry.vuln.clone(),
                            outcome.label().to_string(),
                            entry.fetched_at(),
                        ));
                    }
                    // Online: try refresh, fallback to stale on error
                    match self.fetch_details_network(id, entry.etag.as_deref()).await {
                        Ok(NetworkResult::Fetched(details, etag)) => {
                            if let Some(m) = &self.metrics {
                                m.net_fetched.fetch_add(1, Ordering::Relaxed);
                            }
                            if let Some(store) = &self.details_store {
                                let _ =
                                    store.save(&CachedVuln::new(id, now, details.clone(), etag));
                            }
                            return Ok((
                                details,
                                FetchOutcome::NetworkFetched.label().to_string(),
                                now,
                            ));
                        }
                        Ok(NetworkResult::NotModified) => {
                            if let Some(m) = &self.metrics {
                                m.net_not_modified.fetch_add(1, Ordering::Relaxed);
                            }
                            // Update timestamp (touch)
                            if let Some(store) = &self.details_store {
                                let mut new_entry = entry.clone();
                                new_entry.fetched_at_unix = now
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or(Duration::ZERO)
                                    .as_secs();
                                let _ = store.save(&new_entry);
                            }
                            return Ok((
                                entry.vuln.clone(),
                                FetchOutcome::NetworkNotModified.label().to_string(),
                                now,
                            ));
                        }
                        Err(_) => {
                            // Fallback
                            let outcome = if migrated {
                                FetchOutcome::HitLegacyMigrated
                            } else {
                                FetchOutcome::CacheHitStaleFallback
                            };
                            return Ok((
                                entry.vuln.clone(),
                                outcome.label().to_string(),
                                entry.fetched_at(),
                            ));
                        }
                    }
                }
                CacheStatus::Expired => {
                    if let Some(m) = &self.metrics {
                        m.cache_miss.fetch_add(1, Ordering::Relaxed);
                    }
                    if self.offline {
                        return Err(Arc::new(GuardianError::NetworkError(
                            "Offline and cache expired".to_string(),
                        )));
                    }
                    // Must fetch
                    match self.fetch_details_network(id, entry.etag.as_deref()).await {
                        Ok(NetworkResult::Fetched(details, etag)) => {
                            if let Some(m) = &self.metrics {
                                m.net_fetched.fetch_add(1, Ordering::Relaxed);
                            }
                            if let Some(store) = &self.details_store {
                                let _ =
                                    store.save(&CachedVuln::new(id, now, details.clone(), etag));
                            }
                            return Ok((
                                details,
                                FetchOutcome::NetworkFetched.label().to_string(),
                                now,
                            ));
                        }
                        Ok(NetworkResult::NotModified) => {
                            if let Some(m) = &self.metrics {
                                m.net_not_modified.fetch_add(1, Ordering::Relaxed);
                            }
                            if let Some(store) = &self.details_store {
                                let mut new_entry = entry.clone();
                                new_entry.fetched_at_unix = now
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or(Duration::ZERO)
                                    .as_secs();
                                let _ = store.save(&new_entry);
                            }
                            return Ok((
                                entry.vuln.clone(),
                                FetchOutcome::NetworkNotModified.label().to_string(),
                                now,
                            ));
                        }
                        Err(e) => return Err(Arc::new(e)),
                    }
                }
            }
        }

        // 2. No Cache
        if let Some(m) = &self.metrics {
            m.cache_miss.fetch_add(1, Ordering::Relaxed);
        }
        if self.offline {
            return Err(Arc::new(GuardianError::NetworkError(format!(
                "Offline: No details cached for {}",
                id
            ))));
        }

        match self.fetch_details_network(id, None).await {
            Ok(NetworkResult::Fetched(details, etag)) => {
                if let Some(m) = &self.metrics {
                    m.net_fetched.fetch_add(1, Ordering::Relaxed);
                }
                if let Some(store) = &self.details_store {
                    let _ = store.save(&CachedVuln::new(id, now, details.clone(), etag));
                }
                Ok((
                    details,
                    FetchOutcome::NetworkFetched.label().to_string(),
                    now,
                ))
            }
            Ok(NetworkResult::NotModified) => Err(Arc::new(GuardianError::NetworkError(
                "Unexpected 304 with no cache".to_string(),
            ))),
            Err(e) => Err(Arc::new(e)),
        }
    }

    async fn fetch_details_network(
        &self,
        id: &str,
        etag: Option<&str>,
    ) -> Result<NetworkResult, GuardianError> {
        // Construct URL
        let base_url = &self.api_url;
        let url = if base_url.ends_with("/querybatch") {
            base_url.replace("/querybatch", &format!("/vulns/{}", id))
        } else {
            let parts: Vec<&str> = base_url.split('/').collect();
            let parent = if parts.len() > 1 && parts.last() == Some(&"querybatch") {
                &base_url[..base_url.len() - "/querybatch".len()]
            } else {
                base_url.trim_end_matches('/')
            };
            format!("{}/vulns/{}", parent, id)
        };

        let etag_owned = etag.map(|s| s.to_string());

        // send_with_retry handles metrics, timeout, and retries
        let resp = self
            .send_with_retry(NetOp::Details, |timeout| {
                let mut req = self.client.get(&url).timeout(timeout);
                if let Some(val) = &etag_owned {
                    req = req.header("If-None-Match", val);
                }
                req
            })
            .await?;

        if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
            return Ok(NetworkResult::NotModified);
        }

        if !resp.status().is_success() {
            return Err(GuardianError::NetworkError(format!(
                "OSV API Details Error: {}",
                resp.status()
            )));
        }

        let new_etag = resp
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let val: Value = resp
            .json()
            .await
            .map_err(|e| GuardianError::NetworkError(format!("JSON Parse: {}", e)))?;

        // Verify ID
        if val.get("id").and_then(|v| v.as_str()) != Some(id) {
            return Err(GuardianError::NetworkError(
                "Returned JSON missing ID or ID mismatch".to_string(),
            ));
        }

        Ok(NetworkResult::Fetched(val, new_etag))
    }

    async fn check_packages_async_inner(
        &self,
        packages: Vec<PackageRef>,
        show_details: bool,
    ) -> Result<Vec<Vulnerability>, Arc<GuardianError>> {
        let mut vulnerabilities = Vec::new();
        let chunk_size = CHUNK_SIZE;

        // Note: For B2 Parallel Limits we would use stream::buffer_unordered here.
        // For B1, sequential chunks are fine, but queries inside might be coalesced if we loop?
        // Actually chunks are processed sequentially in current code.
        // Coalescing works if OTHER threads/calls are happening.
        // OR if we parallelize chunks.
        // User asked for "B2 Parallel Limits" later.
        // For now, keep sequential chunk processing but async.

        for chunk in packages.chunks(chunk_size) {
            // Async Query Chunk
            let results = self.query_chunk_async(chunk).await?;

            for (pkg, os_vulns) in chunk.iter().zip(results.into_iter()) {
                if let Some(vulns) = os_vulns {
                    if !vulns.is_empty() {
                        let mut advisories = Vec::new();

                        for v in vulns {
                            let mut advisory = Advisory {
                                id: v.id.clone(),
                                description: v
                                    .summary
                                    .or(v.details)
                                    .unwrap_or_else(|| "No description".to_string()),
                                vulnerable_versions: semver::VersionReq::STAR,
                                crate_name: pkg.name.clone(),
                                details: None,
                                cache_status: None,
                                last_fetched_at: None,
                            };

                            if show_details {
                                // Async Fetch Details
                                // Async Fetch Details with Coalescing
                                match self.fetch_vuln_details_with_coalescing(&advisory.id).await {
                                    Ok((json, status, fetched_at)) => {
                                        advisory.details = Some(json);
                                        advisory.cache_status = Some(status);
                                        advisory.last_fetched_at = Some(
                                            fetched_at
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap_or(std::time::Duration::ZERO)
                                                .as_secs(),
                                        );
                                    }
                                    Err(e) => {
                                        advisory.cache_status = Some(format!("Error: {}", e));
                                    }
                                }
                            }
                            advisories.push(advisory);
                        }

                        vulnerabilities.push(Vulnerability {
                            ecosystem: pkg.ecosystem,
                            package_name: pkg.name.clone(),
                            version: pkg.version.clone(),
                            advisories,
                            locations: Vec::new(),
                        });
                    }
                }
            }
        }
        Ok(vulnerabilities)
    }

    async fn query_chunk_async(
        &self,
        packages: &[PackageRef],
    ) -> Result<Vec<Option<Vec<OsvVuln>>>, Arc<GuardianError>> {
        // Construct cache key
        let key = self.compute_chunk_key(packages);

        // Try cache first (Sync read from cache is fine for now)
        if let Some(cache) = &self.cache {
            if let Some(cached_json) = cache.get(&key) {
                if let Ok(response) = serde_json::from_str::<BatchResponse>(&cached_json) {
                    return Ok(response.results.into_iter().map(|r| r.vulns).collect());
                }
            }
        }

        if self.offline {
            return Err(Arc::new(GuardianError::NetworkError(
                "Offline mode: OSV cache miss.".to_string(),
            )));
        }

        // Prepare Owned Data for Future (Optimistic serialization)
        let queries: Vec<Query> = packages
            .iter()
            .map(|p| Query {
                package: OsvPackage {
                    name: &p.name,
                    ecosystem: match p.ecosystem {
                        Ecosystem::Npm => "npm",
                        Ecosystem::Rust => "Crates.io",
                    },
                },
                version: &p.version,
            })
            .collect();
        let batch = BatchQuery { queries };

        let json_body = serde_json::to_string(&batch).map_err(|e| {
            Arc::new(GuardianError::NetworkError(format!(
                "JSON Serialization Error: {}",
                e
            )))
        })?;

        let url = self.api_url.clone();
        let this = self.clone(); // Clone internal struct
        let key_for_cache = key.clone();

        let future = {
            // Lock Map
            let mut map = self.in_flight_query.lock().unwrap();

            if let Some(fut) = map.get(&key) {
                if let Some(m) = &self.metrics {
                    m.coalesced_waiters.fetch_add(1, Ordering::Relaxed);
                }
                fut.clone()
            } else {
                // Future captures ONLY owned data
                let future = async move {
                    // Use body string directly
                    let resp = this
                        .send_with_retry(NetOp::QueryBatch, |timeout| {
                            this.client
                                .post(&url)
                                .body(json_body.clone())
                                .header("Content-Type", "application/json")
                                .timeout(timeout)
                        })
                        .await
                        .map_err(Arc::new)?;

                    if !resp.status().is_success() {
                        return Err(Arc::new(GuardianError::NetworkError(format!(
                            "OSV API error: {}",
                            resp.status()
                        ))));
                    }

                    let body = resp
                        .text()
                        .await
                        .map_err(|e| Arc::new(GuardianError::NetworkError(e.to_string())))?;

                    // Cache write using cloned key/this
                    if let Some(cache) = &this.cache {
                        let _ = cache.put(&key_for_cache, &body);
                    }

                    let response: BatchResponse = serde_json::from_str(&body).map_err(|e| {
                        Arc::new(GuardianError::NetworkError(format!(
                            "JSON Parse Error: {}",
                            e
                        )))
                    })?;

                    Ok(response.results.into_iter().map(|r| r.vulns).collect())
                }
                .boxed()
                .shared();

                // Insert
                map.insert(key.clone(), future.clone());
                future
            }
        };

        // Await
        let result = future.await;

        // Cleanup
        {
            let mut map = self.in_flight_query.lock().unwrap();
            map.remove(&key);
        }

        result
    }

    fn compute_chunk_key(&self, packages: &[PackageRef]) -> String {
        let mut s = String::new();
        for p in packages {
            s.push_str(&format!("{}:{}:{};", p.ecosystem, p.name, p.version));
        }
        s
    }
}
