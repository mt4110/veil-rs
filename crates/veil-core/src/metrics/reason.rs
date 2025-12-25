use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasonCode {
    // Input/Config
    ConfigInvalid,
    ConfigMissingRequired,
    SchemaViolation,

    // File/Cache
    IoReadFailed,
    IoWriteFailed,
    CacheCorrupt,
    CacheVersionMismatch,
    LockTimeout,
    AtomicRenameFailed,

    // Network/API
    Offline,
    DnsFailed,
    TlsFailed,
    Timeout,
    RateLimited,
    Http4xx,
    Http5xx,

    // Parse/Decode
    JsonParseFailed,
    DecodeFailed,

    // Internal/Unexpected
    NotSupported,
    InternalInvariantBroken,
    Unexpected,
}

impl ReasonCode {
    pub const ALL: &'static [ReasonCode] = &[
        ReasonCode::ConfigInvalid,
        ReasonCode::ConfigMissingRequired,
        ReasonCode::SchemaViolation,
        ReasonCode::IoReadFailed,
        ReasonCode::IoWriteFailed,
        ReasonCode::CacheCorrupt,
        ReasonCode::CacheVersionMismatch,
        ReasonCode::LockTimeout,
        ReasonCode::AtomicRenameFailed,
        ReasonCode::Offline,
        ReasonCode::DnsFailed,
        ReasonCode::TlsFailed,
        ReasonCode::Timeout,
        ReasonCode::RateLimited,
        ReasonCode::Http4xx,
        ReasonCode::Http5xx,
        ReasonCode::JsonParseFailed,
        ReasonCode::DecodeFailed,
        ReasonCode::NotSupported,
        ReasonCode::InternalInvariantBroken,
        ReasonCode::Unexpected,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            ReasonCode::ConfigInvalid => "config_invalid",
            ReasonCode::ConfigMissingRequired => "config_missing_required",
            ReasonCode::SchemaViolation => "schema_violation",
            ReasonCode::IoReadFailed => "io_read_failed",
            ReasonCode::IoWriteFailed => "io_write_failed",
            ReasonCode::CacheCorrupt => "cache_corrupt",
            ReasonCode::CacheVersionMismatch => "cache_version_mismatch",
            ReasonCode::LockTimeout => "lock_timeout",
            ReasonCode::AtomicRenameFailed => "atomic_rename_failed",
            ReasonCode::Offline => "offline",
            ReasonCode::DnsFailed => "dns_failed",
            ReasonCode::TlsFailed => "tls_failed",
            ReasonCode::Timeout => "timeout",
            ReasonCode::RateLimited => "rate_limited",
            ReasonCode::Http4xx => "http_4xx",
            ReasonCode::Http5xx => "http_5xx",
            ReasonCode::JsonParseFailed => "json_parse_failed",
            ReasonCode::DecodeFailed => "decode_failed",
            ReasonCode::NotSupported => "not_supported",
            ReasonCode::InternalInvariantBroken => "internal_invariant_broken",
            ReasonCode::Unexpected => "unexpected",
        }
    }
}

impl fmt::Display for ReasonCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricsV1 {
    pub v: u32,
    pub metrics: MetricsBody,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricsBody {
    // Use String keys for human-readable alphabetical order in output
    pub counts_by_reason: BTreeMap<String, u64>,
}

impl MetricsV1 {
    pub fn new() -> Self {
        Self {
            v: 1,
            metrics: MetricsBody {
                counts_by_reason: BTreeMap::new(),
            },
            meta: None,
        }
    }
}
