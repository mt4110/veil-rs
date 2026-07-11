---
release: TBD
epic: Enterprise JP PII
pr: TBD
status: Draft
created_at: 2026-07-11
branch: feat/rulepack-update-flow
commit: TBD
title: Add RulePack update flow
---

# SOT: Add RulePack Update Flow

## SOT
- Title: Add RulePack update flow
- Status: Draft
- PR: TBD

## What
- [x] Add a v1 RulePack update runbook for staging, verification, atomic promotion, and rollback.
- [x] Tie the update flow to the implemented offline `pinned_digests` verification gate.
- [x] Document that v1 does not perform automatic network updates.
- [x] Mark only the RulePack update flow Phase 7 task complete.

## Verification
- [x] `python3 scripts/check_docs_taxonomy.py` - PASS
- [x] `git diff --check` - PASS
- [x] `rg -n "automatic network|remote_rules_url|pinned_digests|atomic promote|rules_dir|upload" ...` - PASS / reviewed safety wording

## Evidence
- The flow keeps `rules_dir` activation explicit and reviewable.
- The flow does not add a network downloader or mutate active packs in place.
- CLI, LSP, and Local Audit UI continue to share the same RulePack loader gate.

## Non-goals
- [x] Do not implement `veil rules update`.
- [x] Do not enable `remote_rules_url` by default.
- [x] Do not implement public-key or TOFU trust models.
- [x] Do not change scanner behavior.

## Rollback
- Revert this PR as a unit to remove the update-flow runbook and roadmap status update.
