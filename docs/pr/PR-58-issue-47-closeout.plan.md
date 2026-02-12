# PR58 — Issue #47 Closeout — evidence finalization & baseline “quiet main” confirmation

## Context

- PR56 merged ✅
- PR57 merged ✅
- All checks green ✅
- Dependabot open alerts = 0 ✅ (snapshot required)
- Goal: Close Issue #47 with final evidence pointers and a short closing note.
- Secondary goal: Confirm baseline “quiet main” (no warnings; reproducible green) with evidence pinned.

## Definitions

- “Evidence” = committed, pointerable artifacts that allow later re-check without guesswork.
- “Quiet main” = (a) clean working tree, (b) `nix run .#prverify` green, (c) no open Dependabot alerts, (d) no stray warnings in the evidence we pin.

## Scope

### In
- Add closeout SOT doc with:
  - Links: PR56, PR57
  - Evidence pointers:
    - dependabot open alerts snapshot (0)
    - latest prverify report on main (quiet main)
  - final closeout note (short)
- Add optional runbook note about Dependabot lag / snapshot discipline.
- Close Issue #47 with a final comment referencing PR56+PR57 and evidence.

### Out
- No code changes (unless runbook note touches docs only)
- No new CI logic / no policy changes

## Plan

### P0 — Path anchoring (determinism)
- Set repo root via `git rev-parse --show-toplevel`
- Use repo-root-relative canonical paths for Plan/Task/SOT + evidence folders.

### P1 — Evidence capture (RunAlways)
- Capture Dependabot open alerts count with `gh api ... --paginate` and store output in `.local/evidence/issue-47/`.
- Run `nix run .#prverify` on `main` and pin the generated report path.
- Record commit SHAs for PR56 and PR57 (for linking clarity).

### P2 — Documentation pinning
- Write SOT closeout note doc:
  - “Resolved by PR56 + PR57”
  - “Checks green”
  - “Dependabot open alerts = 0 (snapshot at <timestamp>)”
  - “quiet main confirmed by prverify report: <path>”
- (Optional) Add tiny runbook note: Dependabot can lag; always snapshot pre/post and re-check open=0.

### P3 — Issue close
- Comment Issue #47 with the short closing note (include PR links + evidence pointers).
- Close Issue #47 (or confirm already closed).

### P4 — PR hygiene
- Commit docs + evidence
- Open PR58 (docs/evidence-only)
- Merge (preferred merge method per repo policy)
- Verify on main once more if desired (fast sanity)

## Risk Controls

- IF `gh` is not authenticated -> stop; do not produce fake evidence.
- IF Dependabot endpoint errors / permissions -> record error output as evidence; do not claim open=0.
- IF prverify output not found -> stop and locate it; do not handwave.

## Done Definition

- SOT doc exists and points to:
  - dependabot open=0 snapshot file
  - prverify report file for quiet main
  - PR56/PR57 links
- Issue #47 has a final comment and is closed (or confirmed closed)
- All artifacts are committed and reviewable.
