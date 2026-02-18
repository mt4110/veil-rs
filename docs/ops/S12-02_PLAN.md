# S12-02 PLAN — Closeout + Ritual Spec (zsh-safe)

## Scope (One-liner)
Close S12-01 in STATUS (truth sync) and define strict/observation ritual that never breaks in zsh (no-glob nomatch).

## Non-Negotiable Rules
- No exit/return non0/set -e/trap EXIT. Never rely on termination for control.
- Failures are recorded as text: OK/ERROR/SKIP. Continue safely.
- Heavy work must be split. This phase is doc-first (no heavy runs required).

## Inputs (Truth Sources)
- docs/ops/STATUS.md is the ONLY canonical progress board.
- S12-01 is merged (PR #83), so STATUS must become 100% for S12-01.

## Outputs (Deliverables)
- docs/ops/S12-02_PLAN.md
- docs/ops/S12-02_TASK.md
- docs/ops/STATUS.md updated (S12-01=100%, S12-02 started)
- (optional) docs/evidence/ops/obs_YYYYMMDD_s12-02.md

## Pseudocode (stopless / deterministic)

STATE:
  ok = true
  repo_root = git rev-parse --show-toplevel
  status = docs/ops/STATUS.md

IF repo_root is empty:
  PRINT "ERROR: not in git repo"
  ok = false

IF ok == true:
  # STEP 1: Observe (no heavy)
  PRINT "OK: observe git status/log"
  # NOTE: do not use shell glob patterns that can fail on zsh nomatch

IF ok == true:
  # STEP 2: Decide evidence pointer for S12-01 merged truth
  # Prefer a repo-tracked prverify report in docs/evidence/prverify/
  evidence_path = "<best-effort>"
  IF evidence_path not found:
    PRINT "SKIP: evidence path not resolved; keep existing"

IF ok == true:
  # STEP 3: Patch STATUS truth
  # - Set S12-01 Progress => 100% (Merged), Current => "-"
  # - Start S12-02 row (1% or 99% depending on PR stage)
  # - Update Last Updated (Date/By/Agent/Evidence) deterministically
  IF S12-01 row not found:
    PRINT "ERROR: S12-01 row missing"
    ok = false
  ELSE:
    APPLY minimal edits only

IF ok == true:
  # STEP 4: Write S12-02 ritual spec (zsh-safe)
  # Define: observation snippet uses find -name 'pattern' not glob
  PRINT "OK: write ritual guidance to docs/ops/S12-02_TASK.md"

IF ok == true:
  # STEP 5: Record light evidence (optional)
  # Put observed outputs into docs/evidence/ops/obs_...md
  PRINT "SKIP or OK: evidence note"

END:
  IF ok == true:
    PRINT "OK: S12-02 plan complete"
  ELSE:
    PRINT "ERROR: S12-02 plan incomplete (see logs)"
