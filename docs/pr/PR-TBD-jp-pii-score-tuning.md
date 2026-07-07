---
release: TBD
epic: A
pr: TBD
status: Draft
created_at: TBD
branch: feat/jp-pii-score-tuning
commit: d60e912dd641f040cf6abab7cd7cfa02d53e3cf1
title: Tune JP PII negative test context scoring
---

## SOT
- Title: Tune JP PII negative test context scoring
- Status: Draft
- PR: TBD

## What
- [x] Add English `sandbox` and Japanese test-data context words to the default score-dampening list.
- [x] Cover the new context words in the scoring unit test so test/sample/dummy/mock/sandbox contexts remain deterministic.

## Verification
- [x] `cargo fmt --all --check` — passed.
- [x] `git diff --check` — passed.
- [x] `cargo test -p veil-core test_context_modifiers -- --nocapture` — passed.
- [x] `cargo test -p veil-core jp_pii -- --nocapture` — passed.
- [x] `cargo clippy -p veil-core --all-targets --all-features -- -D warnings` — passed.

## Evidence
- [x] Local unit and JP PII fixture tests verify the score-context behavior and existing Japanese PII detection fixtures.

## Non-goals
- [x] Do not rebalance JP PII rule base scores or grade thresholds.
- [x] Do not add broad Japanese context words such as `例`, because they are too likely to dampen real findings.
- [x] Do not change masking, validators, or rule matching behavior.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
