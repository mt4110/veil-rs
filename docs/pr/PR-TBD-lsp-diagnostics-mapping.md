---
release: TBD
epic: A
pr: TBD
status: Draft
created_at: TBD
branch: feat/lsp-diagnostics
commit: 95493211e2179a0f17e3fc9a77d153d776a38fa6
title: Add LSP diagnostics mapping
---

## SOT
- Title: Add LSP diagnostics mapping
- Status: Draft
- PR: TBD

## What
- [x] Add pure `Finding` to LSP `Diagnostic` mapping in `veil-lsp`.
- [x] Use `Finding.utf16_range` as the only diagnostic range source.
- [x] Map Veil severities to LSP severity levels.
- [x] Populate diagnostic `data` with `ruleId`, `score`, `grade`, `maskedSnippet`, and `actions` only.
- [x] Mark only the Phase 4 diagnostics mapping task complete in roadmap docs.

## Verification
- [x] `cargo fmt --all --check`
- [x] `cargo check -p veil-lsp --all-targets`
- [x] `cargo test -p veil-lsp diagnostics -- --nocapture`
- [x] `git diff --check`

## Evidence
- Local `veil-lsp` diagnostics unit tests passed: `4 passed; 0 failed`.
- `cargo check -p veil-lsp --all-targets` completed successfully.
- Unit coverage asserts UTF-16 range use, severity mapping, safe diagnostic data, and finding order preservation.
- Diagnostic data tests assert raw `line_content` and `matched_content` do not leak.

## Non-goals
- [x] Do not call `scan_content` from LSP.
- [x] Do not advertise diagnostics capability from the server yet.
- [x] Do not implement document storage, debounce, or publication.
- [x] Do not implement code actions.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
