---
release: TBD
epic: A
pr: 142
status: Ready
created_at: 2026-07-08
branch: feat/core-partial-mask-unicode
commit: 61581caec0fad6abac006509e70276019d102921
title: Align partial mask with last-four design
---

## SOT
- Title: Align partial mask with last-four design
- Status: Ready
- PR: #142

## What
- [x] Align `MaskMode::Partial` with the design contract to retain only the last four characters.
- [x] Share the partial mask formatting logic between range-based and span-based masking.
- [x] Add multibyte tests so partial masking is safe for Unicode text.

## Verification
- [x] `cargo test -p veil-core masking::tests`
- [x] `cargo fmt --all`
- [x] `cargo test --workspace --lib --bins`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `git diff --check`

## Non-goals
- [x] Do not wire new LSP code actions.
- [x] Do not change redact or plain mask behavior.
- [x] Do not change diagnostic payloads.

## Evidence
- `veil-core` masking tests passed with new Unicode-safe partial mask coverage for both range-based and span-based masking.
- Workspace lib/bin tests passed across `veil-cli`, `veil-config`, `veil-core`, `veil-guardian`, `veil-lsp`, `veil-pro`, and `veil-server`.
- Workspace clippy passed with `-D warnings`.
