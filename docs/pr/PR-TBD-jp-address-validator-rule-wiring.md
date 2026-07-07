---
release: TBD
epic: A
pr: TBD
status: Draft
created_at: TBD
branch: feat/jp-address-validator-rule-wiring
commit: c21dca5f1eea211b41c859a034debd14e2b3eb95
title: Wire JP address validator into address rule
---

## SOT
- Title: Wire JP address validator into address rule
- Status: Draft
- PR: TBD

## What
- Wire `jp_address_prefecture_city_block` into the existing `pii.jp.address.prefecture_heuristic` rule.
- Keep the rule pattern unchanged and let the validator reject adjacent false positives after the regex match.
- Sync the validator setting across `crates/veil/rules/default/pii.toml`, `crates/veil/rules/default/pii_jp.toml`, and `crates/veil/rules_ja/default/pii_jp.toml`.
- Add scanner-level fixture coverage for accepted JP address forms and rejected label/code/version-like cases.
- Add a test that the built-in default JP address rule resolves the expected validator.

## Verification
- [x] `cargo fmt --all --check`
- [x] `git diff --check`
- [x] `cargo test -p veil-core jp_pii -- --nocapture`
- [x] `cargo test -p veil-core test_rules_from_fixtures -- --nocapture`
- [x] `cargo test -p veil-core --test jp_pii_fixture_tests -- --nocapture`
- [x] `cargo test -p veil-core validators -- --nocapture`
- [x] `cargo test -p veil-core jp_security_critical -- --nocapture`
- [x] `cargo clippy -p veil-core --all-targets --all-features -- -D warnings`

## Evidence
- `jp_pii_fixture_tests` passed with 3 tests, including built-in validator wiring.
- `validators` passed with 8 focused validator/resolver tests.
- `test_rules_from_fixtures` passed with the updated address false-positive fixture.
- `jp_security_critical` passed with 4 tests, confirming no critical security RulePack drift.

## Non-goals
- Do not broaden the JP address regex or add naked address detection.
- Do not change scores, severity, presets, or rule enablement.
- Do not modify name detection, MyNumber checksum behavior, or JP security critical boundaries.
- Do not touch UI, Local API, LSP, or evidence schemas.

## Rollback
- Revert this PR as a unit to remove the validator wiring and restore the previous fixture expectations.
