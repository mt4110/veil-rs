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

    let config: Config =
        toml::from_str(&content).with_context(|| "Failed to parse TOML config file")?;

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
