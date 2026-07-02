use crate::config::{Config, RuleConfig};
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

pub const BUILTIN_PRESET_IDS: &[&str] = &[
    "standard-jp",
    "fintech-jp",
    "gov-jp",
    "logs-jp",
    "si-vendor-jp",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PresetFile {
    #[serde(default)]
    rules: HashMap<String, PresetRuleOverride>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PresetRuleOverride {
    enabled: Option<bool>,
    base_score: Option<u32>,
}

pub fn builtin_preset_config(preset_id: &str) -> Result<Config> {
    let source = builtin_preset_source(preset_id)?;
    parse_preset_config(preset_id, source)
}

pub fn apply_builtin_preset_as_base(config: Config, preset_id: &str) -> Result<Config> {
    let mut preset_config = builtin_preset_config(preset_id)?;
    preset_config.merge(config);
    Ok(preset_config)
}

fn builtin_preset_source(preset_id: &str) -> Result<&'static str> {
    match preset_id {
        "standard-jp" => Ok(include_str!("../../veil/presets/standard-jp.toml")),
        "fintech-jp" => Ok(include_str!("../../veil/presets/fintech-jp.toml")),
        "gov-jp" => Ok(include_str!("../../veil/presets/gov-jp.toml")),
        "logs-jp" => Ok(include_str!("../../veil/presets/logs-jp.toml")),
        "si-vendor-jp" => Ok(include_str!("../../veil/presets/si-vendor-jp.toml")),
        _ => bail!(
            "Unknown preset '{}'. Built-in presets: {}",
            preset_id,
            BUILTIN_PRESET_IDS.join(", ")
        ),
    }
}

fn parse_preset_config(preset_id: &str, source: &str) -> Result<Config> {
    let preset: PresetFile = toml::from_str(source)
        .with_context(|| format!("Invalid preset TOML for '{}'", preset_id))?;

    if preset.rules.is_empty() {
        bail!(
            "Preset '{}' must contain at least one rule override",
            preset_id
        );
    }

    let mut config = Config::default();
    config.core.include.clear();
    for (rule_id, rule_override) in preset.rules {
        if rule_override.enabled.is_none() && rule_override.base_score.is_none() {
            bail!(
                "Preset '{}' rule '{}' must set enabled and/or base_score",
                preset_id,
                rule_id
            );
        }

        let rule_config = RuleConfig {
            enabled: rule_override.enabled.unwrap_or(true),
            base_score: rule_override.base_score,
            ..RuleConfig::default()
        };
        config.rules.insert(rule_id, rule_config);
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_builtin_presets_parse_to_rule_overrides() {
        for preset_id in BUILTIN_PRESET_IDS {
            let config = builtin_preset_config(preset_id).unwrap();
            assert!(
                !config.rules.is_empty(),
                "preset '{}' should define rule overrides",
                preset_id
            );
            for rule in config.rules.values() {
                assert!(rule.severity.is_none());
                assert!(rule.pattern.is_none());
                assert!(rule.score.is_none());
                assert!(rule.category.is_none());
                assert!(rule.tags.is_none());
                assert!(rule.validator.is_none());
            }
            assert!(config.core.include.is_empty());
            assert!(config.core.ignore.is_empty());
            assert!(config.core.rules_dir.is_none());
        }
    }

    #[test]
    fn unknown_builtin_preset_errors() {
        let err = builtin_preset_config("minimal-ci").unwrap_err();
        assert!(err.to_string().contains("Unknown preset 'minimal-ci'"));
    }

    #[test]
    fn preset_rejects_fields_outside_enabled_and_base_score() {
        let err = parse_preset_config(
            "bad",
            r#"
[rules."pii.jp.mynumber.keyword"]
enabled = true
severity = "high"
"#,
        )
        .unwrap_err();

        assert!(err.to_string().contains("Invalid preset TOML"));
    }

    #[test]
    fn apply_builtin_preset_as_base_lets_config_override_rules() {
        let mut config = Config::default();
        config.rules.insert(
            "pii.fin.credit_card.keyword".to_string(),
            RuleConfig {
                enabled: false,
                base_score: Some(99),
                ..RuleConfig::default()
            },
        );

        let merged = apply_builtin_preset_as_base(config, "fintech-jp").unwrap();
        let rule = merged.rules.get("pii.fin.credit_card.keyword").unwrap();

        assert!(!rule.enabled);
        assert_eq!(rule.base_score, Some(99));
    }
}
