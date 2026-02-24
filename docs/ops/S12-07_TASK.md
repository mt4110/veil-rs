# S12-07 TASK: SOT命名/プレースホルダ禁止 + stdout継続監査（最小コスト）

## 0) ブランチ
- [ ] `git switch -c s12-07-sot-docname-guard-v1`

## 1) discovery（軽い観測ログ）
- [ ] `ls -la docs/ops docs/pr cmd 2>/dev/null || true` をOBSへ
- [ ] 実装注入先の確認（このPLANでは確定済み）:
  - [ ] `IMPL_DIR = cmd/prverify`

## 2) PLAN/TASK（SOT）
- [ ] `docs/ops/S12-07_PLAN.md` が存在する（この生成で作成済みならOK）
- [ ] `docs/ops/S12-07_TASK.md` が存在する（この生成で作成済みならOK）

## 3) 実装（cmd/prverify）
- [ ] docs/pr 命名バリデータ（digits必須、TBD/XXX/??禁止）
- [ ] STATUS evidence（docs/pr/ を指す場合のみ）実在チェック
- [ ] stdout継続監査（scripts/*.py entrypointの `OK: phase=end` 必須）
- [ ] 違反時 `ERROR:` + `stop=1`、最後に `OK: phase=end stop=<0|1>`

## 4) テスト（最小）
- [ ] unit tests 追加（t.TempDir() fixtureでOK）
- [ ] 対象を絞って実行（例：cmd/prverify配下のみ）

## 5) STATUS.md 更新（開始の合図）
- [ ] `docs/ops/STATUS.md` の `| S12-07 ` 行を **1% (WIP)** に
- [ ] `Last Updated:` を更新

## 6) PR運用（TBD禁止）
- [ ] PR作成（番号確定）
- [ ] `docs/pr/PR-<number>-s12-07-*.md` を作成（**仮名禁止**）
- [ ] CI green → マージ
- [ ] マージ後 `STATUS.md` を 100% (Merged) に

## 7) 終端
- [ ] `OK: phase=end stop=<0|1>` を各手順ログ末尾で満たす

Last Updated: 2026-02-24 (UTC)
