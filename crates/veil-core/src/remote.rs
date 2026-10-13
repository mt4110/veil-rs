//! Compatibility shim for the retired remote rule API. No network I/O is performed.
use crate::model::Rule;

#[derive(thiserror::Error, Debug)]
pub enum RemoteError {
    #[error("Remote rules are disabled. Use a local rule pack through core.rules_dir.")]
    Disabled,
}

/// Always rejects remote loading. Kept so callers receive an explicit migration error.
pub fn fetch_remote_rules(_url: &str, _timeout_secs: u64) -> Result<Vec<Rule>, RemoteError> {
    Err(RemoteError::Disabled)
}
