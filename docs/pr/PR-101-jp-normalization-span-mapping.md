# PR-101: JP Normalization and Span Mapping

## SOT

- Contract decisions: `docs/design/enterprise_jp_pii/00_contract_decisions.md`
- JP PII detection strategy: `docs/design/enterprise_jp_pii/04_jp_pii_detection_strategy.md`
- Testing strategy: `docs/design/enterprise_jp_pii/12_testing_strategy.md`
- Acceptance gate: `docs/design/enterprise_jp_pii/14_bulk_implementation_safety.md` section `14.4 Acceptance Gate`

## What

- Add JP text normalization for fullwidth alphanumerics, fullwidth spaces, JP hyphen variants, colons, and fullwidth parentheses.
- Add normalized-to-original byte span mapping so normalized detection can still mask the original source range.
- Add internal `FindingSpan` and UTF-16 `Range` fields to core findings without exposing them through serialized finding output.
- Keep raw rule matching first, then add normalized matching with duplicate original-span suppression.
- Add a JP PII fixture policy before expanding positive/negative fixture coverage.

## Verification

```bash
cargo test --workspace
npm --prefix crates/veil-pro/frontend run build
python scripts/check_generated_schemas.py
cargo run -p veil-cli -- verify tests/fixtures/evidence/golden.zip --require-complete
```

## Evidence

- `scanner::jp_normalize` unit tests cover fullwidth/separator normalization and normalized span mapping back to original bytes.
- `scanner::tests::scan_content_detects_normalized_jp_pii_with_original_span` verifies normalized JP PII detection, original byte span masking, and UTF-16 range calculation.
- `scripts/check_generated_schemas.py` passes, confirming the new internal span fields do not alter tracked Local API / Evidence schemas.
- Frontend build required local dependency restoration with `npm ci`; the tracked frontend files were not changed.

## Rollback

Revert this PR as a unit if normalized matching causes unacceptable rule compatibility drift, duplicate findings, or span/range instability. Keep the PR-0 contract documents intact; contract changes require an explicit revision.
