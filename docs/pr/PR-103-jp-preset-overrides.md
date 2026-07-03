# PR-103: JP preset override resolver

## Why
- JP PII validators become operational only when teams can choose a scenario-oriented preset.
- The preset data contract should be narrow and deterministic before exposing CLI UX.
- Keeping presets to rule overrides makes review, rollback, and future expansion easier.

## Summary
- Add built-in JP preset TOML files.
- Add a preset resolver in `veil-config`.
- Apply preset rule overrides as a base config layer while preserving repo override behavior.

## Changes
- Add `standard-jp`, `fintech-jp`, `gov-jp`, `logs-jp`, and `si-vendor-jp` preset TOML files.
- Limit preset TOML to `enabled` and `base_score`.
- Reject unknown fields in preset TOML.
- Add `builtin_preset_config` and `apply_builtin_preset_as_base`.
- Apply `RuleConfig.base_score` to resolved built-in and rule-pack rules.
- Cover unknown preset IDs, forbidden fields, enabled filtering, base score overrides, and `logs-jp` with the log rule pack.

## Non-goals
- No `veil scan --preset` CLI UX.
- No `veil init --preset` CLI UX.
- No preset severity field; score remains the source of truth.
- No preset-controlled `rules_dir`, output settings, or core include/ignore settings.

## Impact / Scope
- CLI: no user-facing preset command in this PR.
- CI: adds focused preset resolver tests.
- Docs: SOT only.
- Rules: preset files adjust rule enablement and base scores only.
- Tests: config and core resolver behavior.

## Verification

### Commands
```bash
cargo fmt --all --check
cargo test -p veil-config presets
cargo test -p veil-config
cargo test -p veil-core rules::builtin::tests
cargo test -p veil-core
python scripts/check_generated_schemas.py
cargo test --workspace
```

### Evidence
- Local validation passed before opening the draft PR.
- This PR is stacked on PR #102.

## Rollback
- Revert this PR to remove preset TOML files and resolver behavior while keeping validator support from PR #102.
