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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Finding, Severity};
    use crate::rules::grade::Grade;

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
}
