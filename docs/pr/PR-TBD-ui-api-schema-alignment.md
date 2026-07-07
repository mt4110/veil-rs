---
release: TBD
epic: A
pr: TBD
status: Draft
created_at: TBD
branch: feat/ui-api-schema-alignment
commit: c9884f9cd02312bbec2e220e22b4cb064370d39b
title: Align UI API schema types
---

## SOT
- Title: Align UI API schema types
- Status: Draft
- PR: TBD

## What
- [x] Expand `api-contract.ts` from the scan/policy subset to the existing Local API DTO surface used by the UI.
- [x] Add typed `/api/me` auth context handling in `App.svelte`.
- [x] Align `ConfigConflict.shadowed` with the generated OpenAPI schema as `string[]`.
- [x] Keep policy conflict rendering stable for both known layer names and future string values.
- [x] Mark the Phase 5 API schema alignment roadmap item complete.

## Verification
- [x] `npm ci`
- [x] `npm run check`
- [x] `npm run build`
- [x] `python scripts/check_generated_schemas.py`
- [x] `git diff --check`

## Evidence
- `svelte-check` completed with `0 errors and 0 warnings`.
- Vite production build completed successfully.
- Generated schemas match tracked files and internal refs resolve.
- UI auth/session and policy conflict types now match the Rust DTO/OpenAPI naming contract.

## Non-goals
- [x] Do not add new Local API endpoints.
- [x] Do not change Rust DTO serialization.
- [x] Do not regenerate or hand-edit OpenAPI/JSON Schema files.
- [x] Do not add baseline, doctor, projects, or run metadata UI screens.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
