use crate::formatters::{Formatter, Summary};
use anyhow::Result;
use veil_core::model::Finding;

pub struct MarkdownFormatter;

impl Formatter for MarkdownFormatter {
    fn print(&self, findings: &[Finding], summary: &Summary) -> Result<()> {
        println!("# Veil Security Report");
        println!("\n## Summary");
        println!("- **Total Scanned Files**: {}", summary.scanned_files);
        println!("- **Total Findings**: {}", summary.findings_count);
        println!("- **Duration**: {}ms", summary.duration_ms);

        if findings.is_empty() {
            println!("\n✅ No secrets detected.");
            return Ok(());
        }

        println!("\n## Findings");
        println!("| Severity | Score | Rule ID | File | Line | Match |");
        println!("|---|---|---|---|---|---|");

        for finding in findings {
            let match_content = if !finding.masked_line.is_empty() {
                &finding.masked_line
            } else {
                &finding.line_content
            };
            // Escape pipe chars in content to prevent breaking md table
            let clean_match = match_content.replace("|", "\\|");

            println!(
                "| {:?} | {} | {} | {} | {} | `{}` |",
                finding.severity,
                finding.score,
                finding.rule_id,
                finding.path.display(),
                finding.line_number,
                clean_match
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use veil_core::model::Severity;

    #[test]
    fn test_markdown_output() {
        let formatter = MarkdownFormatter;
        let findings = vec![Finding {
            path: PathBuf::from("test.txt"),
            line_number: 1,
            line_content: "secret=123".to_string(),
            masked_line: "secret=***".to_string(),
            rule_id: "test_rule".to_string(),
            severity: Severity::High,
            score: 80,
            grade: veil_core::rules::grade::Grade::High,
        }];
        let summary = Summary {
            total_files: 1,
            scanned_files: 1,
            skipped_files: 0,
            findings_count: 1,
            duration_ms: 100,
            severity_counts: HashMap::new(),
        };

        let result = formatter.print(&findings, &summary);
        assert!(result.is_ok());
    }
}
