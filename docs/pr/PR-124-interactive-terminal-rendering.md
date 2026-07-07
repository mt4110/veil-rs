---
release: TBD
epic: A
pr: 124
status: Draft
created_at: TBD
branch: feat/interactive-terminal-rendering
commit: 0b6e50e5b7c7fd72778b3519f5a5dcccbdf59ab6
title: Add interactive terminal rendering
---

## SOT
- Title: Add interactive terminal rendering
- Status: Draft
- PR: #124

## What
- [x] Add terminal rendering for the current `veil scan --interactive` finding.
- [x] Render only safe context: position, rule id, path, line, severity, score, grade, and `masked_snippet`.
- [x] Keep raw `line_content` and `matched_content` out of interactive output.
- [x] Advance the state machine from `RenderContext` to `AwaitDecision` after rendering.
- [x] Keep decision input guarded for a follow-up PR.
- [x] Mark the roadmap terminal rendering task complete with the safe masked-context boundary.

## Verification
- [x] `cargo fmt --all --check`
- [x] `cargo test -p veil-cli interactive_scan -- --nocapture`
- [x] `cargo test -p veil-cli scan_interactive_renders_first_finding_until_decision_input_lands -- --nocapture`
- [x] `git diff --check`

## Evidence
- Local interactive unit test result: `7 passed; 0 failed`.
- Local guarded integration test result: `1 passed; 0 failed`.
- Unit coverage asserts the rendered terminal context contains `<REDACTED>` and does not contain the raw matched value.
- SOT was renamed after PR #124 was created.

## Non-goals
- [x] Do not implement decision input.
- [x] Do not implement diff preview.
- [x] Do not implement atomic writes or file mutation.
- [x] Do not print raw secret values.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
