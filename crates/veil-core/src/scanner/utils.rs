use crate::model::{Finding, Rule, Severity};
use crate::rules::grade::Grade;
use crate::scanner::{scan_content, RULE_ID_BINARY_FILE, RULE_ID_MAX_FILE_SIZE};
use std::path::Path;
use veil_config::Config;

pub fn scan_data(path: &Path, data: &[u8], rules: &[Rule], config: &Config) -> Vec<Finding> {
    // 1. Size Check
    let size = data.len() as u64;
    let max_size = config.core.max_file_size.unwrap_or(1_000_000);
    if size > max_size {
        return vec![create_skipped_finding(
            path,
            RULE_ID_MAX_FILE_SIZE,
            format!(
                "File size ({} bytes) exceeds limit ({} bytes)",
                size, max_size
            ),
            Severity::High,
        )];
    }

    // 2. Binary Check (Check first 8KB like git, or just 1KB)
    // 1KB is usually enough for quick check.
    let header_len = std::cmp::min(data.len(), 8192);
    if data[..header_len].contains(&0) {
        return vec![create_skipped_finding(
            path,
            RULE_ID_BINARY_FILE,
            "Binary file detected (skipped)".to_string(),
            Severity::Medium,
        )];
    }

    // 3. UTF-8 Validation & Scan
    match std::str::from_utf8(data) {
        Ok(content) => scan_content(content, path, rules, config),
        Err(_) => vec![create_skipped_finding(
            path,
            RULE_ID_BINARY_FILE,
            "Binary file detected (invalid UTF-8)".to_string(),
            Severity::Medium,
        )],
    }
}

pub fn create_skipped_finding(
    path: &Path,
    rule_id: &str,
    msg: String,
    severity: Severity,
) -> Finding {
    Finding {
        path: path.to_path_buf(),
        line_number: 0,
        line_content: msg,
        rule_id: rule_id.to_string(),
        matched_content: "".to_string(),
        masked_snippet: "".to_string(),
        severity,
        score: if rule_id == RULE_ID_MAX_FILE_SIZE {
            100
        } else {
            0
        },
        grade: if rule_id == RULE_ID_MAX_FILE_SIZE {
            Grade::Critical
        } else {
            Grade::Safe
        },
        context_before: vec![],
        context_after: vec![],
        commit_sha: None,
        author: None,
        date: None,
    }
}
