use anyhow::Result;

use std::path::PathBuf;
// veil_core::rules::Rule needs to be imported directly or via public path
use veil_core::{get_all_rules, Rule};

pub fn check(config_path: Option<&PathBuf>) -> Result<bool> {
    println!("🔍 Validating configuration...");

    // 1. Load effective config
    let config = crate::config_loader::load_effective_config(config_path)?;
    println!("✅ Configuration loaded successfully.");
    println!("   - Config Path: {:?}", config_path);

    // 2. Fetch Rules (Local + Remote)
    let mut remote_rules = Vec::new();
    let remote_url = std::env::var("VEIL_REMOTE_RULES_URL")
        .ok()
        .or_else(|| config.core.remote_rules_url.clone());

    if let Some(url) = remote_url {
        println!("   - Remote Rules URL: {}", url);
        match veil_core::remote::fetch_remote_rules(&url, 3) {
            Ok(rules) => {
                println!("   - Fetched {} remote rules.", rules.len());
                remote_rules = rules;
            }
            Err(e) => {
                println!("   ⚠️ Failed to fetch remote rules: {}", e);
            }
        }
    }

    let rules = get_all_rules(&config, remote_rules);
    println!("   - Total Active Rules: {}", rules.len());

    // 3. Validate Rules (Regex Safety)
    println!("\n🛡️  Performing constraints check...");
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    for rule in &rules {
        if let Err(e) = check_rule_safety(rule) {
            match e {
                SafetyIssue::Error(msg) => errors.push(format!("Rule '{}': {}", rule.id, msg)),
                SafetyIssue::Warning(msg) => warnings.push(format!("Rule '{}': {}", rule.id, msg)),
            }
        }
    }

    if warnings.is_empty() && errors.is_empty() {
        println!("✅ No safety issues found in {} rules.", rules.len());
    } else {
        if !warnings.is_empty() {
            println!("\n⚠️  Warnings:");
            for w in &warnings {
                println!("   - {}", w);
            }
        }
        if !errors.is_empty() {
            println!("\n❌ Errors:");
            for e in &errors {
                println!("   - {}", e);
            }
        }
    }

    if !errors.is_empty() {
        Ok(true) // Should fail exit code
    } else {
        Ok(false) // Success
    }
}

enum SafetyIssue {
    Warning(String),
    #[allow(dead_code)]
    Error(String),
}

fn check_rule_safety(rule: &Rule) -> Result<(), SafetyIssue> {
    let pattern_str = rule.pattern.as_str();

    // 1. Compile Check
    // The Rule struct already holds a compiled Regex, so it is valid by definition.
    // We strictly check for "potential" ReDoS patterns in the string representation.

    // 2. DoS Check (Catastrophic Backtracking)
    let dangerous_fragments = ["(.+)+", "(.*)+", "(.+.*)+", "(.*.+)+"];
    for frag in dangerous_fragments {
        if pattern_str.contains(frag) {
            return Err(SafetyIssue::Warning(format!(
                "Potential ReDoS pattern '{}' detected. Avoid nested quantifiers.",
                frag
            )));
        }
    }

    Ok(())
}

pub fn dump(
    explicit_path: Option<&PathBuf>,
    layer: Option<crate::cli::ConfigLayer>,
    format: Option<crate::cli::ConfigFormat>,
) -> Result<()> {
    use crate::cli::{ConfigFormat, ConfigLayer};
    use crate::config_loader::load_config_layers;
    use veil_config::Config;

    let layers = load_config_layers(explicit_path)?;

    let selected: Option<&Config> = match layer.unwrap_or(ConfigLayer::Effective) {
        ConfigLayer::Org => layers.org.as_ref(),
        ConfigLayer::User => layers.user.as_ref(),
        ConfigLayer::Repo => layers.repo.as_ref(),
        ConfigLayer::Effective => Some(&layers.effective),
    };

    let Some(config) = selected else {
        println!("(no config for this layer)");
        return Ok(());
    };

    let fmt = format.unwrap_or(ConfigFormat::Json);

    match fmt {
        ConfigFormat::Json => {
            let s = serde_json::to_string_pretty(config)?;
            println!("{s}");
        }
        ConfigFormat::Toml => {
            let s = toml::to_string_pretty(config)?;
            println!("{s}");
        }
    }

    Ok(())
}
