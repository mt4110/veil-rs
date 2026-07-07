---
release: TBD
epic: A
pr: TBD
status: Draft
created_at: TBD
branch: feat/lsp-crate-scaffold
commit: e8e784860c71d730689b21476ef9743c5f563c70
title: Add LSP crate scaffold
---

## SOT
- Title: Add LSP crate scaffold
- Status: Draft
- PR: TBD

## What
- [x] Add `crates/veil-lsp` as a workspace member.
- [x] Add the LSP crate file layout from the design doc.
- [x] Add a `veil-lsp` binary scaffold that exits explicitly until server implementation lands.
- [x] Add a minimal server-name unit test.
- [x] Mark only the Phase 4 LSP crate scaffold task complete in roadmap docs.

## Verification
- [x] `cargo fmt --all --check`
- [x] `cargo check -p veil-lsp --all-targets`
- [x] `cargo test -p veil-lsp -- --nocapture`
- [x] `git diff --check`

## Evidence
- Local `veil-lsp` unit test result: `1 passed; 0 failed`.
- `cargo check -p veil-lsp --all-targets` completed successfully.
- Cargo.lock contains only the new `veil-lsp` workspace package entry.
- SOT will be renamed after the PR number is assigned.

## Non-goals
- [x] Do not add `tower-lsp` integration.
- [x] Do not wire `veil lsp` into the existing CLI.
- [x] Do not implement diagnostics, document storage, range mapping, or code actions.
- [x] Do not change scan, UI, or interactive behavior.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
