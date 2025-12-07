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
    #[serde(default = "default_max_file_size")]
    pub max_file_size: u64,
    #[serde(default)]
    pub fail_on_score: u32,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            include: default_include(),
            ignore: Vec::new(),
            max_file_size: default_max_file_size(),
            fail_on_score: 0,
        }
    }
}

fn default_max_file_size() -> u64 {
    1_000_000 // 1MB
}

fn default_include() -> Vec<String> {
    vec![".".to_string()]
}

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
