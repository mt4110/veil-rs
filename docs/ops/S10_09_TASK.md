0. Preflight (Clean rail)

- [x] cd "$(git rev-parse --show-toplevel)"
- [x] git status -sb（clean確認）
- [x] dirtyなら停止：意図変更→別コミット / 不要→restore（復旧後に再確認）
- [x] git fetch origin --prune
- [x] git switch main
- [x] git pull --ff-only
- [x] git switch -c feature/s10-09-prkit-exec-hardening-v1
- [x] git push -u origin HEAD

1. Docs（先に器を固定）

- [x] ls -la docs/ops | rg "S10_"
- [x] docs/ops/S10_09_PLAN.md 作成（このPlanを貼る）
- [x] docs/ops/S10_09_TASK.md 作成（このTaskを貼る）
- [x] SOT作成：
    - [x] docs/pr/PR-TBD-v1-epic-A-s10-09-prkit-exec-hardening.md
    - [x] Evidence placeholder を先に入れる（後で差し替え）

2. 実装対象の探索（作り話禁止：実パス確定するまで実装禁止）

ここで確定したパスが 唯一の編集対象。以降 “推測で別ファイル触る” は禁止。

- [x] prkit候補探索（広めに見る）
    - [x] rg -n "prkit" -S cmd internal scripts .github
- [x] PRKIT_DIR を確定し、Taskに貼る（STOP条件：曖昧なら停止）
    - PRKIT_DIR = "cmd/prkit"

- [x] exec関連の実パス列挙（結果を 保存）
    - [x] rg -n "os/exec|exec\.Command|exec\.CommandContext|CombinedOutput|([^A-Za-z]Output\()|([^A-Za-z]Run\()|command_list|RunCmd" -S "$PRKIT_DIR" cmd internal
- [x] EXEC_CALL_SITES を確定（重複排除＋ソート）して Task に貼る
    - EXEC_CALL_SITES:
        - internal/prkit/check_git.go
        - internal/prkit/review_bundle.go
        - internal/prkit/sot.go
        - internal/prkit/tools.go

- [x] EVIDENCE_SCHEMA_FILES（command_list言及）を確定して貼る
    - EVIDENCE_SCHEMA_FILES:
        - internal/prkit/portable_evidence.go

- [x] STOP判定：波及が大きい（多ディレクトリ/多数ファイル）→ 中央集約のみに縮退（記録追加を後回し）

3. Exec hardening（中央集約：単一入口）

（編集対象は EXEC_CALL_SITES に限定）

- [ ] shell実行の禁止を確認（sh -c 等があれば除去）
- [ ] ExecRunner + ExecSpec + ExecResult を導入（errorはResultへ畳む）
- [ ] stdout/stderr 分離（CombinedOutput は置換対象）
- [ ] 出力正規化（改行統一/上限/UTF-8確定）を入口で固定
- [ ] cwdは repo相対で記録（絶対パスをevidenceへ入れない）
- [ ] envは 必要最小（sorted KV slice）で渡す＆記録

4. command_list（evidenceへ決定論で積む）

- [ ] EVIDENCE_SCHEMA_FILES を読んで “既存schema” に合わせる（増やしすぎ禁止）
- [ ] command_list へ append（順序契約：呼び出し順固定）
- [ ] failure/timeout/spawn失敗も必ず1エントリ残す（ErrorKind固定）

5. テスト（実プロセス実行なし）

- [ ] fake/stub runner で success case を作る
- [ ] fake/stub runner で failure case（exit!=0）を作る
- [ ] （可能なら）timeout/cancel case を作る
- [ ] contract: 同一入力→同一evidence（JSON byte一致 or canonical比較）

6. Gates & Evidence

- [ ] go test ./... -count=1
- [ ] nix run .#prverify
- [ ] docs/evidence/prverify/prverify_<UTC>_<sha>.md を保存
- [ ] SOT の Evidence 行を差し替え
- [ ] TASK のチェックを埋める

7. Commit & Push

- [ ] git status -sb
- [ ] git add -A
- [ ] git commit -m "fix(s10-09): harden prkit exec + deterministic command evidence"
- [ ] git push
