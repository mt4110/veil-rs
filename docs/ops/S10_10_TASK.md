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
- [x] 10 prverifyレポート保存（docs/evidence/prverify/prverify_20260215T032428Z_c725312.md）
- [x] 11 SOT更新（docs/pr/PR-74-v1-epic-A-s10-10-prkit-contract-single-entry.md）
- [x] 12 PR作成（PR #74）
- [x] 13 仕上げ（Task全チェック、STOP/skip/error の記録が残ってる）

## DoD（最終合格条件）
完了宣言は以下がすべて成立した場合のみ許可する（未達なら error STOP）：
- review/meta/warnings.txt == empty
- review/meta/status.txt == empty（seal bundle 作成時点で clean）
- prverify_20260215T032428Z_c725312.md が存在し、SOT/Task/PR本文が同一パスを指す
- Unicode gate（changed files）: Bidi controls == 0 AND Unicode Cf == 0（証拠を repo に保存）

## Fixpack-2: Final Alignment & Audit Gate (DoD = warnings=0)

- [ ] FP2-01 Preflight: on PR head branch / git status clean / fetch origin
- [ ] FP2-02 prverify for CURRENT HEAD (go test, nix run .#prverify)
- [ ] FP2-03 Save prverify under docs/evidence/prverify/prverify_20260215T032428Z_c725312.md
- [ ] FP2-04 Align pointers: SOT Evidence + Task Evidence -> FP2-03 (no PR-TBD residue)
- [ ] FP2-05 Unicode gate:
      - [ ] Identify GitHub-warned file(s)
      - [ ] Scan for Bidi controls + Unicode Cf
      - [ ] Evidence: docs/evidence/security/unicode_scan_20260215T084639Z_c725312.md (PASS)
- [ ] FP2-06 Commit: docs(evidence): seal pointers + unicode evidence + task checkmarks
- [ ] FP2-07 Push: git push
- [ ] FP2-08 PR entry: gh pr edit 74 body -> correct SOT/Evidence/Unicode + DoD
- [ ] FP2-09 Audit seal: MODE=wip bash ops/ci/review_bundle.sh
- [ ] FP2-10 DoD check: review/meta/warnings.txt == empty (record proof)
- [ ] FP2-11 Final confirmation: PR page shows updated body + latest evidence links
