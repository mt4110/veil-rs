use crate::formatters::{DisplayFinding, Formatter, Summary};
use anyhow::Result;

pub struct MarkdownFormatter;

impl Formatter for MarkdownFormatter {
    fn print(&self, findings: &[DisplayFinding], summary: &Summary) -> Result<()> {
        println!("# Veil Security Report");
        println!("\n## Summary");
        println!("**Total Files**: {}", summary.total_files);
        println!(
            "**Findings**: {} total, {} new",
            summary.total_findings, summary.new_findings
        );
        println!("- **Duration**: {}ms", summary.duration_ms);

        if findings.is_empty() {
            println!("\n✅ No secrets detected.");
            return Ok(());
        }

        println!("\n## Findings");
        println!("| Severity | Score | Rule ID | File | Line | Match |");
        println!("|---|---|---|---|---|---|");

        for finding in findings {
            let inner = &finding.inner;
            let match_content = if !inner.masked_snippet.is_empty() {
                &inner.masked_snippet
            } else {
                &inner.matched_content
            };
            // Escape pipe chars in content to prevent breaking md table
            let clean_match = match_content.replace("|", "\\|");

            println!(
                "| {:?} | {} | {} | {} | {} | `{}` |",
                inner.severity,
                inner.score,
                inner.rule_id,
                inner.path.display(),
                inner.line_number,
                clean_match
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatters::{DisplayFinding, FindingStatus};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use veil_core::model::{Finding, Severity};

    #[test]
    fn test_markdown_output() {
        let formatter = MarkdownFormatter;
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
                grade: veil_core::rules::grade::Grade::High,
                context_before: vec![],
                context_after: vec![],
                commit_sha: None,
                author: None,
                date: None,
            },
            status: FindingStatus::New,
        }];
        let summary = Summary {
            total_files: 5,
            scanned_files: 5,
            skipped_files: 2,
            total_findings: 5,
            new_findings: 0,
            baseline_suppressed: 0,
            limit_reached: false,
            file_limit_reached: false,
            duration_ms: 1234,
            baseline_path: None,
            severity_counts: HashMap::new(),
            builtin_skips: Vec::new(),
        };

        let result = formatter.print(&findings, &summary);
        assert!(result.is_ok());
    }
}
