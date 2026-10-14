use crate::config::Config;
use crate::validate::validate_config;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn load_config(path: &Path) -> Result<Config> {
    if !path.exists() {
        // Return default if file doesn't exist
        return Ok(Config::default());
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file at {:?}", path))?;

    let config: Config = toml::from_str(&content).map_err(|error: toml::de::Error| {
        // TOML's Display includes source lines and user-controlled values.
        // Keep only location metadata at the CLI diagnostic boundary.
        if let Some(span) = error.span() {
            let prefix = content.get(..span.start).unwrap_or("");
            let line = prefix.bytes().filter(|&byte| byte == b'\n').count() + 1;
            let column = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;
            anyhow::anyhow!("Failed to parse TOML config file at line {line}, column {column}")
        } else {
            anyhow::anyhow!("Failed to parse TOML config file")
        }
    })?;

    validate_config(&config)?;

    Ok(config)
}

pub fn save_config(config: &Config, path: &Path) -> Result<()> {
    let content =
        toml::to_string_pretty(config).with_context(|| "Failed to serialize config to TOML")?;

    fs::write(path, content)
        .with_context(|| format!("Failed to write config file to {:?}", path))?;

    Ok(())
}
