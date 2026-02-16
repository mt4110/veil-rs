---
release: v0.19.0
epic: B
pr: TBD
status: draft
created_at: 2026-01-16
branch: feat/v0.19.0-sot-autopilot-b
commit: 5a9ee301d228344376c810b23bdd9d62db084b17
title: UX Improvements for SOT (Templates & CI)
---

## Overview
Improve SOT friction by updating PR templates.

## Goals
- [x] Update PR templates to recommend manual SOT creation
- [x] Update `docs/pr/README.md`
- [x] Implement helpful CI SOT check (`pr_sot_guard.yml`)
- [x] Use SHA-based comparison in CI for stability
- [x] Ensure grep safety in CI script

## Non-Goals
- [ ] Rename SOT command (handled in Epic C)

## Design
### PR Templates
Replaced `cat` instructions with manual steps in:
- `00_default.md`
- `10_epic.md`
- `20_release.md`
- `30_hotfix.md`
- `40_docs.md`

### CI Guard
New workflow `.github/workflows/pr_sot_guard.yml`:
- Compares `base_sha` vs `head_sha`.
- Filters out `docs/` and `.github/` changes.
- Fails if code changes exist but no SOT in `docs/pr/` is part of the diff.
- Error message provides copy-pasteable manual creation instructions.

## Verification
- [x] `cargo test -p veil-cli` (Existing)
- [x] Manual: Generated SOT for Epic B
- [x] Manual: Verified PR templates content locally
- [x] Manual: Simulated CI failure cases locally (via script logic)

## Risks / Rollback
- Risks: CI might block valid PRs if logic is buggy.
- Rollback: Revert `pr_sot_guard.yml` or disable SOT check temporarily.

## Audit Notes
- Evidence:
  - Generated SOT file under docs/pr
  - CI logs referencing SOT path
