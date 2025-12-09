use crate::formatters::{Formatter, Summary};
use anyhow::Result;
use serde::Serialize;
use veil_core::model::Finding;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use veil_core::model::Severity;

    #[test]
    fn test_json_output() {
        let formatter = JsonFormatter;
        let findings = vec![Finding {
            path: PathBuf::from("test.txt"),
            line_number: 1,
            line_content: "secret=123".to_string(),
            matched_content: "123".to_string(),
            masked_snippet: "secret=***".to_string(),
            rule_id: "test_rule".to_string(),
            severity: Severity::High,
            score: 80,
            grade: veil_core::rules::grade::Grade::Critical,
            context_before: vec![],
            context_after: vec![],
        }];
        let summary = Summary {
            total_files: 10,
            scanned_files: 8,
            skipped_files: 2,
            findings_count: 5,
            shown_findings: 5,
            truncated: false,
            duration_ms: 1234,
            severity_counts: HashMap::new(),
        };

        // Capture stdout?
        // Testing println! is hard in unit tests directly without redirecting.
        // For now, we just ensure it runs without panic.
        // Or we can modify print to take a writer, but Formatter trait uses print().

        let result = formatter.print(&findings, &summary);
        assert!(result.is_ok());
    }
}

pub struct JsonFormatter;

#[derive(Serialize)]
struct JsonReport<'a> {
    summary: &'a Summary,
    findings: &'a [Finding],
}

impl Formatter for JsonFormatter {
    fn print(&self, findings: &[Finding], summary: &Summary) -> Result<()> {
        let report = JsonReport { summary, findings };
        let json = serde_json::to_string_pretty(&report)?;
        println!("{}", json);
        Ok(())
    }
}
