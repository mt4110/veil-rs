use colored::*;
use veil_core::{Finding, Grade, Severity};

pub fn print_finding(finding: &Finding) {
    let severity_colored = match finding.severity {
        Severity::Low => "sev:LOW".yellow(),
        Severity::Medium => "sev:MED".truecolor(255, 165, 0), // Orange
        Severity::High => "sev:HIGH".red(),
        Severity::Critical => "sev:CRT".red().bold().on_black(),
    };

    let grade_colored = match finding.grade {
        Grade::Safe => "SAFE".green(),
        Grade::Low => "LOW".yellow(),
        Grade::Medium => "MEDIUM".truecolor(255, 165, 0),
        Grade::High => "HIGH".red(),
        Grade::Critical => "CRITICAL".red().bold().on_black(),
    };

    println!(
        "[{}] [{}] (Score: {}) [{}] {}:{}  {}",
        grade_colored,
        severity_colored,
        finding.score,
        finding.rule_id.cyan(),
        finding.path.display(),
        finding.line_number,
        finding.masked_snippet
    );
}
