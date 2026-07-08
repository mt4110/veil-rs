---
release: TBD
epic: A
pr: 140
status: Ready
created_at: 2026-07-08
branch: feat/lsp-mask-code-action
commit: 039405cfadb42ae50f8e57c61d3cdad5672875dc
title: Add LSP mask code action
---

## SOT
- Title: Add LSP mask code action
- Status: Ready
- PR: #140

## What
- [x] Wire a minimal LSP `Mask value` code action for finding diagnostics.
- [x] Reuse the diagnostic UTF-16 range as the only edit range source.
- [x] Suppress mask actions for skip diagnostics and already-redacted selections.
- [x] Advertise `code_action_provider` in server capabilities.

## Verification
- [x] `cargo fmt --all`
- [x] `cargo test -p veil-lsp`
- [x] `cargo test --workspace --lib --bins`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `git diff --check`

## Non-goals
- [x] Do not implement partial mask.
- [x] Do not implement inline ignore comments.
- [x] Do not change LSP diagnostics payload shape.

## Evidence
- `veil-lsp` tests passed with new mask code action coverage and existing UTF-16 fixture coverage.
- Workspace lib/bin tests passed across `veil-cli`, `veil-config`, `veil-core`, `veil-guardian`, `veil-lsp`, `veil-pro`, and `veil-server`.
- Workspace clippy passed with `-D warnings`.
