use anyhow::{bail, Context, Result};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;
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

#[allow(dead_code)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InteractiveDecision {
    Mask,
    SkipFinding,
    IgnoreRuleLine,
    SkipFile,
    Help,
    Quit,
}

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

fn parse_decision(input: &str) -> Option<InteractiveDecision> {
    match input.trim() {
        "m" | "mask" => Some(InteractiveDecision::Mask),
        "n" | "skip" => Some(InteractiveDecision::SkipFinding),
        "i" | "ignore-line" => Some(InteractiveDecision::IgnoreRuleLine),
        "s" | "skip-file" => Some(InteractiveDecision::SkipFile),
        "?" | "h" | "help" => Some(InteractiveDecision::Help),
        "q" | "quit" => Some(InteractiveDecision::Quit),
        _ => None,
    }
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
    render_action_prompt(writer)?;
    Ok(())
}

fn render_action_prompt<W: Write>(writer: &mut W) -> io::Result<()> {
    writeln!(
        writer,
        "Action: mask / skip / ignore-line / skip-file / help / quit"
    )
}

fn render_help<W: Write>(writer: &mut W) -> io::Result<()> {
    writeln!(writer, "Actions:")?;
    writeln!(writer, "  mask        prepare masked write handling")?;
    writeln!(writer, "  skip        move to the next finding")?;
    writeln!(writer, "  ignore-line prepare line ignore handling")?;
    writeln!(writer, "  skip-file   prepare file skip handling")?;
    writeln!(writer, "  help        show actions")?;
    writeln!(writer, "  quit        exit interactive review")?;
    render_action_prompt(writer)
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

pub fn run_interactive(findings: &[Finding]) -> Result<bool> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut input = stdin.lock();
    let mut output = stdout.lock();

    run_with_io(findings, &mut input, &mut output)
}

pub fn run_with_io<R: BufRead, W: Write>(
    findings: &[Finding],
    input: &mut R,
    output: &mut W,
) -> Result<bool> {
    let mut state = InteractiveScanState::new(findings.len());
    if matches!(state.phase(), InteractiveScanPhase::Complete) {
        return Ok(false);
    }

    let mut line = String::new();
    loop {
        if matches!(state.phase(), InteractiveScanPhase::RenderContext) {
            if let Some(index) = state.current_index() {
                render_finding_context(output, &state, &findings[index])
                    .context("render interactive finding context")?;
                output
                    .flush()
                    .context("flush interactive finding context")?;
            }
            state.apply(InteractiveScanAction::ContextRendered)?;
        }

        line.clear();
        let bytes_read = input
            .read_line(&mut line)
            .context("read interactive decision")?;
        if bytes_read == 0 {
            bail!("--interactive reached end of input before a decision");
        }

        match parse_decision(&line) {
            Some(InteractiveDecision::Quit) => {
                state.apply(InteractiveScanAction::QuitRequested)?;
                return Ok(false);
            }
            Some(InteractiveDecision::SkipFinding) => {
                state.apply(InteractiveScanAction::SkipFinding)?;
                if matches!(state.phase(), InteractiveScanPhase::Complete) {
                    return Ok(false);
                }
            }
            Some(InteractiveDecision::Help) => {
                state.apply(InteractiveScanAction::HelpRequested)?;
                render_help(output).context("render interactive help")?;
                output.flush().context("flush interactive help")?;
                state.apply(InteractiveScanAction::HelpDismissed)?;
            }
            Some(InteractiveDecision::Mask) => {
                let Some(index) = state.current_index() else {
                    return Ok(false);
                };
                state.apply(InteractiveScanAction::MaskRequested)?;
                state.apply(InteractiveScanAction::PreviewRendered)?;
                let wrote = apply_mask_atomic(&findings[index])?;
                if wrote {
                    writeln!(output, "Masked: {}", findings[index].path.display())?;
                } else {
                    writeln!(output, "Already masked: {}", findings[index].path.display())?;
                }
                output.flush().context("flush interactive mask result")?;
                state.apply(InteractiveScanAction::WriteConfirmed)?;
                if matches!(state.phase(), InteractiveScanPhase::Complete) {
                    return Ok(false);
                }
            }
            Some(InteractiveDecision::IgnoreRuleLine) => {
                bail!(
                    "--interactive action 'ignore-line' is parsed but ignore handling is not implemented yet"
                );
            }
            Some(InteractiveDecision::SkipFile) => {
                bail!(
                    "--interactive action 'skip-file' is parsed but file skip handling is not implemented yet"
                );
            }
            None => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    writeln!(output, "Unknown action.")?;
                } else {
                    writeln!(output, "Unknown action: {}", trimmed)?;
                }
                render_action_prompt(output)?;
                output.flush().context("flush interactive action prompt")?;
            }
        }
    }
}

fn apply_mask_atomic(finding: &Finding) -> Result<bool> {
    if finding.matched_content.is_empty() {
        bail!("cannot mask finding with empty matched content");
    }

    if finding.masked_snippet.contains('\n') || finding.masked_snippet.contains('\r') {
        bail!("cannot mask finding with multi-line masked snippet");
    }

    let content = fs::read_to_string(&finding.path)
        .with_context(|| format!("read {}", finding.path.display()))?;
    let Some(new_content) = masked_file_content(&content, finding)? else {
        return Ok(false);
    };

    write_atomic(&finding.path, &new_content)?;
    Ok(true)
}

fn masked_file_content(content: &str, finding: &Finding) -> Result<Option<String>> {
    let Some(target_index) = finding.line_number.checked_sub(1) else {
        bail!("finding line number must be 1-based");
    };

    let mut output = String::with_capacity(content.len());
    let mut found_target = false;

    for (index, segment) in content.split_inclusive('\n').enumerate() {
        if index != target_index {
            output.push_str(segment);
            continue;
        }

        found_target = true;
        let (line, ending) = split_line_ending(segment);

        if line == finding.masked_snippet {
            return Ok(None);
        }

        if line != finding.line_content {
            bail!(
                "refusing to mask {}:{} because the line changed since scan",
                finding.path.display(),
                finding.line_number
            );
        }

        if !line.contains(&finding.matched_content) {
            bail!(
                "refusing to mask {}:{} because matched content is no longer present",
                finding.path.display(),
                finding.line_number
            );
        }

        output.push_str(&finding.masked_snippet);
        output.push_str(ending);
    }

    if !found_target {
        bail!(
            "refusing to mask {}:{} because the line was not found",
            finding.path.display(),
            finding.line_number
        );
    }

    Ok(Some(output))
}

fn split_line_ending(segment: &str) -> (&str, &str) {
    if let Some(line) = segment.strip_suffix("\r\n") {
        (line, "\r\n")
    } else if let Some(line) = segment.strip_suffix('\n') {
        (line, "\n")
    } else {
        (segment, "")
    }
}

fn write_atomic(path: &Path, content: &str) -> Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let permissions = fs::metadata(path)
        .with_context(|| format!("read metadata for {}", path.display()))?
        .permissions();
    let mut tmp = tempfile::Builder::new()
        .prefix(".veil-interactive-mask.")
        .tempfile_in(parent)
        .with_context(|| format!("create temp file in {}", parent.display()))?;

    tmp.write_all(content.as_bytes())
        .with_context(|| format!("write temp file for {}", path.display()))?;
    tmp.flush()
        .with_context(|| format!("flush temp file for {}", path.display()))?;
    fs::set_permissions(tmp.path(), permissions)
        .with_context(|| format!("set temp file permissions for {}", tmp.path().display()))?;
    tmp.as_file()
        .sync_all()
        .with_context(|| format!("sync temp file for {}", path.display()))?;
    tmp.persist(path)
        .map_err(|error| error.error)
        .with_context(|| format!("replace {}", path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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
    fn runner_succeeds_without_findings() {
        let mut input = io::Cursor::new(Vec::new());
        let mut output = Vec::new();

        assert!(!run_with_io(&[], &mut input, &mut output).unwrap());
        assert!(output.is_empty());
    }

    #[test]
    fn decision_parser_accepts_scripted_actions() {
        assert_eq!(parse_decision("m\n"), Some(InteractiveDecision::Mask));
        assert_eq!(parse_decision("mask"), Some(InteractiveDecision::Mask));
        assert_eq!(
            parse_decision("n\n"),
            Some(InteractiveDecision::SkipFinding)
        );
        assert_eq!(
            parse_decision("skip"),
            Some(InteractiveDecision::SkipFinding)
        );
        assert_eq!(
            parse_decision("i\n"),
            Some(InteractiveDecision::IgnoreRuleLine)
        );
        assert_eq!(
            parse_decision("ignore-line"),
            Some(InteractiveDecision::IgnoreRuleLine)
        );
        assert_eq!(parse_decision("s\n"), Some(InteractiveDecision::SkipFile));
        assert_eq!(
            parse_decision("skip-file"),
            Some(InteractiveDecision::SkipFile)
        );
        assert_eq!(parse_decision("?\n"), Some(InteractiveDecision::Help));
        assert_eq!(parse_decision("help"), Some(InteractiveDecision::Help));
        assert_eq!(parse_decision("q\n"), Some(InteractiveDecision::Quit));
        assert_eq!(parse_decision("quit"), Some(InteractiveDecision::Quit));
        assert_eq!(parse_decision(""), None);
        assert_eq!(parse_decision("unknown"), None);
    }

    #[test]
    fn runner_accepts_scripted_quit() {
        let findings = vec![test_finding()];
        let mut input = io::Cursor::new(b"q\n".to_vec());
        let mut output = Vec::new();

        assert!(!run_with_io(&findings, &mut input, &mut output).unwrap());
        let rendered = String::from_utf8(output).unwrap();

        assert!(rendered.contains("Finding 1/1"));
        assert!(rendered.contains("Action: mask / skip / ignore-line / skip-file / help / quit"));
    }

    #[test]
    fn runner_accepts_scripted_skip_then_quit() {
        let mut second = test_finding();
        second.line_number = 43;
        let findings = vec![test_finding(), second];
        let mut input = io::Cursor::new(b"n\nq\n".to_vec());
        let mut output = Vec::new();

        assert!(!run_with_io(&findings, &mut input, &mut output).unwrap());
        let rendered = String::from_utf8(output).unwrap();

        assert!(rendered.contains("Finding 1/2"));
        assert!(rendered.contains("Finding 2/2"));
    }

    #[test]
    fn runner_masks_scripted_decision_with_atomic_write() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secret.txt");
        fs::write(&path, "token = raw-secret-value\nnext = safe\n").unwrap();
        let findings = vec![test_finding_at(path.clone())];
        let mut input = io::Cursor::new(b"mask\n".to_vec());
        let mut output = Vec::new();

        assert!(!run_with_io(&findings, &mut input, &mut output).unwrap());
        let rendered = String::from_utf8(output).unwrap();

        assert!(rendered.contains("Masked:"));
        assert_eq!(
            fs::read_to_string(path).unwrap(),
            "token = <REDACTED>\nnext = safe\n"
        );
    }

    #[test]
    fn atomic_mask_rejects_stale_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secret.txt");
        fs::write(&path, "token = changed-value\n").unwrap();
        let finding = test_finding_at(path.clone());

        let err = apply_mask_atomic(&finding).unwrap_err().to_string();

        assert!(err.contains("line changed since scan"));
        assert_eq!(fs::read_to_string(path).unwrap(), "token = changed-value\n");
    }

    #[test]
    fn atomic_mask_preserves_crlf_line_endings() {
        let finding = test_finding();

        let masked = masked_file_content("token = raw-secret-value\r\nnext = safe\r\n", &finding)
            .unwrap()
            .unwrap();

        assert_eq!(masked, "token = <REDACTED>\r\nnext = safe\r\n");
    }

    #[test]
    fn atomic_mask_is_idempotent_for_already_masked_line() {
        let finding = test_finding();

        assert!(masked_file_content("token = <REDACTED>\n", &finding)
            .unwrap()
            .is_none());
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
        assert!(rendered.contains("File: src/main.rs:1"));
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
        test_finding_at(PathBuf::from("src/main.rs"))
    }

    fn test_finding_at(path: PathBuf) -> Finding {
        Finding {
            path,
            line_number: 1,
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
                    line: 0,
                    character: 8,
                },
                end: Position {
                    line: 0,
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
