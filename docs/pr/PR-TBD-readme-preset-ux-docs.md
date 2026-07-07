---
release: TBD
epic: A
pr: TBD
status: Draft
created_at: TBD
branch: feat/readme-preset-ux-docs
commit: 8859eed4b6afa341ad9efc1cf12bca8f45d9a6f5
title: Document JP preset UX in README
---

## SOT
- Title: Document JP preset UX in README
- Status: Draft
- PR: TBD

## What
- [x] Add JP preset quickstart guidance to `README.md`.
- [x] Add matching preset quickstart guidance to `README_EN.md`.
- [x] Document `veil init --preset`, `veil scan --preset`, `veil config dump --preset --layer preset`, and the `logs-jp` RulePack requirement.
- [x] Mention the dashboard preset selector, `Include Suppressed`, and coverage/limit UI indicators.
- [x] Mark the Phase 2 docs/README task complete in roadmap docs.

## Verification
- [x] `git diff --check` — passed.
- [x] `rg -n "JP Preset Quickstart|Japan Preset Quickstart|docs/README更新|Include Suppressed|coverageComplete" ...` — passed.

## Evidence
- [x] `README.md` and `README_EN.md` now expose the implemented preset UX.
- [x] `docs/design/enterprise_jp_pii/13_implementation_roadmap.md` marks `docs/README更新` complete.
- [x] `docs/design/enterprise_jp_pii/DETAIL_DESIGN.md` mirrors the roadmap completion state.

## Non-goals
- [x] Do not change CLI, Local API, or UI behavior.
- [x] Do not add `CoreConfig.preset`.
- [x] Do not implement wizard inference logic.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
