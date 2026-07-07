---
release: TBD
epic: A
pr: TBD
status: Draft
created_at: TBD
branch: feat/interactive-scripted-decisions
commit: 52df729320e1086744a637ebcc64a599a461c843
title: Add interactive scripted decisions
---

## SOT
- Title: Add interactive scripted decisions
- Status: Draft
- PR: TBD

## What
- [x] Add scripted decision parsing for `veil scan --interactive`.
- [x] Wire interactive scan to read decisions from stdin.
- [x] Support deterministic `quit`, `skip`, and `help` flows without mutating files.
- [x] Keep `mask`, `ignore-line`, and `skip-file` parsed but guarded until write handling lands.
- [x] Add unit coverage for scripted decisions and CLI coverage for `q\n`.
- [x] Mark the Phase 3 scripted stdin test task complete in roadmap docs.

## Verification
- [x] `cargo fmt --all --check`
- [x] `cargo test -p veil-cli interactive_scan -- --nocapture`
- [x] `cargo test -p veil-cli scan_interactive_accepts_scripted_quit -- --nocapture`
- [x] `git diff --check`

## Evidence
- Local interactive unit test result: `12 passed; 0 failed`.
- Local scripted stdin integration test result: `1 passed; 0 failed`.
- CLI integration test drives `veil scan --interactive` with `q\n` on stdin and expects success.
- SOT will be renamed after the PR number is assigned.

## Non-goals
- [x] Do not implement atomic write.
- [x] Do not mutate source files, config, ignore state, or baselines.
- [x] Do not auto-select mask or apply any decision beyond review navigation.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
