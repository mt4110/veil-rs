---
release: TBD
epic: A
pr: TBD
status: Draft
created_at: TBD
branch: feat/interactive-atomic-write
commit: fc7024780cc65ee4e7716d98e867c134c1355151
title: Add interactive atomic write
---

## SOT
- Title: Add interactive atomic write
- Status: Draft
- PR: TBD

## What
- [x] Connect the interactive `mask` decision to file mutation.
- [x] Replace only the scanned target line with the existing safe `masked_snippet`.
- [x] Write masked content through a temp file, flush, preserve permissions, sync, and persist atomically.
- [x] Refuse to write when the target line changed after scan.
- [x] Preserve LF/CRLF endings for the replaced line.
- [x] Add unit and CLI coverage for scripted `mask\n`.
- [x] Mark the Phase 3 atomic write task complete in roadmap docs.

## Verification
- [x] `cargo fmt --all --check`
- [x] `cargo test -p veil-cli interactive_scan -- --nocapture`
- [x] `cargo test -p veil-cli scan_interactive -- --nocapture`
- [x] `git diff --check`

## Evidence
- Local interactive unit test result: `15 passed; 0 failed`.
- Local scripted interactive integration result: `2 passed; 0 failed`.
- Unit coverage asserts stale-line refusal, CRLF preservation, and already-masked idempotence.
- CLI coverage asserts `mask\n` redacts the file and removes the raw AWS key.
- SOT will be renamed after the PR number is assigned.

## Non-goals
- [x] Do not implement `ignore-line` or `skip-file` mutation.
- [x] Do not change non-interactive `veil scan` behavior.
- [x] Do not implement multi-finding batching or file-level grouping.
- [x] Do not change the masking algorithm or placeholder policy.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
