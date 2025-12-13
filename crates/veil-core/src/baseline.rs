use crate::model::{Finding, Severity};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub const BASELINE_SCHEMA_V1: &str = "veil.baseline.v1";

#[derive(Debug, Serialize, Deserialize)]
pub struct BaselineEntry {
    pub fingerprint: String,
    pub rule_id: String,
    pub path: String,
    pub line: usize,
    pub severity: Severity,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BaselineSnapshot {
    pub schema: String,
    pub generated_at: DateTime<Utc>,
    pub tool: String,
    pub entries: Vec<BaselineEntry>,
}

pub fn generate_fingerprint(finding: &Finding) -> String {
    let input = format!(
        "{}|{}|{}|{}",
        finding.rule_id,
        finding.path.to_string_lossy(),
        finding.line_number,
        finding.masked_snippet
    );

    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let digest = hasher.finalize();
    format!("sha256:{:x}", digest)
}

pub fn from_findings(findings: &[Finding], tool_version: &str) -> BaselineSnapshot {
    let entries = findings
        .iter()
        .map(|f| BaselineEntry {
            fingerprint: generate_fingerprint(f),
            rule_id: f.rule_id.clone(),
            path: f.path.to_string_lossy().into_owned(),
            line: f.line_number,
            severity: f.severity.clone(),
        })
        .collect();

    BaselineSnapshot {
        schema: BASELINE_SCHEMA_V1.to_string(),
        generated_at: Utc::now(),
        tool: format!("veil-rs {}", tool_version),
        entries,
    }
}

pub fn load_baseline(path: &Path) -> Result<BaselineSnapshot> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let snapshot: BaselineSnapshot = serde_json::from_reader(reader)?;

    if snapshot.schema != BASELINE_SCHEMA_V1 {
        anyhow::bail!(
            "Unsupported baseline schema: {} (expected {})",
            snapshot.schema,
            BASELINE_SCHEMA_V1
        );
    }
    Ok(snapshot)
}

pub fn save_baseline(path: &Path, snapshot: &BaselineSnapshot) -> Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, snapshot)?;
    Ok(())
}

/// Result of applying a baseline to a set of findings.
#[derive(Debug, Clone)]
pub struct BaselineResult {
    /// Findings that match the baseline (suppressed)
    pub suppressed: Vec<Finding>,
    /// Findings that do NOT match the baseline (new)
    pub new: Vec<Finding>,
}

impl BaselineSnapshot {
    /// Returns a Set of fingerprints for efficient lookup
    pub fn fingerprint_set(&self) -> std::collections::HashSet<String> {
        self.entries.iter().map(|e| e.fingerprint.clone()).collect()
    }
}

/// Partitions findings into new and suppressed based on the baseline.
pub fn apply_baseline(
    findings: Vec<Finding>,
    baseline: Option<&BaselineSnapshot>,
) -> BaselineResult {
    if let Some(snapshot) = baseline {
        let known_fingerprints = snapshot.fingerprint_set();
        let (suppressed, new): (Vec<Finding>, Vec<Finding>) = findings
            .into_iter()
            .partition(|f| known_fingerprints.contains(&generate_fingerprint(f)));

        BaselineResult { suppressed, new }
    } else {
        // No baseline provided, all findings are considered "new" (or rather, just findings)
        // In the context of "New vs Legacy", everything is technically "New" to the report if no baseline exists.
        BaselineResult {
            suppressed: Vec::new(),
            new: findings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Finding, Severity};
    use crate::rules::grade::Grade;

    fn create_test_finding(path: &str, line: usize, secret: &str) -> Finding {
        Finding {
            path: path.into(),
            line_number: line,
            line_content: format!("{} = {}", "key", secret),
            rule_id: "test.rule".into(),
            matched_content: secret.into(),
            masked_snippet: "key = <REDACTED>".to_string(),
            severity: Severity::High,
            score: 80,
            grade: Grade::High,
            context_before: vec![],
            context_after: vec![],
            commit_sha: None,
            author: None,
            date: None,
        }
    }

    #[test]
    fn test_apply_baseline_partitioning() {
        let f1 = create_test_finding("src/main.rs", 10, "secret1");
        let f2 = create_test_finding("src/lib.rs", 20, "secret2");
        let _f3 = create_test_finding("src/main.rs", 10, "secret1"); // Same as f1

        let findings = vec![f1.clone(), f2.clone()];

        // Create baseline with ONLY f1
        let baseline = from_findings(std::slice::from_ref(&f1), "0.0.0");

        let result = apply_baseline(findings, Some(&baseline));

        assert_eq!(result.suppressed.len(), 1, "Should suppress f1");
        assert_eq!(result.new.len(), 1, "Should mark f2 as new");

        // Check contents
        assert_eq!(result.suppressed[0].matched_content, "secret1");
        assert_eq!(result.new[0].matched_content, "secret2");
    }

    #[test]
    fn test_apply_baseline_none() {
        let f1 = create_test_finding("src/main.rs", 10, "secret1");
        let findings = vec![f1];

        let result = apply_baseline(findings, None);

        assert_eq!(result.new.len(), 1);
        assert!(result.suppressed.is_empty());
    }

    #[test]
    fn fingerprint_is_stable_for_same_input() {
        let f = Finding {
            path: "src/config.py".into(),
            line_number: 42,
            line_content: "aws_key = AKIA123...".into(),
            rule_id: "creds.aws.access_key".into(),
            matched_content: "AKIA123...".into(),
            masked_snippet: "aws_key = <REDACTED>".into(),
            severity: Severity::High,
            score: 80,
            grade: Grade::High,
            context_before: vec![],
            context_after: vec![],
            commit_sha: None,
            author: None,
            date: None,
        };

        let fp1 = generate_fingerprint(&f);
        let fp2 = generate_fingerprint(&f);

        assert_eq!(fp1, fp2);
    }

    #[test]
    fn snapshot_roundtrip_json() {
        let f = Finding {
            path: "src/config.py".into(),
            line_number: 42,
            line_content: "aws_key = AKIA123...".into(),
            rule_id: "creds.aws.access_key".into(),
            matched_content: "AKIA123...".into(),
            masked_snippet: "aws_key = <REDACTED>".into(),
            severity: Severity::High,
            score: 80,
            grade: Grade::High,
            context_before: vec![],
            context_after: vec![],
            commit_sha: None,
            author: None,
            date: None,
        };

        let snapshot = from_findings(&[f], "0.9.1-test");

        let buf = serde_json::to_vec(&snapshot).unwrap();
        let decoded: BaselineSnapshot = serde_json::from_slice(&buf).unwrap();

        assert_eq!(decoded.schema, BASELINE_SCHEMA_V1);
        assert_eq!(decoded.entries.len(), 1);
        assert_eq!(decoded.tool, "veil-rs 0.9.1-test");
    }

    #[test]
    fn test_apply_baseline_empty_findings() {
        // Baseline exists, but we found nothing in scan
        let f1 = create_test_finding("src/main.rs", 10, "secret1");
        let baseline = from_findings(&[f1], "0.0.0");

        let findings = vec![];
        let result = apply_baseline(findings, Some(&baseline));

        assert!(result.new.is_empty());
        assert!(result.suppressed.is_empty());
    }

    #[test]
    fn test_apply_baseline_empty_snapshot() {
        // Did scan, found secrets, but baseline is empty (fresh init?)
        let f1 = create_test_finding("src/main.rs", 10, "secret1");
        let findings = vec![f1.clone()];

        let baseline = from_findings(&[], "0.0.0"); // Empty baseline
        let result = apply_baseline(findings, Some(&baseline));

        assert_eq!(result.new.len(), 1);
        assert!(result.suppressed.is_empty());
    }

    #[test]
    fn test_apply_baseline_duplicates() {
        // If we have duplicate findings (e.g. same secret reported twice or same line hit by multiple rules?
        // Logic says fingerprint depends on rule_id, path, line, snippet. So diff rule = diff fingerprint).
        // Let's testing identical findings appearing twice in input (maybe concurrency bug or just file duplication?)

        // Use separate findings to avoid borrow checker issues with clones if any
        let f1 = create_test_finding("src/main.rs", 10, "secret1");
        let f1_copy = create_test_finding("src/main.rs", 10, "secret1");

        let findings = vec![f1.clone(), f1_copy];

        // Baseline has f1
        let baseline = from_findings(&[f1], "0.0.0");

        let result = apply_baseline(findings, Some(&baseline));

        // Both should be suppressed because they match the known fingerprint set
        assert_eq!(result.suppressed.len(), 2);
        assert!(result.new.is_empty());
    }
}
