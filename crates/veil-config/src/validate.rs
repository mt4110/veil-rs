use crate::config::{Config, MaskMode};
use anyhow::{bail, Result};

pub fn validate_config(config: &Config) -> Result<()> {
    // Security: veil.toml cannot set mask_mode = "plain"
    // This forces unsafe output to be an explicit CLI opt-in
    if config.output.mask_mode == Some(MaskMode::Plain) {
        bail!("'plain' mask_mode is not allowed in config file. Use --unsafe or --mask-mode plain CLI flag instead.");
    }

    // Validate bounds
    if let Some(size) = config.core.max_file_size {
        if size == 0 || size > 500 * 1024 * 1024 {
            bail!("Invalid config field 'core.max_file_size': must be between 1 and 524288000 (500MB)");
        }
    }

    if let Some(count) = config.core.max_file_count {
        if count == 0 || count > 1_000_000 {
            bail!("Invalid config field 'core.max_file_count': must be between 1 and 1,000,000");
        }
    }

    if let Some(limit) = config.output.max_findings {
        if limit == 0 || limit > 100_000 {
            bail!("Invalid config field 'output.max_findings': must be between 1 and 100,000");
        }
    }

    for (id, rule) in &config.rules {
        if let Some(pattern) = &rule.pattern {
            if pattern.is_empty() {
                bail!("Rule '{}' has empty pattern", id);
            }
            if pattern.len() > 1024 {
                bail!(
                    "Rule '{}' has a pattern exceeding the maximum length of 1024 characters (current: {}). Consider simplifying or splitting the regex.",
                    id, pattern.len()
                );
            }
            // Validate regex compilation
            if let Err(e) = regex::Regex::new(pattern) {
                bail!("Rule '{}' has invalid regex: {}", id, e);
            }
        }

        if let Some(severity) = &rule.severity {
            match severity.to_lowercase().as_str() {
                "low" | "medium" | "high" | "critical" => {}
                _ => bail!(
                    "Invalid config field 'severity' for rule '{}': {}. Must be one of: low, medium, high, critical",
                    id,
                    severity
                ),
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, MaskMode};

    #[test]
    fn test_fail_fast_plain_mode() {
        let mut config = Config::default();
        config.output.mask_mode = Some(MaskMode::Plain);

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("'plain' mask_mode is not allowed"));
    }

    #[test]
    fn test_valid_modes() {
        let mut config = Config::default();
        config.output.mask_mode = Some(MaskMode::Redact);
        assert!(validate_config(&config).is_ok());

        config.output.mask_mode = Some(MaskMode::Partial);
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_oversized_regex() {
        let mut config = Config::default();
        let long_pattern = "a".repeat(1025);

        let rule = crate::config::RuleConfig {
            enabled: true,
            severity: Some("high".to_string()),
            pattern: Some(long_pattern),
            score: Some(10),
            category: None,
            tags: None,
            base_score: None,
            context_lines_before: None,
            context_lines_after: None,
            description: None,
            placeholder: None,
        };
        config.rules.insert("test_rule".to_string(), rule);

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Consider simplifying or splitting the regex"));
    }

    #[test]
    fn test_invalid_bounds() {
        let mut config = Config::default();

        // Test oversized file size
        config.core.max_file_size = Some(1024 * 1024 * 1024); // 1GB (over limit)
        let result = validate_config(&config);
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be between 1 and 524288000"));

        config.core.max_file_size = None;

        // Test oversized file count
        config.core.max_file_count = Some(2_000_000);
        let result = validate_config(&config);
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be between 1 and 1,000,000"));
    }
}
