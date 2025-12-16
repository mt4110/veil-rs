use crate::models::{Ecosystem, PackageRef};
use crate::GuardianError;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Deserialize, Debug)]
struct PnpmLock {
    #[serde(default)]
    packages: HashMap<String, serde::de::IgnoredAny>,
    #[serde(default)]
    snapshots: HashMap<String, serde::de::IgnoredAny>,
}

pub fn parse_pnpm_lock(content: &str) -> Result<Vec<PackageRef>, GuardianError> {
    let lock: PnpmLock = serde_yaml::from_str(content)
        .map_err(|e| GuardianError::LockfileParseError(format!("Failed to parse YAML: {}", e)))?;

    // Use HashSet of (name, version) for deduplication
    let mut refs = HashSet::new();

    // Helper to process a key
    let mut process_key = |key: &str| {
        // Skip non-semver protocols
        if key.starts_with("file:") || key.starts_with("link:") || key.starts_with("workspace:") {
            return;
        }

        let clean_key = if let Some(idx) = key.find('(') {
            &key[..idx]
        } else {
            key
        };

        // v5/v6 Slash-style: /@scope/pkg/1.2.3 or /pkg/1.2.3
        if let Some(path) = clean_key.strip_prefix('/') {
            // Strip leading slash

            // Strip peer/hash part (everything after first _, if any)
            let path = if let Some(idx) = path.find('_') {
                &path[..idx]
            } else {
                path
            };

            // Structure is usually [scope?]/pkg/version
            // Split by '/' to find where version starts
            // e.g. @scope/pkg/1.2.3 -> ["@scope", "pkg", "1.2.3"]
            // e.g. lodash/4.17.15 -> ["lodash", "4.17.15"]
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                // If the first part starts with @, we need at least 3 parts (@scope, pkg, ver)
                // Otherwise we need 2 parts (pkg, ver)
                if parts[0].starts_with('@') {
                    if parts.len() >= 3 {
                        let name = format!("{}/{}", parts[0], parts[1]);
                        let version = parts[2].to_string();
                        refs.insert((name, version));
                    }
                } else {
                    let name = parts[0].to_string();
                    let version = parts[1].to_string();
                    refs.insert((name, version));
                }
            }
        }
        // v9 name@version style: lodash@4.17.15 or @scope/pkg@1.2.3
        else if let Some(idx) = clean_key.rfind('@') {
            // Determine if this @ is a separator or part of scope
            if idx > 0 {
                let name = clean_key[..idx].to_string();
                let version = clean_key[idx + 1..].to_string();
                refs.insert((name, version));
            }
        }
    };

    // Process 'packages' (common in v5/v6/v9)
    for key in lock.packages.keys() {
        process_key(key);
    }

    // Process 'snapshots' (new in v9 for describing the graph, but keys are essentially packages)
    for key in lock.snapshots.keys() {
        process_key(key);
    }

    let mut refs_vec: Vec<_> = refs.into_iter().collect();
    refs_vec.sort_by(|a, b| match a.0.cmp(&b.0) {
        std::cmp::Ordering::Equal => a.1.cmp(&b.1),
        other => other,
    });

    Ok(refs_vec
        .into_iter()
        .map(|(name, version)| PackageRef {
            ecosystem: Ecosystem::Npm,
            name,
            version,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_v6_slash_style() {
        let content = r#"
packages:
  /@rushstack/eslint-config/2.5.1(eslint@7.32.0):
    resolution: {integrity: sha512-xxx}
  /lodash/4.17.15:
    resolution: {integrity: sha512-xxx}
  /tslib/2.3.1_12345:
    resolution: {integrity: sha512-xxx}
"#;
        let deps = parse_pnpm_lock(content).unwrap();
        let map: HashMap<_, _> = deps
            .iter()
            .map(|d| (d.name.as_str(), d.version.as_str()))
            .collect();

        assert_eq!(map.get("@rushstack/eslint-config"), Some(&"2.5.1"));
        assert_eq!(map.get("lodash"), Some(&"4.17.15"));
        assert_eq!(map.get("tslib"), Some(&"2.3.1"));
    }

    #[test]
    fn test_parse_v9_at_style() {
        let content = r#"
packages:
  lodash@4.17.15:
    resolution: {integrity: sha512-xxx}
  '@types/node@18.0.0':
    resolution: {integrity: sha512-xxx}
snapshots:
  jest-config@30.0.3(@types/node@18.0.0):
    dependencies: {}
"#;
        let deps = parse_pnpm_lock(content).unwrap();
        let map: HashMap<_, _> = deps
            .iter()
            .map(|d| (d.name.as_str(), d.version.as_str()))
            .collect();

        assert_eq!(map.get("lodash"), Some(&"4.17.15"));
        assert_eq!(map.get("@types/node"), Some(&"18.0.0"));
        assert_eq!(map.get("jest-config"), Some(&"30.0.3"));
    }
}
