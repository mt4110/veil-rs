use crate::models::{Advisory, Ecosystem};
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct ScanResult {
    pub vulnerabilities: Vec<Vulnerability>,
    pub scanned_crates: usize,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Vulnerability {
    pub ecosystem: Ecosystem,
    pub package_name: String,
    pub version: String,
    pub advisories: Vec<Advisory>,
    pub locations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Unknown,
}

impl Severity {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical" => Severity::Critical,
            "high" => Severity::High,
            "medium" => Severity::Medium,
            "low" => Severity::Low,
            "moderate" => Severity::Medium, // OSV sometimes uses MODERATE
            _ => Severity::Unknown,
        }
    }
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

                // 1. Group by (Ecosystem, Package, Version) to merge duplicates
                // This handles cases where multiple lockfiles (or even the same lockfile in some parsers)
                // produce duplicate entries for the same package version.
                let mut grouped_vulns: std::collections::BTreeMap<
                    (String, String, String),
                    Vulnerability,
                > = std::collections::BTreeMap::new();

                for vuln in &self.vulnerabilities {
                    let key = (
                        vuln.ecosystem.to_string(),
                        vuln.package_name.clone(),
                        vuln.version.clone(),
                    );

                    if let Some(existing) = grouped_vulns.get_mut(&key) {
                        // Merge locations
                        existing.locations.extend(vuln.locations.iter().cloned());

                        // Merge advisories (dedup by ID)
                        for adv in &vuln.advisories {
                            if !existing.advisories.iter().any(|a| a.id == adv.id) {
                                existing.advisories.push(adv.clone());
                            }
                        }
                    } else {
                        grouped_vulns.insert(key, vuln.clone());
                    }
                }

                let mut vulns: Vec<Vulnerability> = grouped_vulns.into_values().collect();

                // Sort Vulnerabilities (already sorted by BTreeMap key, but explicit sort for safety/clarity)
                vulns.sort_by(|a, b| {
                    (a.ecosystem.to_string(), &a.package_name, &a.version).cmp(&(
                        b.ecosystem.to_string(),
                        &b.package_name,
                        &b.version,
                    ))
                });

                // 0. Cache Stats Collection
                let mut stats_hit_fresh = 0;
                let mut stats_hit_stale = 0;
                let mut stats_network = 0;
                let mut stats_offline = 0;
                let mut stats_error = 0;
                let mut _stats_unknown = 0;

                let mut has_cache_info = false;

                for vuln in &vulns {
                    for advisory in &vuln.advisories {
                        if advisory.cache_status.is_some() {
                            has_cache_info = true;
                        }
                        match advisory.cache_status.as_deref() {
                            Some("Network") | Some("Fetched") => stats_network += 1,
                            Some(s) if s.contains("Fresh") => stats_hit_fresh += 1, // "Hit (Fresh)"
                            Some(s) if s.contains("Stale") => stats_hit_stale += 1, // "Hit (Stale)"
                            Some(s) if s.starts_with("Error") => stats_error += 1,
                            Some("Offline") => stats_offline += 1,
                            _ => _stats_unknown += 1,
                        }
                    }
                }

                let mut out = String::new();
                out.push_str(&format!(
                    "Found {} vulnerabilities in {} packages scanned:\n",
                    vulns.len(),
                    self.scanned_crates
                ));

                if has_cache_info {
                    out.push_str(&format!(
                        "Cache: {} fresh, {} stale, {} network, {} offline, {} error\n\n",
                        stats_hit_fresh, stats_hit_stale, stats_network, stats_offline, stats_error
                    ));
                } else {
                    out.push('\n');
                }

                for vuln in &mut vulns {
                    // 2. Sort Advisories (Severity -> ID)
                    vuln.advisories.sort_by(|a, b| {
                        let sev_a = get_severity_score(a);
                        let sev_b = get_severity_score(b);
                        // Critical (0) < High (1) ... so distinct from Severity enum order?
                        // Severity enum: Critical, High, Medium, Low, Unknown
                        // Ord derivation makes Critical < High (based on definition order).
                        // We want Critical to be "smallest" if we sort ascending?
                        // Or we want Critical First.
                        // Defined: Critical=0, High=1.
                        // sort() is ascending. 0 comes before 1. So Critical comes first. Perfect.
                        sev_a.cmp(&sev_b).then_with(|| a.id.cmp(&b.id))
                    });

                    // Sort locations
                    vuln.locations.sort();
                    vuln.locations.dedup();

                    out.push_str(&format!(
                        "- {} v{} ({})\n",
                        vuln.package_name, vuln.version, vuln.ecosystem
                    ));

                    if !vuln.locations.is_empty() {
                        out.push_str(&format!("  Locations: {}\n", vuln.locations.join(", ")));
                    }

                    for advisory in &vuln.advisories {
                        out.push_str(&format!("  [{}] {}\n", advisory.id, advisory.description));

                        // Action Line
                        let fix = recommend_fix(advisory, &vuln.version);
                        out.push_str(&format!("    Fix: {}\n", fix));

                        // 3. Cache Status
                        if let Some(status) = &advisory.cache_status {
                            out.push_str(&format!("    Status: {}\n", status));
                        }

                        // 4. Details
                        if let Some(details) = &advisory.details {
                            // Summary
                            if let Some(summary) = details.get("summary").and_then(|s| s.as_str()) {
                                if summary != advisory.description {
                                    out.push_str(&format!("    Summary: {}\n", summary));
                                }
                            }

                            // Severity Display
                            if let Some(severity) =
                                details.get("severity").and_then(|v| v.as_array())
                            {
                                if let Some(first) = severity.first() {
                                    if let Some(score) = first.get("score").and_then(|s| s.as_str())
                                    {
                                        out.push_str(&format!("    Severity: {}\n", score));
                                    }
                                }
                            }

                            // References (Sorted)
                            if let Some(refs) = details.get("references").and_then(|v| v.as_array())
                            {
                                if !refs.is_empty() {
                                    out.push_str("    References:\n");
                                    // Extract URLs and sort them
                                    let mut urls: Vec<&str> = refs
                                        .iter()
                                        .filter_map(|r| r.get("url").and_then(|u| u.as_str()))
                                        .collect();
                                    urls.sort();

                                    for url in urls.into_iter().take(3) {
                                        out.push_str(&format!("      - {}\n", url));
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

fn recommend_fix(advisory: &Advisory, current_version: &str) -> String {
    // Attempt to find a fixed version in OSV details
    // details["affected"][]["ranges"][]["events"][]["fixed"]
    if let Some(details) = &advisory.details {
        if let Some(affected) = details.get("affected").and_then(|v| v.as_array()) {
            let current = semver::Version::parse(current_version).ok();
            let mut candidates = Vec::new();

            for aff in affected {
                if let Some(ranges) = aff.get("ranges").and_then(|r| r.as_array()) {
                    for range in ranges {
                        if let Some(events) = range.get("events").and_then(|e| e.as_array()) {
                            for event in events {
                                if let Some(fixed) = event.get("fixed").and_then(|v| v.as_str()) {
                                    if let Ok(fixed_ver) = semver::Version::parse(fixed) {
                                        candidates.push(fixed_ver);
                                    } else {
                                        // Fallback for non-semver versions?
                                        // For now, only collect valid semvers.
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Sort candidates
            candidates.sort();

            if let Some(curr) = current {
                // Find smallest fixed > current
                if let Some(upgrade) = candidates.iter().find(|&v| v > &curr) {
                    return format!("Upgrade to >= {}", upgrade);
                }
            } else {
                // If current version parsing failed, just suggest the first available fixed version (assuming it's sorted ascending, likely smallest)
                if let Some(first) = candidates.first() {
                    return format!("Upgrade to >= {}", first);
                }
            }
        }
    }
    "No fixed version available (mitigation required)".to_string()
}

fn get_severity_score(advisory: &Advisory) -> Severity {
    if let Some(details) = &advisory.details {
        if let Some(severity) = details
            .get("database_specific")
            .and_then(|d| d.get("severity"))
            .and_then(|s| s.as_str())
        {
            return Severity::from_str(severity);
        }
        // Fallback to standard severity field if needed, but OSV usually puts it in specific places or as CVSS.
        // For now, let's assume we can parse "severity" from the top level if we extracted it,
        // but the Advisory struct has `details` which is raw JSON.
        // Let's look for "severity" -> [ { "type": "CVSS_V3", "score": "..." } ]
        // Parsing CVSS score to "Critical/High" is hard without a calculator.
        // However, GitHub/OSV often provide "database_specific": { "severity": "MODERATE" }.
        if let Some(db_specific) = details.get("database_specific") {
            if let Some(severity_str) = db_specific.get("severity").and_then(|s| s.as_str()) {
                return Severity::from_str(severity_str);
            }
        }
    }
    Severity::Unknown
}
