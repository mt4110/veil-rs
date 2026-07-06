---
release: TBD
epic: A
pr: TBD
status: Draft
created_at: TBD
branch: feat/sot-template-helper
commit: 7f9ca1de4942f9ceebd11154c5cc5e08a2ec9990
title: Add PR SOT template helper
---

## SOT
- Title: Add PR SOT template helper
- Status: Draft
- PR: TBD

## What
- Add `veil sot new --slug <slug>` support for slug-only SOT names such as `docs/pr/PR-TBD-sot-template-helper.md`.
- Add `veil sot new --pr <number>` support for direct `PR-<number>-<slug>.md` creation.
- Add `--status` and `--date`; default `created_at` to `TBD` so generation does not depend on the current clock.
- Align the built-in SOT template and `docs/pr/sot_template.md` with the standard SOT / What / Verification / Evidence / Non-goals / Rollback sections.
- Update `docs/pr/README.md` to document the helper instead of the stale manual-only workflow.

## Verification
- [x] `cargo fmt --all --check`
- [x] `git diff --check`
- [x] `cargo test -p veil-cli sot_new -- --nocapture`
- [x] `cargo test -p veil-cli sot_rename -- --nocapture`
- [x] `cargo run -p veil-cli -- sot new --slug sot-template-helper --title "Add PR SOT template helper"`
- [x] `cargo run -p veil-cli -- sot new --pr 999 --slug sample --title "Sample" --dry-run`
- [x] `cargo clippy -p veil-cli --all-targets --all-features -- -D warnings`

## Evidence
- `cargo test -p veil-cli sot_new -- --nocapture` passed with 5 `sot_new_test` tests.
- `cargo test -p veil-cli sot_rename -- --nocapture` passed with 3 `sot_rename_test` tests.
- The dry-run command printed the `docs/pr/PR-999-sample.md` content to stdout without writing a file.
- The generated SOT file is this file: `docs/pr/PR-TBD-sot-template-helper.md`.

## Non-goals
- Do not weaken or bypass `.github/workflows/pr_sot_guard.yml`.
- Do not introduce a separate SOT script while the existing `veil sot` command already owns this workflow.
- Do not touch JP PII address validator wiring; that remains the next small PR.

## Rollback
- Revert this PR as a unit to restore the previous `veil sot new` behavior and docs.
