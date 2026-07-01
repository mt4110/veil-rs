# PR-100: PR-0 Contract Alignment

## SOT

- Contract decisions: `docs/design/enterprise_jp_pii/00_contract_decisions.md`
- Acceptance gate: `docs/design/enterprise_jp_pii/14_bulk_implementation_safety.md` section `14.4 Acceptance Gate`
- Generated schema output: repository root `schemas/`

## What

- Add Local API DTOs as Rust source and generate OpenAPI / JSON Schema from those DTOs.
- Align Local API, Evidence report, RunMeta, baseline fingerprinting, and schema validation to the v4.4 contract.
- Keep Evidence report raw-free with all findings, while Local API defaults to effective findings.
- Replace the PR-0 acceptance wrapper with a non-shell Python runner that only executes the 14.4 command list.

## Verification

```bash
cargo test --workspace
npm --prefix crates/veil-pro/frontend run build
cargo run -p veil-pro --bin export_local_api_schema -- --out-dir schemas
python scripts/check_generated_schemas.py
cargo run -p veil-cli -- verify tests/fixtures/evidence/golden.zip --require-complete
```

## Evidence

- `RunMeta.result.limitReasons` is required and `RunMeta.result.additionalProperties=false` is checked by `scripts/check_generated_schemas.py`.
- JSON Schema and OpenAPI internal `$ref` resolution are checked by `scripts/check_generated_schemas.py`.
- `SafeFindingApiV1.baselineFingerprint` is a required API field and remains distinct from `findingId`.

## Rollback

Revert this PR as a unit if generated DTO/schema contracts drift or the acceptance gate fails. Do not silently alter `00_contract_decisions.md`; contract changes require an explicit revision.
