use super::cache::Cache;
use super::details::{CachePolicy, CacheStatus, CachedVuln};
use super::details_store::DetailsStore;
use crate::models::{Advisory, Ecosystem, PackageRef};
use crate::report::Vulnerability;
use crate::GuardianError;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::env;
use std::time::SystemTime;

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
struct BatchResponse {
    results: Vec<QueryResult>,
}

#[derive(Deserialize)]
struct QueryResult {
    vulns: Option<Vec<OsvVuln>>,
}

#[derive(Deserialize, Clone)]
struct OsvVuln {
    id: String,
    // OSV batch response doesn't always contain details/summary unless requested?
    // Actually standard OSV response includes 'summary' or 'details'.
    summary: Option<String>,
    details: Option<String>,
}

pub struct OsvClient {
    client: Client,
    cache: Option<Cache>,
    details_store: Option<DetailsStore>,
    offline: bool,
    api_url: String,
}

impl OsvClient {
    pub fn new(offline: bool, api_url: Option<String>) -> Self {
        // Priority: Argument -> Env Var -> Default
        let api_url = api_url
            .or_else(|| env::var("OSV_API_URL").ok())
            .unwrap_or_else(|| DEFAULT_OSV_URL.to_string());

        Self {
            client: Client::new(),
            cache: Cache::new(),
            details_store: DetailsStore::new(),
            offline,
            api_url,
        }
    }

    /// For testing: inject a custom store and/or URL
    pub fn new_custom(offline: bool, store: Option<DetailsStore>, api_url: Option<String>) -> Self {
        let api_url = api_url
            .or_else(|| env::var("OSV_API_URL").ok())
            .unwrap_or_else(|| DEFAULT_OSV_URL.to_string());

        Self {
            client: Client::new(),
            cache: Cache::new(),
            details_store: store,
            offline,
            api_url,
        }
    }

    /// Fetch vulnerability details with Smart Offline policy.
    /// Returns (Raw JSON, CacheStatus, FetchedAt).
    pub fn fetch_vuln_details(
        &self,
        id: &str,
    ) -> Result<(Value, CacheStatus, SystemTime), GuardianError> {
        let policy = CachePolicy::default();
        let now = SystemTime::now();
        let force_refresh = env::var("VEIL_OSV_FORCE_REFRESH").is_ok();

        // 1. Load from cache
        let cached = self.details_store.as_ref().and_then(|s| s.load(id));

        // 2. Determine status
        let status = if let Some(entry) = &cached {
            policy.classify(entry.fetched_at(), now)
        } else {
            CacheStatus::Expired // Effectively expired (missing)
        };

        // 3. Logic: Should we fetch?
        // - Force Refresh -> Yes (unless offline)
        // - Fresh -> No
        // - Stale -> Yes (try), fallback to cache if fail
        // - Expired/Missing -> Yes, fail if offline (unless cached exists, then warn/use? see below)

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
            match self.perform_fetch(id) {
                Ok(json) => {
                    // Save to cache
                    if let Some(store) = &self.details_store {
                        let entry = CachedVuln::new(id, now, json.clone());
                        let _ = store.save(&entry); // Ignore save errors
                    }
                    return Ok((json, CacheStatus::Fresh, now));
                }
                Err(e) => {
                    // Start of fallback logic
                    if let Some(entry) = cached {
                        // If we have stale/expired cache and fetch failed, use it
                        return Ok((entry.vuln.clone(), status, entry.fetched_at()));
                    } else {
                        // No cache, fetch failed -> Error
                        return Err(e);
                    }
                }
            }
        }

        // Offline or Fresh case
        if let Some(entry) = cached {
            Ok((entry.vuln.clone(), status, entry.fetched_at()))
        } else {
            // Offline and no cache
            Err(GuardianError::NetworkError(format!(
                "Offline: No details cached for {}",
                id
            )))
        }
    }

    fn perform_fetch(&self, id: &str) -> Result<Value, GuardianError> {
        // Construct URL: Replace "querybatch" with "vulns/{id}" in base URL
        let base_url = &self.api_url;
        let url = if base_url.ends_with("/querybatch") {
            // https://api.osv.dev/v1/querybatch -> https://api.osv.dev/v1/vulns/
            base_url.replace("/querybatch", &format!("/vulns/{}", id))
        } else {
            // Fallback logic
            let parent = if base_url.ends_with('/') {
                &base_url[..base_url.len() - 1]
            } else {
                // split by / and take all but last
                let parts: Vec<&str> = base_url.split('/').collect();
                if parts.len() > 1 && parts.last() == Some(&"querybatch") {
                    &base_url[..base_url.len() - "/querybatch".len()]
                } else {
                    base_url
                }
            };

            format!("{}/vulns/{}", parent.trim_end_matches('/'), id)
        };

        let resp = self
            .client
            .get(&url)
            .send()
            .map_err(|e| GuardianError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(GuardianError::NetworkError(format!(
                "OSV API Details Error: {}",
                resp.status()
            )));
        }

        let val: Value = resp
            .json()
            .map_err(|e| GuardianError::NetworkError(format!("JSON Parse: {}", e)))?;

        // Verify it is a valid vulnerability object (has "id")
        if val.get("id").and_then(|v| v.as_str()) != Some(id) {
            return Err(GuardianError::NetworkError(
                "Returned JSON missing ID or ID mismatch".to_string(),
            ));
        }

        Ok(val)
    }

    pub fn check_packages(
        &self,
        packages: &[PackageRef],
        show_details: bool,
    ) -> Result<Vec<Vulnerability>, GuardianError> {
        let mut vulnerabilities = Vec::new();

        for chunk in packages.chunks(CHUNK_SIZE) {
            let results = self.query_chunk(chunk)?;

            for (pkg, os_vulns) in chunk.iter().zip(results.into_iter()) {
                if let Some(vulns) = os_vulns {
                    if !vulns.is_empty() {
                        let mut advisories = Vec::new();

                        for v in vulns {
                            let mut advisory = Advisory {
                                id: v.id.clone(),
                                // Use summary if available, else truncated details, else placeholder
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
                                match self.fetch_vuln_details(&advisory.id) {
                                    Ok((json, status, fetched_at)) => {
                                        advisory.details = Some(json);
                                        advisory.cache_status = Some(format!("{:?}", status));
                                        advisory.last_fetched_at = Some(
                                            fetched_at
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap_or(std::time::Duration::ZERO)
                                                .as_secs(),
                                        );
                                    }
                                    Err(e) => {
                                        // On failure, we keep the basic advisory but maybe append error to description?
                                        // Or just log it?
                                        // For now, silently fail details fetching (user sees basic info)
                                        // Maybe better to indicate failure in cache_status?
                                        advisory.cache_status = Some(format!("Error: {}", e));
                                    }
                                }
                            }
                            advisories.push(advisory);
                        }

                        vulnerabilities.push(Vulnerability {
                            crate_name: pkg.name.clone(),
                            version: pkg.version.clone(),
                            advisories,
                        });
                    }
                }
            }
        }

        Ok(vulnerabilities)
    }

    fn query_chunk(
        &self,
        packages: &[PackageRef],
    ) -> Result<Vec<Option<Vec<OsvVuln>>>, GuardianError> {
        // Construct cache key for this chunk
        // Key idea: unique signature of the query content.
        let key = self.compute_chunk_key(packages);

        // Try cache first
        if let Some(cache) = &self.cache {
            if let Some(cached_json) = cache.get(&key) {
                if let Ok(response) = serde_json::from_str::<BatchResponse>(&cached_json) {
                    return Ok(response.results.into_iter().map(|r| r.vulns).collect());
                }
            }
        }

        if self.offline {
            return Err(GuardianError::NetworkError(
                "Offline mode: OSV cache miss. Run once online to populate cache (or disable --offline)."
                    .to_string(),
            ));
        }

        // Online query
        let queries: Vec<Query> = packages
            .iter()
            .map(|p| Query {
                package: OsvPackage {
                    name: &p.name,
                    ecosystem: match p.ecosystem {
                        Ecosystem::Npm => "npm",
                        // For Rust, we use local DB, but if we extended OSV for rust: "Crates.io"
                        Ecosystem::Rust => "Crates.io",
                    },
                },
                version: &p.version,
            })
            .collect();

        let batch = BatchQuery { queries };

        let url = &self.api_url;

        let resp = self
            .client
            .post(url)
            .json(&batch)
            .send()
            .map_err(|e| GuardianError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(GuardianError::NetworkError(format!(
                "OSV API error: {}",
                resp.status()
            )));
        }

        let body = resp
            .text()
            .map_err(|e| GuardianError::NetworkError(e.to_string()))?;

        // Cache write
        if let Some(cache) = &self.cache {
            let _ = cache.put(&key, &body);
        }

        let response: BatchResponse = serde_json::from_str(&body)
            .map_err(|e| GuardianError::NetworkError(format!("JSON Parse Error: {}", e)))?;
        Ok(response.results.into_iter().map(|r| r.vulns).collect())
    }

    fn compute_chunk_key(&self, packages: &[PackageRef]) -> String {
        // Simple serialization of inputs
        let mut s = String::new();
        for p in packages {
            s.push_str(&format!("{}:{}:{};", p.ecosystem, p.name, p.version));
        }
        s
    }
}
