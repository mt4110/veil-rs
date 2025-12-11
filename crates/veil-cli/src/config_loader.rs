use anyhow::Result;
use std::path::PathBuf;
use veil_config::{load_config, Config};

#[derive(Debug, Clone)]
pub struct ConfigLayers {
    #[allow(dead_code)]
    pub org: Option<Config>,
    #[allow(dead_code)]
    pub user: Option<Config>,
    #[allow(dead_code)]
    pub repo: Option<Config>,
    pub effective: Config,
}

/// New entry point for loading configuration with layers
pub fn load_config_layers(explicit_path: Option<&PathBuf>) -> Result<ConfigLayers> {
    let org = load_org_config()?;
    let user = load_user_config()?;
    let repo = load_repo_config(explicit_path)?;

    // Merge logic: Org -> User -> Repo
    let effective = merge_configs(org.as_ref(), user.as_ref(), repo.as_ref());

    Ok(ConfigLayers {
        org,
        user,
        repo,
        effective,
    })
}

/// wrapper for backward compatibility
pub fn load_effective_config(config_path: Option<&PathBuf>) -> Result<Config> {
    Ok(load_config_layers(config_path)?.effective)
}

fn merge_configs(org: Option<&Config>, user: Option<&Config>, repo: Option<&Config>) -> Config {
    let mut final_config = Config::default();

    if let Some(org_cfg) = org {
        final_config.merge(org_cfg.clone());
    }

    if let Some(user_cfg) = user {
        final_config.merge(user_cfg.clone());
    }

    if let Some(repo_cfg) = repo {
        final_config.merge(repo_cfg.clone());
    }

    final_config
}

fn load_org_config() -> Result<Option<Config>> {
    // Legacy support for VEIL_ORG_RULES
    if let Ok(org_path) = std::env::var("VEIL_ORG_RULES") {
        let path = PathBuf::from(&org_path);
        if path.exists() {
            match load_config(&path) {
                Ok(c) => return Ok(Some(c)),
                Err(e) => eprintln!("Warning: Failed to load Org config at {:?}: {}", path, e),
            }
        } else {
            eprintln!(
                "Warning: VEIL_ORG_RULES set to {:?} but file not found.",
                path
            );
        }
    }

    // Future: Load VEIL_ORG_CONFIG or /etc/veil/org.toml
    Ok(None)
}

fn load_user_config() -> Result<Option<Config>> {
    // Future: Load $XDG_CONFIG_HOME/veil/veil.toml
    Ok(None)
}

fn load_repo_config(explicit_path: Option<&PathBuf>) -> Result<Option<Config>> {
    let config_file = explicit_path
        .cloned()
        .unwrap_or_else(|| PathBuf::from("veil.toml"));

    match load_config(&config_file) {
        Ok(c) => Ok(Some(c)),
        Err(e) => {
            // Fail if explicit path was given and missing
            if explicit_path.is_some() && !config_file.exists() {
                anyhow::bail!("Config file not found: {:?}", config_file);
            }
            // If default path logic failed (e.g. file missing), just return None (use defaults)
            // Note: The previous logic returned Config::default() here, but for "repo layer"
            // specifically, returning None is cleaner if no file exists.
            if config_file.exists() {
                // Return error if file exists but failed to parse
                return Err(anyhow::anyhow!(e));
            }
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_repo_only_config() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("veil.toml");

        let config_toml = r#"
[core]
fail_on_score = 99
"#;
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{}", config_toml).unwrap();

        let path_buf = file_path.to_path_buf();
        let layers = load_config_layers(Some(&path_buf)).unwrap();

        assert!(layers.repo.is_some());
        assert_eq!(layers.effective.core.fail_on_score, Some(99));
    }

    // Since VEIL_ORG_RULES reads env var which might be shared, we must be careful.
    // Rust tests run in parallel. Setting env var might affect others.
    // For now, we skip heavy env var manipulation test here or assume serial execution if needed.
}
