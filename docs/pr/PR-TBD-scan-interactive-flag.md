---
release: TBD
epic: A
pr: TBD
status: Draft
created_at: TBD
branch: feat/scan-interactive-flag
commit: 2b4730101a59c98913801c5d34d9fdde06993133
title: Add scan interactive flag
---

## SOT
- Title: Add scan interactive flag
- Status: Draft
- PR: TBD

## What
- [x] Add `veil scan --interactive` as an accepted CLI flag.
- [x] Thread the flag through `main.rs` into `commands::scan::scan`.
- [x] Guard the flag with an explicit exit-code-2 error until the interactive state machine lands.
- [x] Mark the Phase 3 flag task complete as a guarded stub.
- [x] Cover the guarded behavior with an integration test.

## Verification
- [x] `cargo fmt --all --check`
- [x] `cargo test -p veil-cli scan_interactive_flag_is_guarded_until_state_machine_lands -- --nocapture`
- [x] `git diff --check`

## Evidence
- Local test result: `1 passed; 0 failed` for `scan_interactive_flag_is_guarded_until_state_machine_lands`.
- SOT will be renamed after PR creation.

## Non-goals
- [x] Do not implement finding iteration state machine.
- [x] Do not implement terminal rendering, diff preview, or writes.
- [x] Do not change normal `veil scan` behavior when `--interactive` is absent.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
