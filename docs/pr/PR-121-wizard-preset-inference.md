---
release: TBD
epic: A
pr: 121
status: Draft
created_at: TBD
branch: feat/wizard-preset-inference
commit: c6ef29f30e4af2865a43c54cb4d51724ccd120cc
title: Add wizard preset inference
---

## SOT
- Title: Add wizard preset inference
- Status: Draft
- PR: #121

## What
- [x] Add deterministic `veil init --wizard` preset inference for root-level project signals.
- [x] Detect log audit signals (`logs/`, `*.log`, `*.jsonl`, `*.ndjson`) before fintech directory signals.
- [x] Detect fintech directory signals (`payments`, `billing`, `kyc`, `account`) before Japanese README signals.
- [x] Display the detected preset candidate in the wizard without auto-applying it.
- [x] Cover inference priority and no-signal behavior with unit tests.

## Verification
- [x] `cargo fmt --all --check`
- [x] `cargo test -p veil-cli infer_wizard_preset -- --nocapture`
- [x] `git diff --check`

## Evidence
- Local test result: `5 passed; 0 failed` for `infer_wizard_preset`.
- SOT was renamed from the temporary PR placeholder name after PR #121 was created.

## Non-goals
- [x] Do not add `CoreConfig.preset`.
- [x] Do not auto-apply inferred presets from the wizard.
- [x] Do not redesign the existing wizard profile questions.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
