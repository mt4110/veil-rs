use crate::model::Rule;
use crate::validators::resolve_validator;
use anyhow::{Context, Result};
use rayon::prelude::*;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct RulePackManifest {
    pub pack: PackMetadata,
    #[serde(default)]
    pub files: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PackMetadata {
    pub id: String,
    pub version: u32,
    pub schema_version: u32,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RuleFile {
    #[serde(rename = "rules")]
    rules: Vec<RuleConfigRaw>,
}

// Intermediate struct to match TOML structure before converting to core::Rule
#[derive(Debug, Deserialize)]
struct RuleConfigRaw {
    id: String,
    enabled: Option<bool>,
    description: String,
    pattern: String,
    severity: Option<String>,
    score: Option<u32>,
    category: Option<String>,
    tags: Option<Vec<String>>,
    base_score: Option<u32>,
    context_lines_before: Option<u8>,
    context_lines_after: Option<u8>,
    validator: Option<String>,
    // placeholder is optional, but we will enforce canonicalization later
    placeholder: Option<String>,
}

pub fn load_rule_pack(dir: &Path) -> Result<Vec<Rule>> {
    let manifest_path = dir.join("00_manifest.toml");
    let mut rules = Vec::new();
    let mut loaded_ids = HashSet::new();

    if manifest_path.exists() {
        let content = fs::read_to_string(&manifest_path)
            .with_context(|| format!("Failed to read manifest at {:?}", manifest_path))?;
        let manifest = parse_manifest(&content)
            .with_context(|| format!("Failed to parse manifest at {:?}", manifest_path))?;

        if !manifest.files.is_empty() {
            for file_name in &manifest.files {
                let file_path = dir.join(file_name);
                load_rules_from_path(&file_path, &mut rules, &mut loaded_ids)?;
            }
        } else {
            load_rules_auto(dir, &mut rules, &mut loaded_ids)?;
        }
    } else {
        load_rules_auto(dir, &mut rules, &mut loaded_ids)?;
    }

    Ok(rules)
}

pub fn load_rule_templates_parallel(root: &Path) -> Result<Vec<Rule>> {
    if !root.exists() {
        anyhow::bail!("Rule template root missing: {:?}", root);
    }
    if !root.is_dir() {
        anyhow::bail!("Rule template root is not a directory: {:?}", root);
    }

    let mut paths = Vec::new();
    collect_rule_paths_recursive(root, &mut paths)?;
    paths.sort();
    load_rule_paths_parallel(paths)
}

fn collect_rule_paths_recursive(dir: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .with_context(|| format!("Failed to read rule template directory {:?}", dir))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    entries.sort();

    for path in entries {
        if path.is_dir() {
            collect_rule_paths_recursive(&path, paths)?;
        } else if is_rule_toml(&path) {
            paths.push(path);
        }
    }

    Ok(())
}

fn is_rule_toml(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "toml")
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name != "00_manifest.toml")
}

fn load_rule_paths_parallel(paths: Vec<PathBuf>) -> Result<Vec<Rule>> {
    let parsed_rules: Result<Vec<Vec<Rule>>> = paths
        .par_iter()
        .map(|path| parse_rules_from_path(path))
        .collect();

    let mut rules = Vec::new();
    let mut ids = HashSet::new();

    for file_rules in parsed_rules? {
        append_rules_with_duplicate_check(file_rules, &mut rules, &mut ids)?;
    }

    Ok(rules)
}

fn load_rules_auto(dir: &Path, rules: &mut Vec<Rule>, ids: &mut HashSet<String>) -> Result<()> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().is_some_and(|ext| ext == "toml")
                && p.file_name().unwrap() != "00_manifest.toml"
        })
        .collect();
    paths.sort();

    for p in paths {
        load_rules_from_path(&p, rules, ids)?;
    }
    Ok(())
}

fn load_rules_from_path(
    path: &Path,
    rules: &mut Vec<Rule>,
    ids: &mut HashSet<String>,
) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("Rule file missing: {:?}", path);
    }
    let content = fs::read_to_string(path)?;
    // Use the parsed rules
    load_rules_from_content(&content, rules, ids, Some(path))
}

fn parse_rules_from_path(path: &Path) -> Result<Vec<Rule>> {
    if !path.exists() {
        anyhow::bail!("Rule file missing: {:?}", path);
    }
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read rule file {:?}", path))?;
    parse_rules_from_content(&content, Some(path))
}

pub fn parse_manifest(content: &str) -> Result<RulePackManifest> {
    toml::from_str(content).map_err(Into::into)
}

pub fn load_rules_from_content(
    content: &str,
    rules: &mut Vec<Rule>,
    ids: &mut HashSet<String>,
    source: Option<&Path>,
) -> Result<()> {
    let parsed_rules = parse_rules_from_content(content, source)?;
    append_rules_with_duplicate_check(parsed_rules, rules, ids)
}

fn parse_rules_from_content(content: &str, source: Option<&Path>) -> Result<Vec<Rule>> {
    let raw_file: RuleFile = toml::from_str(content)
        .with_context(|| format!("Failed to parse rule file content (source: {:?})", source))?;

    let mut rules = Vec::new();
    for raw in raw_file.rules {
        let regex = Regex::new(&raw.pattern)
            .with_context(|| format!("Invalid regex for rule {}: {}", raw.id, raw.pattern))?;

        let validator_id = raw.validator.clone();
        let validator = match validator_id.as_deref() {
            Some(id) => Some(resolve_validator(id).with_context(|| {
                format!(
                    "Unknown validator '{}' for rule '{}' (source: {:?})",
                    id, raw.id, source
                )
            })?),
            None => None,
        };

        let rule = Rule {
            id: raw.id.clone(),
            enabled: raw.enabled.unwrap_or(true),
            pattern: regex,
            description: raw.description,
            severity: raw.severity.as_deref().unwrap_or("medium").into(),
            score: raw.score.unwrap_or(50),
            category: raw.category.unwrap_or_else(|| "other".to_string()),
            tags: raw.tags.unwrap_or_default(),
            base_score: raw.base_score,
            context_lines_before: raw.context_lines_before.unwrap_or(0),
            context_lines_after: raw.context_lines_after.unwrap_or(0),
            validator_id,
            validator,
            placeholder: raw.placeholder,
        };

        rules.push(rule);
    }
    Ok(rules)
}

fn append_rules_with_duplicate_check(
    parsed_rules: Vec<Rule>,
    rules: &mut Vec<Rule>,
    ids: &mut HashSet<String>,
) -> Result<()> {
    for rule in parsed_rules {
        if !ids.insert(rule.id.clone()) {
            anyhow::bail!("Duplicate rule ID found: {}", rule.id);
        }
        rules.push(rule);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    fn setup_test_dir(name: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push("veil_test_packs");
        dir.push(name);
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_load_manifest_order() {
        let dir = setup_test_dir("manifest_order");

        // manifest
        let manifest = r#"
files = ["b.toml", "a.toml"]

[pack]
id = "test.pack"
version = 1
schema_version = 1
"#;
        File::create(dir.join("00_manifest.toml"))
            .unwrap()
            .write_all(manifest.as_bytes())
            .unwrap();

        // a.toml (Loaded SECOND)
        let a = r#"
[[rules]]
id = "rule.a"
description = "A"
pattern = "a"
"#;
        File::create(dir.join("a.toml"))
            .unwrap()
            .write_all(a.as_bytes())
            .unwrap();

        // b.toml (Loaded FIRST)
        let b = r#"
[[rules]]
id = "rule.b"
description = "B"
pattern = "b"
"#;
        File::create(dir.join("b.toml"))
            .unwrap()
            .write_all(b.as_bytes())
            .unwrap();

        let rules = load_rule_pack(&dir).unwrap();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].id, "rule.b");
        assert_eq!(rules[1].id, "rule.a");
    }

    #[test]
    fn test_load_auto_order() {
        let dir = setup_test_dir("auto_order");

        // No manifest

        // a.toml
        let a = r#"
[[rules]]
id = "rule.a"
description = "A"
pattern = "a"
"#;
        File::create(dir.join("a.toml"))
            .unwrap()
            .write_all(a.as_bytes())
            .unwrap();

        // b.toml
        let b = r#"
[[rules]]
id = "rule.b"
description = "B"
pattern = "b"
"#;
        File::create(dir.join("b.toml"))
            .unwrap()
            .write_all(b.as_bytes())
            .unwrap();

        let rules = load_rule_pack(&dir).unwrap();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].id, "rule.a");
        assert_eq!(rules[1].id, "rule.b");
    }

    #[test]
    fn test_duplicate_id_error() {
        let dir = setup_test_dir("duplicate_id");

        // a.toml
        let a = r#"
[[rules]]
id = "rule.common"
description = "A"
pattern = "a"
"#;
        File::create(dir.join("a.toml"))
            .unwrap()
            .write_all(a.as_bytes())
            .unwrap();

        // b.toml
        let b = r#"
[[rules]]
id = "rule.common"
description = "B"
pattern = "b"
"#;
        File::create(dir.join("b.toml"))
            .unwrap()
            .write_all(b.as_bytes())
            .unwrap();

        let res = load_rule_pack(&dir);
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("Duplicate rule ID found"));
    }

    #[test]
    fn test_unknown_validator_errors() {
        let mut rules = Vec::new();
        let mut ids = HashSet::new();
        let content = r#"
[[rules]]
id = "rule.with.unknown.validator"
description = "Unknown validator"
pattern = "a"
validator = "missing_validator"
"#;

        let res = load_rules_from_content(content, &mut rules, &mut ids, None);

        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("Unknown validator 'missing_validator'"));
    }

    #[test]
    fn test_known_validator_is_resolved() {
        let mut rules = Vec::new();
        let mut ids = HashSet::new();
        let content = r#"
[[rules]]
id = "rule.with.known.validator"
description = "Known validator"
pattern = "[0-9]{16}"
validator = "luhn"
"#;

        load_rules_from_content(content, &mut rules, &mut ids, None).unwrap();

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].validator_id.as_deref(), Some("luhn"));
        assert!(rules[0].validator.is_some());
    }

    #[test]
    fn test_rule_enabled_flag_is_loaded() {
        let mut rules = Vec::new();
        let mut ids = HashSet::new();
        let content = r#"
[[rules]]
id = "rule.disabled"
enabled = false
description = "Disabled by default"
pattern = "a"
"#;

        load_rules_from_content(content, &mut rules, &mut ids, None).unwrap();

        assert_eq!(rules.len(), 1);
        assert!(!rules[0].enabled);
    }

    #[test]
    fn test_load_rule_templates_parallel_recurses_deterministically() {
        let dir = setup_test_dir("templates_parallel_order");
        fs::create_dir_all(dir.join("templates/pii/kv")).unwrap();
        fs::create_dir_all(dir.join("templates/secret/leak")).unwrap();

        let b = r#"
[[rules]]
id = "rule.b"
description = "B"
pattern = "b"
"#;
        File::create(dir.join("templates/secret/leak/b.toml"))
            .unwrap()
            .write_all(b.as_bytes())
            .unwrap();

        let a = r#"
[[rules]]
id = "rule.a"
description = "A"
pattern = "a"
"#;
        File::create(dir.join("templates/pii/kv/a.toml"))
            .unwrap()
            .write_all(a.as_bytes())
            .unwrap();

        let rules = load_rule_templates_parallel(&dir).unwrap();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].id, "rule.a");
        assert_eq!(rules[1].id, "rule.b");
    }

    #[test]
    fn test_load_rule_templates_parallel_rejects_duplicate_ids() {
        let dir = setup_test_dir("templates_parallel_duplicate");
        fs::create_dir_all(dir.join("templates/pii/kv")).unwrap();
        fs::create_dir_all(dir.join("templates/secret/leak")).unwrap();

        let a = r#"
[[rules]]
id = "rule.common"
description = "A"
pattern = "a"
"#;
        File::create(dir.join("templates/pii/kv/a.toml"))
            .unwrap()
            .write_all(a.as_bytes())
            .unwrap();

        let b = r#"
[[rules]]
id = "rule.common"
description = "B"
pattern = "b"
"#;
        File::create(dir.join("templates/secret/leak/b.toml"))
            .unwrap()
            .write_all(b.as_bytes())
            .unwrap();

        let res = load_rule_templates_parallel(&dir);
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("Duplicate rule ID found"));
    }

    #[test]
    #[ignore = "loads the large repository template corpus"]
    fn test_jp_security_templates_1000_loads_parallel() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../veil/rules_ja/templates/jp_security_templates_1000");

        let rules = load_rule_templates_parallel(&root).unwrap();
        assert_eq!(rules.len(), 1000);
    }
}
