# PR-104: JP preset CLI UX

## Why
- Operators need a direct way to apply JP presets during scan and init workflows.
- Preset behavior must remain explainable through config layer inspection.
- CLI UX should be added only after validator and preset resolver contracts are stable.

## Summary
- Add CLI flags for applying built-in presets.
- Make preset config visible as its own layer.
- Keep the preset as a base layer so user, org, and repo config can override it.

## Changes
- Add `veil scan --preset <ID>`.
- Add `veil init --preset <ID>`.
- Add `veil config dump --preset <ID> --layer preset`.
- Allow `veil config dump --preset <ID> --layer effective` to show merged results.
- Use `logs-jp` to generate the log rule pack during init.
- Add CLI tests for preset scan scoring, unknown preset failure, config dump visibility, and init output.

## Non-goals
- No new preset fields beyond PR #103.
- No server/API preset execution beyond the existing explicit rejection.
- No extra preset policy layer semantics beyond base-layer config merge.

## Impact / Scope
- CLI: scan, init, and config dump gain preset flags.
- CI: adds CLI tests for preset UX and config layer behavior.
- Docs: SOT only.
- Rules: no direct rule changes in this PR.
- Tests: CLI-focused integration tests.

## Verification

### Commands
```bash
cargo fmt --all --check
cargo test -p veil-cli
cargo test -p veil-core scanner::
cargo test --workspace
python scripts/check_generated_schemas.py
cargo run -p veil-cli -- verify tests/fixtures/evidence/golden.zip --require-complete
npm --prefix crates/veil-pro/frontend run build
```

### Evidence
- Local validation passed before opening the draft PR.
- This PR is stacked on PR #103.

## Rollback
- Revert this PR to remove preset CLI UX while keeping preset resolver support from PR #103.
