# PR-110: JP PII normalization foundation

## SOT
This PR keeps the JP PII normalization layer narrow and deterministic:

- Generic JP normalization preserves the long vowel mark `ー`.
- Rules that want to treat `ー` as a separator must opt in at the regex level.
- Normalized matching continues to map findings back to original byte spans.
- Address validation starts as a helper contract, not a broader default detection rollout.

## What
- Preserve `ー` in `jp_normalize` while still normalizing fullwidth alphanumeric characters, fullwidth spaces, JP hyphen variants, colons, and parentheses.
- Add scanner coverage proving that MyNumber separator handling with `ー` requires explicit rule opt-in and still works with `jp_mynumber_len12`.
- Add `jp_address_prefecture_city_block` and register it in the validator resolver allowlist.
- Bound the address validator to the earliest prefecture match and to block-like address numbers inside the address segment.
- Add JP PII fixtures for fullwidth address positives and label-only negatives.

## Verification
```bash
cargo fmt --all --check
git diff --check
cargo test -p veil-core jp_normalize -- --nocapture
cargo test -p veil-core validators -- --nocapture
cargo test -p veil-core jp_pii -- --nocapture
cargo test -p veil-core jp_security_critical -- --nocapture
cargo test -p veil-core scan_content_requires_rule_opt_in_for_choonpu_separator -- --nocapture
cargo clippy -p veil-core --all-targets --all-features -- -D warnings
```

## Evidence
- SOT Guard requires this document because PR #110 changes `crates/` and `tests/fixtures/`.
- `jp_security_critical` tests are part of the validation set to ensure PR #109's opt-in RulePack boundary is unchanged.
- The address validator is intentionally not wired into default address rule behavior in this PR.

## Non-goals
- Do not broaden naked JP address detection.
- Do not wire the new address validator into default rules yet.
- Do not change PR #109 `jp_security_critical` templates or enable additional variants.
- Do not treat `ー` as a generic hyphen in normalization.

## Rollback
Revert PR #110 to remove the helper validator, fixture additions, and `ー` preservation test changes. Existing JP PII rules remain isolated because the address validator is not connected to default rules in this PR.
