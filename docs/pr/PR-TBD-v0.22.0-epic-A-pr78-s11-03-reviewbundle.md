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

## Copilot Review Evidence
- Artifacts:
  - docs/pr/evidence/pr78/copilot.json
  - docs/pr/evidence/pr78/copilot.sha256
  - docs/pr/evidence/pr78/copilot.meta.txt
- Source: GitHub API via `gh api` (REST)
- Bound to PR Head SHA: (see meta file)

## Post-Fix Notes (C0/C1)

### C0: Copilot Evidence block dedupe
- Action: Removed duplicate Copilot evidence section to keep SOT single-source and reviewable.
- Result: One canonical Copilot evidence block remains, pointing to:
  - docs/pr/evidence/pr78/copilot.json
  - docs/pr/evidence/pr78/copilot.sha256
  - docs/pr/evidence/pr78/copilot.meta.txt

### C1: CI fetch origin/main (temporary stabilization)
- Action: Added a dedicated `git fetch origin main` step before `nix run .#go-test`.
- Why: Determinism tests rely on a base ref (format-patch base); default checkout may not include origin/main.
- Note: This is an operational patch; S11-04 hermetic refactor is the structural fix.
