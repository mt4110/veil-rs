# PR-TBD: JP critical security RulePack

## SOT
- Date: 2026-07-04
- Branch: `feat/jp-critical-template-rulepack`
- Status: Draft

## What
- Add an opt-in `jp_security_critical` RulePack under `crates/veil/rules_ja/packs`.
- Promote 37 critical `secret` / `finance` `kv` templates from the inactive `jp_security_templates_1000` corpus.
- Add fixture-backed tests for representative hits and label-only false positives.
- Document the pack boundary and the exact promotion command.

## Why
- #108 added the promotion mechanism and inactive template corpus.
- The next stable step is a small executable RulePack that can be loaded explicitly without expanding defaults.
- `kv` is promoted first because enabling both `kv` and `lv` produces duplicate findings on ordinary `key: value` lines.

## Non-goals
- No default RulePack expansion.
- No `lv`, `schema`, or `leak` promotion.
- No address/name validators.
- No `CoreConfig.preset` field.
- No preset `severity` changes.

## Verification
- `cargo fmt --all --check`
- `git diff --check`
- `cargo test -p veil-core jp_security_critical -- --nocapture`
- `cargo test -p veil-core rules::pack::`
- `cargo test -p veil-cli commands::rules::tests::`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `python scripts/check_generated_schemas.py`
- `python` pack manifest/TOML consistency check: `rows=37 toml=37 manifest_files=37 missing=0 extra=0 manifest_missing=0 manifest_extra=0`
- `npm --prefix crates/veil-pro/frontend ci`
- `npm --prefix crates/veil-pro/frontend run build`

## Evidence
- Template corpus remains inactive and separate from executable packs.
- `rules list` loads 37 promoted `log.jp.*` rules when `[core].rules_dir` points to `jp_security_critical`.
- Positive fixtures use dummy values only.
- Negative fixtures assert label-only text does not trigger promoted rules.
