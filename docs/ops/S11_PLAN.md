# S11_PLAN (S11-00 Kickoff: STATUS board pinned)

## Objective
Create a single canonical progress board:
- docs/ops/STATUS.md
And pin S11 workflow:
- docs/ops/S11_PLAN.md
- docs/ops/S11_TASK.md

## Definition of Done (DoD)
- STATUS.md exists and contains:
  - Milestone table covering S11..S15
  - Progress % fields
  - "Update Checklist" rule that forces every PR to update STATUS
- S11_PLAN/TASK explicitly encode "non-stalling design":
  - No zsh glob assumptions (commands must not rely on shell-specific glob)
  - Env-get patterns are explicit (repo-root resolution)
  - One command => one artifact (no hidden side effects)

## Non-stalling design constraints
- Always resolve repo root via: `git rev-parse --show-toplevel`
- Never depend on zsh-only behavior (glob qualifiers, nomatch, etc.)
- Prefer `bash -lc '...'` for deterministic, contained commands
- Each command should produce a single observable result:
  - create/update exactly one file OR
  - run one verification gate

## Plan (Pseudo-code)

PHASE 0: Preflight
  repo_root = git rev-parse --show-toplevel
  IF repo_root is empty: STOP (not a git repo)
  IF git status is dirty: STOP (must start clean)

PHASE 1: Branch
  branch = "s11-00-status-board-v1"
  IF current branch != branch:
    create/switch branch

PHASE 2: Create artifacts (1 command => 1 artifact)
  create docs/ops/STATUS.md
  create docs/ops/S11_PLAN.md
  create docs/ops/S11_TASK.md

PHASE 3: Sanity checks (no stall)
  Ensure files are tracked under git
  Ensure no accidental placeholders leak:
    - "TBD" is allowed, but must be explicit (not implicit missing)
  Ensure STATUS table row order is stable (S11..S15)

PHASE 4: Gates
  Run repo’s canonical verification (no new gates invented):
    - nix run .#prverify  (if available in this repo)
    - otherwise: run the established minimal doc gate(s)
  IF any gate fails: STOP (fix then rerun)

PHASE 5: Commit
  git add docs/ops/STATUS.md docs/ops/S11_PLAN.md docs/ops/S11_TASK.md
  commit message: "docs(ops): add STATUS board and S11 kickoff plan/task"

PHASE 6: Push & PR
  push branch
  open PR

EXIT CONDITIONS
  - Branch pushed
  - PR created
  - Evidence link recorded in STATUS.md "Last Updated"

