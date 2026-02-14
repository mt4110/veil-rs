# S10-08 Task — prkit contract tests + CLI testability hardening

## Progress
- S10-07: 100% (done)
- S10-08: 0% -> 100% in this PR (single PR policy)

## 0. Preflight (Clean rail)
- [x] cd "$(git rev-parse --show-toplevel)"
- [x] git fetch origin --prune
- [x] git status -sb
- [x] IF cmd/prkit/main.go is modified:
  - [x] Inspect: git diff -- cmd/prkit/main.go
  - [x] IF change is unintended: git restore --source=HEAD --worktree --staged cmd/prkit/main.go
  - [x] ELSE intended: git add cmd/prkit/main.go && git commit -m "fix(prkit): <s10-08 intent>"
- [x] Delete stale local branch if exists: git branch -D feature/s10-07-any-v1 2>/dev/null || true
- [x] Sync main: git switch main && git pull --ff-only
- [x] Back to work branch: git switch feature/s10-08-prkit-next-v1
- [x] Rebase onto main: git rebase main
- [x] git status -sb (must be clean)

## 1. Path confirmation (avoid made-up paths)
- [x] List S10 ops docs: ls -la docs/ops | rg "S10"
- [x] Confirm S10-07 naming pattern:
  - [x] rg -n "S10-07|s10-07|prkit" docs/ops docs/pr
- [x] Decide final file names for S10-08 docs (must match existing convention)

## 2. Write docs (Plan/Task/SOT)
- [x] Create docs/ops/S10_08_PLAN.md (paste from Plan)
- [x] Create docs/ops/S10_08_TASK.md (this file)
- [ ] Create SOT:
  - [ ] docs/pr/PR-TBD-v1-epic-A-s10-08-prkit-contract-tests.md
  - [ ] Include: contract summary + acceptance + evidence placeholder

## 3. Implementation (minimal refactor for testability)
- [x] Identify current entrypoints:
  - [x] open cmd/prkit/main.go and locate flag parsing + output formatting
- [x] Implement/adjust:
  - [x] Provide Run(args, stdout, stderr) int (or equivalent) to allow pure tests
  - [x] Keep S10-07 contract unchanged:
    - [x] --help path: exit=0, stdout empty, stderr Usage (deterministic)
    - [x] unknown flag: exit=2, stdout FAIL JSON, stderr error+Usage
  - [x] Ensure JSON is stable (key order, newline, no timestamps, no absolute paths)

## 4. Tests (contract tests)
- [/] Add contract tests that call Run() directly (no exec)
- [/] Test case A: --help
  - [x] assert exit==0
  - [x] assert stdout==""
  - [x] assert stderr matches expected (contains Usage; golden if adopted)
- [/] Test case B: unknown flag
  - [x] assert exit==2
  - [x] assert stdout == exact FAIL JSON
  - [x] assert stderr contains "flag provided but not defined" (or your exact chosen string) + Usage
- [x] Normalize line endings to "\n" in tests to avoid OS variance

## 5. Gates (must pass)
- [x] go test ./... (PASS)
- [x] nix run .#prverify (PASS)
- [x] Save prverify report:
  - [x] docs/evidence/prverify/prverify_20260214T115121Z_c0a1a07.md
- [x] Update SOT to reference the evidence file path

## 6. PR prep (one PR completes S10-08)
- [ ] git status -sb (clean)
- [ ] git push -u origin feature/s10-08-prkit-next-v1
- [ ] PR body includes:
  - [ ] SOT path
  - [ ] Evidence path
  - [ ] Contract summary (S10-07 preserved; S10-08 adds tests)
