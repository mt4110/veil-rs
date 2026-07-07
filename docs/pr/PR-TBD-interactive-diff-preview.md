---
release: TBD
epic: A
pr: TBD
status: Draft
created_at: TBD
branch: feat/interactive-diff-preview
commit: 3c27b450ab8c563dfb496f826d4a50e7f38b3a47
title: Add interactive diff preview
---

## SOT
- Title: Add interactive diff preview
- Status: Draft
- PR: TBD

## What
- [x] Add a safe `Mask preview` block to interactive terminal rendering.
- [x] Render before/after lines as `- <MATCH>` redacted source and `+ <REDACTED>` masked output.
- [x] Hide raw content when the matched raw value cannot be located in the source line.
- [x] Keep the interactive state at decision input; do not auto-select mask.
- [x] Mark the Phase 3 diff preview task complete with the safe redacted-preview boundary.

## Verification
- [x] `cargo fmt --all --check`
- [x] `cargo test -p veil-cli interactive_scan -- --nocapture`
- [x] `cargo test -p veil-cli scan_interactive_renders_first_finding_until_decision_input_lands -- --nocapture`
- [x] `git diff --check`

## Evidence
- Local interactive unit test result: `8 passed; 0 failed`.
- Local guarded integration test result: `1 passed; 0 failed`.
- Unit coverage asserts unmatched raw content is hidden and raw matched values do not appear in preview output.
- SOT will be renamed after PR creation.

## Non-goals
- [x] Do not implement decision input.
- [x] Do not implement atomic writes or file mutation.
- [x] Do not display raw secret values.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
