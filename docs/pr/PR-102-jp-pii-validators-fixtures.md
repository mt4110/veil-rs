# PR-102: JP PII validators and fixture harness

## Why
- PR #101 made JP normalization span-safe, but detection still needed semantic filtering.
- JP PII rules should reject common false positives before presets make the rules operational.
- The validator contract must be documented and tested before preset and CLI UX layers depend on it.

## Summary
- Add deterministic validator resolution for TOML rule packs.
- Add the first JP PII validators and a fixture harness covering both positive and negative cases.
- Update the JP PII detection strategy SOT to match the PR #101 implementation contract.

## Changes
- Add `validator_id` to loaded rules and resolve TOML `validator` names through an allowlisted registry.
- Fail rule-pack loading for unknown validators instead of silently ignoring them.
- Add `jp_mynumber_len12`, `jp_phone_mobile`, and `luhn` validators.
- Reject known test card numbers by default.
- Add positive JP PII fixtures with expected rule IDs.
- Add negative fixtures for order numbers, version numbers, dummy/test/example context, fullwidth non-JP secrets, and test cards.

## Non-goals
- No preset TOML or preset override resolver.
- No `veil scan --preset` or `veil init --preset` CLI UX.
- No full J-LIS MyNumber check digit validation.
- No address or name validators.

## Impact / Scope
- CLI: filter and scan paths now honor validators through loaded rules.
- CI: adds fixture coverage and schema drift verification.
- Docs: updates the enterprise JP PII detection strategy.
- Rules: JP MyNumber, JP mobile phone, and credit-card rules get validator bindings.
- Tests: adds validator unit tests and JP PII positive/negative fixture tests.

## Verification

### Commands
```bash
cargo fmt --all --check
cargo test -p veil-core scanner::
cargo test -p veil-core
cargo test -p veil-cli filter_load_rules_pack
cargo test -p veil-cli --test filter_config_test
cargo test -p veil-cli --test json_contract_tests
python scripts/check_generated_schemas.py
cargo run -p veil-cli -- verify tests/fixtures/evidence/golden.zip --require-complete
npm --prefix crates/veil-pro/frontend run build
cargo test --workspace
```

### Evidence
- Local validation passed before opening the draft PR.
- GitHub SOT Guard requires this file because the PR changes code.

## Rollback
- Revert this PR to remove validator registry wiring, validator-bound rules, and JP PII fixtures together.
