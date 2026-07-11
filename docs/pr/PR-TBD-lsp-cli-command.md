---
release: TBD
epic: LSP
pr: TBD
status: Draft
created_at: 2026-07-11
branch: feat/lsp-cli-command
commit: 4ff7a60
title: Add CLI entrypoint for LSP
---

# SOT: LSP CLI Entrypoint

## SOT
- Title: Add CLI entrypoint for LSP
- Status: Draft
- PR: TBD

## What
- [x] Add `veil lsp` as a first-class CLI subcommand.
- [x] Load the LSP server config through the existing CLI config loader.
- [x] Support `--config` and `--preset` so CLI and editor diagnostics share the same effective config path.
- [x] Keep the standalone `veil-lsp` binary behavior intact by preserving default-config stdio startup.

## Verification
- [x] `git diff --check` - pass
- [x] `cargo check -p veil-cli` - pass
- [x] `cargo test -p veil-cli cli_tests` - pass
- [x] `cargo test -p veil-lsp` - pass

## Evidence
- Local verification completed after rebasing onto `origin/main`.
- Commit under review: `4ff7a60`

## Non-goals
- [x] Do not add editor-specific workspace configuration discovery beyond the existing CLI config loader.
- [x] Do not change diagnostic rules, code actions, masking behavior, or LSP protocol capabilities.
- [x] Do not remove the standalone `veil-lsp` binary entrypoint.

## Rollback
- Revert this PR as a unit. The CLI `lsp` subcommand and injected-config server startup will be removed, while the previous standalone `veil-lsp` default startup path is restored.
