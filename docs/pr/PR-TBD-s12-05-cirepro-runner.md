# PR-TBD SOT — s12-05-cirepro-runner

## SOT
- PR: #TBD
- Repo: veil-rs
- Phase: S12-05 (CI Repro Runner Alignment)
- Branch: HEAD
- Head: <will be updated by user>
- Board: docs/ops/STATUS.md
- Ops evidence: docs/evidence/ops/obs_20260222_s12-05.md

## What
- Refactored `ci-repro` logic to use the `prkit` `ExecRunner` contract (`Runner.Run`) and DI for core side-effects.
- Re-use `check_git` helpers for deterministic git status checks in `ci-repro`.
- Add `FindRepoRoot()` bootstrap command to decouple early path finding from strict global Runner context.
- Update `ci-repro` test suites to utilize DI pattern safely rather than mutating package-level variables.

## Run Instructions (ci-repro via prkit runner)
- Representative run: `go run ./cmd/prkit ci-repro run --run-id smoke`
- Step-by-step: `go run ./cmd/prkit ci-repro step <subcommand>`

## Verification
Local (representative):
- `nix develop -c go test ./...` : PASS
- `nix run .#prverify` : PASS
- `go run ./cmd/prkit ci-repro run --run-id smoke` : PASS (No SecurityViolation/panic, output correctly logged).

## Evidence
- prverify report: `.local/prverify/prverify_20260222T010312Z_7d2b2a6.md`
- ci-repro output: `.local/obs/s12-05_20260222T005352Z/final_cirepro_run.log`

## Rollback
- Revert the merge commit for this PR:
  - `git revert <merge_commit_sha>`
