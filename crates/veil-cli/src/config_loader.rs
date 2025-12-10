use anyhow::Result;
use std::path::PathBuf;
use veil_config::{load_config, Config};

pub fn load_effective_config(config_path: Option<&PathBuf>) -> Result<Config> {
    let mut final_config = Config::default();

    // 1. Org Config (VEIL_ORG_RULES)
    if let Ok(org_path) = std::env::var("VEIL_ORG_RULES") {
        let path = PathBuf::from(&org_path);
        if path.exists() {
            match load_config(&path) {
                Ok(org_config) => final_config.merge(org_config),
                Err(e) => eprintln!("Warning: Failed to load Org config at {:?}: {}", path, e),
            }
        } else {
            eprintln!(
                "Warning: VEIL_ORG_RULES set to {:?} but file not found.",
                path
            );
        }
    }

    // 2. Project Config
    let config_file = config_path
        .cloned()
        .unwrap_or_else(|| PathBuf::from("veil.toml"));

    // If explicit config path given, fail if missing. If default, fallback to default.
    let project_config = match load_config(&config_file) {
        Ok(c) => c,
        Err(e) => {
            if config_path.is_some() && !config_file.exists() {
                anyhow::bail!("Config file not found: {:?}", config_file);
            }
            if config_file.exists() {
                return Err(e);
            }
            Config::default()
        }
    };

    final_config.merge(project_config);

    Ok(final_config)
}
