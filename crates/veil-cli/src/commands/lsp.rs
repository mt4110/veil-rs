use std::path::PathBuf;

use anyhow::Result;

pub fn run(config_path: Option<&PathBuf>, preset_id: Option<&str>) -> Result<()> {
    let config = crate::config_loader::load_effective_config_with_preset(config_path, preset_id)?;

    if let Some(preset_id) = preset_id {
        eprintln!(
            "Using preset '{}' as the base config layer; user/org/repo config may override it.",
            preset_id
        );
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(veil_lsp::server::run_stdio_with_config(config));
    Ok(())
}
