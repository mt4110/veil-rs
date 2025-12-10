use crate::formatters::{Formatter, Summary};
use anyhow::Result;
use prettytable::{format, Cell, Row, Table};
use veil_core::model::Finding;

pub struct TableFormatter;

impl Formatter for TableFormatter {
    fn print(&self, findings: &[Finding], summary: &Summary) -> Result<()> {
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
            Cell::new("Severity").style_spec("b"),
            Cell::new("Score").style_spec("b"),
            Cell::new("Rule ID").style_spec("b"),
            Cell::new("File:Line").style_spec("b"),
            Cell::new("Match").style_spec("b"),
        ]));

        // Clone findings to sort
        let mut sorted_findings = findings.to_vec();
        // Sort: Path ASC, Line ASC
        sorted_findings.sort_by(|a, b| a.path.cmp(&b.path).then(a.line_number.cmp(&b.line_number)));

        for finding in sorted_findings {
            let severity_str = format!("{:?}", finding.severity);
            let sev_style = match finding.score {
                s if s >= 90 => "Fr", // Red foreground
                s if s >= 70 => "Fy", // Yellow foreground
                _ => "",
            };

            let path_str = finding.path.display().to_string();
            let file_loc = if path_str.len() > 40 {
                // Truncate path if too long: ".../some/long/path.rs"
                format!(
                    "...{}:{}",
                    &path_str[path_str.len() - 40..],
                    finding.line_number
                )
            } else {
                format!("{}:{}", path_str, finding.line_number)
            };

            // Truncate match if too long
            let content_to_show = &finding.masked_snippet;
            let match_content = if content_to_show.len() > 50 {
                format!("{}...", &content_to_show[..50])
            } else if !content_to_show.is_empty() {
                content_to_show.clone()
            } else {
                // Fallback if masked_snippet empty
                if finding.line_content.len() > 50 {
                    format!("{}...", &finding.line_content[..50])
                } else {
                    finding.line_content.clone()
                }
            };

            table.add_row(Row::new(vec![
                Cell::new(&severity_str).style_spec(sev_style),
                Cell::new(&finding.score.to_string()),
                Cell::new(&finding.rule_id),
                Cell::new(&file_loc),
                Cell::new(&match_content),
            ]));
        }

        table.printstd();

        println!("\nScan Summary:");
        println!("  Total Scanned: {}", summary.scanned_files);
        println!("  Findings: {}", summary.findings_count);
        println!("  Time: {}ms", summary.duration_ms);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use veil_core::model::{Finding, Severity};

    #[test]
    fn test_table_output() {
        let formatter = TableFormatter;
        let findings = vec![Finding {
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
        }];
        let summary = Summary {
            total_files: 10,
            scanned_files: 8,
            skipped_files: 2,
            findings_count: 5,
            shown_findings: 5,
            limit_reached: false,
            duration_ms: 1234,
            severity_counts: HashMap::new(),
        };

        let result = formatter.print(&findings, &summary);
        assert!(result.is_ok());
    }
}
