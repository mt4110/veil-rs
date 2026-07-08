---
release: TBD
epic: A
pr: TBD
status: Draft
created_at: 2026-07-08
branch: feat/lsp-inline-ignore-action
commit: 8772577c8b9f1895aa3e6d13e8878df420200cd5
title: Add LSP inline ignore code action
---

## SOT
- Title: Add LSP inline ignore code action
- Status: Draft
- PR: TBD

## What
- [x] Add LSP `Add inline ignore` code action for findings that advertise `ignore`.
- [x] Resolve comment syntax from document language or file extension.
- [x] Hide inline ignore for JSON and other commentless formats.
- [x] Reuse UTF-16 positions for insertion edits without using `masked_snippet` as a range source.

## Verification
- [x] `cargo test -p veil-lsp`
- [x] `cargo fmt --all`
- [x] `cargo test --workspace --lib --bins`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `git diff --check`

## Non-goals
- [x] Do not implement partial mask.
- [x] Do not change diagnostic payload shape.
- [x] Do not add inline ignore for JSON/commentless formats.

## Evidence
- `veil-lsp` tests passed with new inline ignore coverage for Rust and JSON/commentless suppression.
- Workspace lib/bin tests passed across `veil-cli`, `veil-config`, `veil-core`, `veil-guardian`, `veil-lsp`, `veil-pro`, and `veil-server`.
- Workspace clippy passed with `-D warnings`.
