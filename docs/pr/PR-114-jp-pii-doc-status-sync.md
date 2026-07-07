---
release: TBD
epic: A
pr: 114
status: Draft
created_at: TBD
branch: feat/jp-pii-doc-status-sync
commit: 7739ab94661048f7061b0901519750d846a9161d
title: Sync JP PII implementation status docs
---

## SOT
- Title: Sync JP PII implementation status docs
- Status: Draft
- PR: 114

## What
- [x] Mark the JP address validator as implemented and wired after #112.
- [x] Mark Phase 1 score tuning as complete for the #113 negative-context dampening scope.
- [x] Keep `Name validator` and `jp_mynumber_checksum` unchecked because they are still follow-up work.

## Verification
- [x] `git diff --check` — passed.
- [x] `rg -n "Address validatorは未実装|Address validatorを実装する。現状|validatorなしの住所ヒューリスティック|\[ \] score調整" docs/design/enterprise_jp_pii/04_jp_pii_detection_strategy.md docs/design/enterprise_jp_pii/13_implementation_roadmap.md docs/design/enterprise_jp_pii/DETAIL_DESIGN.md docs/design/enterprise_jp_pii/implementation/task_breakdown.md` — no matches.
- [x] `cargo run -p veil-cli -- sot --help` — passed.

## Evidence
- [x] Docs now reference #112 address validator wiring and #113 score context tuning in the implementation status files.

## Non-goals
- [x] Do not change runtime behavior.
- [x] Do not mark the Name validator or J-LIS MyNumber checksum as complete.
- [x] Do not rebalance JP PII base scores or grade thresholds.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
