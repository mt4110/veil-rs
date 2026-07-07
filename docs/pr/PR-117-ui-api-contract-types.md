---
release: TBD
epic: A
pr: 117
status: Draft
created_at: TBD
branch: feat/ui-api-contract-types
commit: c6edb2ef6bfb1ccd05c3e6cd7edff3d2aac8eb5f
title: Add UI API contract types
---

## SOT
- Title: Add UI API contract types
- Status: Draft
- PR: 117

## What
- [x] Add `api-contract.ts` with the UI-facing subset of the generated Local API/OpenAPI contract.
- [x] Type `ScanView` request/response/findings against the Local API contract instead of `any` and inline request shapes.
- [x] Declare `Dashboard` props so `App.svelte` -> `Dashboard.svelte` type checking is explicit.
- [x] Mark the #106 UI client contract reflection task complete in the JP PII implementation breakdown.

## Verification
- [x] `npm run check` in `crates/veil-pro/frontend` — passed.
- [x] `npm run build` in `crates/veil-pro/frontend` — passed.
- [x] `git diff --check` — passed.

## Evidence
- [x] `crates/veil-pro/frontend/src/lib/api-contract.ts` mirrors the Local API types consumed by the scan UI.
- [x] `crates/veil-pro/frontend/src/lib/ScanView.svelte` consumes `ScanRequest`, `ScanResponse`, and `SafeFindingApiV1`.
- [x] `docs/design/enterprise_jp_pii/implementation/task_breakdown.md` records the completed #106 UI contract reflection task.

## Non-goals
- [x] Do not add the `includeSuppressed` UI toggle.
- [x] Do not change limit reached / `coverageComplete` presentation.
- [x] Do not add new frontend package dependencies or generated TypeScript tooling.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
