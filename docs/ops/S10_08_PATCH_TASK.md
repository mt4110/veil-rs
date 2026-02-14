# S10-08 Patch Task（固定実行順）
## 0. Preflight（Clean rail）
- [x] cd "$(git rev-parse --show-toplevel)"
- [x] git fetch origin --prune
- [x] git status -sb

## 1. A-fix: unknown flag stderr 重複の除去
- [ ] Locate parse error branch in `cmd/prkit/main.go`
- [ ] Edit `cmd/prkit/main.go` to remove redundant error print
- [ ] Quick local sanity: `go run ./cmd/prkit --unknown-flag`

## 2. (推奨) contract_test を補強（エラー行count=1）
- [ ] Update `cmd/prkit/contract_test.go` assertions

## 3. B-fix: prverify に go test -count=1 ./cmd/prkit を追加
- [ ] Find prverify entry in `cmd/prverify`
- [ ] Insert new gate step `go test -count=1 ./cmd/prkit`
- [ ] Ensure stdout/stderr are captured

## 4. Commit（Fixpack）
- [ ] git commit -m "fix(s10-08): dedupe prkit flag error + include cmd/prkit tests in prverify"

## 5. Gates（証拠）
- [ ] go test ./... -count=1
- [ ] nix run .#prverify

## 6. Docs更新（SOT/Task）
- [ ] Update SOT evidence path
- [ ] Update docs/ops/S10_08_TASK.md
