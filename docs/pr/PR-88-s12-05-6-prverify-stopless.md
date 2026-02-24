# PR-88 — S12-05.6: prverify stopless hardening (no os.Exit + bugfixes)

## SOT

- Scope: S12-05.6 — cmd/prverify stopless hardening
- Branch: s12-05-6-prverify-stopless-fix-v1
- **Auto-merge disabled — do NOT enable auto-merge.**
- Deliverables:
  - cmd/prverify/main.go (stopless patch: os.Exit removed, stdout ERROR, stop=<0|1>)
  - docs/ops/S12-05-6_PLAN.md (pseudocode PLAN)
  - docs/ops/S12-05-6_TASK.md (ordered task list)
  - docs/ops/STATUS.md (S12-05.6 row added, 1% WIP)

## What

S12-05.5 (PR #87) の差分で観測された地雷を除去する stopless hardening パッチ。

| Blocker                       | 対応                                        |
| ----------------------------- | ------------------------------------------- |
| A: os.Exit(1) at L437         | FIXED: hasError flag + PASS: output instead |
| B: ERROR: stderr-only         | FIXED: stdout mirror added (9 locations)    |
| C: dep-guard double cmd.Run() | Audit-SKIP: already single-run (L339)       |
| D: git diff cmd.Dir missing   | Audit-SKIP: cmd.Dir=root already set (L810) |

Invariants enforced:
- `OK: phase=end stop=<0|1>` always printed (stop derived from hasError)
- `PASS: All checks passed.` only when !hasError
- Zero `os.Exit` in cmd/prverify/main.go

## Verification

```
rg "os\.Exit" cmd/prverify/main.go  # exit code 1 = CLEAN (zero matches)
rg 'stop=0'   cmd/prverify/main.go  # exit code 1 = CLEAN (no hardcode)
go build ./cmd/prverify              # OK: build clean
```

## Evidence

- Build: `go build ./cmd/prverify` → OK
- Lint: `rg "os\.Exit" cmd/prverify/main.go` → no matches (exit code 1)
- SOT docs: docs/ops/S12-05-6_PLAN.md, docs/ops/S12-05-6_TASK.md
