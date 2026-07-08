---
release: TBD
epic: A
pr: 136
status: Ready
created_at: 2026-07-08
branch: feat/lsp-diagnostics-publish
commit: 1225c4b1b010c65a02aaea35b82e690bbcc9c9e0
title: Publish LSP diagnostics
---

## SOT
- Title: Publish LSP diagnostics
- Status: Ready
- PR: #136

## What
- [x] Store opened LSP documents with URI, text, and version.
- [x] Apply full and incremental `didChange` content updates before scanning.
- [x] Run `scan_content` from the LSP server and convert findings through the existing pure diagnostics mapping.
- [x] Publish diagnostics on `didOpen` and `didChange`, including the document version.
- [x] Clear diagnostics on `didClose`.
- [x] Keep `Finding.utf16_range` as the only diagnostic range source.
- [x] Keep raw `matched_content` and raw lines out of diagnostic `data`.

## Verification
- [x] `cargo fmt --all`
- [x] `cargo fmt --all -- --check`
- [x] `cargo test -p veil-lsp`
- [x] `cargo test --workspace --lib --bins`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `git diff --check`

## Evidence
- Local `veil-lsp` tests passed: `10 passed; 0 failed`.
- Workspace lib/bin tests passed across `veil-cli`, `veil-config`, `veil-core`, `veil-guardian`, `veil-lsp`, `veil-pro`, and `veil-server`.
- Workspace clippy passed with `-D warnings`.
- `document_store` tests cover full document updates, UTF-16 incremental ranges, and invalid surrogate-pair positions.
- `server` tests cover default-rule scan wiring, UTF-16 diagnostic range preservation, and raw-value exclusion from diagnostic data.

## Non-goals
- [x] Do not implement code actions.
- [x] Do not implement debounce or stale revision cancellation yet.
- [x] Do not add preset/config file loading to LSP startup yet.
- [x] Do not advertise LSP pull diagnostics; this PR uses `publishDiagnostics` notifications only.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
