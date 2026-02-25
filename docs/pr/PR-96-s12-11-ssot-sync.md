# PR: S12-11 verify/create SSOT sync (contract+SHA256SUMS+seal), budget, evidence scan, light tests

## Goal
SSOT（1. contract.json, 2. SHA256SUMS, 3. SHA256SUMS.sha256）を実装（verify, create）の真実と一致させ、閉世界（closed world）、budget上限（ファイル数・バイト数）、狙い撃ちのevidenceスキャンを導入して検証の堅牢化を行う。

## Changes
1. **errors.go**:
   - `E_MISSING`, `E_EXTRA`, `E_SEAL`, `E_BUDGET` の追加。
   - `VError` に `Reason` プロパティを追加。
   - output format (`ERROR: <reason> ... stop=1`) に適合する `Line()` メソッドの追加。
2. **verify.go (Budget & SSOT Sync)**:
   - `VerifyOptions` 構造体を導入（`BudgetBytes`, `BudgetFiles`, `EvidenceScan`）。
   - TARストリームのパース時にファイル数・バイト数の Tracking を行い、Budget を超過した場合は即座に `stop=1` を返す（`budget_exceeded`）。
   - Evidence 対象ファイルのコンテンツ内に禁止された絶対パスや file スキームが含まれないかスキャン。
3. **checksums.go (Closed World & Safety)**:
   - `VerifyChecksumCompleteness` で不足ファイル (`missing_file`) と超過ファイル (`extra_file`) を厳密に検出。
   - `ParseSHA256SUMS` において、マニフェスト内のパスに対する安全検証（親ディレクトリ走破、OSドライブレター指定、絶対パス等の禁止）を追加。
   - SSOT シール（`SHA256SUMS.sha256`）の不一致で `seal_broken` を返すように修正。
4. **create.go / main.go**:
   - create 中のセルフオーディットで失敗した場合、`create_generated_invalid_bundle stop=1` を出力し、正常時は `OK: create stop=0` を出力するように統一。
5. **Tests**:
   - 変更が意図通りに動作することを保証するため、`evidence_scan_test.go` を新設および `verify_test.go`, `verify_binding_test.go`, `hermetic_repo_test.go` へ `VerifyOptions` を適用。

## Review Instructions
- 動作は全て "stopless" (exit 0) のまま、エラー内容は `ERROR: <reason> ... stop=1` 出力で表現されているか確認してください。
- ファイルサイズおよびファイル数の budget が安全に動作しているか（tar の展開中に制限値を越えれば止まるか）確認してください。
- 破壊的変更がないか（すべてのエラーが graceful に報告されるか）確認してください。

## State
- STATUS: 99% (Review)
- S12-11 PLAN: `docs/ops/S12-11_PLAN.md`
- S12-11 TASK: `docs/ops/S12-11_TASK.md`
