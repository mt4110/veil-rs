# S10-07 Task — prkit-exec-v1 Hardening & Portable Evidence

## 0. Setup & Pre-flight（状態固定）
- [ ] `cd "$(git rev-parse --show-toplevel)"`
- [ ] `test "$(git rev-parse --abbrev-ref HEAD)" = "feature/s10-07-any-v1"`
- [ ] `git status -sb`（dirtyなら STOP：差分提示して終了）
- [ ] `git rev-parse HEAD`
- [ ] `git diff --stat`

## 1. Define scanners（検出力維持・自己ヒット回避）
- [ ] 変数連結で abs-path シグネチャを作る（docs本文にリテラルを残さない）
  - [ ] 下を実行：
    ```bash
    U="U"; S="sers"; H="h"; O="ome"
    HOME_PATTERN="(/${U}${S}/[^/]+|/${H}${O}/[^/]+|[A-Za-z]:\\\\[^\\\\]+)"
    echo "$HOME_PATTERN" | rg -n '^\(' >/dev/null && echo "OK: pattern built"
    ```

## 2. Blocker check（早期に地雷を踏む）
- [ ] `nix run .#prverify`
- [ ] FAIL した場合：原因を分類して以下の該当セクションから潰す（推測で先へ進まない）

## 3. Hardening prkit（cmd/prkit/main.go）
- [ ] `go run ./cmd/prkit --help`（exit 0 を確認）
- [ ] `cmd/prkit/main.go` を編集：
  - [ ] `--run` が `DryRun` 前提でブロックされている箇所があれば解除
  - [ ] `--run` と `--dry-run` 同時指定は **エラーで拒否**（相互排他）
- [ ] 相互排他の負のテスト（両方指定で落ちること）
  - [ ] `go run ./cmd/prkit --run --dry-run --help && echo "ERROR: expected failure" || echo "OK: rejected conflicting flags"`

## 4. Hardening SOT path（internal/prkit/sot.go）
- [ ] `internal/prkit/sot.go` を編集：
  - [ ] 表示/生成パスを repo-relative にする（`filepath.Rel`）
  - [ ] rel が repo外（`..` 始まり等）なら error（出力も生成も拒否）
  - [ ] 表示は `filepath.ToSlash(rel)`（OS差吸収）
- [ ] dry-run で出力確認（※実際のフラグは `--help` の表示に従う）
  - [ ] 例（存在する場合のみ実行）：
    ```bash
    go run ./cmd/prkit --sot-new --epic A --slug test --release v1
    ```
- [ ] 出力が repo-relative であることを確認（abs-path が出たら STOP）

## 5. Portable Evidence（docs abs-path scan）
- [ ] `rubric="docs/ops docs/pr"`
- [ ] `rg -n --hidden -E "$HOME_PATTERN" $rubric || true`
- [ ] hits があれば：
  - [ ] 対象docsを編集して “環境依存の絶対パス” を除去（プレースホルダ化 / 相対化）
  - [ ] 再実行して 0 hits を確認

## 6. Markdown fence check（docs）
- [ ] docs の ``` 数が奇数のファイルを列挙（0件になるまで直す）
  - [ ] 下を実行：
    ```bash
    python3 - <<'PY'
import pathlib, re
root = pathlib.Path("docs")
bad = []
for p in root.rglob("*.md"):
    s = p.read_text(encoding="utf-8", errors="ignore")
    n = len(re.findall(r"```", s))
    if n % 2 == 1:
        bad.append((str(p), n))
for f, n in bad:
    print(f"{f}\tcount={n}")
PY
    ```

## 7. Final Gate（真偽確定）
- [ ] `nix run .#prverify`（Must PASS）
- [ ] `git status -sb`（Must clean）

## 8. review_bundle（ログに相対パスで残す）
- [ ] `MODE=wip bash ops/ci/review_bundle.sh`
- [ ] `ls -la .local/review-bundles/`
- [ ] `ls -t .local/review-bundles/*.tar.gz | head -n 1`

## 9. Commit / Push / PR
- [ ] `git add -A`
- [ ] `git commit -m "feat(s10-07): harden prkit exec + portable evidence"`
- [ ] `git push -u origin HEAD`
- [ ] `git fetch origin --prune`
- [ ] `git log --oneline origin/main..HEAD`
- [ ] 既存PR確認：`gh pr list --head "$(git rev-parse --abbrev-ref HEAD)" --state all`
- [ ] PR作成（無ければ）：
  - [ ] `gh pr create --base main --head "$(git rev-parse --abbrev-ref HEAD)" --title "feat(s10-07): harden prkit exec + portable evidence" --body $'- Plan: docs/ops/S10_07_PLAN.md\n- Task: docs/ops/S10_07_TASK.md\n- Gate: nix run .#prverify (PASS required)\n- Bundle: .local/review-bundles/* (wip)'`

## Exit Conditions（PASS/FAIL を埋めて終了）
- [ ] EC1: git status is clean
- [ ] EC2: prverify exits 0
- [ ] EC3: docs abs-path matches 0 hits
- [ ] EC4: fence matches 0 odd-counts
- [ ] EC5: SOT ambiguity not present（prverify PASSで担保）
- [ ] EC6: review_bundle generated
- [ ] EC7: origin/main..HEAD has commits
