use crate::config::{Config, MaskMode};
use anyhow::{bail, Result};

pub fn validate_config(config: &Config) -> Result<()> {
    // Security: veil.toml cannot set mask_mode = "plain"
    // This forces unsafe output to be an explicit CLI opt-in
    if config.output.mask_mode == Some(MaskMode::Plain) {
        bail!("'plain' mask_mode is not allowed in config file. Use --unsafe or --mask-mode plain CLI flag instead.");
    }

    for (id, rule) in &config.rules {
        if let Some(pattern) = &rule.pattern {
            if pattern.is_empty() {
                bail!("Rule '{}' has empty pattern", id);
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
                    "Invalid severity for rule '{}': {}. Must be one of: low, medium, high, critical",
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
}
