---
release: TBD
epic: A
pr: 123
status: Draft
created_at: TBD
branch: feat/interactive-state-machine
commit: 08121a11a6be8b6835bc9fc4b6edf19e0c42bdda
title: Add interactive scan state machine
---

## SOT
- Title: Add interactive scan state machine
- Status: Draft
- PR: #123

## What
- [x] Add an `interactive_scan` state machine for Phase 3 interactive scan flow.
- [x] Model context rendering, decision wait, help, write preview, write confirmation, completion, and quit phases.
- [x] Model supported decisions for mask, skip finding, ignore rule line, skip file, help, confirm/cancel write, and quit.
- [x] Initialize the state machine from `veil scan --interactive` scan results.
- [x] Keep terminal rendering guarded until the next PR.
- [x] Mark the roadmap state-machine task complete with the renderer boundary.

## Verification
- [x] `cargo fmt --all --check`
- [x] `cargo test -p veil-cli interactive_scan -- --nocapture`
- [x] `cargo test -p veil-cli scan_interactive_flag_is_guarded_until_state_machine_lands -- --nocapture`
- [x] `git diff --check`

## Evidence
- Local state-machine test result: `6 passed; 0 failed` for `interactive_scan`.
- Local guarded integration test result: `1 passed; 0 failed`.
- SOT was renamed after PR #123 was created.

## Non-goals
- [x] Do not implement terminal rendering.
- [x] Do not implement diff preview.
- [x] Do not implement atomic writes or file mutation.
- [x] Do not change normal non-interactive `veil scan` output.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
