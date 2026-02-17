# S11-05 TASK — Closeout ritual

## 0. Preflight
- [ ] `git status -sb` が clean
- [ ] `git fetch origin --prune`

## 1. Evidence (main HEAD)
- [ ] `git switch main && git pull --ff-only`
- [ ] `nix run .#prverify` (PASS)
- [ ] `.local/prverify/prverify_*.md` の最新を `docs/evidence/prverify/` にコピーしてコミット

## 2. STATUS.md 正規化
- [ ] S11-03 / S11-04 を 100% に更新
- [ ] S11-05 を 0% で追加（または既存があれば更新）
- [ ] `- Evidence:` を最新 prverify に更新
- [ ] `- Last Updated:` を更新

## 3. SOT (check-sot 対策)
- [ ] PR番号確定後に `docs/pr/PR-<NUM>-s11-05-reviewbundle-closeout.md` を作成
- [ ] SOT / What / Verification / Evidence を標準形式で埋める

## 4. Gates
- [ ] `nix run .#prverify` (PASS)
- [ ] `go test ./...` (PASS)

## 5. Push/PR
- [ ] push
- [ ] PR 作成（title/body）
- [ ] checks green を確認
