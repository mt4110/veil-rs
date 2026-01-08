use crate::model::Rule;
use crate::rules::pack::{load_rules_from_content, parse_manifest};
use regex::Regex;
use rust_embed::RustEmbed;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
use veil_config::Config;

#[derive(RustEmbed)]
#[folder = "../veil/rules/default/"]
struct DefaultRulesAssets;

// Sanity check path (Correct relative path)
const _CHECK: &str = include_str!("../../../veil/rules/default/00_manifest.toml");

static DEFAULT_RULES: OnceLock<Vec<Rule>> = OnceLock::new();

pub fn get_default_rules() -> Vec<Rule> {
    DEFAULT_RULES
        .get_or_init(|| {
            load_embedded_rules().unwrap_or_else(|e| {
                eprintln!("Failed to load default rules: {}", e);
                vec![]
            })
        })
        .clone()
}

fn load_embedded_rules() -> anyhow::Result<Vec<Rule>> {
    let mut rules = Vec::new();
    let mut loaded_ids = HashSet::new();

    // Try to load manifest
    if let Some(f) = DefaultRulesAssets::get("00_manifest.toml") {
        let content = std::str::from_utf8(f.data.as_ref())?;
        if let Ok(manifest) = parse_manifest(content) {
            if !manifest.files.is_empty() {
                for file_name in &manifest.files {
                    if let Some(rule_file) = DefaultRulesAssets::get(file_name) {
                        let rule_content = std::str::from_utf8(rule_file.data.as_ref())?;
                        if let Err(e) = load_rules_from_content(
                            rule_content,
                            &mut rules,
                            &mut loaded_ids,
                            Some(std::path::Path::new(file_name)),
                        ) {
                            eprintln!("Failed to load rule file {}: {}", file_name, e);
                        }
                    } else {
                        eprintln!("Warning: Manifest referenced file '{}' but it was not found in assets.", file_name);
                    }
                }
            } else {
                load_auto(&mut rules, &mut loaded_ids)?;
            }
        } else {
            eprintln!("Failed to parse manifest");
            load_auto(&mut rules, &mut loaded_ids)?;
        }
    } else {
        load_auto(&mut rules, &mut loaded_ids)?;
    }

    // println!("DEBUG: Loaded {} rules: {:?}", rules.len(), rules.iter().map(|r| &r.id).collect::<Vec<_>>());
    Ok(rules)
}

fn load_auto(rules: &mut Vec<Rule>, ids: &mut HashSet<String>) -> anyhow::Result<()> {
    // RustEmbed iter() returns filenames.
    // We filter for .toml and sort.
    let mut files: Vec<String> = DefaultRulesAssets::iter()
        .map(|f| f.to_string())
        .filter(|name| name.ends_with(".toml") && name != "00_manifest.toml")
        .collect();
    files.sort();

    for name in files {
        if let Some(f) = DefaultRulesAssets::get(&name) {
            let rule_content = std::str::from_utf8(f.data.as_ref())?;
            load_rules_from_content(rule_content, rules, ids, Some(std::path::Path::new(&name)))?;
        }
    }
    Ok(())
}

pub fn get_all_rules(config: &Config, extra_rules: Vec<Rule>) -> Vec<Rule> {
    // defaults is Vec<Rule> (cloned from static, loaded from assets)
    let defaults = get_default_rules();
    let mut rule_map: HashMap<String, Rule> =
        defaults.into_iter().map(|r| (r.id.clone(), r)).collect();

    // Load from rules_dir if configured
    if let Some(rules_dir) = &config.core.rules_dir {
        let path = std::path::Path::new(rules_dir);
        if path.exists() {
            match crate::rules::pack::load_rule_pack(path) {
                Ok(rules) => {
                    // Merge pack rules (override defaults by ID)
                    for r in rules {
                        rule_map.insert(r.id.clone(), r);
                    }
                }
                Err(e) => eprintln!("Error loading rule pack from {:?}: {}", path, e),
            }
        }
    }

    // Merge extra/remote rules
    // If ID exists (matches builtin), we overwrite it (Remote updates/fixes builtin)
    // If ID is new, we insert it.
    for rule in extra_rules {
        rule_map.insert(rule.id.clone(), rule);
    }

    // Merge overrides from Config
    for (id, rule_conf) in &config.rules {
        if let Some(pattern_str) = &rule_conf.pattern {
            // New rule or Overwrite pattern (Pure TOML Rule)
            if let Ok(regex) = Regex::new(pattern_str) {
                let rule = Rule {
                    id: id.clone(),
                    pattern: regex,
                    description: rule_conf
                        .description
                        .clone()
                        .unwrap_or_else(|| "Custom rule".to_string()),
                    severity: rule_conf.severity.as_deref().unwrap_or("medium").into(),
                    score: rule_conf.score.unwrap_or(50) as u32,
                    category: rule_conf.category.clone().unwrap_or("custom".to_string()),
                    tags: rule_conf.tags.clone().unwrap_or_default(),
                    base_score: rule_conf.base_score,
                    context_lines_before: rule_conf.context_lines_before.unwrap_or(2),
                    context_lines_after: rule_conf.context_lines_after.unwrap_or(0),
                    validator: None,
                    placeholder: rule_conf.placeholder.clone(),
                };
                rule_map.insert(id.clone(), rule);
            } else {
                eprintln!("Invalid Regex pattern for rule {}: {}", id, pattern_str);
            }
        } else {
            // Override existing rule (Builtin OR Remote)
            if let Some(rule) = rule_map.get_mut(id) {
                if let Some(sev) = &rule_conf.severity {
                    rule.severity = sev.as_str().into();
                }
                if let Some(desc) = &rule_conf.description {
                    rule.description = desc.clone();
                }
                if let Some(score) = rule_conf.score {
                    rule.score = score as u32;
                }
                if let Some(cat) = &rule_conf.category {
                    rule.category = cat.clone();
                }
                if let Some(tags) = &rule_conf.tags {
                    rule.tags = tags.clone();
                }
                if let Some(ph) = &rule_conf.placeholder {
                    rule.placeholder = Some(ph.clone());
                }
            }
        }
    }

    rule_map
        .into_values()
        .filter(|r| config.rules.get(&r.id).map(|rc| rc.enabled).unwrap_or(true))
        .collect()
}
