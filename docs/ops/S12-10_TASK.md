# S12-10 TASK — Reviewbundle Pack Contract v1

## 実行ルール（必読）
- exit系禁止（終了コード依存の設計にしない）
- 失敗は `ERROR:` 出力 + `stop=1`
- 重い処理は強制しない（必要なら任意/CIで担保）
- 各ステップは “観測ログ（OBS）” を残す（監査可能性）

## フェーズ0：準備（軽い）
- [x] main 最新化（S12-09後処理）
- [x] S12-10 ブランチ作成（`s12-10-reviewbundle-pack-contract-v1`）

## フェーズ1：Discovery（超重要・軽い）
- [x] `cmd/reviewbundle` の実装位置と契約関連ワードを観測（rgは範囲限定）
- [x] bundle の出力先（`.local/out` 等）を浅く観測（削除しない）
- [ ] evidence_report の実体（ファイル名/パス/形式）を特定
- [ ] 既存の “verify/manifest/sha” 相当処理があるか確認

**成果物（OBS）**
- `.local/obs/s12-10_discovery_*/rg_cmd_reviewbundle_contract.txt`
- `.local/obs/s12-10_discovery_*/rg_*_reviewbundle.txt`

## フェーズ2：ドキュメント固定（このPRの核）
- [x] `docs/ops/S12-10_PLAN.md` 作成（疑似コードつき）
- [x] `docs/ops/S12-10_TASK.md` 作成（順序固定チェックリスト）
- [x] `docs/ops/REVIEWBUNDLE_PACK_CONTRACT_v1.md` 作成（契約本文）
- [ ] `docs/ops/STATUS.md` 更新（S12-09=100% / S12-10=1% WIP）

## フェーズ3：manifest v1（実装：小さく）
- [ ] manifest の ファイル名/配置 を確定（Discovery結果で固定）
- [ ] bundle 生成時に manifest を生成（strictは必須）
- [ ] files の列挙規則を固定（pathソート）
- [ ] sha256 計算の予算（budget）方針を決める（端末保護）

## フェーズ4：verify コマンド（stopless）
- [ ] `cmd/reviewbundle verify`（または相当サブコマンド）を追加
- [ ] verify は bundle だけを読む（repo全スキャンしない）
- [ ] missing/extra/size/sha256/evidence_report 禁止物を検出し `stop=1`
- [ ] 例外で落ちない（unexpected は `ERROR: unexpected_exception`）

## フェーズ5：テスト（小さく・軽く）
- [ ] 小さい fixture bundle（数KB〜数十KB）を `testdata/` 等に用意
- [ ] verify OK / manifest欠落 / sha不一致 / extra file をテスト
- [ ] ローカルで重い場合は実行を強制しない（CIで担保）

## フェーズ6：運用破綻防止（契約に書く）
- [ ] 出力墓地化（cemetery）方針を契約に明記
  - 上書き禁止 / 衝突時の退避先 / 退避ログ
- [ ] 掃除（cleanup）は “削除実行” ではなく “候補列挙” を基本に（誤爆防止）
- [ ] 互換（compat）方針（versioning / deprecate）を明記

## フェーズ7：PR 最終チェック（軽い）
- [ ] `docs/ops` の差分確認（嘘がない）
- [ ] コマンド例が exit を使っていないか確認
- [ ] CI が通る前提の最小実装になっているか確認

## DoD
- strict bundle: manifest v1 + verify OK
- 改ざんで ERROR + stop=1
- docs と STATUS が一致し、嘘がない
