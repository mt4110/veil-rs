# S10-09 Fixpack v2 (Audit-Grade)

## Goal
- ExecRunner / ExecSpec 契約を一本化し、実運用で確実に動く形に硬化する
- portable evidence に絶対パスを混入させない（argv + stdout/stderr）
- review_bundle の stdout/stderr 混線と Getwd 依存を排除し、長期運用で腐らない形にする
- テストは実プロセスを踏まない（FakeRunner / pure function で契約固定）

## Non-goals
- 新機能追加
- ネットワーク依存の統合テスト
- “過去の証拠” の改変（docs/evidence/prverify の既存ファイル delete/rename/modify）

---

## Control Flow (Pseudo)

PHASE 0: Clean rail
- IF git status dirty:
  - IF intended changes: commit OR stash (policyに従う)
  - ELSE: restore
  - RECHECK; IF still dirty -> error STOP

PHASE 1: 対象箇所の実パス確定（作り話禁止）
- TARGETS (確定):
  - internal/prkit/exec.go
  - internal/prkit/fake_runner.go
  - internal/prkit/tools.go
  - internal/prkit/check_git.go
  - internal/prkit/review_bundle.go
  - internal/prkit/sot.go
  - cmd/prkit/main.go
  - docs/pr/PR-TBD-v1-epic-A-s10-09-prkit-exec-hardening.md
  - docs/ops/S10_09_PLAN.md
  - docs/ops/S10_09_TASK.md
  - docs/evidence/prverify/prverify_20260215T012259Z_e70ca0f.md

PHASE 2: ExecSpec 契約の一本化（最重要ブロッカー）
- RULE: ExecSpec は “argvフル” を正とする
  - spec.Argv[0] = 実行ファイル名
  - spec.Argv[1:] = 引数
  - spec.Name は互換用に残すなら runner 内で吸収（呼び出し側は触らない）
- IF spec.Argv len==0: error STOP
- Runner(Prod/Fake) の双方で:
  - 実行名/引数の解釈が一致する
  - evidence.command_list.Argv が常に同一形式で記録される

PHASE 3: cwd 契約の硬化（repo外脱出禁止）
- RULE: ExecSpec.Dir は “repo root 相対” として解釈する
  - 空なら "."
  - 絶対パスは error STOP
  - Clean + Join(repoRoot, rel) 後に:
    - IF repo外 -> error STOP
- evidence.command_list.CwdRel は slash 正規化で固定

PHASE 4: env 契約の硬化（曖昧禁止）
- RULE: ExecSpec.Env は “差分（override）” として解釈
  - 実効 env = inherit_host_env + overrides
  - evidence:
    - EnvMode: "inherit+delta"
    - EnvKV: overrides をキーソートで記録（必要最小）
    - EnvHash: 実効 env 全体から決定論ハッシュ
- IF strict/full-env をやりたくなったら今回は STOP（別Sへ）

PHASE 5: portable evidence の絶対パス対策（argv + stdout/stderr）
- argv:
  - review_bundle の scriptPath を絶対にしない（repo相対で固定）
- stdout/stderr:
  - runner 側で repoRoot prefix を "<REPO_ROOT>" に置換（または "./"）
  - redaction をした事実は evidence に残す（Redactions フィールド等）
- IF /Users や drive letter が evidence に混入 -> error STOP

PHASE 6: review_bundle hardening（高リスク部）
- findReviewBundleScript:
  - os.Getwd 依存を廃止
  - repoRoot + 候補relpath で探索
- generateReviewBundle:
  - stdout/stderr を混ぜない
  - OK: 行は stdout を後ろから走査して最後の OK: を拾う
  - 返す bundlePath は repo相対で固定（絶対なら Rel(repoRoot, abs)）

PHASE 7: Tests（execしない）
- FakeRunner + pure function で契約を固定
  - ExecSpec argv解釈が Name に依存しないこと
  - Dir escape ("..") が error になること
  - redaction で "<REPO_ROOT>" に置換されること
  - review_bundle の stderr 混線が解析に影響しないこと

PHASE 8: Gates & Evidence
- go test ./... -count=1 PASS
- nix run .#prverify PASS （clean rail で取る）
- docs/evidence/prverify に最終1本だけ追加（既存は改変しない）
- SOT / TASK 更新
