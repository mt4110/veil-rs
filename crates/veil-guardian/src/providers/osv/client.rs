use super::cache::Cache;
use crate::models::{Advisory, Ecosystem, PackageRef};
use crate::report::Vulnerability;
use crate::GuardianError;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use std::env;

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
    offline: bool,
}

impl OsvClient {
    pub fn new(offline: bool) -> Self {
        Self {
            client: Client::new(),
            cache: Cache::new(),
            offline,
        }
    }

    pub fn check_packages(
        &self,
        packages: &[PackageRef],
    ) -> Result<Vec<Vulnerability>, GuardianError> {
        let mut vulnerabilities = Vec::new();

        for chunk in packages.chunks(CHUNK_SIZE) {
            let results = self.query_chunk(chunk)?;

            for (pkg, os_vulns) in chunk.iter().zip(results.into_iter()) {
                if let Some(vulns) = os_vulns {
                    if !vulns.is_empty() {
                        let advisories = vulns
                            .into_iter()
                            .map(|v| Advisory {
                                id: v.id,
                                // Use summary if available, else truncated details, else placeholder
                                description: v
                                    .summary
                                    .or(v.details)
                                    .unwrap_or_else(|| "No description".to_string()),
                                vulnerable_versions: semver::VersionReq::STAR, // Correctly, we matched exact version, so it IS vulnerable.
                                crate_name: pkg.name.clone(),
                            })
                            .collect();

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

        let url = env::var("OSV_API_URL").unwrap_or_else(|_| DEFAULT_OSV_URL.to_string());

        let resp = self
            .client
            .post(&url)
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
