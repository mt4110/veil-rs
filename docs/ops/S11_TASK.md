# S11_TASK (S11-00 Kickoff: STATUS board pinned)

## Progress
- S10: 100%
- S11-00: 0% -> 99% (Review) -> 100% (Merged)

## Task (Deterministic Order)

### 0) Preflight (clean start)
- [ ] Run (single output: show status)
  - `bash -lc 'cd "$(git rev-parse --show-toplevel)" && git status -sb'`
- [ ] Confirm ops S11 files do not exist yet (single output: grep list)
  - `bash -lc 'cd "$(git rev-parse --show-toplevel)" && ls -1 docs/ops | rg -n "^S11_" || true'`

### 1) Branch
- [ ] Create/switch branch (single result: branch set)
  - `bash -lc 'cd "$(git rev-parse --show-toplevel)" && git switch -c s11-00-status-board-v1 2>/dev/null || git switch s11-00-status-board-v1'`

### 2) Create artifacts (1 command => 1 file)
- [ ] Create `docs/ops/STATUS.md`
- [ ] Create `docs/ops/S11_PLAN.md`
- [ ] Create `docs/ops/S11_TASK.md`

### 3) Content sanity (no stall)
- [ ] Ensure files are tracked (single output: list tracked)
  - `bash -lc 'cd "$(git rev-parse --show-toplevel)" && git ls-files docs/ops/STATUS.md docs/ops/S11_PLAN.md docs/ops/S11_TASK.md'`
- [ ] Ensure STATUS contains S11..S15 rows (single output: rg hits)
  - `bash -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "^\| S11-|^\| S12|^\| S13|^\| S14|^\| S15" docs/ops/STATUS.md'`

### 4) Gates (use canonical project gate)
- [ ] Gate: prverify (single result: PASS/FAIL)
  - `bash -lc 'cd "$(git rev-parse --show-toplevel)" && nix run .#prverify'`
  - If this repo does not have prverify: replace with the repo’s standard doc/test gate and keep it stable.

### 5) Commit
- [ ] Stage (single result: index updated)
  - `bash -lc 'cd "$(git rev-parse --show-toplevel)" && git add docs/ops/STATUS.md docs/ops/S11_PLAN.md docs/ops/S11_TASK.md'`
- [ ] Commit (single result: commit created)
  - `bash -lc 'cd "$(git rev-parse --show-toplevel)" && git commit -m "docs(ops): add STATUS board and S11 kickoff plan/task"'`

### 6) Push & PR
- [ ] Push (single result: remote updated)
  - `bash -lc 'cd "$(git rev-parse --show-toplevel)" && git push -u origin s11-00-status-board-v1'`

## After Merge (Operational Rule)
- Every PR must update `docs/ops/STATUS.md`:
  - Progress % and Current row(s)
  - Last Updated (Date/By/Evidence)

