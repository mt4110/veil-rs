use crate::model::Finding;

#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    pub findings: Vec<Finding>,
    pub total_files: usize,
    pub scanned_files: usize,
    pub skipped_files: usize,
    pub limit_reached: bool,
    pub file_limit_reached: bool,
    pub max_file_size_reached: bool,
    pub read_error_reached: bool,
    pub builtin_skips: std::collections::HashSet<String>,
}

/// Classification of the legacy scan_file/scan_data sentinel findings.
/// Keep sentinel interpretation in one place until the public API can evolve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileScanStatus {
    Scanned,
    Binary,
    TooLarge,
    ReadError,
}

impl FileScanStatus {
    pub fn from_findings(findings: &[Finding]) -> Self {
        match findings.first().map(|finding| finding.rule_id.as_str()) {
            Some(super::RULE_ID_BINARY_FILE) => Self::Binary,
            Some(super::RULE_ID_MAX_FILE_SIZE) => Self::TooLarge,
            Some(super::RULE_ID_READ_ERROR) => Self::ReadError,
            _ => Self::Scanned,
        }
    }
}

pub fn incomplete_scan_reason(
    finding_limit: bool,
    file_limit: bool,
    size_limit: bool,
    read_error: bool,
) -> Option<&'static str> {
    if file_limit {
        Some("file count limit reached (core.max_file_count)")
    } else if size_limit {
        Some("file size limit reached (core.max_file_size)")
    } else if read_error {
        Some("file read or UTF-8 decoding error")
    } else if finding_limit {
        Some("finding limit reached (--limit or output.max_findings)")
    } else {
        None
    }
}
