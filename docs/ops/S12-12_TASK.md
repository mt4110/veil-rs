# S12-12 TASK: reviewbundle contract v1 enforce

> stopless / heavy禁止 / OBS必須

## 0) STATUS closeout + kickoff
- [ ] STATUS: S12-11 を 100% (Merged PR #96) に更新
- [ ] STATUS: S12-12 行を追加し 1% (WIP) にする

## 1) Discovery (OBS)
- [ ] cmd/reviewbundle の contract 関連を rg で観測
- [ ] 直近 strict bundle があれば contract.json を抽出してキーを観測
- [ ] 契約doc v1 を先頭だけ読む（重くしない）

## 2) contract v1 実装
- [ ] contract_v1.go を追加（struct + Parse + Validate）
- [ ] 必須フィールドの最小セットを確定（Discovery結果に合わせる）
- [ ] schema/version mismatch を reason 固定で出す

## 3) verify へ統合
- [ ] verify の flow に contract validate を挿入（stopless）
- [ ] contract↔SHA256SUMS の cross-check を実装
- [ ] エラーは必ず `ERROR: <reason> ... stop=1`

## 4) create へ統合
- [ ] create が contract.json を struct から生成（安定）
- [ ] self-audit が contract validate を通す（verify 呼び出しを再利用）

## 5) Tests (light)
- [ ] contract parse OK/NG（invalid json / missing field / schema mismatch）
- [ ] contract↔sums mismatch（file list mismatch 等）

## 6) Docs
- [ ] docs/pr/PR-97-s12-12-contract-enforce.md を作る
- [ ] TASK のチェックを更新
- [ ] STATUS を更新（このPRの Current/Evidence を指す）

## 7) Final
- [ ] go test は対象限定で回す（reviewbundle中心）
- [ ] 最後に prverify を回して証拠を PR doc に貼る
