# S10-09 Fixpack v2 (Audit-Grade)

## 0) Preflight
- [ ] cd "$(git rev-parse --show-toplevel)"
- [ ] git fetch origin --prune
- [ ] git status -sb
- [ ] git diff --cached --name-status

## 1) STOP: prverify が append-only か（過去証拠を壊さない）
- [ ] git diff --name-status origin/main...HEAD -- docs/evidence/prverify | sort
  - [ ] IF D/R/M が出たら -> STOP

### 1-A) STOP解除（安全に origin/main へ同期）
- [ ] git restore --source=origin/main --staged --worktree -- docs/evidence/prverify
- [ ] git add docs/evidence/prverify/prverify_20260215T012259Z_e70ca0f.md
- [ ] git diff --name-status origin/main...HEAD -- docs/evidence/prverify | sort
  - [ ] 期待: A が 1本のみ

## 2) STOP: docs の file URL 根絶（文字列を含めずに検出）
- [ ] rg -n 'file:/{3}' docs || true
  - [ ] ヒットがあれば修正 -> 0件になるまで STOP 継続

## 3) ブロッカー修正: ExecSpec 契約の一本化（Argvフル）
- [ ] rg -n 'type ExecSpec|CommandContext\\(|spec\\.Name|spec\\.Argv' internal/prkit -S

### 3-A) 修正対象（確定パス）
- [ ] internal/prkit/exec.go
- [ ] internal/prkit/fake_runner.go
- [ ] internal/prkit/check_git.go
- [ ] internal/prkit/tools.go
- [ ] internal/prkit/sot.go

### 3-B) 実装修正（最低ライン）
- [ ] ProdExecRunner: 実行名 = (spec.Name が空なら spec.Argv[0])、引数 = spec.Argv[1:]
- [ ] FakeExecRunner: 記録する Argv 形式を Prod と一致させる
- [ ] spec.Argv len==0 は error（failure evidence を出す）

## 4) cwd 契約（repo相対 + repo外脱出禁止）
- [ ] internal/prkit/exec.go:
  - [ ] spec.Dir が空なら "."
  - [ ] spec.Dir が絶対なら error
  - [ ] Join(repoRoot, Clean(spec.Dir)) 後、repo外なら error
  - [ ] evidence.CwdRel は slash 正規化で固定

## 5) env 契約（inherit+delta）
- [ ] internal/prkit/exec.go:
  - [ ] spec.Env は “差分 override” として merge
  - [ ] evidence.EnvMode = "inherit+delta"
  - [ ] evidence.EnvKV = override をキーソートで記録
  - [ ] evidence.EnvHash = 実効 env から決定論ハッシュ

## 6) review_bundle hardening（Getwd廃止 + stderr混線廃止）
- [ ] internal/prkit/review_bundle.go:
  - [ ] os.Getwd 依存を 제거（repoRoot 基準で候補探索）
  - [ ] scriptPath は repo相対で固定
  - [ ] stdout/stderr を混ぜない（解析は stdout のみ）
  - [ ] OK: 行は stdout の末尾側から探索（最後の OK: を採用）
  - [ ] bundlePath は repo相対で返す（絶対なら Rel(repoRoot, abs)）

## 7) portable evidence: 絶対パス redaction（argv + stdout/stderr）
- [ ] internal/prkit/exec.go:
  - [ ] repoRoot prefix を "<REPO_ROOT>" に置換
  - [ ] evidence に Redactions を残す（何をしたか嘘をつかない）
- [ ] 監査（evidence / prverify）
  - [ ] E="docs/evidence/prverify/prverify_20260215T012259Z_e70ca0f.md"
  - [ ] rg -n "file:/{3}|/Users/|[A-Za-z]:\\\\" "$E" || true  -> 0件

## 8) SOT ブロッカー: file URL を相対 backtick に統一
- [ ] FILE="docs/pr/PR-TBD-v1-epic-A-s10-09-prkit-exec-hardening.md"
- [ ] rg -n 'file:/{3}' "$FILE" || true
- [ ] Evidence 行を `docs/evidence/prverify/prverify_20260215T012259Z_e70ca0f.md` に修正
- [ ] rg -n 'file:/{3}' "$FILE" || true  -> 0件

## 9) Gates（clean rail で取る）
- [ ] git status -sb （dirty なら STOP）
- [ ] go test ./... -count=1
- [ ] nix run .#prverify
- [ ] docs/evidence/prverify/prverify_<UTC>_<sha>.md を保存（最終PASS 1本だけ add）

## 10) Commit & Push
- [ ] git status -sb
- [ ] git diff --cached --name-status
- [ ] git commit -m "fix(s10-09): fixpack v2 (execspec contract + portable evidence + review_bundle hardening)"
- [ ] git push
