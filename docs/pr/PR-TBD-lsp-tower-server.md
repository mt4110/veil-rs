---
release: TBD
epic: A
pr: TBD
status: Draft
created_at: TBD
branch: feat/lsp-tower-server
commit: 0fb837ac3962c5fdd1d9f886cb4e1d35235e25c4
title: Add LSP tower server
---

## SOT
- Title: Add LSP tower server
- Status: Draft
- PR: TBD

## What
- [x] Add `tokio` and `tower-lsp` dependencies to `veil-lsp`.
- [x] Replace the scaffold exit path with a stdio `tower-lsp` server entrypoint.
- [x] Implement minimal `LanguageServer` initialize/initialized/shutdown handling.
- [x] Advertise incremental text document sync and no diagnostics/code actions yet.
- [x] Mark only the Phase 4 tower-lsp integration task complete in roadmap docs.

## Verification
- [x] `cargo fmt --all --check`
- [x] `cargo check -p veil-lsp --all-targets`
- [x] `cargo test -p veil-lsp -- --nocapture`
- [x] `git diff --check`

## Evidence
- Local `veil-lsp` unit test result: `2 passed; 0 failed`.
- `cargo check -p veil-lsp --all-targets` completed successfully.
- Unit coverage asserts incremental text sync is advertised and diagnostics/code actions remain unset.
- Cargo.lock records the `tower-lsp` dependency set.
- SOT will be renamed after the PR number is assigned.

## Non-goals
- [x] Do not map findings to diagnostics.
- [x] Do not implement document storage or debounce.
- [x] Do not implement code actions.
- [x] Do not wire `veil lsp` into `veil-cli`.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
