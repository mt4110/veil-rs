use crate::db::GuardianDb;

use crate::providers::{npm, osv, pnpm, yarn};
use crate::report::{ScanResult, Vulnerability};
use crate::GuardianError;
use crate::Metrics;
use semver::Version;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub struct ScanOptions {
    pub offline: bool,
    pub show_details: bool,
    pub osv_api_url: Option<String>,
    pub metrics: Option<Arc<Metrics>>,
}

pub fn scan_lockfile(path: &Path, options: ScanOptions) -> Result<ScanResult, GuardianError> {
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| GuardianError::LockfileParseError("Invalid filename".to_string()))?;

    if filename == "package-lock.json" {
        return scan_npm(path, options);
    }

    if filename == "pnpm-lock.yaml" {
        return scan_pnpm(path, options);
    }

    if filename == "yarn.lock" {
        return scan_yarn(path, options);
    }

    // Default to Cargo.lock
    scan_cargo(path)
}

fn scan_cargo(path: &Path) -> Result<ScanResult, GuardianError> {
    let lockfile = cargo_lock::Lockfile::load(path)
        .map_err(|e| GuardianError::LockfileParseError(e.to_string()))?;

    let db = GuardianDb::load_builtin()?;
    let mut result = ScanResult::new();

    // Iterate over all packages
    for package in lockfile.packages {
        let name = package.name.as_str();
        let version_str = package.version.to_string();

        // Count scanned packages
        result.scanned_crates += 1;

        if let Ok(version) = Version::parse(&version_str) {
            let advisories = db.check_vulnerabilities(name, &version);
            if !advisories.is_empty() {
                let owned_advisories: Vec<_> = advisories.into_iter().cloned().collect();
                result.vulnerabilities.push(Vulnerability {
                    ecosystem: crate::models::Ecosystem::Rust,
                    package_name: name.to_string(),
                    version: version_str,
                    advisories: owned_advisories,
                    locations: vec![path.to_string_lossy().to_string()],
                });
            }
        }
    }

    Ok(result)
}

fn scan_npm(path: &Path, options: ScanOptions) -> Result<ScanResult, GuardianError> {
    let start_parse = std::time::Instant::now();
    let packages = npm::parse_package_lock(path)?;
    if let Some(m) = &options.metrics {
        m.time_parse_ms
            .fetch_add(start_parse.elapsed().as_millis() as u64, Ordering::Relaxed);
    }

    let show_details = options.show_details;
    let client = osv::OsvClient::new(
        options.offline,
        options.osv_api_url,
        options.metrics.clone(),
    );
    let mut vulns = client.check_packages(&packages, show_details)?;
    for vuln in &mut vulns {
        vuln.locations.push(path.to_string_lossy().to_string());
    }

    Ok(ScanResult {
        scanned_crates: packages.len(),
        vulnerabilities: vulns,
    })
}

fn scan_pnpm(path: &Path, options: ScanOptions) -> Result<ScanResult, GuardianError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| GuardianError::LockfileParseError(e.to_string()))?;

    let start_parse = std::time::Instant::now();
    let packages = pnpm::parse_pnpm_lock(&content)
        .map_err(|e| GuardianError::LockfileParseError(e.to_string()))?;
    if let Some(m) = &options.metrics {
        m.time_parse_ms
            .fetch_add(start_parse.elapsed().as_millis() as u64, Ordering::Relaxed);
    }

    let show_details = options.show_details;
    let client = osv::OsvClient::new(
        options.offline,
        options.osv_api_url,
        options.metrics.clone(),
    );
    let mut vulns = client.check_packages(&packages, show_details)?;
    for vuln in &mut vulns {
        vuln.locations.push(path.to_string_lossy().to_string());
    }

    Ok(ScanResult {
        scanned_crates: packages.len(),
        vulnerabilities: vulns,
    })
}

fn scan_yarn(path: &Path, options: ScanOptions) -> Result<ScanResult, GuardianError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| GuardianError::LockfileParseError(e.to_string()))?;

    let start_parse = std::time::Instant::now();
    let packages = yarn::parse_yarn_lock(&content)
        .map_err(|e| GuardianError::LockfileParseError(e.to_string()))?;
    if let Some(m) = &options.metrics {
        m.time_parse_ms
            .fetch_add(start_parse.elapsed().as_millis() as u64, Ordering::Relaxed);
    }

    let show_details = options.show_details;
    let client = osv::OsvClient::new(
        options.offline,
        options.osv_api_url,
        options.metrics.clone(),
    );
    let mut vulns = client.check_packages(&packages, show_details)?;
    for vuln in &mut vulns {
        vuln.locations.push(path.to_string_lossy().to_string());
    }

    Ok(ScanResult {
        scanned_crates: packages.len(),
        vulnerabilities: vulns,
    })
}
