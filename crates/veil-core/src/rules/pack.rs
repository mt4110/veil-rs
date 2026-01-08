use crate::model::Rule;
use anyhow::{Context, Result};
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
    description: String,
    pattern: String,
    severity: Option<String>,
    score: Option<u32>,
    category: Option<String>,
    tags: Option<Vec<String>>,
    base_score: Option<u32>,
    context_lines_before: Option<u8>,
    context_lines_after: Option<u8>,
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

pub fn parse_manifest(content: &str) -> Result<RulePackManifest> {
    toml::from_str(content).map_err(Into::into)
}

pub fn load_rules_from_content(
    content: &str,
    rules: &mut Vec<Rule>,
    ids: &mut HashSet<String>,
    source: Option<&Path>,
) -> Result<()> {
    let raw_file: RuleFile = toml::from_str(content)
        .with_context(|| format!("Failed to parse rule file content (source: {:?})", source))?;

    for raw in raw_file.rules {
        if ids.contains(&raw.id) {
            anyhow::bail!("Duplicate rule ID found: {}", raw.id);
        }

        let regex = Regex::new(&raw.pattern)
            .with_context(|| format!("Invalid regex for rule {}: {}", raw.id, raw.pattern))?;

        let rule = Rule {
            id: raw.id.clone(),
            pattern: regex,
            description: raw.description,
            severity: raw.severity.as_deref().unwrap_or("medium").into(),
            score: raw.score.unwrap_or(50),
            category: raw.category.unwrap_or_else(|| "other".to_string()),
            tags: raw.tags.unwrap_or_default(),
            base_score: raw.base_score,
            context_lines_before: raw.context_lines_before.unwrap_or(0),
            context_lines_after: raw.context_lines_after.unwrap_or(0),
            validator: None, // No validators from TOML
            placeholder: raw.placeholder,
        };

        ids.insert(rule.id.clone());
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
}
