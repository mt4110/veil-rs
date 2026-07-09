---
release: TBD
epic: A
pr: TBD
status: Ready
created_at: 2026-07-09
branch: feat/docs-taxonomy-cleanup
commit: TBD
title: Add public docs taxonomy
---

## SOT
- Title: Add public docs taxonomy
- Status: Ready
- PR: TBD

## What
- [x] Add `docs/README.md` as the public docs portal and taxonomy entry point.
- [x] Add `docs/SSOT.md` as the current public product SSOT.
- [x] Add status headers to existing public entry docs so active/reference/archive intent is explicit.
- [x] Add `scripts/check_docs_taxonomy.py` to verify every public Markdown document is classified.
- [x] Remove absolute local path links and private document links from public entry docs.
- [x] Keep private design notes out of the public docs tree.

## Verification
- [x] `python3 scripts/check_docs_taxonomy.py` - PASS
- [x] `rg -n "/Users/masakitakemura|\\.private" docs/README.md docs/SSOT.md docs/cli/README.md docs/runbook/exception-registry.md docs/ai/SSOT.md docs/ops/STATUS.md docs/pr/README.md docs/dogfood/README.md scripts/check_docs_taxonomy.py` - no matches
- [x] `git diff --check` - PASS

## Evidence
- Local command output observed on 2026-07-09 in `feat/docs-taxonomy-cleanup`.
- CI evidence will be the GitHub pull request checks for this branch.

## Non-goals
- [x] Does not publish `.private/` or `.design/` documents.
- [x] Does not rewrite historical evidence logs that contain old absolute build paths.
- [x] Does not change product behavior or generated schema contracts.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
