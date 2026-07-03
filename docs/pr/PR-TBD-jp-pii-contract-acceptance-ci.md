# PR-TBD: JP PII contract acceptance CI gate

## Why (Background)
- #106 merged Local API and minimal UI support for JP preset selection.
- The JP PII SOT still marked the contract acceptance gate and Local API/UI preset support as unfinished.
- Previous review feedback correctly required CI enforcement before marking schema drift / acceptance gates complete.

## Summary
- Add a `contract-acceptance` CI job that invokes `python scripts/check_contract_acceptance.py`.
- Install Rust, Node, frontend dependencies, Python 3.12, and pinned `PyYAML` so the CI gate is reproducible.
- Sync JP PII roadmap/task breakdown text after #106 without marking unfinished UI completion work as done.

## Changes
- `.github/workflows/ci.yml` now runs the contract acceptance wrapper after the stable job succeeds.
- `13_implementation_roadmap.md` and `DETAIL_DESIGN.md` now mark the acceptance gate and #106 preset path complete.
- `implementation/task_breakdown.md` now marks Local API preset resolution, `logs-jp` RulePack enforcement, and the minimal UI request path complete.
- Generated UI client alignment, `includeSuppressed` UI, and `coverageComplete` UI remain unchecked.

## Non-goals (Not changed)
- No `CoreConfig.preset` field.
- No address/name validators.
- No J-LIS MyNumber checksum implementation.
- No score calibration.
- No large UI workflow redesign.

## Verification

### Commands
```bash
cargo fmt --all --check
python scripts/check_generated_schemas.py
cargo run -p veil-cli -- verify tests/fixtures/evidence/golden.zip --require-complete
npm --prefix crates/veil-pro/frontend run build
python scripts/check_contract_acceptance.py
```

## Rollback
- Revert this PR as a unit to remove the CI gate and restore the SOT checkboxes to their pre-#106 follow-up state.
