# PR-90 — S12-06C: .local を"散らからない宇宙"にする

## SOT

- Scope: S12-06C — .local durability GC (policy doc + cmd/localgc + unit tests)
- Branch: s12-06-local-durability-gc-v1
- **Auto-merge disabled — do NOT enable auto-merge.**
- Deliverables:
  - docs/ops/LOCAL_STORAGE_POLICY_v1.md (new: canonical .local policy)
  - cmd/localgc/main.go (new: stopless GC tool, dry-run default, double-lock apply)
  - cmd/localgc/main_test.go (new: 4 tests)

## What

S12-06C は `.local/` の肥大化・ゴミ化で「再現不能/ディスク死」が起きる未来を先に潰す。

| DoD                      | 実装                                         |
| ------------------------ | -------------------------------------------- |
| .local ポリシー doc      | docs/ops/LOCAL_STORAGE_POLICY_v1.md          |
| GCツール dry-run default | cmd/localgc: `--mode dry-run` が何も消さない |
| 二重ロック apply         | `--mode apply` かつ `--apply` 両方必須       |
| stopless                 | rc=0 常時, `OK: phase=end stop=<0            | 1>` 必須 |
| 単体テスト               | 4 tests PASS                                 |

Fast inventory (discovery):

```
.local/ subdirs: archive/ bin/ cache/ ci/ evidence/ fp2/ fp3/
                 handoff/ lock_backup/ obs/ prverify/ review-bundles/
各サブディレクトリはほぼ空 (count=0)。スクラッチファイルは .local/ 直下に集積。
```

## Verification

```
go build ./cmd/localgc                          → OK: build clean
go test ./cmd/localgc/... -v
  → PASS: TestLocalgcStopless (dry-run/plan/bad-flag)
  → PASS: TestLocalgcDoubleLock (3 subcases)
  → PASS: TestLocalgcDryRunSafe
  → PASS: TestLocalgcOutputFormat
rg "os\.Exit|log\.Fatal|log\.Panic" cmd/localgc/main.go
  → main.go:51 os.Exit(run(...)) のみ (entry point, acceptable)
```

## Evidence

- Build: clean
- Tests: 4 new tests PASS
- Double-lock verified: no files deleted without both `--mode apply` AND `--apply`
- Dry-run safety: 10 files created, 0 deleted after dry-run
