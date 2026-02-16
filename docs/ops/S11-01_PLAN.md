# S11-01 PLAN — STATUS.md Enforcement ("forget → fail")

## Goal
- Enforce a discipline: **S11 PR must update `docs/ops/STATUS.md`**, or the gate fails.
- Enforcement lives in **prverify** (veil-rs canonical gate).

## Non-Goals
- Redesigning STATUS format.
- Enforcing STATUS updates for non-S11 branches (scope is S11 first, expand later only if proven useful).

## Deliverables
- docs/ops/STATUS.md updated: S11-00=100% (Merged), S11-01=0% (start).
- docs/ops/S11-01_PLAN.md
- docs/ops/S11-01_TASK.md
- prverify change: fail when S11 branch diff does not include STATUS.md update.
- Deterministic tests for the enforcement logic (no network, no time dependency).
- Evidence: new prverify report committed under docs/evidence/prverify/.

## Definition of Done (DoD)
- On a branch matching `^s11-` (or equivalent CI head ref), prverify:
  - PASS when `docs/ops/STATUS.md` is modified in diff vs base.
  - FAIL (clear error + remediation) when missing.
- prverify behavior is deterministic:
  - Base ref resolution has stable fallback.
  - File list ordering is stable (sorted).
- `nix run .#prverify` passes on this PR, producing evidence file committed.

## Design (Pseudo-code)
- Inputs:
  - baseRefCandidates = ["origin/main", "main"]
  - branchName = (CI head ref if present) else `git rev-parse --abbrev-ref HEAD`

try:
  baseRef = first(baseRefCandidates where `git rev-parse --verify <ref>` succeeds)
catch:
  error("cannot resolve base ref; expected origin/main or main")

try:
  diffFiles = `git diff --name-only --diff-filter=ACMRT <baseRef>...HEAD`
  diffFiles = sort(diffFiles)
catch:
  error("cannot compute diff vs base ref")

if branchName does NOT match "^s11-":
  skip("status enforcement not applicable (non-S11 branch)")

if diffFiles is empty:
  skip("no changes vs base (nothing to enforce)")

# exceptions are intentionally minimal: this is a discipline gate
if "docs/ops/STATUS.md" NOT IN diffFiles:
  error(
    "S11 requires STATUS.md update, but diff lacks docs/ops/STATUS.md",
    remediation = [
      "edit docs/ops/STATUS.md to reflect current phase progress",
      "commit and re-run prverify"
    ]
  )

# Optional strictness (recommended but keep simple first):
try:
  statusText = `git show HEAD:docs/ops/STATUS.md`
catch:
  error("STATUS.md missing at HEAD (unexpected)")

if statusText does NOT contain "S11-":
  error("STATUS.md does not mention S11 (format drift)")

pass("STATUS.md updated")
