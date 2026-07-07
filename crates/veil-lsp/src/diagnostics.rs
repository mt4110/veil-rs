use serde_json::json;
use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticSeverity, NumberOrString, Position as LspPosition, Range as LspRange,
};
use veil_core::model::{Finding, Range, Severity};

pub fn findings_to_diagnostics(findings: &[Finding]) -> Vec<Diagnostic> {
    findings.iter().map(finding_to_diagnostic).collect()
}

pub fn finding_to_diagnostic(finding: &Finding) -> Diagnostic {
    Diagnostic {
        range: range_to_lsp(finding.utf16_range),
        severity: Some(severity_to_lsp(&finding.severity)),
        code: Some(NumberOrString::String(finding.rule_id.clone())),
        code_description: None,
        source: Some("veil".to_string()),
        message: format!(
            "Sensitive data detected by {} (grade {}, score {})",
            finding.rule_id, finding.grade, finding.score
        ),
        related_information: None,
        tags: None,
        data: Some(json!({
            "ruleId": finding.rule_id,
            "score": finding.score,
            "grade": finding.grade.to_string(),
            "maskedSnippet": finding.masked_snippet,
            "actions": ["mask", "ignore"],
        })),
    }
}

fn severity_to_lsp(severity: &Severity) -> DiagnosticSeverity {
    match severity {
        Severity::Critical | Severity::High => DiagnosticSeverity::ERROR,
        Severity::Medium => DiagnosticSeverity::WARNING,
        Severity::Low => DiagnosticSeverity::INFORMATION,
    }
}

fn range_to_lsp(range: Range) -> LspRange {
    LspRange {
        start: position_to_lsp(range.start),
        end: position_to_lsp(range.end),
    }
}

fn position_to_lsp(position: veil_core::model::Position) -> LspPosition {
    LspPosition {
        line: position.line,
        character: position.character,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tower_lsp::lsp_types::{Position, Range as LspRange};
    use veil_core::model::{FindingSpan, Position as CorePosition};
    use veil_core::rules::grade::Grade;

    fn finding_with(severity: Severity) -> Finding {
        Finding {
            path: PathBuf::from("fixture.txt"),
            line_number: 3,
            line_content: "token = raw-secret-value".to_string(),
            rule_id: "secret.test".to_string(),
            matched_content: "raw-secret-value".to_string(),
            masked_snippet: "token = <REDACTED>".to_string(),
            severity,
            score: 92,
            grade: Grade::Critical,
            span: FindingSpan {
                byte_start: 8,
                byte_end: 24,
            },
            utf16_range: Range {
                start: CorePosition {
                    line: 2,
                    character: 8,
                },
                end: CorePosition {
                    line: 2,
                    character: 24,
                },
            },
            context_before: Vec::new(),
            context_after: Vec::new(),
            commit_sha: None,
            author: None,
            date: None,
        }
    }

    #[test]
    fn diagnostic_range_uses_utf16_range() {
        let finding = finding_with(Severity::High);
        let diagnostic = finding_to_diagnostic(&finding);

        assert_eq!(
            diagnostic.range,
            LspRange {
                start: Position {
                    line: 2,
                    character: 8
                },
                end: Position {
                    line: 2,
                    character: 24
                }
            }
        );
    }

    #[test]
    fn severity_matches_lsp_contract() {
        assert_eq!(
            severity_to_lsp(&Severity::Critical),
            DiagnosticSeverity::ERROR
        );
        assert_eq!(severity_to_lsp(&Severity::High), DiagnosticSeverity::ERROR);
        assert_eq!(
            severity_to_lsp(&Severity::Medium),
            DiagnosticSeverity::WARNING
        );
        assert_eq!(
            severity_to_lsp(&Severity::Low),
            DiagnosticSeverity::INFORMATION
        );
    }

    #[test]
    fn diagnostic_data_contains_safe_fields_only() {
        let finding = finding_with(Severity::Critical);
        let diagnostic = finding_to_diagnostic(&finding);
        let data = diagnostic.data.expect("diagnostic data");
        let data_text = data.to_string();

        assert_eq!(
            data.get("ruleId").and_then(|value| value.as_str()),
            Some("secret.test")
        );
        assert_eq!(
            data.get("maskedSnippet").and_then(|value| value.as_str()),
            Some("token = <REDACTED>")
        );
        assert_eq!(data.get("score").and_then(|value| value.as_u64()), Some(92));
        assert_eq!(
            data.get("grade").and_then(|value| value.as_str()),
            Some("CRITICAL")
        );
        assert_eq!(
            data.get("actions")
                .and_then(|value| value.as_array())
                .map(Vec::len),
            Some(2)
        );
        assert!(!data_text.contains("raw-secret-value"));
        assert!(!data_text.contains("token = raw-secret-value"));
    }

    #[test]
    fn diagnostics_preserve_finding_order() {
        let findings = vec![finding_with(Severity::Low), finding_with(Severity::High)];
        let diagnostics = findings_to_diagnostics(&findings);

        assert_eq!(diagnostics.len(), 2);
        assert_eq!(
            diagnostics[0].severity,
            Some(DiagnosticSeverity::INFORMATION)
        );
        assert_eq!(diagnostics[1].severity, Some(DiagnosticSeverity::ERROR));
    }
}
