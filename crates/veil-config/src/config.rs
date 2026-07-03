use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub core: CoreConfig,
    #[serde(default)]
    pub masking: MaskingConfig,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub rules: HashMap<String, RuleConfig>,
}

#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub mask_mode: Option<MaskMode>,
    pub show_snippets: bool,
    pub max_findings: Option<usize>,
    pub max_findings_is_set: bool,
}

impl Serialize for OutputConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let field_count = usize::from(self.mask_mode.is_some())
            + 1
            + usize::from(self.max_findings_is_set && self.max_findings.is_some());
        let mut state = serializer.serialize_struct("OutputConfig", field_count)?;
        if let Some(mask_mode) = self.mask_mode {
            state.serialize_field("mask_mode", &mask_mode)?;
        }
        state.serialize_field("show_snippets", &self.show_snippets)?;
        if self.max_findings_is_set {
            if let Some(max_findings) = self.max_findings {
                state.serialize_field("max_findings", &max_findings)?;
            }
        }
        state.end()
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            mask_mode: None,
            show_snippets: true,
            max_findings: Some(1000),
            max_findings_is_set: false,
        }
    }
}

impl<'de> Deserialize<'de> for OutputConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawOutputConfig {
            #[serde(default)]
            mask_mode: Option<MaskMode>,
            #[serde(default = "default_true")]
            show_snippets: bool,
            #[serde(default)]
            max_findings: Option<Option<usize>>,
        }

        let raw = RawOutputConfig::deserialize(deserializer)?;
        let max_findings_is_set = raw.max_findings.is_some();
        let max_findings = raw.max_findings.unwrap_or_else(default_max_findings);

        Ok(Self {
            mask_mode: raw.mask_mode,
            show_snippets: raw.show_snippets,
            max_findings,
            max_findings_is_set,
        })
    }
}

fn default_max_findings() -> Option<usize> {
    Some(1000)
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")] // "redact", "partial", "plain"
pub enum MaskMode {
    #[default]
    Redact,
    Partial,
    Plain,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CoreConfig {
    #[serde(default = "default_include")]
    pub include: Vec<String>,
    #[serde(default)]
    pub ignore: Vec<String>,
    pub max_file_size: Option<u64>,
    pub max_file_count: Option<usize>,
    pub fail_on_score: Option<u32>,
    pub remote_rules_url: Option<String>,
    pub rules_dir: Option<String>,
}

impl Config {
    pub fn merge(&mut self, other: Config) {
        // Merge Core
        // For lists, we usually append? Or should project override?
        // Appending seems safer for "ignore" (Org says ignore X, Project says ignore Y -> Ignore X+Y).
        // For include, likely same?
        self.core.include.extend(other.core.include);
        self.core.ignore.extend(other.core.ignore);

        // Scalars: Override if other has value
        if let Some(val) = other.core.max_file_size {
            self.core.max_file_size = Some(val);
        }
        if let Some(val) = other.core.max_file_count {
            self.core.max_file_count = Some(val);
        }
        if let Some(val) = other.core.fail_on_score {
            self.core.fail_on_score = Some(val);
        }
        if let Some(val) = other.core.remote_rules_url {
            self.core.remote_rules_url = Some(val);
        }
        if let Some(val) = other.core.rules_dir {
            self.core.rules_dir = Some(val);
        }

        // Merge Rules (field-wise override/insert)
        for (id, rule) in other.rules {
            match self.rules.entry(id) {
                Entry::Occupied(mut entry) => entry.get_mut().merge(rule),
                Entry::Vacant(entry) => {
                    entry.insert(rule);
                }
            }
        }

        // Merge Masking (Simple override for now as fields are not Option)
        if other.masking.placeholder != default_placeholder() {
            self.masking.placeholder = other.masking.placeholder;
        }

        // Merge Output
        if let Some(mode) = other.output.mask_mode {
            self.output.mask_mode = Some(mode);
        }
        // show_snippets default is true. If other is false, override.
        if !other.output.show_snippets {
            self.output.show_snippets = false;
        }
        if other.output.max_findings_is_set || other.output.max_findings != default_max_findings() {
            self.output.max_findings = other.output.max_findings;
            self.output.max_findings_is_set = other.output.max_findings_is_set;
        }
    }
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            include: default_include(),
            ignore: Vec::new(),
            max_file_size: None,  // Default handled at usage site
            max_file_count: None, // Default handled at usage site
            fail_on_score: None,
            remote_rules_url: None,
            rules_dir: None,
        }
    }
}

fn default_include() -> Vec<String> {
    vec![".".to_string()]
}
// Remove default_max_file_size as it's no longer used in serde default

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MaskingConfig {
    #[serde(default = "default_placeholder")]
    pub placeholder: String,
}

impl Default for MaskingConfig {
    fn default() -> Self {
        Self {
            placeholder: default_placeholder(),
        }
    }
}

fn default_placeholder() -> String {
    "<REDACTED>".to_string()
}

#[derive(Debug, Clone)]
pub struct RuleConfig {
    pub enabled: bool,
    pub enabled_is_set: bool,
    pub severity: Option<String>,
    pub pattern: Option<String>,
    pub score: Option<u8>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub base_score: Option<u32>,
    pub context_lines_before: Option<u8>,
    pub context_lines_after: Option<u8>,
    pub validator: Option<String>,
    pub description: Option<String>,
    pub placeholder: Option<String>,
}

impl Serialize for RuleConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let field_count = usize::from(self.enabled_is_set)
            + usize::from(self.severity.is_some())
            + usize::from(self.pattern.is_some())
            + usize::from(self.score.is_some())
            + usize::from(self.category.is_some())
            + usize::from(self.tags.is_some())
            + usize::from(self.base_score.is_some())
            + usize::from(self.context_lines_before.is_some())
            + usize::from(self.context_lines_after.is_some())
            + usize::from(self.validator.is_some())
            + usize::from(self.description.is_some())
            + usize::from(self.placeholder.is_some());
        let mut state = serializer.serialize_struct("RuleConfig", field_count)?;
        if self.enabled_is_set {
            state.serialize_field("enabled", &self.enabled)?;
        }
        if let Some(severity) = &self.severity {
            state.serialize_field("severity", severity)?;
        }
        if let Some(pattern) = &self.pattern {
            state.serialize_field("pattern", pattern)?;
        }
        if let Some(score) = self.score {
            state.serialize_field("score", &score)?;
        }
        if let Some(category) = &self.category {
            state.serialize_field("category", category)?;
        }
        if let Some(tags) = &self.tags {
            state.serialize_field("tags", tags)?;
        }
        if let Some(base_score) = self.base_score {
            state.serialize_field("base_score", &base_score)?;
        }
        if let Some(context_lines_before) = self.context_lines_before {
            state.serialize_field("context_lines_before", &context_lines_before)?;
        }
        if let Some(context_lines_after) = self.context_lines_after {
            state.serialize_field("context_lines_after", &context_lines_after)?;
        }
        if let Some(validator) = &self.validator {
            state.serialize_field("validator", validator)?;
        }
        if let Some(description) = &self.description {
            state.serialize_field("description", description)?;
        }
        if let Some(placeholder) = &self.placeholder {
            state.serialize_field("placeholder", placeholder)?;
        }
        state.end()
    }
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            enabled_is_set: true,
            severity: None,
            pattern: None,
            score: None,
            category: None,
            tags: None,
            base_score: None,
            context_lines_before: None,
            context_lines_after: None,
            validator: None,
            description: None,
            placeholder: None,
        }
    }
}

impl<'de> Deserialize<'de> for RuleConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawRuleConfig {
            enabled: Option<bool>,
            severity: Option<String>,
            pattern: Option<String>,
            score: Option<u8>,
            category: Option<String>,
            tags: Option<Vec<String>>,
            base_score: Option<u32>,
            context_lines_before: Option<u8>,
            context_lines_after: Option<u8>,
            validator: Option<String>,
            description: Option<String>,
            placeholder: Option<String>,
        }

        let raw = RawRuleConfig::deserialize(deserializer)?;
        let enabled_is_set = raw.enabled.is_some();

        Ok(Self {
            enabled: raw.enabled.unwrap_or(true),
            enabled_is_set,
            severity: raw.severity,
            pattern: raw.pattern,
            score: raw.score,
            category: raw.category,
            tags: raw.tags,
            base_score: raw.base_score,
            context_lines_before: raw.context_lines_before,
            context_lines_after: raw.context_lines_after,
            validator: raw.validator,
            description: raw.description,
            placeholder: raw.placeholder,
        })
    }
}

impl RuleConfig {
    fn merge(&mut self, other: RuleConfig) {
        if other.pattern.is_some() {
            *self = other;
            return;
        }

        if other.enabled_is_set || !other.enabled {
            self.enabled = other.enabled;
            self.enabled_is_set = other.enabled_is_set || !other.enabled;
        }
        if other.severity.is_some() {
            self.severity = other.severity;
        }
        if other.score.is_some() {
            self.score = other.score;
        }
        if other.category.is_some() {
            self.category = other.category;
        }
        if other.tags.is_some() {
            self.tags = other.tags;
        }
        if other.base_score.is_some() {
            self.base_score = other.base_score;
        }
        if other.context_lines_before.is_some() {
            self.context_lines_before = other.context_lines_before;
        }
        if other.context_lines_after.is_some() {
            self.context_lines_after = other.context_lines_after;
        }
        if other.validator.is_some() {
            self.validator = other.validator;
        }
        if other.description.is_some() {
            self.description = other.description;
        }
        if other.placeholder.is_some() {
            self.placeholder = other.placeholder;
        }
    }
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backward_compatibility() {
        // Missing max_file_count in the toml shouldn't prevent parsing
        let toml_str = r#"
        [core]
        max_file_size = 1024
        "#;

        let config: Config = toml::from_str(toml_str).expect("Failed to parse old config format");

        assert_eq!(config.core.max_file_size, Some(1024));
        assert_eq!(config.core.max_file_count, None);
    }

    #[test]
    fn merge_copies_non_default_max_findings() {
        let mut base = Config::default();
        let mut other = Config::default();
        other.output.max_findings = Some(25);

        base.merge(other);

        assert_eq!(base.output.max_findings, Some(25));
    }

    #[test]
    fn merge_copies_explicit_default_max_findings() {
        let mut base = Config::default();
        base.output.max_findings = Some(25);
        let other: Config = toml::from_str(
            r#"
[output]
max_findings = 1000
"#,
        )
        .unwrap();

        base.merge(other);

        assert_eq!(base.output.max_findings, Some(1000));
    }

    #[test]
    fn merge_does_not_let_default_max_findings_override_existing_layer() {
        let mut base = Config::default();
        base.output.max_findings = Some(25);
        let other = Config::default();

        base.merge(other);

        assert_eq!(base.output.max_findings, Some(25));
    }

    #[test]
    fn serialize_omits_implicit_max_findings() {
        let config: Config = toml::from_str(
            r#"
[output]
show_snippets = true
"#,
        )
        .unwrap();

        let saved = toml::to_string(&config).unwrap();
        let reparsed: Config = toml::from_str(&saved).unwrap();

        assert!(!saved.contains("max_findings"));
        assert_eq!(reparsed.output.max_findings, Some(1000));
        assert!(!reparsed.output.max_findings_is_set);
    }

    #[test]
    fn serialize_preserves_explicit_default_max_findings() {
        let config: Config = toml::from_str(
            r#"
[output]
max_findings = 1000
"#,
        )
        .unwrap();

        let saved = toml::to_string(&config).unwrap();
        let reparsed: Config = toml::from_str(&saved).unwrap();

        assert!(saved.contains("max_findings = 1000"));
        assert_eq!(reparsed.output.max_findings, Some(1000));
        assert!(reparsed.output.max_findings_is_set);
    }

    #[test]
    fn merge_combines_rule_fields_without_erasing_lower_layer_values() {
        let mut base = Config::default();
        base.rules.insert(
            "pii.fin.credit_card.keyword".to_string(),
            RuleConfig {
                enabled: false,
                enabled_is_set: true,
                base_score: Some(85),
                ..RuleConfig::default()
            },
        );
        let other: Config = toml::from_str(
            r#"
[rules."pii.fin.credit_card.keyword"]
description = "repo-specific copy"
"#,
        )
        .unwrap();

        base.merge(other);
        let rule = base.rules.get("pii.fin.credit_card.keyword").unwrap();

        assert!(!rule.enabled);
        assert_eq!(rule.base_score, Some(85));
        assert_eq!(rule.description.as_deref(), Some("repo-specific copy"));
    }

    #[test]
    fn merge_applies_explicit_rule_enabled_override() {
        let mut base = Config::default();
        base.rules.insert(
            "pii.fin.credit_card.keyword".to_string(),
            RuleConfig {
                base_score: Some(85),
                ..RuleConfig::default()
            },
        );
        let other: Config = toml::from_str(
            r#"
[rules."pii.fin.credit_card.keyword"]
enabled = false
"#,
        )
        .unwrap();

        base.merge(other);

        assert!(!base.rules["pii.fin.credit_card.keyword"].enabled);
    }

    #[test]
    fn merge_treats_programmatic_enabled_false_as_explicit_disable() {
        let mut base = Config::default();
        base.rules.insert(
            "pii.fin.credit_card.keyword".to_string(),
            RuleConfig {
                base_score: Some(85),
                ..RuleConfig::default()
            },
        );
        let mut other = Config::default();
        other.rules.insert(
            "pii.fin.credit_card.keyword".to_string(),
            RuleConfig {
                enabled: false,
                ..RuleConfig::default()
            },
        );

        base.merge(other);

        assert!(!base.rules["pii.fin.credit_card.keyword"].enabled);
    }

    #[test]
    fn merge_treats_programmatic_enabled_true_as_explicit_enable() {
        let mut base = Config::default();
        base.rules.insert(
            "pii.fin.credit_card.keyword".to_string(),
            RuleConfig {
                enabled: false,
                enabled_is_set: true,
                base_score: Some(85),
                ..RuleConfig::default()
            },
        );
        let mut other = Config::default();
        other.rules.insert(
            "pii.fin.credit_card.keyword".to_string(),
            RuleConfig {
                enabled: true,
                ..RuleConfig::default()
            },
        );

        base.merge(other);

        assert!(base.rules["pii.fin.credit_card.keyword"].enabled);
    }

    #[test]
    fn serialize_omits_implicit_rule_enabled() {
        let config: Config = toml::from_str(
            r#"
[rules."pii.fin.credit_card.keyword"]
base_score = 85
"#,
        )
        .unwrap();

        let saved = toml::to_string(&config).unwrap();
        let reparsed: Config = toml::from_str(&saved).unwrap();
        let rule = reparsed.rules.get("pii.fin.credit_card.keyword").unwrap();

        assert!(!saved.contains("enabled"));
        assert!(rule.enabled);
        assert!(!rule.enabled_is_set);
    }

    #[test]
    fn serialize_preserves_explicit_rule_enabled_true() {
        let config: Config = toml::from_str(
            r#"
[rules."pii.fin.credit_card.keyword"]
enabled = true
base_score = 85
"#,
        )
        .unwrap();

        let saved = toml::to_string(&config).unwrap();
        let reparsed: Config = toml::from_str(&saved).unwrap();
        let rule = reparsed.rules.get("pii.fin.credit_card.keyword").unwrap();

        assert!(saved.contains("enabled = true"));
        assert!(rule.enabled);
        assert!(rule.enabled_is_set);
    }

    #[test]
    fn merge_pattern_replacement_reenables_rule_by_default() {
        let mut base = Config::default();
        base.rules.insert(
            "custom.replacement".to_string(),
            RuleConfig {
                enabled: false,
                enabled_is_set: true,
                validator: Some("luhn".to_string()),
                ..RuleConfig::default()
            },
        );
        let other: Config = toml::from_str(
            r#"
[rules."custom.replacement"]
pattern = "password"
"#,
        )
        .unwrap();

        base.merge(other);
        let rule = base.rules.get("custom.replacement").unwrap();

        assert!(rule.enabled);
        assert_eq!(rule.pattern.as_deref(), Some("password"));
    }

    #[test]
    fn merge_pattern_replacement_clears_omitted_lower_layer_fields() {
        let mut base = Config::default();
        base.rules.insert(
            "custom.replacement".to_string(),
            RuleConfig {
                validator: Some("luhn".to_string()),
                placeholder: Some("<CARD>".to_string()),
                base_score: Some(90),
                context_lines_before: Some(3),
                context_lines_after: Some(1),
                ..RuleConfig::default()
            },
        );
        let other: Config = toml::from_str(
            r#"
[rules."custom.replacement"]
pattern = "password"
"#,
        )
        .unwrap();

        base.merge(other);
        let rule = base.rules.get("custom.replacement").unwrap();

        assert_eq!(rule.pattern.as_deref(), Some("password"));
        assert!(rule.validator.is_none());
        assert!(rule.placeholder.is_none());
        assert!(rule.base_score.is_none());
        assert!(rule.context_lines_before.is_none());
        assert!(rule.context_lines_after.is_none());
    }
}
