
# S10-02: pr-kit dry-run v1

## What
Introduce pr-kit (v1: dry-run only) that emits portable evidence JSON v1 for audit-friendly PR rituals.

## Why
We need a stable, diff-friendly “portable-first” evidence unit before automating PR rituals.
This step makes “what/where/how” visible without touching files.

## Scope
- Add: cmd/prkit/main.go
- Add: internal/prkit/{portable_evidence.go,run.go,tools.go,check_git.go}
- Add/Update: docs/ops/S10_02_PLAN.md, docs/ops/S10_02_TASK.md
- Update: docs/ops/S10_evidence.md (includes dirty FAIL → Clean Rail Re-run PASS)

## Non-goals
- Creating PRs/branches/commits (beyond this implementation PR)
- Running prverify as part of pr-kit (dry-run outputs planned checks only)
- Generating/modifying repo files (output-only)

## Contracts / Invariants
- v1 requires --dry-run
- Always outputs a single portable JSON evidence (stable ordering; no map-based ordering drift)
- Logic is in Go (shell remains thin)
- First check only: git_clean_worktree via git status --porcelain=v1

## Verification
- `go test ./...`
- `unset GOROOT && go run ./cmd/prkit --dry-run`
- expect: status=PASS, exit_code=0 (clean worktree)
- expect: checks[0].name == "git_clean_worktree"
- expect: artifact_hashes == []

## Evidence Notes
docs/ops/S10_evidence.md includes:
- S10-02 initial run (dirty) => FAIL (exit_code=2; details show untracked paths)
- S10-02 Clean Rail Re-run => PASS (exit_code=0)

## Rollback
Revert the merge/squash commit for this PR:
```bash
git revert <commit>
```
