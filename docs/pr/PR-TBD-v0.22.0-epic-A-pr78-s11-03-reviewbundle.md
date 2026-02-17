# SOT — PR #78 — S11-03 Reviewbundle Go Hardening

## Scope
- PR: #78
- Epic: A
- Phase: S11-03
- Goal: reviewbundle Go hardening + deterministic contract (create/verify)

## Evidence
- prverify: docs/evidence/prverify/prverify_20260216T110132Z_9863b52.md

## Verification
- nix run .#prverify (PASS)

## Copilot Review Evidence (Captured)

This PR includes a machine-captured snapshot of Copilot review output for auditability and reviewer convenience.

- Source: GitHub API via `gh api` (REST)
- PR: #78
- Bound to PR Head SHA: (see meta file)
- Artifacts:
  - docs/pr/evidence/pr78/copilot.json
  - docs/pr/evidence/pr78/copilot.meta.txt
  - docs/pr/evidence/pr78/copilot.sha256
  - docs/pr/evidence/pr78/copilot.raw/ (raw API captures)

### Local Verification (Human)
- Inspect Copilot-only items:
  - `jq '.items_copilot[] | {kind,at,author,path,line}' docs/pr/evidence/pr78/copilot.json`
- Integrity check:
  - `shasum -a 256 docs/pr/evidence/pr78/copilot.json` (or `sha256sum`)
  - Compare with `docs/pr/evidence/pr78/copilot.sha256`


## Copilot Review Evidence (PR #78)

### Stored Artifacts
- docs/pr/evidence/pr78/copilot.json
- docs/pr/evidence/pr78/copilot.sha256
- docs/pr/evidence/pr78/copilot.meta.txt

### Capture Contract
- Purpose: Preserve Copilot review feedback as tamper-evident evidence for PR review/audit.
- Tamper-evidence:
  - copilot.sha256 binds the exact bytes of copilot.json.
  - copilot.meta.txt records capture command, PR HEAD SHA, timestamp, and tool versions.

### Notes / Triage
- MUST (PR #78 scope):
  - Fixpack: create must emit warnings.txt when warnings_count > 0 (contract compliance).
  - Any doc inconsistency that impacts audit reading (duplicate sections / numbering).
- DEFER (S11-04+):
  - Hermetic determinism tests (synthetic git repo) to remove dependency on fetched base refs.
  - Repo-agnostic knobs (BASE_REF, repo name) if/when we generalize beyond veil-rs.
