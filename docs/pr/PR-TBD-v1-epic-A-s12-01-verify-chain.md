# PR-TBD — S12-01 A: Verify Chain v1

## SOT
- Plan/Task:
  - docs/ops/S12-01_PLAN.md
  - docs/ops/S12-01_TASK.md
- Status:
  - docs/ops/STATUS.md (S12-00 merged, S12-01 wip)

## What
- Kick off S12-01(A) and turn verification invariants into enforceable tests + gates.

## Verification
- rg: `rg -n "^\| S12|Verify Chain" docs/ops/STATUS.md docs/ops/S12-01_PLAN.md`
- go test ./... (PASS)
- nix run .#prverify (PASS, if applicable)
