use crate::formatters::{Formatter, Summary};

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

        for finding in findings {
            let severity_str = format!("{:?}", finding.severity);
            let sev_style = match finding.score {
                s if s >= 90 => "Fr", // Red foreground
                s if s >= 70 => "Fy", // Yellow foreground
                _ => "",
            };

            let file_loc = format!("{}:{}", finding.path.display(), finding.line_number);

            // Truncate match if too long
            let match_content = if finding.masked_line.len() > 50 {
                format!("{}...", &finding.masked_line[..50])
            } else if !finding.masked_line.is_empty() {
                finding.masked_line.clone()
            } else {
                // Fallback if masked_line empty (shouldn't happen for matches)
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
