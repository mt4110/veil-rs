use anyhow::{bail, Context, Result};
use std::io::{self, Write};
use veil_core::Finding;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractiveScanPhase {
    RenderContext,
    AwaitDecision,
    Help,
    WritePreview,
    ConfirmWrite,
    Complete,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractiveScanAction {
    ContextRendered,
    MaskRequested,
    SkipFinding,
    IgnoreRuleLine,
    SkipFile,
    HelpRequested,
    HelpDismissed,
    PreviewRendered,
    WriteConfirmed,
    WriteCancelled,
    QuitRequested,
}

const SUPPORTED_ACTIONS: [InteractiveScanAction; 11] = [
    InteractiveScanAction::ContextRendered,
    InteractiveScanAction::MaskRequested,
    InteractiveScanAction::SkipFinding,
    InteractiveScanAction::IgnoreRuleLine,
    InteractiveScanAction::SkipFile,
    InteractiveScanAction::HelpRequested,
    InteractiveScanAction::HelpDismissed,
    InteractiveScanAction::PreviewRendered,
    InteractiveScanAction::WriteConfirmed,
    InteractiveScanAction::WriteCancelled,
    InteractiveScanAction::QuitRequested,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractiveScanState {
    current_index: usize,
    total_findings: usize,
    phase: InteractiveScanPhase,
}

#[allow(dead_code)]
impl InteractiveScanState {
    pub fn new(total_findings: usize) -> Self {
        let phase = if total_findings == 0 {
            InteractiveScanPhase::Complete
        } else {
            InteractiveScanPhase::RenderContext
        };

        Self {
            current_index: 0,
            total_findings,
            phase,
        }
    }

    pub fn phase(&self) -> InteractiveScanPhase {
        self.phase
    }

    pub fn position(&self) -> Option<(usize, usize)> {
        if self.total_findings == 0 || matches!(self.phase, InteractiveScanPhase::Complete) {
            None
        } else {
            Some((self.current_index + 1, self.total_findings))
        }
    }

    pub fn current_index(&self) -> Option<usize> {
        if self.total_findings == 0 || matches!(self.phase, InteractiveScanPhase::Complete) {
            None
        } else {
            Some(self.current_index)
        }
    }

    pub fn apply(&mut self, action: InteractiveScanAction) -> Result<()> {
        if action == InteractiveScanAction::QuitRequested
            && !matches!(self.phase, InteractiveScanPhase::Complete)
        {
            self.phase = InteractiveScanPhase::Quit;
            return Ok(());
        }

        match (self.phase, action) {
            (InteractiveScanPhase::RenderContext, InteractiveScanAction::ContextRendered) => {
                self.phase = InteractiveScanPhase::AwaitDecision;
            }
            (InteractiveScanPhase::AwaitDecision, InteractiveScanAction::MaskRequested) => {
                self.phase = InteractiveScanPhase::WritePreview;
            }
            (InteractiveScanPhase::AwaitDecision, InteractiveScanAction::SkipFinding)
            | (InteractiveScanPhase::AwaitDecision, InteractiveScanAction::IgnoreRuleLine)
            | (InteractiveScanPhase::AwaitDecision, InteractiveScanAction::SkipFile) => {
                self.advance_finding();
            }
            (InteractiveScanPhase::AwaitDecision, InteractiveScanAction::HelpRequested) => {
                self.phase = InteractiveScanPhase::Help;
            }
            (InteractiveScanPhase::Help, InteractiveScanAction::HelpDismissed) => {
                self.phase = InteractiveScanPhase::AwaitDecision;
            }
            (InteractiveScanPhase::WritePreview, InteractiveScanAction::PreviewRendered) => {
                self.phase = InteractiveScanPhase::ConfirmWrite;
            }
            (InteractiveScanPhase::ConfirmWrite, InteractiveScanAction::WriteConfirmed) => {
                self.advance_finding();
            }
            (InteractiveScanPhase::ConfirmWrite, InteractiveScanAction::WriteCancelled) => {
                self.phase = InteractiveScanPhase::AwaitDecision;
            }
            _ => bail!(
                "invalid interactive transition: {:?} from {:?}",
                action,
                self.phase
            ),
        }

        Ok(())
    }

    fn advance_finding(&mut self) {
        if self.current_index + 1 >= self.total_findings {
            self.phase = InteractiveScanPhase::Complete;
        } else {
            self.current_index += 1;
            self.phase = InteractiveScanPhase::RenderContext;
        }
    }
}

pub fn supported_actions() -> &'static [InteractiveScanAction] {
    &SUPPORTED_ACTIONS
}

pub fn render_finding_context<W: Write>(
    writer: &mut W,
    state: &InteractiveScanState,
    finding: &Finding,
) -> io::Result<()> {
    let Some((position, total)) = state.position() else {
        return Ok(());
    };

    writeln!(writer, "Finding {}/{}", position, total)?;
    writeln!(writer, "Rule: {}", finding.rule_id)?;
    writeln!(
        writer,
        "File: {}:{}",
        finding.path.display(),
        finding.line_number
    )?;
    writeln!(
        writer,
        "Severity: {}  Score: {}  Grade: {}",
        finding.severity, finding.score, finding.grade
    )?;
    writeln!(writer, "Snippet:")?;
    writeln!(writer, "{}", finding.masked_snippet.trim())?;
    render_mask_preview(writer, finding)?;
    writeln!(writer)?;
    writeln!(
        writer,
        "Action: mask / skip / ignore-line / skip-file / help / quit"
    )?;
    Ok(())
}

pub fn render_mask_preview<W: Write>(writer: &mut W, finding: &Finding) -> io::Result<()> {
    writeln!(writer, "Mask preview:")?;
    writeln!(writer, "- {}", safe_before_line(finding))?;
    writeln!(writer, "+ {}", finding.masked_snippet.trim())?;
    Ok(())
}

fn safe_before_line(finding: &Finding) -> String {
    if finding.matched_content.is_empty()
        || !finding.line_content.contains(&finding.matched_content)
    {
        return "<sensitive content hidden>".to_string();
    }

    finding
        .line_content
        .replacen(&finding.matched_content, "<MATCH>", 1)
}

pub fn run_guarded_until_decision_input_lands(findings: &[Finding]) -> Result<bool> {
    let mut state = InteractiveScanState::new(findings.len());
    if matches!(state.phase(), InteractiveScanPhase::Complete) {
        return Ok(false);
    }

    if let Some(index) = state.current_index() {
        let stdout = io::stdout();
        let mut writer = stdout.lock();
        render_finding_context(&mut writer, &state, &findings[index])
            .context("render interactive finding context")?;
    }
    state.apply(InteractiveScanAction::ContextRendered)?;

    bail!(
        "--interactive rendered finding context at {:?} for {} finding(s), but decision input is not implemented yet. Supported actions: {}.",
        state.position(),
        findings.len(),
        supported_actions().len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use veil_core::{FindingSpan, Grade, Position, Range, Severity};

    #[test]
    fn state_starts_complete_without_findings() {
        let state = InteractiveScanState::new(0);

        assert_eq!(state.phase(), InteractiveScanPhase::Complete);
        assert_eq!(state.position(), None);
    }

    #[test]
    fn state_skips_findings_until_complete() {
        let mut state = InteractiveScanState::new(2);

        assert_eq!(state.phase(), InteractiveScanPhase::RenderContext);
        assert_eq!(state.position(), Some((1, 2)));

        state.apply(InteractiveScanAction::ContextRendered).unwrap();
        state.apply(InteractiveScanAction::SkipFinding).unwrap();
        assert_eq!(state.phase(), InteractiveScanPhase::RenderContext);
        assert_eq!(state.position(), Some((2, 2)));

        state.apply(InteractiveScanAction::ContextRendered).unwrap();
        state.apply(InteractiveScanAction::SkipFinding).unwrap();
        assert_eq!(state.phase(), InteractiveScanPhase::Complete);
        assert_eq!(state.position(), None);
    }

    #[test]
    fn state_masks_after_preview_confirmation() {
        let mut state = InteractiveScanState::new(1);

        state.apply(InteractiveScanAction::ContextRendered).unwrap();
        state.apply(InteractiveScanAction::MaskRequested).unwrap();
        assert_eq!(state.phase(), InteractiveScanPhase::WritePreview);

        state.apply(InteractiveScanAction::PreviewRendered).unwrap();
        assert_eq!(state.phase(), InteractiveScanPhase::ConfirmWrite);

        state.apply(InteractiveScanAction::WriteConfirmed).unwrap();
        assert_eq!(state.phase(), InteractiveScanPhase::Complete);
    }

    #[test]
    fn state_returns_from_help_to_decision() {
        let mut state = InteractiveScanState::new(1);

        state.apply(InteractiveScanAction::ContextRendered).unwrap();
        state.apply(InteractiveScanAction::HelpRequested).unwrap();
        assert_eq!(state.phase(), InteractiveScanPhase::Help);

        state.apply(InteractiveScanAction::HelpDismissed).unwrap();
        assert_eq!(state.phase(), InteractiveScanPhase::AwaitDecision);
    }

    #[test]
    fn state_rejects_invalid_transition() {
        let mut state = InteractiveScanState::new(1);

        let err = state
            .apply(InteractiveScanAction::WriteConfirmed)
            .unwrap_err()
            .to_string();
        assert!(err.contains("invalid interactive transition"));
    }

    #[test]
    fn guarded_runner_succeeds_without_findings() {
        assert!(!run_guarded_until_decision_input_lands(&[]).unwrap());
    }

    #[test]
    fn renderer_prints_safe_finding_context() {
        let state = InteractiveScanState::new(1);
        let finding = test_finding();
        let mut output = Vec::new();

        render_finding_context(&mut output, &state, &finding).unwrap();
        let rendered = String::from_utf8(output).unwrap();

        assert!(rendered.contains("Finding 1/1"));
        assert!(rendered.contains("Rule: pii.test"));
        assert!(rendered.contains("File: src/main.rs:42"));
        assert!(rendered.contains("Severity: HIGH  Score: 90  Grade: CRITICAL"));
        assert!(rendered.contains("token = <REDACTED>"));
        assert!(rendered.contains("Mask preview:"));
        assert!(rendered.contains("- token = <MATCH>"));
        assert!(rendered.contains("+ token = <REDACTED>"));
        assert!(!rendered.contains("raw-secret-value"));
    }

    #[test]
    fn mask_preview_hides_unmatched_raw_content() {
        let mut finding = test_finding();
        finding.matched_content = "different-raw-value".to_string();
        let mut output = Vec::new();

        render_mask_preview(&mut output, &finding).unwrap();
        let rendered = String::from_utf8(output).unwrap();

        assert!(rendered.contains("- <sensitive content hidden>"));
        assert!(rendered.contains("+ token = <REDACTED>"));
        assert!(!rendered.contains("raw-secret-value"));
        assert!(!rendered.contains("different-raw-value"));
    }

    fn test_finding() -> Finding {
        Finding {
            path: PathBuf::from("src/main.rs"),
            line_number: 42,
            line_content: "token = raw-secret-value".to_string(),
            rule_id: "pii.test".to_string(),
            matched_content: "raw-secret-value".to_string(),
            masked_snippet: "token = <REDACTED>".to_string(),
            severity: Severity::High,
            score: 90,
            grade: Grade::Critical,
            span: FindingSpan::default(),
            utf16_range: Range {
                start: Position {
                    line: 41,
                    character: 8,
                },
                end: Position {
                    line: 41,
                    character: 24,
                },
            },
            context_before: vec![],
            context_after: vec![],
            commit_sha: None,
            author: None,
            date: None,
        }
    }
}
