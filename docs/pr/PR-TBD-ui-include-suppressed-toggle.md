---
release: TBD
epic: A
pr: TBD
status: Draft
created_at: TBD
branch: feat/ui-include-suppressed-toggle
commit: 6772bc59782baa88b0896eeaa90f9cfbc887712f
title: Add include suppressed scan toggle
---

## SOT
- Title: Add include suppressed scan toggle
- Status: Draft
- PR: TBD

## What
- [x] Add an `Include Suppressed` toggle to the scan form.
- [x] Send `includeSuppressed: true` only when the toggle is enabled.
- [x] Keep Active Findings based on `effectiveFindings`, even when suppressed findings are included.
- [x] Add a Status column so suppressed findings are visually distinguishable from active findings.
- [x] Mark the #106 `includeSuppressed` UI toggle task complete.

## Verification
- [x] `npm ci` in `crates/veil-pro/frontend` — passed.
- [x] `npm run check` in `crates/veil-pro/frontend` — passed.
- [x] `npm run build` in `crates/veil-pro/frontend` — passed.
- [x] `git diff --check` — passed.

## Evidence
- [x] `crates/veil-pro/frontend/src/lib/ScanView.svelte` sends the typed `includeSuppressed` request flag.
- [x] `crates/veil-pro/frontend/src/lib/ScanView.svelte` renders Active/Suppressed status badges in scan results.
- [x] `docs/design/enterprise_jp_pii/implementation/task_breakdown.md` records the completed #106 toggle task.

## Non-goals
- [x] Do not change Local API response semantics.
- [x] Do not add limit reached / `coverageComplete` UI presentation.
- [x] Do not update frontend dependencies or address existing npm audit findings.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
