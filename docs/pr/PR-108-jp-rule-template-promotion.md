# PR-108: JP Rule Template Promotion

## SOT
- Date: 2026-07-04
- Branch: `feat/jp-template-promote-cli`
- Status: Draft

## What
- Store the 1000 generated JP security templates under `crates/veil/rules_ja/templates` as inactive template inventory.
- Add a deterministic parallel template loader for recursive TOML, regex, and validator validation.
- Add `veil rules promote-templates` to filter by `category`, `variant`, `severity`, and `score`, then write an executable RulePack.
- Keep promoted RulePacks outside the template root so inactive inventory and executable rules cannot be accidentally mixed.

## Verification
- `cargo fmt --all --check`
- `git diff --check`
- `python3` manifest/TOML check: `rows=1000 toml=1000 missing=0 extra=0`
- `cargo test -p veil-core rules::pack::`
- `cargo test -p veil-core rules::pack::tests::test_jp_security_templates_1000_loads_parallel -- --ignored`
- `cargo test -p veil-cli commands::rules::tests::`
- `cargo run -q -p veil-cli -- rules promote-templates --templates-dir crates/veil/rules_ja/templates/jp_security_templates_1000 --out-dir /tmp/veil-promoted-rules-dry-run --category finance --variant kv --severity critical --min-score 95 --dry-run`
- Promoted a 9-rule finance/kv/critical RulePack and verified it with `veil rules list`.
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `python scripts/check_generated_schemas.py`
- `npm --prefix crates/veil-pro/frontend run build` after `npm --prefix crates/veil-pro/frontend ci`

## Evidence
- Template corpus is inactive and not referenced by the default RulePack manifest.
- Promotion refuses empty selections, accidental all-template promotion without `--allow-all`, unsafe manifest paths, duplicate ids, non-empty output dirs without `--force`, and output dirs inside the template root.
- The PR was built from a clean `origin/main` worktree to avoid mixing unrelated local dirty changes.
