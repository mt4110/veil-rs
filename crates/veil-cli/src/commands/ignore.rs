use anyhow::Result;
use std::path::PathBuf;
use veil_config::{load_config, loader::save_config};

pub fn ignore(path: &str, config_path: Option<&PathBuf>) -> Result<()> {
    let config_file = config_path
        .cloned()
        .unwrap_or_else(|| PathBuf::from("veil.toml"));

    let mut config = load_config(&config_file)?;

    if !config.core.ignore.contains(&path.to_string()) {
        config.core.ignore.push(path.to_string());
        save_config(&config, &config_file)?;
        println!("Added '{}' to ignore list in {:?}", path, config_file);
    } else {
        println!("'{}' is already in ignore list.", path);
    }

    Ok(())
}
