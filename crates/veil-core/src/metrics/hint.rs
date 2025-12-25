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

    // Phase 12: Action Blueprint
    pub fn action_id(&self) -> &'static str {
        match self {
            HintCode::RetryLater => "A-WAIT-001",
            HintCode::CheckNetwork => "A-NET-001",
            HintCode::CheckDns => "A-NET-002",
            HintCode::CheckTlsClock => "A-SEC-001",
            HintCode::ClearCache => "A-DISK-001",
            HintCode::UpgradeTool => "A-UPG-001",
            HintCode::ReduceParallelism => "A-PERF-001",
            HintCode::CheckTokenPermissions => "A-IAM-001",
            HintCode::Unknown => "Z-UNMAPPED", // Fallback for stability
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            HintCode::RetryLater => "Retry operation later (Transient)",
            HintCode::CheckNetwork => "Verify network connectivity",
            HintCode::CheckDns => "Check DNS resolution",
            HintCode::CheckTlsClock => "Verify system clock & TLS",
            HintCode::ClearCache => "Clear local cache",
            HintCode::UpgradeTool => "Upgrade veil-rs to latest",
            HintCode::ReduceParallelism => "Reduce concurrency/parallelism",
            HintCode::CheckTokenPermissions => "Verify token permissions",
            HintCode::Unknown => "Unmapped hint event",
        }
    }

    pub fn effort(&self) -> &'static str {
        match self {
            HintCode::UpgradeTool | HintCode::ClearCache => "S",
            HintCode::CheckTokenPermissions | HintCode::RetryLater => "M",
            _ => "L",
        }
    }

    pub fn playbook_ref(&self) -> Option<&'static str> {
        // Future: specific docs links
        None
    }

    pub fn suggested_paths(&self) -> &'static [&'static str] {
        match self {
            HintCode::CheckNetwork | HintCode::CheckDns => &["/etc/hosts", "/etc/resolv.conf"],
            _ => &[],
        }
    }
}

impl fmt::Display for HintCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
