use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HintCode {
    RetryLater,
    CheckNetwork,
    CheckDns,
    CheckTlsClock,
    ClearCache,
    UpgradeTool,
    ReduceParallelism,
    CheckTokenPermissions,
    // Future proofing
    #[serde(other)]
    Unknown,
}

impl HintCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            HintCode::RetryLater => "retry_later",
            HintCode::CheckNetwork => "check_network",
            HintCode::CheckDns => "check_dns",
            HintCode::CheckTlsClock => "check_tls_clock",
            HintCode::ClearCache => "clear_cache",
            HintCode::UpgradeTool => "upgrade_tool",
            HintCode::ReduceParallelism => "reduce_parallelism",
            HintCode::CheckTokenPermissions => "check_token_permissions",
            HintCode::Unknown => "unknown",
        }
    }
}

impl fmt::Display for HintCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
