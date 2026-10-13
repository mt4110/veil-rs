use crate::formatters::{DisplayFinding, Formatter, Summary};
use anyhow::Result;
use serde::Serialize;
use veil_core::model::Finding;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatters::{DisplayFinding, FindingStatus};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use veil_core::model::Severity;

    #[test]
    fn test_json_output() {
        let formatter = JsonFormatter::new(false);
        let findings = vec![DisplayFinding {
            inner: Finding {
                path: PathBuf::from("test.txt"),
                line_number: 1,
                line_content: "secret=123".to_string(),
                matched_content: "123".to_string(),
                masked_snippet: "secret=***".to_string(),
                rule_id: "test_rule".to_string(),
                severity: Severity::High,
                score: 80,
                grade: veil_core::rules::grade::Grade::Critical,
                span: Default::default(),
                utf16_range: Default::default(),
                context_before: vec![],
                context_after: vec![],
                commit_sha: None,
                author: None,
                date: None,
            },
            status: FindingStatus::New,
        }];

        let summary = Summary {
            total_files: 10,
            scanned_files: 8,
            skipped_files: 2,
            total_findings: 5,
            new_findings: 0,
            baseline_suppressed: 0,
            limit_reached: false,
            file_limit_reached: false,
            max_file_size_reached: false,
            read_error_reached: false,
            duration_ms: 1234,
            baseline_path: None,
            severity_counts: HashMap::new(),
            builtin_skips: Vec::new(),
        };

        let result = formatter.print(&findings, &summary);
        assert!(result.is_ok());
    }
}

pub struct JsonFormatter {
    reveal_raw: bool,
}

impl JsonFormatter {
    pub fn new(reveal_raw: bool) -> Self {
        Self { reveal_raw }
    }

    fn output_finding(&self, finding: &Finding) -> Finding {
        let mut output = finding.clone();
        if !self.reveal_raw {
            // Finding is the internal raw model. Its raw fields must not cross the
            // normal CLI output boundary, including unclassified neighboring lines.
            output.line_content = output.masked_snippet.clone();
            output.matched_content = veil_core::DEFAULT_PLACEHOLDER.to_string();
            for line in output
                .context_before
                .iter_mut()
                .chain(output.context_after.iter_mut())
            {
                *line = veil_core::DEFAULT_PLACEHOLDER.to_string();
            }
        }
        output
    }
}

const SCHEMA_VERSION: &str = "veil-v1";

#[derive(Serialize)]
struct JsonReport<'a> {
    #[serde(rename = "schemaVersion")]
    schema_version: &'a str,
    summary: &'a Summary,
    findings: Vec<Finding>,
}

impl Formatter for JsonFormatter {
    fn print(&self, findings: &[DisplayFinding], summary: &Summary) -> Result<()> {
        let inner_findings = findings
            .iter()
            .map(|f| self.output_finding(&f.inner))
            .collect();

        let report = JsonReport {
            schema_version: SCHEMA_VERSION,
            summary,
            findings: inner_findings,
        };
        let json = serde_json::to_string_pretty(&report)?;
        println!("{}", json);
        Ok(())
    }
}
