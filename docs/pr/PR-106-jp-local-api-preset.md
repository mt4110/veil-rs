# PR-106: JP Local API preset support

## Why (Background)
- #104 added JP preset support to the CLI path.
- Local API scans still rejected implemented JP preset names, so UI/API scans could drift from CLI behavior.
- The `logs-jp` preset must not silently pass with zero findings when the log RulePack is absent.

## Summary
- Resolve Local API `ScanRequest.preset` values as built-in JP preset base config layers.
- Keep `CoreConfig.preset` out of scope; preset selection remains an API/CLI argument applied before repo/org/user overrides.
- Add the smallest frontend path needed to pass preset selection into `/api/scan`.

## Changes
- Local API accepts `standard-jp`, `fintech-jp`, `gov-jp`, `si-vendor-jp`, and `logs-jp`.
- Built-in preset layers are applied through `veil_config::apply_builtin_preset_as_base`.
- `logs-jp` scans require a valid `rules/log` RulePack and return guidance when it is missing.
- `logs-jp` required rule IDs are shared from `veil-config` to keep CLI and API behavior aligned.
- Frontend scan requests include the selected preset when it is not the default.
- Tests cover Local API preset score overrides and missing `logs-jp` RulePack guidance.

## Non-goals (Not changed)
- No `CoreConfig.preset` field.
- No address/name validators.
- No J-LIS MyNumber checksum implementation.
- No large UI workflow redesign.
- No new `severity` keys in JP preset TOML files; `base_score` remains the canonical score override.

## Impact / Scope
- CLI: no behavior change intended; shared `logs-jp` constants preserve the existing CLI guard.
- Local API: JP preset names now resolve to effective config instead of being rejected.
- UI: minimal preset selector added to the existing scan form.
- Docs: this SOT records the code PR boundary after the design SOT sync in #105.
- Rules: `logs-jp` continues to require the log RulePack.
- Tests: API and existing CLI/config tests exercise the shared behavior.

## Verification

### Commands
```bash
cargo fmt --all --check
python scripts/check_generated_schemas.py
cargo test -p veil-pro
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
npm --prefix crates/veil-pro/frontend run build
```

### Notes / Evidence
- `api::tests::local_api_fintech_preset_applies_base_score_override` verifies that `fintech-jp` base score overrides apply in the Local API scan path.
- `api::tests::scan_logs_preset_without_rule_pack_returns_guidance` verifies that `logs-jp` returns explicit guidance when `rules/log` is absent.
- Existing CLI preset tests pass after moving `LOGS_JP_REQUIRED_RULE_IDS` into `veil-config`.
- `scripts/check_generated_schemas.py` passes; generated schema output remains stable.
- `npm --prefix crates/veil-pro/frontend ci` was run locally before the frontend build to restore dependencies.

## Rollback
- Revert this PR as a unit to restore Local API preset rejection and remove the frontend preset selector while keeping the CLI preset support from #104.
