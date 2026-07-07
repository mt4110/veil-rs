use anyhow::{bail, Result};

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

pub fn run_guarded_until_renderer_lands(total_findings: usize) -> Result<bool> {
    let state = InteractiveScanState::new(total_findings);
    if matches!(state.phase(), InteractiveScanPhase::Complete) {
        return Ok(false);
    }

    bail!(
        "--interactive initialized finding iteration state at {:?} for {} finding(s), but terminal rendering is not implemented yet. Supported actions: {}.",
        state.position(),
        total_findings,
        supported_actions().len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(!run_guarded_until_renderer_lands(0).unwrap());
    }
}
