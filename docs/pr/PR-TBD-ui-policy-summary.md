---
release: TBD
epic: A
pr: TBD
status: Draft
created_at: TBD
branch: feat/ui-policy-summary
commit: 313f35d55ea645232c690d1e3828d6962bfa551e
title: Show policy summary in UI
---

## SOT
- Title: Show policy summary in UI
- Status: Draft
- PR: TBD

## What
- [x] Add UI contract types for `/api/policy`.
- [x] Add a `PolicyView` that fetches and renders the active policy summary.
- [x] Replace the governance placeholder with live policy data.
- [x] Mark the Phase 5 Policy explain roadmap item complete.

## Verification
- [x] `npm ci`
- [x] `npm run check`
- [x] `npm run build`
- [x] `git diff --check`

## Evidence
- `svelte-check` completed with `0 errors and 0 warnings`.
- Vite production build completed successfully.
- Policy UI reads `/api/policy` and displays effective rules, preset, org/repo config presence, layers, warnings, and conflicts.

## Non-goals
- [x] Do not change backend policy calculation.
- [x] Do not add policy editing.
- [x] Do not change Local API schemas.
- [x] Do not implement the broader Svelte state machine work.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
