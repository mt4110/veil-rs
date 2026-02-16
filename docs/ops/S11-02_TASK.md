# S11-02 TASK — SOT guidance truth (ordered)

## Phase 0 — Safety snapshot
- [x] cd "$(git rev-parse --show-toplevel)"
- [x] git status -sb（dirtyなら理由を1行メモ。嘘は書かない）
- [x] git rev-parse --abbrev-ref HEAD（s11-02-sot-guidance-truth-v1 であること）

## Phase 1 — Update STATUS (SOT board)
- [x] docs/ops/STATUS.md を更新
  - [x] S11-01 = 100% (Merged), Current = -
  - [x] S11-02 = 0%, Current = S11-02 Planning
  - [x] Last Updated Evidence = docs/evidence/prverify/prverify_20260216T100922Z_ebec5cd.md

## Phase 2 — Create PLAN/TASK files
- [x] docs/ops/S11-02_PLAN.md 作成
- [x] docs/ops/S11-02_TASK.md 作成（このファイル）

## Phase 3 — Locate all stale guidance (truth discovery)
- [x] rg -n "veil sot new|SOT Missing|Check SOT existence" .github docs -S
- [x] ヒット一覧（ファイル/行）をメモ（差分レビューの根拠）

## Phase 4 — Replace guidance with truthful manual steps (message-only)
- [x] .github/workflows/pr_sot_guard.yml
  - [x] 判定ロジックは変更禁止
  - [x] echo/表示文言のみ `veil sot new` を排除し、手動手順へ
- [x] .github/PULL_REQUEST_TEMPLATE/*.md
  - [x] `veil sot new ...` を “手動SOT作成手順” に置換（周辺構造は維持）
- [x] docs/pr/README.md / docs/pr/sot_template.md / docs/v0.20.0-planning/...
  - [x] 同様に置換（[LEGACY] 注記があるなら保持）

## Phase 5 — Assert zero remaining refs
- [x] rg -n "veil sot new" .github docs -S が 0 件であること（証拠）

## Phase 6 — Gates + Evidence
- [x] nix run .#prverify（PASS）
- [x] 生成された prverify evidence を docs/evidence/prverify にコミット
  - Evidence: `docs/evidence/prverify/prverify_20260216T110132Z_9863b52.md`

## Phase 7 — Commit & PR
- [ ] git add docs/ops docs .github
- [ ] commit（message-only + STATUS 更新が分かる題）
- [ ] git push -u origin s11-02-sot-guidance-truth-v1
- [ ] PR 作成（本文テンプレ使用）
