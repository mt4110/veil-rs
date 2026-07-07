---
release: TBD
epic: A
pr: 116
status: Draft
created_at: TBD
branch: feat/jp-name-validator
commit: 09cf41da04dea80de8877ce043dd50b2f52dc2c1
title: Wire JP name validator into name rule
---

## SOT
- Title: Wire JP name validator into name rule
- Status: Draft
- PR: 116

## What
- [x] Add `jp_person_name_keyword` validator for labeled JP/EN person-name findings.
- [x] Wire `pii.person.name.keyword` to the validator in default JP rule packs.
- [x] Add positive and negative JP PII fixtures for labeled names, placeholders, and instruction text.
- [x] Add a built-in rule resolution test for the name validator.
- [x] Mark the Name validator item complete in the JP PII design/task docs.

## Verification
- [x] `cargo fmt --all --check` — passed.
- [x] `git diff --check` — passed.
- [x] `cargo test -p veil-core person_name -- --nocapture` — passed.
- [x] `cargo test -p veil-core jp_pii -- --nocapture` — passed.
- [x] `cargo test -p veil-core --all-features jp_pii -- --nocapture` — passed.
- [x] `cargo clippy -p veil-core --all-targets --all-features -- -D warnings` — passed.

## Evidence
- [x] `tests/fixtures/jp_pii/positive/person_name_keyword.txt` covers a valid labeled name.
- [x] `tests/fixtures/jp_pii/negative/name_labels_without_values.txt` covers empty labels, input instructions, and placeholder/dev values.

## Non-goals
- [x] Do not add unlabeled name detection.
- [x] Do not infer names without `pii.person.name.keyword` matching first.
- [x] Do not change scores or severity bands.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
