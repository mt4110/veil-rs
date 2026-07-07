---
release: TBD
epic: A
pr: TBD
status: Draft
created_at: TBD
branch: feat/ui-evidence-export-availability
commit: 95493211e2179a0f17e3fc9a77d153d776a38fa6
title: Show evidence export after scans
---

## SOT
- Title: Show evidence export after scans
- Status: Draft
- PR: TBD

## What
- [x] Move the Evidence ZIP export action to the shared scan results toolbar.
- [x] Keep the export action visible for zero-finding, violation, and incomplete scan results.
- [x] Add a download icon to the export button.
- [x] Mark the Phase 5 Evidence ZIP UX roadmap item complete.

## Verification
- [x] `npm ci`
- [x] `npm run check`
- [x] `npm run build`
- [x] `git diff --check`

## Evidence
- `svelte-check` completed with `0 errors and 0 warnings`.
- Vite production build completed successfully.
- The export button now depends on `scanStats` instead of `sortedFindings.length > 0`.

## Non-goals
- [x] Do not change Evidence ZIP generation or backend cache behavior.
- [x] Do not add filtering, search, or policy explain UI.
- [x] Do not change Local API schemas.
- [x] Do not address existing npm audit findings.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
