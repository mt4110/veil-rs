use super::cache::Cache;
use super::details::{CachePolicy, CacheStatus, CachedVuln};
use super::details_store::DetailsStore;
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
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

// Type aliases for Coalescing
type DetailsResult = Result<(Value, String, SystemTime), Arc<GuardianError>>;
type QueryBatchResult = Result<Vec<Option<Vec<OsvVuln>>>, Arc<GuardianError>>;

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

        // Split cache_dir into sub-caches if provided, or pass None to use defaults/env
        let (query_cache, details_cache) = if let Some(base) = cache_dir {
            (Some(base.join("osv")), Some(base.join("details")))
        } else {
            (None, None)
        };

        Self {
            client: Client::new(),
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

        Self {
            client: Client::new(),
            cache: if !offline { Cache::new(None) } else { None },
            details_store: store,
            offline,
            api_url,
            metrics: None,
            rt: Arc::new(rt),
            in_flight_details: Arc::new(Mutex::new(HashMap::new())),
            in_flight_query: Arc::new(Mutex::new(HashMap::new())),
        }
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
}

impl OsvClientInternal {
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
        let policy = CachePolicy::default();
        let now = SystemTime::now();
        let force_refresh = OsvClient::env_flag("VEIL_OSV_FORCE_REFRESH"); // Use static method

        // 1. Load from cache (Sync FS op, assume fast enough or use spawn_blocking)
        // Ideally fs ops should be blocking task.
        let cached = self.details_store.as_ref().and_then(|s| s.load(id));

        // 2. Determine status
        let status = if let Some(entry) = &cached {
            policy.classify(entry.fetched_at(), now)
        } else {
            CacheStatus::Expired
        };

        // Cache Metrics
        if let Some(m) = &self.metrics {
            match status {
                CacheStatus::Fresh => m.cache_fresh.fetch_add(1, Ordering::Relaxed),
                CacheStatus::Stale => m.cache_stale.fetch_add(1, Ordering::Relaxed),
                CacheStatus::Expired => m.cache_miss.fetch_add(1, Ordering::Relaxed),
            };
        }

        // 3. Logic: Should we fetch?
        let should_fetch = if self.offline {
            false
        } else if force_refresh {
            true
        } else {
            match status {
                CacheStatus::Fresh => false,
                CacheStatus::Stale | CacheStatus::Expired => true,
            }
        };

        if should_fetch {
            match self.perform_fetch_async(id).await {
                Ok(json) => {
                    // Save to cache
                    if let Some(store) = &self.details_store {
                        let entry = CachedVuln::new(id, now, json.clone());
                        let _ = store.save(&entry);
                    }
                    return Ok((json, "Network".to_string(), now));
                }
                Err(e) => {
                    // Fallback
                    if let Some(entry) = cached {
                        let status_str = match status {
                            CacheStatus::Fresh => "Hit (Fresh)",
                            CacheStatus::Stale => "Hit (Stale)",
                            CacheStatus::Expired => "Hit (Expired)",
                        };
                        return Ok((
                            entry.vuln.clone(),
                            format!("{} [Offline Fallback]", status_str),
                            entry.fetched_at(),
                        ));
                    } else {
                        return Err(Arc::new(e));
                    }
                }
            }
        }

        // Offline or Fresh case
        if let Some(entry) = cached {
            let status_str = match status {
                CacheStatus::Fresh => "Hit (Fresh)",
                CacheStatus::Stale => "Hit (Stale)",
                CacheStatus::Expired => "Hit (Expired)",
            };
            Ok((
                entry.vuln.clone(),
                status_str.to_string(),
                entry.fetched_at(),
            ))
        } else {
            Err(Arc::new(GuardianError::NetworkError(format!(
                "Offline: No details cached for {}",
                id
            ))))
        }
    }

    async fn perform_fetch_async(&self, id: &str) -> Result<Value, GuardianError> {
        // Construct URL
        let base_url = &self.api_url;
        let url = if base_url.ends_with("/querybatch") {
            base_url.replace("/querybatch", &format!("/vulns/{}", id))
        } else {
            // ... simplified fallback logic or copy verbatim ...
            // Copying logic:
            let parts: Vec<&str> = base_url.split('/').collect();
            let parent = if parts.len() > 1 && parts.last() == Some(&"querybatch") {
                &base_url[..base_url.len() - "/querybatch".len()]
            } else {
                base_url.trim_end_matches('/')
            };
            format!("{}/vulns/{}", parent, id)
        };

        if let Some(m) = &self.metrics {
            m.req_details.fetch_add(1, Ordering::Relaxed);
        }

        let start = std::time::Instant::now();
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| GuardianError::NetworkError(e.to_string()))?;

        if let Some(m) = &self.metrics {
            m.time_osv_details_ms
                .fetch_add(start.elapsed().as_millis() as u64, Ordering::Relaxed);
        }

        if !resp.status().is_success() {
            return Err(GuardianError::NetworkError(format!(
                "OSV API Details Error: {}",
                resp.status()
            )));
        }

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

        Ok(val)
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
                    if let Some(m) = &this.metrics {
                        m.req_querybatch.fetch_add(1, Ordering::Relaxed);
                    }
                    let start = std::time::Instant::now();

                    // Use body string directly
                    let resp = this
                        .client
                        .post(&url)
                        .body(json_body)
                        .header("Content-Type", "application/json")
                        .send()
                        .await
                        .map_err(|e| Arc::new(GuardianError::NetworkError(e.to_string())))?;

                    if let Some(m) = &this.metrics {
                        m.time_osv_query_ms
                            .fetch_add(start.elapsed().as_millis() as u64, Ordering::Relaxed);
                    }

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
