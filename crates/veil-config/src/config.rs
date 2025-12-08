use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub core: CoreConfig,
    #[serde(default)]
    pub masking: MaskingConfig,
    #[serde(default)]
    pub rules: HashMap<String, RuleConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CoreConfig {
    #[serde(default = "default_include")]
    pub include: Vec<String>,
    #[serde(default)]
    pub ignore: Vec<String>,
    pub max_file_size: Option<u64>,
    pub fail_on_score: Option<u32>,
    pub remote_rules_url: Option<String>,
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
        if let Some(val) = other.core.fail_on_score {
            self.core.fail_on_score = Some(val);
        }
        if let Some(val) = other.core.remote_rules_url {
            self.core.remote_rules_url = Some(val);
        }

        // Merge Rules (Override/Insert)
        for (id, rule) in other.rules {
            self.rules.insert(id, rule);
        }

        // Merge Masking (Override if set? MaskingConfig doesn't have Options yet, it uses defaults)
        // Ideally MaskingConfig should also be Options, but let's leave it for now unless requested.
        // Assuming Project config overrides Org config for placeholder if we implemented it.
    }
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            include: default_include(),
            ignore: Vec::new(),
            max_file_size: None, // Default handled at usage site
            fail_on_score: None,
            remote_rules_url: None,
        }
    }
}

fn default_include() -> Vec<String> {
    vec![".".to_string()]
}
// Remove default_max_file_size as it's no longer used in serde default

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RuleConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub severity: Option<String>,
    pub pattern: Option<String>,
    pub score: Option<u8>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
}

fn default_true() -> bool {
    true
}
