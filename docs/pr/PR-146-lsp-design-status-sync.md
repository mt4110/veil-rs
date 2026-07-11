---
release: TBD
epic: LSP
pr: 146
status: Ready
created_at: 2026-07-11
branch: feat/lsp-design-status-sync
commit: 0696022
title: Sync LSP design implementation status
---

# SOT: Sync LSP Design Implementation Status

## SOT
- Title: Sync LSP design implementation status
- Status: Ready
- PR: #146

## What
- [x] Mark LSP code actions complete in the enterprise JP PII roadmap now that mask, partial mask, and inline ignore quick fixes are on `main`.
- [x] Mark the `veil lsp` CLI entrypoint and Neovim startup note complete after PR #145.
- [x] Update the partial-mask SOT with its landed `main` commit instead of leaving the evidence as `TBD`.

## Verification
- [x] `python3 scripts/check_docs_taxonomy.py` - PASS
- [x] `git diff --check` - PASS

## Evidence
- `crates/veil-lsp/src/code_actions.rs` contains `Mask value`, `Partial mask`, and `Add inline ignore` quick fixes.
- `docs/design/enterprise_jp_pii/06_lsp_design.md` already documents the Neovim `vim.lsp.start` example for `veil lsp --preset fintech-jp`.
- Partial mask landed on `main` as `16eb4f3 Add LSP partial mask code action`.

## Non-goals
- [x] Do not change LSP behavior.
- [x] Do not publish or modify `.private/` design documents.
- [x] Do not invent a PR number for commit-level evidence.

## Rollback
- Revert this docs-only PR to restore the previous implementation-status wording.
