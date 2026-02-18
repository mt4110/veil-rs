# S12-00 TASK — Kickoff

## 0) Preflight (truth)
- [ ] `git status -sb`
- [ ] `rg -n "^\| S12" docs/ops/STATUS.md`

## 1) Goal 1-line を確定
- [ ] docs/ops/S12_00_PLAN.md の `GOAL_1LINE:` を 1行で埋める（TBDを消す）

## 2) STATUS を更新（S12: 1% (WIP), Current: S12-00 Kickoff）
- [ ] S12 行の `%` と `Current` を更新
- [ ] `Last Updated:` がある場合は日付更新（なければ無視でOK）

## 3) ガード（禁止混入）
- [ ] `rg -n "file[:]//|/[U]sers/" docs/ops/S12_00_PLAN.md docs/ops/S12_00_TASK.md docs/ops/STATUS.md || true`

## 4) Commit / Push
- [ ] `git add docs/ops/S12_00_PLAN.md docs/ops/S12_00_TASK.md docs/ops/STATUS.md`
- [ ] `git commit -m "docs(ops): kickoff S12-00 (define 1-line goal)" || true`
- [ ] `git push -u origin <BRANCH>`

## 5) PR作成→番号確保→SOT作成→PR本文更新
- [ ] PRをdraftで作成
- [ ] PR番号を取得
- [ ] docs/pr/PR-<PRNUM>-s12-00-kickoff.md を作成
- [ ] PR本文のSOT/Evidenceを更新

## 6) CI緑確認
- [ ] `gh pr checks --watch || true`
