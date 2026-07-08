---
release: TBD
epic: A
pr: TBD
status: Draft
created_at: 2026-07-08
branch: feat/lsp-max-file-size-diagnostic
commit: 6dde2b2840a96b64600779c9205f9870d28a87ef
title: Publish LSP max file size skip diagnostic
---

## SOT
- Title: Publish LSP max file size skip diagnostic
- Status: Draft
- PR: TBD

## What
- [x] Publish a synthetic `MAX_FILE_SIZE` LSP diagnostic when an open document exceeds the scan size limit.
- [x] Short-circuit LSP scanning before `scan_content` for oversized open documents.
- [x] Keep `Finding.utf16_range` as the range source for regular findings.
- [x] Keep raw document content out of skip diagnostic `data`.

## Verification
- [x] `cargo fmt --all`
- [x] `cargo fmt --all -- --check`
- [x] `cargo test -p veil-lsp`
- [x] `cargo test --workspace --lib --bins`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `git diff --check`

## Evidence
- `veil-lsp` tests passed: `16 passed; 0 failed`.
- Workspace lib/bin tests passed across `veil-cli`, `veil-config`, `veil-core`, `veil-guardian`, `veil-lsp`, `veil-pro`, and `veil-server`.
- Workspace clippy passed with `-D warnings`.
- `diagnostics` tests cover safe `MAX_FILE_SIZE` diagnostic payload and zero-range placement.
- `server` tests cover oversized text short-circuiting before normal finding scan.

## Non-goals
- [x] Do not change secret or PII finding mapping for non-skipped diagnostics.
- [x] Do not implement code actions.
- [x] Do not add preset/config file loading to LSP startup yet.
- [x] Do not add binary/read-error skip diagnostics to LSP in this PR.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
