use crate::db::GuardianDb;
use crate::report::{ScanResult, Vulnerability};
use crate::GuardianError;
use semver::Version;
use std::path::Path;

pub fn scan_lockfile(path: &Path) -> Result<ScanResult, GuardianError> {
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
