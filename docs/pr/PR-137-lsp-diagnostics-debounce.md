---
release: TBD
epic: A
pr: 137
status: Ready
created_at: 2026-07-08
branch: feat/lsp-diagnostics-debounce
commit: d7f84f175cd9ed6506d36aa83dcee011a40956bf
title: Debounce LSP diagnostics publishing
---

## SOT
- Title: Debounce LSP diagnostics publishing
- Status: Ready
- PR: #137

## What
- [x] Add per-document `scan_revision` tracking to the LSP document store.
- [x] Debounce `didChange` diagnostics publication by 200ms.
- [x] Abort and replace pending scans when newer edits arrive for the same document.
- [x] Drop stale or closed-document scan results before publish.
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
- `veil-lsp` tests passed: `14 passed; 0 failed`.
- Workspace lib/bin tests passed across `veil-cli`, `veil-config`, `veil-core`, `veil-guardian`, `veil-lsp`, `veil-pro`, and `veil-server`.
- Workspace clippy passed with `-D warnings`.
- `document_store` tests now cover revision reset and latest-generation tracking.
- `server` tests cover debounce interval and latest-revision gating.

## Non-goals
- [x] Do not implement code actions.
- [x] Do not add preset/config file loading to LSP startup yet.
- [x] Do not add size-limit skip diagnostics yet.
- [x] Do not advertise LSP pull diagnostics; this PR still uses `publishDiagnostics` notifications only.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
