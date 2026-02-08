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

## Change Summary
- **Epic Definition**: `docs/epics/EpicD_v0.22.2.md`
- **Runbook**: `docs/runbook/exceptions_v1.md` (Schema, Rules)
- **UX Guidelines**: `docs/runbook/guardrail-fail-ux.md` (Fail Template)

## Invariants kept
- **Green Verification**: `nix run .#prverify` remains green.

## Risk
- Low. Pure documentation update.

## Verification
- `nix run .#prverify` (PASS).
- Manual review of generated markdown files.
