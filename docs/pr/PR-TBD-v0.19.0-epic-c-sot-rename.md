---
release: v0.19.0
epic: C
pr: TBD
status: draft
created_at: 2026-01-19
branch: feat/v0.19.0-sot-autopilot-c
commit: 54ba9336d985620b1d3a337c8881b0edb324c15d
title: Implement veil sot rename
---

## Overview
Implement `veil sot rename` validation and logic to streamline SOT file naming upon PR creation, and improve CI SOT checks.

## Goals
- [x] Implement `veil sot rename --pr <N>`
- [x] Auto-detect SOT file if only one TBD triggers
- [x] Support dry-run and force
- [x] Update pr: field in front matter
- [x] Update `docs/pr/README.md` with new workflow
- [x] De-hardcode version in CI guard messages

## Non-Goals
- [ ] Strict content validation (Epic D)

## Design
### CLI
- `veil sot rename --pr 123`
- Finds `docs/pr/PR-TBD-*.md`
- Renames to `docs/pr/PR-123-*.md`
- Updates YAML frontmatter `pr: TBD` -> `pr: 123`

## Verification
- [x] `cargo test -p veil-cli` (All 6 tests passed)
- [x] Manual: Renamed generated SOT
- [x] Manual: Verified dry-run output

## Risks / Rollback
- Risks: Renaming wrong file or failing to update frontmatter.
- Rollback: `git mv` back and revert file content.

## Audit Notes
- Evidence:
  - CI logs
  - Automated tests covering collision and dry-run scenarios.
