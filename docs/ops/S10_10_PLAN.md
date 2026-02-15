# S10-10: PRKit contract single-entry consolidation

## Goal
Converge all PRKit/Ritual entry points onto a single contract path:
"same contract -> same verification -> same evidence".

## Non-Goals
- Large redesign beyond PRKit contract boundary.
- Behavior changes not required for contract convergence.
- New features (only contract hardening + normalization).

## Hard Constraints
- No fiction: all referenced paths/symbols MUST be confirmed via rg/ls.
- Determinism first: if nondeterminism leaks into contract/evidence -> STOP and close contract first.
- Keep evidence noise low: store only necessary outputs.

## STOP Conditions
- Entry points cannot be enumerated deterministically (too many/unclear) -> STOP, define single canonical entry contract first.
- Contract requires new global behavior changes across unrelated packages -> STOP, narrow scope to PRKit boundary.
- Determinism cannot be closed without expanding surface area (e.g. requires changing dozens of files) -> STOP, isolate normalization layer first.

## Discovery (no fiction)
Pseudo:
- Run: ls/rg to confirm S10-10 docs exist.
- Run: rg for entry points, contract boundary, evidence writers.
- Record candidate paths (repo-relative) into Task.md (audit log).

Commands (minimal):
- ls -la docs/ops | rg '^S10_'
- rg -n 'S10_10|S10-10' docs -S
- rg -n 'package prkit|cmd/prkit|internal/prkit' -S
- rg -n 'func (main|Run|collect|evidence|portable|sot|contract)' internal cmd -S

## Decide contract shape (branching)
Pseudo:
- Identify all entry points:
  FOR each candidate file (confirmed: cmd/prkit/main.go, internal/prkit/run.go):
    - Confirm actual entry function symbols (via rg).
    - If file is CLI-only wrapper -> eligible to slim.
    - If file owns contract orchestration -> candidate canonical entry.
- Choose canonical entry:
  IF multiple orchestration flows exist (found: RunDryRun, RunExecuteMode, ScaffoldSOT in internal/prkit/run.go and sot.go):
    - Define one canonical contract function (symbol name: Run in internal/prkit/exec.go or similar).
    - Route all other flows into it.
  ELSE:
    - Keep as-is; only harden contract + normalize nondeterminism.

## Implementation (contract convergence)
Pseudo:
- Move/route CLI parsing so that:
  - cmd/prkit becomes minimal wrapper.
  - internal/prkit owns contract orchestration.
- Normalize nondeterminism:
  IF contract uses time/env/unstable ordering (found: time.Now in run.go, sot.go):
    - Prefer existing injection points (e.g. prkit.Now variable).
    - If none exist, introduce minimal injection ONLY within prkit boundary + tests.
  ELSE:
    - Continue.

## Verification
- go test ./... -count=1
- nix run .#prverify
- Save prverify report under docs/evidence/prverify/
- 
## S10-10 Fixpack (Post-PR Hardening): Evidence alignment + PR entry + Unicode scan

### Goal
Close remaining audit gaps:
- Evidence aligns with PR HEAD commit
- PR description points to correct SOT/Evidence paths
- Hidden/bidi unicode warning is verified (0) or fixed with evidence

### Hard Constraints
- NO FICTION: any path/symbol must be confirmed by rg/ls/test -f before editing docs.
- Minimal noise: record only necessary outputs as evidence.
- STOP if scope expands beyond docs/prkit boundary.

### STOP Conditions
- Cannot identify PR HEAD SHA deterministically -> STOP and resolve branch/PR state first.
- prverify output does not correspond to HEAD even after rerun -> STOP (investigate prverify generation path).
- Unicode scan reports non-trivial/uncertain characters that might be intentional -> STOP (manual review required).

### Pseudocode
1) Preflight
- Confirm repo clean, branch is PR #74 head branch, get HEAD SHA, get PR number.

2) Evidence Alignment
- Run go test and prverify on HEAD.
- Confirm generated prverify filename includes HEAD short SHA and file exists.
- Update:
  - docs/pr/PR-74-...md Evidence link (relative path)
  - docs/ops/S10_10_TASK.md Evidence link (relative path)
  - (optional) docs/ops/S10_10_PLAN.md note that evidence refreshed for HEAD

3) PR Entry Fix
- Update PR description to point to:
  - SOT: docs/pr/PR-74-...md
  - Evidence: docs/evidence/prverify/prverify_<UTC>_<HEAD7>.md
- Verify PR description no longer mentions PR-TBD.

4) Unicode Scan & Evidence
- Scan cmd/prkit/contract_test.go for:
  - bidi controls (RLO/LRO/isolates)
  - Cf/Cc control chars (except \n \r \t)
- Branch:
  IF scan result == 0:
    - Add evidence record (PR comment OR repo evidence file) stating scan passed (include command output).
  ELSE:
    - Remove offending chars, add commit, rerun scan => must be 0
    - Add evidence record describing what changed and why

5) Gates
- go test ./... -count=1
- (optional) nix run .#prverify if code changed in C3
- Ensure docs + PR + evidence align
