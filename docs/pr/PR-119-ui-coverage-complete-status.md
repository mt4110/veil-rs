---
release: TBD
epic: A
pr: 119
status: Draft
created_at: TBD
branch: feat/ui-coverage-complete-status
commit: c7e8b31249d2c315e16c55c41b75937272410de7
title: Show scan coverage completeness
---

## SOT
- Title: Show scan coverage completeness
- Status: Draft
- PR: 119

## What
- [x] Treat `status=incomplete`, `limitReached`, or `coverageComplete=false` as the scan UI incomplete state.
- [x] Show partial/full coverage in the scan result stat cards.
- [x] Render `limitReasons` as compact chips when a scan is incomplete.
- [x] Mark the #106 limit reached / `coverageComplete` UI task complete.

## Verification
- [x] `npm ci` in `crates/veil-pro/frontend` — passed.
- [x] `npm run check` in `crates/veil-pro/frontend` — passed.
- [x] `npm run build` in `crates/veil-pro/frontend` — passed.
- [x] `git diff --check` — passed.

## Evidence
- [x] `crates/veil-pro/frontend/src/lib/ScanView.svelte` consumes `coverageComplete`, `limitReached`, `limitReasons`, and API `status`.
- [x] `crates/veil-pro/frontend/src/lib/ScanView.svelte` displays a Coverage card and limit reason chips for partial scans.
- [x] `docs/design/enterprise_jp_pii/implementation/task_breakdown.md` records the completed #106 coverage UI task.

## Non-goals
- [x] Do not change Local API contract or scan semantics.
- [x] Do not change `includeSuppressed` request behavior.
- [x] Do not update frontend dependencies or address existing npm audit findings.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
