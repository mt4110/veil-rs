# Contract Fix Summary v4.4

v4.4 is the final pre-PR-0 contract synchronization pass. It consolidates ambiguity fixes and closes the last schema strictness issues before implementation.

## Contract fixes now locked

1. `scripts/check_generated_schemas.py` is the only generated-schema validation script name.
2. The implementation repo root `schemas/` is the schema output and tracking location.
3. JSON Schema generation uses `schemars`; OpenAPI generation uses `utoipa`.
4. `json-schema.report.json` embeds `$defs.SafeFindingApiV1` and does not depend on relative external refs.
5. `SeverityCounts` is a strict zero-filled object with required `Low`, `Medium`, `High`, `Critical` keys.
6. `suppressedSeverityCounts` and `coverageComplete` are required wherever `EvidenceSummary` appears.
7. New RulePack definitions use `base_score`; legacy `score` / `severity` are migration-only inputs.
8. Evidence ZIP baseline entry is `veil.baseline.json`; legacy `baseline.json` Evidence is unsupported and must be regenerated.
9. Evidence `report.json.findings` contains raw-free all findings, including `new` and `suppressed` when baseline is used.
10. `GET /api/runs/{runId}` returns full `RunMetaV1`, not a subset DTO.
11. `RunMeta.result.limitReasons` is required even when empty.
12. `RunMeta.result` is strict (`additionalProperties=false`) in JSON Schema and OpenAPI.
13. `RunMetaV1.extensions` is the only extension namespace; known v1 fields remain strict.
14. `baselineFingerprint` and `findingId` remain separate identifiers.
15. Acceptance gate commands are defined only in `14_bulk_implementation_safety.md` section `14.4`.
16. Design pack PR location is `docs/design/enterprise_jp_pii/`; `.private/` is only a working copy.

## Implementation start condition

Start with `PR-0 Contract Alignment`: Rust DTOs, schema generation, generated-schema diff checks, and the acceptance gate. No product feature work should begin before PR-0 proves the contract mechanically.
