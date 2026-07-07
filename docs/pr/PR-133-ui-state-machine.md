---
release: TBD
epic: A
pr: 133
status: Draft
created_at: TBD
branch: feat/ui-state-machine
commit: 966c7e5f97b0a9bbfac87fb36bdf59306a056d92
title: Add UI state machine types
---

## SOT
- Title: Add UI state machine types
- Status: Draft
- PR: #133

## What
- [x] Add shared UI state machine types for auth, dashboard view, policy view, and scan view.
- [x] Replace App auth booleans with an explicit `AuthState`.
- [x] Type Dashboard navigation with `DashboardView`.
- [x] Replace PolicyView loading/error/data booleans with a `PolicyViewState`.
- [x] Keep ScanView on the same states while moving the state type to the shared module.
- [x] Mark the Phase 5 Svelte state machine roadmap item complete.

## Verification
- [x] `npm ci`
- [x] `npm run check`
- [x] `npm run build`
- [x] `git diff --check`

## Evidence
- `svelte-check` completed with `0 errors and 0 warnings`.
- Vite production build completed successfully.
- Auth, dashboard, policy, and scan view states now use named TypeScript unions.

## Non-goals
- [x] Do not change backend APIs.
- [x] Do not redesign scan result rendering.
- [x] Do not add new UI routes.
- [x] Do not implement API schema alignment.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
