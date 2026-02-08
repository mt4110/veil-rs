---
release: v0.22.2
epic: D
pr: TBD
status: draft
created_at: 2026-02-08
branch: feat/v0.22.2-epic-d-kickoff
commit: 4786cd8bd2cebb6df03c92c173bdcf1d14d220e6
title: TBD
---

## Intent
- Kickoff **Epic D (v0.22.2)**: Establish sustainable Guardrail Operations & Exceptions.
- Define governance, schema, and UX for handling False Positives/Negatives.

> [!CAUTION]
> **Drift Check Constraints**:
> - This file (`PR-TBD-...`) is currently **ignored** by drift-check SOT detection.
> - If renamed to `PR-41-...`, it **MUST** include audit keywords (e.g. `SQLX_OFFLINE`) or `prverify` will FAIL.
> - Do not rename until PR41 is ready to merge and fully compliant.

## Change Summary
- **Epic Definition**: `docs/epics/EpicD_v0.22.2.md`
- **Runbook**: `docs/runbook/exceptions_v1.md` (Schema, Rules)
- **UX Guidelines**: `docs/runbook/guardrail-fail-ux.md` (Fail Template)
- **Corrections**: Docs corrected to match `driftError.Print` (Reason/Fix/Next) and PR42 enforcement timeline.

## Invariants kept
- **Green Verification**: `nix run .#prverify` remains green.

## Risk
- Low. Pure documentation update.

## Verification
- `nix run .#prverify` (PASS).
- Manual review of generated markdown files.
