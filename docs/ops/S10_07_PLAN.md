# S10-07 Plan — prkit-exec-v1 Hardening & Portable Evidence

goal:
- `prkit` の Execution Mode（`--run`）を正式に有効化し、実行ブロック要因（`if !DryRun` など）を除去する
- Documentation / Evidence から “環境依存の絶対パス” 混入を排除し、Portable Evidence を強制する
- SOT 生成・ログ出力は repo-relative を基本とし、repo外参照を拒否する

scope:
- internal/prkit/**/*.go
- cmd/prkit/main.go
- docs/ops/S10_evidence.md（append only, 必要なら）
- docs/pr/（SOT生成の出力が repo-relative であることの保証）
- NO changes: crates/ や prkit 以外のコアロジック

exit_conditions:
- EC1: `git status` が clean
- EC2: `nix run .#prverify` が exit 0（最終ゲートで再実行してPASS）
- EC3: docs abs-path 検査が 0 hits（検出力を弱めない）
- EC4: markdown fence 検査が 0 件（未閉じなし）
- EC5: SOT ambiguity が発生していない（prverify PASSで担保）
- EC6: review_bundle が生成され、生成物パスがログに残る
- EC7: `origin/main..HEAD` にコミット差分がある（PR作成可能）

plan:

STEP0: Safety Snapshot
- run: `cd "$(git rev-parse --show-toplevel)"`
- run: `git rev-parse --abbrev-ref HEAD`
- IF branch == main THEN error: "main作業禁止"
- run: `git status -sb`
- IF dirty THEN error: "dirty開始は禁止（差分提示して停止）"

STEP1: Determine S10-07 Purpose (Docs-grounded)
- run: `ls -la docs/ops | rg -n 'S10_|S10-' || true`
- run: `rg -n --hidden 'S10-07|S10_07|prkit-exec|prkit' docs/ops docs/pr 2>/dev/null || true`
- IF purpose cannot be grounded THEN stop: "探索コマンドを最大3本提案して停止（推測禁止）"

STEP2: Define target files (paths fixed)
- PLAN := `docs/ops/S10_07_PLAN.md`
- TASK := `docs/ops/S10_07_TASK.md`

STEP3: Blocker check (early)
- run: `nix run .#prverify`
- IF FAIL THEN
  - classify reason (SOT ambiguity / docs abs-path / fence / other)
  - encode branching in TASK before implementation
  - IF unknown THEN error: "未知ブロッカー（ログ根拠を貼って停止）"

STEP4: Hardening prkit (cmd/prkit/main.go)
- action: `--run` が “dry-run以外では即return” になっている場合は解除
- action: `--run` と `--dry-run` は相互排他（同時指定はエラー）にする
- verify:
  - `go run ./cmd/prkit --help` が exit 0
  - `go run ./cmd/prkit --dry-run ...` が exit 0
  - `--run` と `--dry-run` 同時指定が reject される

STEP5: Hardening SOT output (internal/prkit/sot.go)
- action: SOT出力パスは repo-relative を基本にする
  - `filepath.Rel(repoRoot, path)` を使用
  - rel が `..` 始まり等で repo外なら error（出力・生成とも拒否）
  - 表示は `filepath.ToSlash(rel)`（OS差の吸収）
- verify:
  - `--sot-new` dry-run の出力が repo-relative である
  - repo外パスが出ない / 生成されない

STEP6: Portable Evidence enforcement (docs)
- action: docs 配下を “環境依存の絶対パス” 検出でスキャン
- NOTE: 検出力は弱めない。自己ヒットは docs側の書き方で回避する（リテラル禁止）
- verify: 0 hits

STEP7: Markdown fence check (docs)
- action: docs の ``` 数が奇数のファイルを列挙して修正
- verify: 0 files

STEP8: Final gate
- run: `nix run .#prverify`
- IF FAIL THEN error: "最終ゲートでFAIL（ログ根拠を貼って停止）"
- run: `git status -sb`
- IF dirty THEN error: "最終ゲート後にdirty（禁止）"

STEP9: Bundle + Commit
- run: `MODE=wip bash ops/ci/review_bundle.sh`
- verify: `.local/review-bundles/` に生成物がある（相対パスでログ化）
- action: commit / push / PR作成

END
