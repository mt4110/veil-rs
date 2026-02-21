# S12-04 TASK — CI Repro Ritual Capsule (Go)

## Discovery results (pinned paths)

| Item           | Path / Value                                           |
| -------------- | ------------------------------------------------------ |
| CLI framework  | Custom `flag.FlagSet`（cobra/urfave 不使用）           |
| Entry point    | `cmd/prkit/main.go` → `internal/prkit/cli.go`:`Run()`  |
| CLI config     | `internal/prkit/cli.go`:`cliConfig` struct             |
| Flag parsing   | `internal/prkit/cli.go`:`parseCLIConfig()`             |
| Dispatch       | `internal/prkit/cli.go`:`executeCLI()`                 |
| ExecRunner     | `internal/prkit/exec.go`:`ExecRunner` interface        |
| FakeRunner     | `internal/prkit/fake_runner.go`:`FakeExecRunner`       |
| Git helpers    | `internal/prkit/check_git.go`                          |
| Contract tests | `cmd/prkit/contract_test.go`                           |
| Module path    | `veil-rs`                                              |
| Time mock      | `internal/prkit/portable_evidence.go`:`Now = time.Now` |

---

## 0) Discovery [x]
- [x] Find prkit entry point → `cmd/prkit/main.go`
- [x] Find CLI framework → custom `flag.FlagSet`
- [x] Find ExecRunner → `internal/prkit/exec.go`
- [x] Find test patterns → `cmd/prkit/contract_test.go`

## 1) Add CLI routing: `ci-repro` subcommand [ ]
- [ ] Modify `internal/prkit/cli.go`:`Run()` to detect `ci-repro` as first positional arg
- [ ] Route `ci-repro run` and `ci-repro step <name>` to new handler
- [ ] Parse flags: `--out-dir`, `--run-id`, `--with-strict`
- [ ] **FlagSet は `flag.ContinueOnError` 必須**（os.Exit 地雷回避）
- [ ] No `os.Exit`, no `panic`, no `log.Fatal`

## 2) Implement runner core [ ]
- [ ] Create `internal/prkit/cirepro/` package (new)
  - [ ] `cirepro.go` — orchestrator
  - [ ] `step.go` — step definitions + execution **（ExecRunner 経由、テストで FakeExecRunner 差替）**
  - [ ] `gitprobe.go` — **既存 `check_git.go` の helper を優先活用**（使えない場合は理由を1行ログ）
  - [ ] `summary.go` — summary formatter
  - [ ] `status_snapshot.go` — STATUS.md extractor
  - [ ] `writer.go` — atomic write helper（summary/snapshot 用）
- [ ] Define canonical step list (01-04, fixed order)
- [ ] Implement run_id (default + override)
- [ ] Implement git probes (CLEAN/DIRTY/UNKNOWN)

## 3) Output writers (I/O contract) [ ]
- [ ] Ensure out-dir exists
- [ ] Always write summary (even on repo detection failure)
- [ ] Always write status snapshot (even on STATUS.md missing)
- [ ] Step logs: attempted=capture, skipped=write `SKIP: <reason>`
- [ ] Atomic write (rename pattern)

## 4) Step execution (stopless + SKIP rules) [ ]
- [ ] **ExecRunner 経由で実行**（nix/go/git 直接呼ばない）
- [ ] Implement SKIP rules per PLAN
  - `run`: `--with-strict` なし → step03/04 SKIP
  - `step <name>`: 明示指定 → 実行（安全 SKIP は適用）
- [ ] Implement command strings per PLAN
- [ ] For each step: record started_utc / ended_utc / duration_ms
- [ ] **ログ先頭ヘッダー**: cmd / reason / started_utc / ended_utc を固定出力

## 5) Summary formatter (deterministic) [ ]
- [ ] Implement fixed-format markdown (4-row table)
- [ ] `overall=ERROR` if any step ERROR
- [ ] `error_steps` ascending list or `NONE`

## 6) STATUS snapshot extractor [ ]
- [ ] Read `docs/ops/STATUS.md`
- [ ] Extract `| S12-` lines
- [ ] Fallback: `ERROR: no S12 rows found`

## 7) Tests [ ]
- [ ] **全テスト `FakeExecRunner` 使用（nix/外部コマンド一切不使用）**
- [ ] `--run-id fixed` produces fixed filenames
- [ ] SKIP writes step log with `SKIP: ...` + cmd header
- [ ] dirty tree → prverify/strict skipped
- [ ] repo missing → summary + snapshot still written
- [ ] summary format stability (string contains required sections)

## 8) Gates [ ]
- [ ] `go test ./...` PASS
- [ ] Manual: `go run ./cmd/prkit ci-repro step go-test --run-id smoke`
- [ ] Manual: `go run ./cmd/prkit ci-repro run --run-id smoke`
- [ ] Files in `.local/obs/ci_smoke_*` exist
