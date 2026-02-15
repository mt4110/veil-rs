- [x] 00 Preflight（clean rail / branch create / push）
- [x] 01 S10-10 docs 実在確認（ls docs/ops | rg 'S10_' confirmed files exist）
- [x] 02 Docs作成/更新（Plan/Task/SOT。file: + // 混入ゼロ確認）
- [x] 03 Discovery: entry points / contract / evidence の候補抽出
      - [x] 03.1 候補パス一覧（repo-relative）
            - cmd/prkit/main.go
            - internal/prkit/run.go
            - internal/prkit/sot.go
            - internal/prkit/exec.go
            - internal/prkit/cli.go
      - [x] 03.2 各候補の実在シンボル（rg 確定済）
            - cmd/prkit/main.go: Run
            - internal/prkit/run.go: RunDryRun, RunExecuteMode
            - internal/prkit/sot.go: ScaffoldSOT
            - internal/prkit/exec.go: Init, Run
- [x] 04 Decide: canonical entry の決定（Planに明記: prkit.Run in cli.go）
- [x] 05 Implement: contract single-entry 化
      - [x] cmd 側被せ皮化
      - [x] internal/prkit 側で契約集約
- [x] 06 Determinism: 非決定性の閉鎖
      - [x] time.Now -> prkit.Now() に置換
      - [x] USER env の安定化
- [x] 07 Tests（fake/stub優先、決定論の観点: TestDeterminism 追加）
- [x] 08 go test ./... -count=1（PASS証拠）
- [x] 09 nix run .#prverify（PASS証拠）
- [x] 10 prverifyレポート保存（docs/evidence/prverify/prverify_20260215T031938Z_16ff4b6.md）
- [x] 11 SOT更新（docs/pr/PR-TBD-v1-epic-A-s10-10-prkit-contract-single-entry.md）
- [x] 12 PR作成（PR #74）
- [x] 13 仕上げ（Task全チェック、STOP/skip/error の記録が残ってる）

## DoD（最終合格条件）
review/meta/warnings.txt == empty を最終ゴールにする。
これが空でない限り “監査ギャップが残存” と見なし、STOP → 原因除去 → 再生成する。

## Fixpack-2 (PR #74): Evidence/Bundle/PR入口/Unicode

- [ ] FP2-00 Preflight（branch/HEAD確定、git status clean確認）
- [ ] FP2-01 Untracked除去（prverify_out.txt / test_out.txt を消す。理由ログ残す）
- [ ] FP2-02 go test（go test ./... -count=1 PASS）
- [ ] FP2-03 prverify（nix run .#prverify PASS）
- [ ] FP2-04 prverifyファイル実在確認（docs/evidence/prverify/prverify_<UTC>_<HEAD7>.md）
- [ ] FP2-05 SOT Evidenceリンク更新（PR-74 SOT を HEAD の prverify に一致）
- [ ] FP2-06 Task Evidenceリンク更新（S10_10_TASK を HEAD の prverify に一致）
- [ ] FP2-07 PR本文更新（SOT/Evidence を HEAD のものに一致）
- [ ] FP2-08 Review bundle 再生成（ops/ci/review_bundle.sh）
- [ ] FP2-10 Unicode scan（0件なら docs/evidence/security/unicode_scan_<UTC>_<HEAD7>.md 保存）
- [ ] FP2-11 PR-TBD 検査（禁止スコープのみ：SOT/Task/PR本文）
- [ ] FP2-12 Gates再確認（必要なら prverify 再実行、最終整合OK）

## Final Audit Gate: review bundle warnings must be empty
- [ ] MODE=wip bash ops/ci/review_bundle.sh
- [ ] latest bundle を特定（ls -t ... | head -n 1）
- [ ] review/meta/warnings.txt を抽出して空であることを確認
- [ ] IF 非空 → error STOP（原因を潰して再生成、空になるまで繰り返す）
