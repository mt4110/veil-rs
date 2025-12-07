use crate::config::Config;
use anyhow::{bail, Result};

pub fn validate_config(config: &Config) -> Result<()> {
    // core.include can be empty if user provides paths via CLI

    for (id, rule_config) in &config.rules {
        if let Some(severity) = &rule_config.severity {
            match severity.to_lowercase().as_str() {
                "low" | "medium" | "high" | "critical" => {}
                _ => bail!(
                    "Invalid severity for rule '{}': {}. Must be one of: low, medium, high, critical",
                    id,
                    severity
                ),
            }
        }
    }

    Ok(())
}
