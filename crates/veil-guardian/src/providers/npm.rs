use crate::models::{Ecosystem, PackageRef};
use crate::GuardianError;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct PackageLock {
    #[allow(dead_code)]
    name: Option<String>,
    #[allow(dead_code)]
    version: Option<String>,
    #[allow(dead_code)]
    lockfile_version: Option<u32>, // Can be 1, 2, or 3
    #[serde(default)]
    dependencies: HashMap<String, DependencyV1>,
    #[serde(default)]
    packages: HashMap<String, PackageV2>,
}

#[derive(Debug, Deserialize)]
struct DependencyV1 {
    version: String,
    #[serde(default)]
    dependencies: HashMap<String, DependencyV1>,
}

#[derive(Debug, Deserialize)]
struct PackageV2 {
    version: String,
    #[serde(default)]
    link: bool,
    #[allow(dead_code)]
    name: Option<String>, // Sometimes present, but usually inferred from key
}

pub fn parse_package_lock(path: &Path) -> Result<Vec<PackageRef>, GuardianError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| GuardianError::LockfileParseError(format!("Failed to read file: {}", e)))?;

    let lock: PackageLock = serde_json::from_str(&content)
        .map_err(|e| GuardianError::LockfileParseError(format!("Invalid JSON: {}", e)))?;

    let mut refs = Vec::new();

    // Strategy 1: "packages" (npm v2/v3) - Preferred if present and non-empty
    // We check for > 1 because "packages" usually contains "" (root) even if empty otherwise?
    // Actually npm v7+ always puts packages.
    if !lock.packages.is_empty() {
        for (key, pkg) in lock.packages {
            if key.is_empty() {
                continue;
            } // Skip root
            if pkg.link {
                continue;
            } // Skip symlinks

            // Key is likely "node_modules/foo" or "node_modules/@scope/foo"
            // We need to extract the actual package name.
            let name = extract_name_from_path(&key);

            // Skip non-semver refs (like URLs or git refs) for now.
            // A simple heuristic: if it contains '/', it might be a path/url, UNLESS it's a scope.
            // But 'version' in package-lock can be a URL.
            // For now, let's just take it if it doesn't look like a URL.
            // Better: OSV expects semantic versions.
            if is_valid_version(&pkg.version) {
                refs.push(PackageRef {
                    ecosystem: Ecosystem::Npm,
                    name: name.to_string(),
                    version: pkg.version,
                });
            }
        }
    } else {
        // Strategy 2: "dependencies" (npm v1) - Recursive
        parse_deps_v1(&lock.dependencies, &mut refs);
    }

    refs.sort_by(|a, b| {
        (a.name.as_str(), a.version.as_str()).cmp(&(b.name.as_str(), b.version.as_str()))
    });
    refs.dedup_by(|a, b| a.name == b.name && a.version == b.version);
    Ok(refs)
}

fn parse_deps_v1(deps: &HashMap<String, DependencyV1>, refs: &mut Vec<PackageRef>) {
    for (name, dep) in deps {
        if is_valid_version(&dep.version) {
            refs.push(PackageRef {
                ecosystem: Ecosystem::Npm,
                name: name.clone(),
                version: dep.version.clone(),
            });
        }
        parse_deps_v1(&dep.dependencies, refs);
    }
}

fn extract_name_from_path(path: &str) -> &str {
    // "node_modules/foo" -> "foo"
    // "node_modules/@scope/bar" -> "@scope/bar"
    // "node_modules/a/node_modules/b" -> "b"

    // We want the last chunk after 'node_modules/'
    if let Some(idx) = path.rfind("node_modules/") {
        &path[idx + 13..]
    } else {
        path
    }
}

fn is_valid_version(v: &str) -> bool {
    // Basic semver check: starts with digit.
    // npm versions can be "1.2.3", "^1.2.3" (in package.json, but lockfile usually has resolved "1.2.3")
    // Lockfile 'version' can also be "git+ssh://..." or "file:..."
    v.chars().next().is_some_and(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_name() {
        assert_eq!(extract_name_from_path("node_modules/foo"), "foo");
        assert_eq!(
            extract_name_from_path("node_modules/@scope/bar"),
            "@scope/bar"
        );
        assert_eq!(extract_name_from_path("node_modules/a/node_modules/b"), "b");
        assert_eq!(extract_name_from_path("foo"), "foo");
    }
}
