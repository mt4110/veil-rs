# S10-08 Plan — prkit contract tests + CLI testability hardening

## Goal
S10-07で確立した prkit の flag/usage 契約を、将来の変更で破壊できないようにする。
- `prkit --help` : exit=0, stdout empty, stderr has Usage (deterministic)
- unknown flag   : exit=2, stdout = FAIL JSON, stderr has flag error + Usage (deterministic)

加えて、契約テストを安定運用できるように CLI 実装をテスト可能な形へ寄せる。

## Non-goals
- 新しいサブコマンド追加（必要になったら別タスク化）
- 実行環境依存の統合テスト（flakeの温床は避ける）
- GitHub API / network を叩くテスト（決定論を壊す）

## Constraints (Determinism)
- テストはローカル/CIで同じ結果になること（時刻・絶対パス・環境差分を出力に混ぜない）
- JSONはキー順/改行/スペースを固定（golden比較しやすく）
- stderr の Usage 文字列が揺れないようにする（Go flag の出力をラップするなら完全固定）

## Files (target)
- Code:
  - cmd/prkit/main.go
  - (必要なら新設) internal/prkit/*  ※CLIロジック分離
- Tests:
  - cmd/prkit/contract_test.go (or internal/prkit/contract_test.go)
  - (golden) testdata/prkit/*.golden (必要なら)
- Docs:
  - docs/ops/S10_08_PLAN.md
  - docs/ops/S10_08_TASK.md
  - docs/pr/PR-TBD-v1-epic-A-s10-08-prkit-contract-tests.md
- Evidence:
  - docs/evidence/prverify/prverify_<UTC>_<sha>.md

## Control Flow (Pseudo)
PHASE 0: Preflight / Clean rail
IF git status not clean:
  - IF changes are intended for S10-08: commit with meaningful msg
  - ELSE: restore changes
  - RECHECK clean; if still dirty -> error STOP

PHASE 1: Locate existing contract docs/tests
FOR each candidate path in docs/ops + docs/pr:
  - search "S10-07" "prkit" "flag contract"
  - IF found authoritative contract statement: record lines + path
  - ELSE continue
IF contract statement not found -> error STOP (spec missing)

PHASE 2: Make CLI testable (minimal refactor)
- Prefer pattern:
  - func Run(args []string, stdout, stderr io.Writer) int
  - main() only wires os.Args + os.Stdout/Stderr
- Ensure:
  - `--help` is handled deterministically (no side-effects)
  - unknown flag path prints exact FAIL JSON to stdout and exact error+Usage to stderr

PHASE 3: Add contract tests (must be stable)
- Test cases:
  1) args=["--help"] :
     - exit==0
     - stdout==""
     - stderr contains "Usage" (and matches golden if adopted)
  2) args=["--definitely-unknown-flag"] :
     - exit==2
     - stdout equals FAIL JSON (exact match)
     - stderr contains flag error + Usage (stable)
- Avoid executing external processes; call Run() directly.
- If golden:
  - store expected stdout/stderr in testdata
  - normalize line endings to "\n"

PHASE 4: Docs + SOT update
- Update S10-08 plan/task
- Add PR SOT with:
  - scope
  - contract summary
  - evidence pointer placeholder

PHASE 5: Evidence
- go test ./... PASS
- nix run .#prverify PASS
- save prverify report under docs/evidence/prverify/
- update PR SOT to point to evidence file

STOP Conditions
- Any nondeterministic output detected (time/path/env): STOP and remove it
- Tests flaky or depend on OS differences: STOP and refactor to pure Run() tests
