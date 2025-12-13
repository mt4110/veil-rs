use anyhow::Result;
use std::path::{Path, PathBuf};
use veil_config::{load_config, Config};

#[derive(Debug, Clone)]
pub struct ConfigLayers {
    pub org: Option<Config>,
    pub user: Option<Config>,
    pub repo: Option<Config>,
    pub effective: Config,
}

/// New entry point for loading configuration with layers
pub fn load_config_layers(explicit_path: Option<&PathBuf>) -> Result<ConfigLayers> {
    let org = load_org_config()?;
    let user = load_user_config()?;
    let repo = load_repo_config(explicit_path)?;

    // Merge logic: User -> Org -> Repo (later overrides earlier)
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

    // Layer 1: User Config (Base Preferences)
    if let Some(user_cfg) = user {
        final_config.merge(user_cfg.clone());
    }

    // Layer 2: Org Config (Policy Defaults)
    // Org overrides User
    if let Some(org_cfg) = org {
        final_config.merge(org_cfg.clone());
    }

    // Layer 3: Repo Config (Project Specific)
    // Repo overrides Org Policy (for now, until Hard Policy is implemented)
    if let Some(repo_cfg) = repo {
        final_config.merge(repo_cfg.clone());
    }

    final_config
}

fn load_org_config() -> Result<Option<Config>> {
    // 1. Explicit: VEIL_ORG_CONFIG (strict)
    if let Ok(path_str) = std::env::var("VEIL_ORG_CONFIG") {
        let path = PathBuf::from(&path_str);
        if !path.exists() {
            anyhow::bail!("VEIL_ORG_CONFIG set but file not found: {:?}", path);
        }
        let cfg = load_config(&path)
            .map_err(|e| anyhow::anyhow!("Failed to load VEIL_ORG_CONFIG {:?}: {}", path, e))?;
        return Ok(Some(cfg));
    }

    // 2. XDG/HOME: org.toml (soft)
    if let Some(path) = resolve_xdg_path("org.toml") {
        if let Some(cfg) = try_load_soft(&path) {
            return Ok(Some(cfg));
        }
    }

    // 3. /etc/veil/org.toml (soft)
    let etc_path = PathBuf::from("/etc/veil/org.toml");
    if let Some(cfg) = try_load_soft(&etc_path) {
        return Ok(Some(cfg));
    }

    // 4. Legacy: VEIL_ORG_RULES (soft fallback)
    if let Ok(path_str) = std::env::var("VEIL_ORG_RULES") {
        let path = PathBuf::from(&path_str);
        if let Some(cfg) = try_load_soft(&path) {
            return Ok(Some(cfg));
        } else {
            eprintln!(
                "Warning: VEIL_ORG_RULES set to {:?} but file not usable.",
                path
            );
        }
    }

    Ok(None)
}

fn load_user_config() -> Result<Option<Config>> {
    // 1. Explicit: VEIL_USER_CONFIG (strict)
    if let Ok(path_str) = std::env::var("VEIL_USER_CONFIG") {
        let path = PathBuf::from(&path_str);
        if !path.exists() {
            anyhow::bail!("VEIL_USER_CONFIG set but file not found: {:?}", path);
        }
        let cfg = load_config(&path)
            .map_err(|e| anyhow::anyhow!("Failed to load VEIL_USER_CONFIG {:?}: {}", path, e))?;
        return Ok(Some(cfg));
    }

    // 2. XDG/HOME: veil.toml (soft)
    // Note: We use "veil.toml" as the standard user config name, consistent with repo config.
    if let Some(path) = resolve_xdg_path("veil.toml") {
        if let Some(cfg) = try_load_soft(&path) {
            return Ok(Some(cfg));
        }
    }

    Ok(None)
}

fn resolve_xdg_path(file_name: &str) -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("veil").join(file_name));
    }

    if let Ok(home) = std::env::var("HOME") {
        return Some(
            PathBuf::from(home)
                .join(".config")
                .join("veil")
                .join(file_name),
        );
    }

    None
}

fn try_load_soft(path: &Path) -> Option<Config> {
    if !path.exists() {
        return None;
    }
    match load_config(path) {
        Ok(cfg) => Some(cfg),
        Err(e) => {
            eprintln!("Warning: Failed to load config at {:?}: {}", path, e);
            None
        }
    }
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
