---
release: TBD
epic: A
pr: 139
status: Ready
created_at: 2026-07-08
branch: feat/lsp-utf16-range-fixture
commit: 2865d3b13ce0cd4618b35d93d9d3f97be3629d2f
title: Add LSP UTF-16 range fixtures
---

## SOT
- Title: Add LSP UTF-16 range fixtures
- Status: Ready
- PR: #139

## What
- [x] Add fixture-driven `veil-lsp` tests for UTF-16 diagnostic ranges.
- [x] Cover emoji-prefixed and fullwidth-prefixed email diagnostics.
- [x] Assert that diagnostic `data` remains raw-free while range positions stay UTF-16-based.

## Verification
- [x] `cargo fmt --all`
- [x] `cargo test -p veil-lsp`
- [x] `cargo test --workspace --lib --bins`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `git diff --check`

## Non-goals
- [x] Do not add code actions.
- [x] Do not change LSP publish behavior.
- [x] Do not introduce disk-based binary or read-error classification into LSP.

## Evidence
- `veil-lsp` tests passed with the new fixture-driven UTF-16 range contract test.
- Workspace lib/bin tests passed across `veil-cli`, `veil-config`, `veil-core`, `veil-guardian`, `veil-lsp`, `veil-pro`, and `veil-server`.
- Workspace clippy passed with `-D warnings`.
