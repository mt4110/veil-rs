use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn run(config_path: Option<&PathBuf>, preset_id: Option<&str>, quiet: bool) -> Result<()> {
    let config = crate::config_loader::load_effective_config_with_preset(config_path, preset_id)?;
    if let Some(preset_id) = preset_id.filter(|_| !quiet) {
        eprintln!(
            "Using preset '{}' as the base config layer; user/org/repo config may override it.",
            preset_id
        );
    }
    // Explicit LSP use starts the standalone server. Normal scans neither spawn
    // this process nor link the server's async/dependency graph.
    let status = std::process::Command::new("veil-lsp")
        .env("VEIL_LSP_CONFIG_JSON", serde_json::to_string(&config)?)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context(
            "Cannot start standalone veil-lsp. Build/install crates/veil-lsp and add it to PATH.",
        )?;
    if !status.success() {
        anyhow::bail!("Standalone veil-lsp exited unsuccessfully: {status}");
    }
    Ok(())
}
