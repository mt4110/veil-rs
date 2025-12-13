use crate::db::GuardianDb;

use crate::providers::{npm, osv};
use crate::report::{ScanResult, Vulnerability};
use crate::GuardianError;
use semver::Version;
use std::path::Path;

pub struct ScanOptions {
    pub offline: bool,
}

pub fn scan_lockfile(path: &Path, options: ScanOptions) -> Result<ScanResult, GuardianError> {
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| GuardianError::LockfileParseError("Invalid filename".to_string()))?;

    if filename == "package-lock.json" {
        return scan_npm(path, options);
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
                    crate_name: name.to_string(),
                    version: version_str,
                    advisories: owned_advisories,
                });
            }
        }
    }

    Ok(result)
}

fn scan_npm(path: &Path, options: ScanOptions) -> Result<ScanResult, GuardianError> {
    let packages = npm::parse_package_lock(path)?;
    let client = osv::OsvClient::new(options.offline);
    let vulns = client.check_packages(&packages)?;

    Ok(ScanResult {
        scanned_crates: packages.len(),
        vulnerabilities: vulns,
    })
}
