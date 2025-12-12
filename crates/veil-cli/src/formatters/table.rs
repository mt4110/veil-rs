use crate::formatters::{DisplayFinding, Formatter, Summary};
use anyhow::Result;
use prettytable::{color, format, Attr, Cell, Row, Table};
use veil_core::model::Severity;

pub struct TableFormatter;

impl Formatter for TableFormatter {
    fn print(&self, findings: &[DisplayFinding], summary: &Summary) -> Result<()> {
        if findings.is_empty() {
            println!("No secrets found.");
            println!("\nScan Summary:");
            println!("  Total Scanned: {}", summary.scanned_files);
            println!("  Time: {}ms", summary.duration_ms);
            return Ok(());
        }

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

        table.set_titles(Row::new(vec![
            Cell::new("Severity").with_style(Attr::Bold),
            Cell::new("Score").with_style(Attr::Bold),
            Cell::new("Rule ID").with_style(Attr::Bold),
            Cell::new("File").with_style(Attr::Bold), // Simplification
            Cell::new("Line").with_style(Attr::Bold),
            Cell::new("Match").with_style(Attr::Bold),
        ]));

        for finding in findings {
            let inner = &finding.inner;
            table.add_row(Row::new(vec![
                Cell::new(&format!("{:?}", inner.severity))
                    .with_style(Attr::Bold)
                    .with_style(Attr::ForegroundColor(match inner.severity {
                        Severity::Critical | Severity::High => color::RED,
                        Severity::Medium => color::YELLOW,
                        Severity::Low => color::BLUE,
                    })),
                Cell::new(&inner.score.to_string()),
                Cell::new(&inner.rule_id),
                Cell::new(&inner.path.display().to_string()),
                Cell::new(&inner.line_number.to_string()),
                Cell::new(&inner.matched_content),
            ]));
        }

        table.printstd();

        println!("\nScan Summary:");
        println!("  Total Scanned: {}", summary.scanned_files);
        println!("  Total Findings: {}", summary.total_findings);
        println!("  Time: {}ms", summary.duration_ms);

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
    fn test_table_output() {
        let formatter = TableFormatter;
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
            duration_ms: 1234,
            baseline_path: None,
            severity_counts: HashMap::new(),
        };

        // Table print output captures invalid compilation if we don't return Result
        let result = formatter.print(&findings, &summary);
        assert!(result.is_ok());
    }
}
