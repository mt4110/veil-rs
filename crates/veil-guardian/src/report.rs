use crate::models::Advisory;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct ScanResult {
    pub vulnerabilities: Vec<Vulnerability>,
    pub scanned_crates: usize,
}

#[derive(Debug, Serialize)]
pub struct Vulnerability {
    pub crate_name: String,
    pub version: String,
    pub advisories: Vec<Advisory>,
}

impl ScanResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_clean(&self) -> bool {
        self.vulnerabilities.is_empty()
    }
}

pub enum OutputFormat {
    Human,
    Json,
}

impl ScanResult {
    pub fn display(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => serde_json::to_string_pretty(self).unwrap_or_default(),
            OutputFormat::Human => {
                if self.is_clean() {
                    return format!(
                        "No vulnerabilities found in {} packages.",
                        self.scanned_crates
                    );
                }

                let mut out = String::new();
                out.push_str(&format!(
                    "Found {} vulnerabilities in {} packages scanned:\n\n",
                    self.vulnerabilities.len(),
                    self.scanned_crates
                ));

                for vuln in &self.vulnerabilities {
                    out.push_str(&format!("- {} v{}\n", vuln.crate_name, vuln.version));
                    for advisory in &vuln.advisories {
                        out.push_str(&format!("  [{}] {}\n", advisory.id, advisory.description));

                        // Display status if available
                        if let Some(status) = &advisory.cache_status {
                            let fetched = advisory
                                .last_fetched_at
                                .map(|t| {
                                    // Simple formatting or iso8601 if possible.
                                    // Since we don't have chrono here easily, maybe just "(timestamp: ...)" or skip.
                                    // Or just "last_fetched: <unix>"?
                                    // Ideally human readable.
                                    // Let's stick to status for now or simple "Status: Fresh"
                                    // User requirement: "cache: fresh|stale + last_fetched"
                                    format!(" (fetched: {})", t)
                                })
                                .unwrap_or_default();
                            out.push_str(&format!("    Cache: {}{}\n", status, fetched));
                        }

                        // Extra Details Extraction (Best Effort)
                        if let Some(details) = &advisory.details {
                            // Summary
                            if let Some(summary) = details.get("summary").and_then(|s| s.as_str()) {
                                if summary != advisory.description {
                                    // dedupe
                                    out.push_str(&format!("    Summary: {}\n", summary));
                                }
                            }

                            // Severity
                            if let Some(severity) =
                                details.get("severity").and_then(|v| v.as_array())
                            {
                                // Take first severity score
                                if let Some(first) = severity.first() {
                                    if let Some(score) = first.get("score").and_then(|s| s.as_str())
                                    {
                                        out.push_str(&format!("    Severity: {}\n", score));
                                    }
                                }
                            }

                            // References (Top 3)
                            if let Some(refs) = details.get("references").and_then(|v| v.as_array())
                            {
                                if !refs.is_empty() {
                                    out.push_str("    References:\n");
                                    for r in refs.iter().take(3) {
                                        if let Some(url) = r.get("url").and_then(|u| u.as_str()) {
                                            out.push_str(&format!("      - {}\n", url));
                                        }
                                    }
                                }
                            }
                        }
                        out.push('\n');
                    }
                }
                out
            }
        }
    }
}
