# S11-01 TASK — STATUS.md Enforcement ("forget → fail")

## Phase 0 — Safety / Truth snapshot
- [ ] `cd "$(git rev-parse --show-toplevel)"`
- [ ] `git status -sb` (record clean/dirty)
- [ ] `git rev-parse --abbrev-ref HEAD` (record branch)

## Phase 1 — STATUS board update (start the board)
- [ ] Edit `docs/ops/STATUS.md`
  - [ ] Set S11-00 = 100% (Merged / PR #75)
  - [ ] Ensure S11-01 exists and remains 0% (Current = status enforcement)
- [ ] `git add docs/ops/STATUS.md`

## Phase 2 — Write S11-01 Plan/Task docs
- [ ] Ensure files exist:
  - [ ] `docs/ops/S11-01_PLAN.md`
  - [ ] `docs/ops/S11-01_TASK.md`
- [ ] `git add docs/ops/S11-01_PLAN.md docs/ops/S11-01_TASK.md`

## Phase 3 — Implement enforcement in prverify (core)
- [ ] Locate prverify entrypoint and diff logic (deterministic):
  - [ ] `git ls-files | rg -n '^cmd/prverify/.*\.go$' || true`
  - [ ] `rg -n "diff|name-only|STATUS\.md|ops/STATUS" cmd/prverify -S || true`
- [ ] Add rule:
  - [ ] If branch is S11 (`^s11-`) AND diff vs base lacks `docs/ops/STATUS.md` → FAIL with remediation
  - [ ] Deterministic sorting for file list
  - [ ] Base ref fallback: origin/main -> main -> error
- [ ] `git add <edited prverify go files>`

## Phase 4 — Tests (deterministic)
- [ ] Add unit tests for enforcement logic:
  - [ ] PASS case: includes STATUS.md
  - [ ] FAIL case: missing STATUS.md
  - [ ] Non-S11 branch: skip
  - [ ] Base ref resolution fallback behavior
- [ ] `git add <test files>`

## Phase 5 — Gates / Evidence
- [ ] Run: `nix run .#prverify`
- [ ] Confirm newest evidence path under `docs/evidence/prverify/`
- [ ] `git add docs/evidence/prverify/prverify_*.md`

## Phase 6 — Commit / Push
- [ ] `git status -sb` (ensure intended files only)
- [ ] Commit message (example):
  - `feat(prverify): enforce STATUS.md updates for S11 branches`
- [ ] `git push -u origin "$(git rev-parse --abbrev-ref HEAD)"`

## Phase 7 — PR create
- [ ] Prepare `.local/s11-01_pr.md` (SOT/What/Verification/Evidence)
- [ ] `gh pr create ...`
