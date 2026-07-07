---
release: TBD
epic: A
pr: 115
status: Draft
created_at: TBD
branch: feat/jp-mynumber-checksum
commit: 271da44ecbe6b4f763c322884455cfd0c1f77eeb
title: Add JP MyNumber checksum feature flag
---

## SOT
- Title: Add JP MyNumber checksum feature flag
- Status: Draft
- PR: 115

## What
- [x] Add a default-off `jp_mynumber_checksum` feature to `veil-core`.
- [x] Forward the feature from `veil-cli` to `veil-core`.
- [x] Keep `jp_mynumber_len12` backward-compatible by enforcing checksum only when the feature is enabled.
- [x] Update JP PII positive fixtures/examples to use checksum-valid MyNumber samples.
- [x] Mark the design/task docs item complete while keeping the Name validator follow-up unchecked.

## Verification
- [x] `cargo fmt --all --check` — passed.
- [x] `git diff --check` — passed.
- [x] `cargo test -p veil-core mynumber -- --nocapture` — passed.
- [x] `cargo test -p veil-core --features jp_mynumber_checksum mynumber -- --nocapture` — passed.
- [x] `cargo test -p veil-core --all-features jp_pii -- --nocapture` — passed.
- [x] `cargo clippy -p veil-core --all-targets --all-features -- -D warnings` — passed.
- [x] `cargo check -p veil-cli --all-features` — passed.

## Evidence
- [x] Feature-enabled tests reject `1234-5678-9012` and accept checksum-valid `1234-5678-9018`.
- [x] JP PII fixtures pass with all features enabled.

## Non-goals
- [x] Do not enable checksum validation by default.
- [x] Do not rename the existing `jp_mynumber_len12` validator id.
- [x] Do not implement the Name validator.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
