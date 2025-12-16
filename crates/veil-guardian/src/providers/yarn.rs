use crate::models::{Ecosystem, PackageRef};
use crate::GuardianError;
use std::collections::HashSet;

/// Parses a `yarn.lock` file content and extracts package references.
///
/// This function distinguishes between Yarn Berry (v2+, YAML-based with `__metadata`)
/// and Yarn Classic (v1, stanza-based).
pub fn parse_yarn_lock(content: &str) -> Result<Vec<PackageRef>, GuardianError> {
    // strict Berry detection: try parsing as YAML and look for top-level keys
    if let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(content) {
        if let Some(mapping) = value.as_mapping() {
            // Berry lockfiles ALWAYS have a __metadata key at the root
            if mapping
                .keys()
                .any(|k| matches!(k, serde_yaml::Value::String(s) if s == "__metadata"))
            {
                return parse_yarn_berry(mapping);
            }
        }
    }

    // Fallback to Classic parser if not a valid Berry YAML structure
    parse_yarn_classic(content)
}

fn parse_yarn_berry(mapping: &serde_yaml::Mapping) -> Result<Vec<PackageRef>, GuardianError> {
    let mut packages = HashSet::new();

    for (key, value) in mapping {
        let key_str = match key.as_str() {
            Some(k) => k,
            None => continue, // Should not happen in valid Berry lockfile
        };

        // Skip metadata key
        if key_str == "__metadata" {
            continue;
        }

        // Berry keys can be comma-separated: "pkg@protocol:ver, pkg@protocol:ver2"
        // We need to check if ANY of the selectors denote a protocol we should skip.
        // If *all* selectors are skipped protocols, we skip the entry.
        // However, usually an entry represents a resolved package. If it resolves to a workspace/patch,
        // it applies to all selectors.
        // For simplicity and safety: if the key implies a local/workspace protocol, we skip it.

        // Protocol list to skip:
        // workspace:, patch:, portal:, link:, file:
        // We check if the key STARTS with these patterns after resolution identifiers.
        // Berry keys format: "package-name@protocol:version"
        // But the key in the mapping is the *descriptor*.
        // Example: "lodash@npm:4.17.21" -> OK
        // Example: "mypkg@workspace:." -> SKIP

        if key_str.contains("@workspace:")
            || key_str.contains("@patch:")
            || key_str.contains("@portal:")
            || key_str.contains("@link:")
            || key_str.contains("@file:")
        {
            continue;
        }

        // Extract version from the value
        let version = match value.get("version").and_then(|v| v.as_str()) {
            Some(v) => v,
            None => continue, // Entry without version? Skip.
        };

        // Extract package name from the key.
        // The key might be "a@npm:1, b@npm:1". We take the first one to determine the name.
        let first_selector = key_str.split(',').next().unwrap_or(key_str).trim();

        // Parse name from "name@..."
        // Handles scoped packages "@scope/pkg@..."
        let name = parse_package_name(first_selector);

        if !name.is_empty() {
            packages.insert((name.to_string(), version.to_string()));
        }
    }

    let mut result_vec: Vec<_> = packages.into_iter().collect();
    result_vec.sort();

    Ok(result_vec
        .into_iter()
        .map(|(name, version)| PackageRef {
            ecosystem: Ecosystem::Npm,
            name,
            version,
        })
        .collect())
}

fn parse_yarn_classic(content: &str) -> Result<Vec<PackageRef>, GuardianError> {
    let mut packages = HashSet::new();
    let mut current_keys: Vec<String> = Vec::new();

    for line in content.lines() {
        let line = line.trim_end();

        // Skip comments and empty lines
        if line.trim().starts_with('#') || line.trim().is_empty() {
            continue;
        }

        // Header detection: Line starts with non-whitespace (and not a comment/empty check above)
        // Headers end with ':' and can be comma-separated strings inside or outside quotes.
        // Example: "lodash@^4.17.15, lodash@4.17.21":
        // Example: lodash@^4.17.15, lodash@4.17.21:
        if !line.starts_with(' ') && line.ends_with(':') {
            // New stanza starts, reset keys
            current_keys.clear();

            // Remove trailing ':'
            let header = &line[..line.len() - 1];

            for sel in split_selectors_quote_aware(header) {
                if should_skip_protocol(&sel) {
                    continue;
                }
                current_keys.push(sel);
            }
            continue;
        } else if line.trim_start().starts_with("version ") {
            // Line is like: "  version \"1.2.3\""
            // Get text after "version "
            let rest = line.trim_start()[8..].trim();
            // Strip quotes
            let version = if rest.starts_with('"') && rest.ends_with('"') {
                &rest[1..rest.len() - 1]
            } else {
                rest
            };

            // Associate this version with all current keys
            for key in &current_keys {
                // Key is like "lodash@^4.17.15" or "@types/node@*"
                // Extract name
                let name = parse_package_name(key);
                if !name.is_empty() {
                    packages.insert((name.to_string(), version.to_string()));
                }
            }
        }
    }

    let mut result_vec: Vec<_> = packages.into_iter().collect();
    result_vec.sort();

    Ok(result_vec
        .into_iter()
        .map(|(name, version)| PackageRef {
            ecosystem: Ecosystem::Npm,
            name,
            version,
        })
        .collect())
}

///
/// Examples:
/// "lodash@npm:4.17.21" -> "lodash"
/// "@scope/pkg@npm:1.2.3" -> "@scope/pkg"
/// "foo@^1.0.0" -> "foo"
fn should_skip_protocol(selector: &str) -> bool {
    selector.contains("@file:")
        || selector.contains("@link:")
        || selector.contains("@workspace:")
        || selector.contains("@portal:")
        || selector.contains("@patch:")
        // Safety for malformed inputs
        || selector.starts_with("file:")
        || selector.starts_with("link:")
        || selector.starts_with("workspace:")
        || selector.starts_with("portal:")
        || selector.starts_with("patch:")
}

/// Split `a@x, b@y` but do NOT split commas inside quotes.
/// Examples:
/// `"lodash@^4, lodash@4"` -> [`lodash@^4, lodash@4`]
fn split_selectors_quote_aware(header: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut prev_backslash = false;

    for ch in header.chars() {
        match ch {
            '"' if !prev_backslash => {
                in_quotes = !in_quotes;
                cur.push(ch);
            }
            ',' if !in_quotes => {
                let part = cur.trim().trim_matches('"').trim();
                if !part.is_empty() {
                    out.push(part.to_string());
                }
                cur.clear();
            }
            '\\' => {
                prev_backslash = true;
                cur.push(ch);
                continue;
            }
            _ => cur.push(ch),
        }
        prev_backslash = false;
    }

    let part = cur.trim().trim_matches('"').trim();
    if !part.is_empty() {
        out.push(part.to_string());
    }

    out
}

fn parse_package_name(selector: &str) -> String {
    // Find the last '@' that is NOT the first character (to handle scopes)
    // Actually, Berry descriptors are usually "name@protocol:range".
    // Classic descriptors are "name@range".
    // Scoped: "@scope/name@range".

    // Strategy:
    // 1. If starts with @, skip it temporarily to find the separator.
    let search_start = if selector.starts_with('@') { 1 } else { 0 };

    match selector[search_start..].find('@') {
        Some(idx) => selector[..search_start + idx].to_string(),
        None => selector.to_string(), // Fallback, though unlikely for valid keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_name() {
        assert_eq!(parse_package_name("lodash@npm:1.0.0"), "lodash");
        assert_eq!(parse_package_name("@scope/pkg@npm:1.0.0"), "@scope/pkg");
        assert_eq!(parse_package_name("foo@^1.2.3"), "foo");
        assert_eq!(
            parse_package_name("no-version-separator"),
            "no-version-separator"
        );
    }

    #[test]
    fn test_berry_parsing_valid() {
        let content = r#"
__metadata:
  version: 6
  cacheKey: 8

"lodash@npm:4.17.21":
  version: 4.17.21
  resolution: "lodash@npm:4.17.21"
  languageName: node
  linkType: hard

"@scope/pkg@npm:^1.2.0, @scope/pkg@npm:1.2.3":
  version: 1.2.3
  resolution: "@scope/pkg@npm:1.2.3"
  languageName: node
  linkType: hard
"#;
        let result = parse_yarn_lock(content).unwrap();
        assert_eq!(result.len(), 2);

        // Result is sorted by name
        // @scope/pkg comes before lodash
        assert_eq!(result[0].name, "@scope/pkg");
        assert_eq!(result[0].version, "1.2.3");
        assert_eq!(result[1].name, "lodash");
        assert_eq!(result[1].version, "4.17.21");
    }

    #[test]
    fn test_berry_parsing_skips_protocols() {
        let content = r#"
__metadata:
  version: 6

"workspace-pkg@workspace:.":
  version: 0.0.0-use.local

"patched-pkg@patch:foo@npm%3A1.0.0#./patches/foo.patch":
  version: 1.0.0

"portal-pkg@portal:./pkgs/portal-pkg":
  version: 0.0.0-use.local
"#;
        let result = parse_yarn_lock(content).unwrap();
        assert!(
            result.is_empty(),
            "Should skip workspace, patch, and portal protocols"
        );
    }
    #[test]
    fn test_classic_parsing() {
        let content = r#"# THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.
# yarn lockfile v1


"@types/node@*":
  version "18.0.0"
  resolved "https://registry.yarnpkg.com/@types/node/-/node-18.0.0.tgz#..."
  integrity sha512-..."

lodash@^4.17.15, lodash@4.17.21:
  version "4.17.21"
  resolved "https://registry.yarnpkg.com/lodash/-/lodash-4.17.21.tgz#..."

"fsevents@^1.2.7":
  version "1.2.9"
  optionalDependencies:
    nan "^2.12.1"
"#;
        let result = parse_yarn_lock(content).unwrap();
        // Should have 3 packages: @types/node, lodash, fsevents
        assert_eq!(result.len(), 3);

        // Sorted Check
        // 1. @types/node
        assert_eq!(result[0].name, "@types/node");
        assert_eq!(result[0].version, "18.0.0");

        // 2. fsevents
        assert_eq!(result[1].name, "fsevents");
        assert_eq!(result[1].version, "1.2.9");

        // 3. lodash
        assert_eq!(result[2].name, "lodash");
        assert_eq!(result[2].version, "4.17.21");
    }
    #[test]
    fn test_classic_header_quote_aware_split() {
        let content = r#"
# yarn lockfile v1

"lodash@^4.17.15, lodash@4.17.21":
  version "4.17.21"

"@scope/pkg@^1.0.0, @scope/pkg@1.2.3":
  version "1.2.3"
"#;

        let result = parse_yarn_lock(content).unwrap();
        assert_eq!(result.len(), 2);

        assert_eq!(result[0].name, "@scope/pkg");
        assert_eq!(result[0].version, "1.2.3");
        assert_eq!(result[1].name, "lodash");
        assert_eq!(result[1].version, "4.17.21");
    }

    #[test]
    fn test_classic_skips_protocols() {
        let content = r#"
# yarn lockfile v1

"local@file:../local":
  version "0.0.0"

"linked@link:../linked":
  version "0.0.0"

"ok@^1.0.0":
  version "1.0.1"
"#;

        let result = parse_yarn_lock(content).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "ok");
        assert_eq!(result[0].version, "1.0.1");
    }
}
