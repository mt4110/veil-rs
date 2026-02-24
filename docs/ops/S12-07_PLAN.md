# S12-07 PLAN: SOT命名/プレースホルダ禁止 + stdout契約の継続監査（最小コスト固め）

## 0. SOT（Source of Truth）
- Progress board: `docs/ops/STATUS.md`
- このフェーズの計画/手順: このファイル（`docs/ops/S12-07_PLAN.md`）と `docs/ops/S12-07_TASK.md`
- 実装チェックの注入先（discoveryで確定→焼き付け済み）:
  - `IMPL_DIR = cmd/prverify`
- 観測ログ（discovery証拠）:
  - `OBS_DIR = .local/obs/s12-07_freeze_20260224T061833Z`

## 1. Goal（狙い）
1) **SOTの命名/プレースホルダ禁止**を「ツールで強制」して再発不能にする  
2) **stdout契約（STDOUT_CONTRACT_v1）の継続監査**を「軽い静的監査」で回し、未来の逸脱を最小コストで潰す

## 2. Non-Goal（やらない）
- 過去の全ドキュメントを大規模リネーム（今はやらない）
- すべてのバイナリ/全コマンドの出力を完全に形式検証（コスト過大）
- “PR番号が確定する前” に docs/pr を仮置きする（**TBD禁止**）

## 3. Contracts / Invariants（不変条件）
- **止まらない**：例外/exitコード依存で制御しない（失敗は `ERROR:` と `stop=1` で表現）
- stdoutは原則 `OK:` / `ERROR:` / `SKIP:` の機械可読ライン
- フェーズ終端は必ず `OK: phase=end stop=<0|1>` を出す（成功/失敗に関係なく）
- 変更は最小：必要な箇所だけ触る

## 4. Spec（仕様）
### 4.1 SOT doc naming rule（docs/pr）
- 許可: `docs/pr/PR-<digits>-<slug>.md`（digitsは1文字以上の数字）
- 禁止（例）:
  - `docs/pr/PR-TBD-...`
  - `docs/pr/PR-XXX-...`
  - `docs/pr/PR-??-...`
  - `docs/pr/PR-<non-digit>-...`（PR-xx 等）
- 追加で禁止（運用を壊すやつ）:
  - ファイル名または本文に `PR-TBD` / `PR-XXX` / `PR-??` が残っている

### 4.2 STATUS evidence rule（最小）
- `docs/ops/STATUS.md` 内で Evidence が `docs/pr/` を指す場合：
  - **そのパスが実在**していること
  - （可能なら）ファイル名が上記ルールに一致すること
- Evidence が `docs/pr/` 以外を指すものは、このフェーズでは強制対象外（最小コスト）

### 4.3 stdout契約の継続監査（軽い静的監査）
- 対象（最小）:
  - `scripts/*.py` のうち、`if __name__ == "__main__":` を含むもの（= entrypoint疑い）
- 要求:
  - ファイル内に `OK: phase=end` が存在すること  
  - （将来の逸脱検知用に）`OK:`/`ERROR:`/`SKIP:` のプレフィクス運用が明確であること  
- ここは “強すぎる形式検証” ではなく、**逸脱の早期発見**を狙う

## 5. Implementation Plan（段階・止まらない）
### PHASE 0: discovery（light）
- `cmd/prverify` を実装注入先として採用（このPLANに焼き付け済み）
- 既存のチェック入口（main/runner）とテスト配置を確認する

### PHASE 1: implement guard（medium）
- `cmd/prverify` に以下の2チェックを追加（既存のprverify/check-sotフレームに合わせる）
  1) `SOT doc naming/placeholder ban`
  2) `stdout contract ongoing audit (python entrypoints)`
- 違反時：
  - `ERROR: <reason>` を出す
  - `stop=1`
- 常に最後に：
  - `OK: phase=end stop=<0|1>`

### PHASE 2: tests（light）
- unit test を最小で追加（fixtureは `t.TempDir()` でOK）
  - valid: `docs/pr/PR-91-s12-07-foo.md`
  - invalid: `docs/pr/PR-TBD-s12-07-foo.md`
  - invalid: `docs/pr/PR-xx-s12-07-foo.md`
  - status evidence -> missing file を検出
  - python entrypoint に `OK: phase=end` が無いケースを検出

### PHASE 3: docs & SOT update（light）
- `docs/ops/STATUS.md` の S12-07 を 1% (WIP) に（PR作業開始の合図）
- PR番号確定後のみ `docs/pr/PR-<number>-s12-07-*.md` を作成（**TBD禁止**）

## 6. DoD（Definition of Done）
- placeholder docs/pr が **チェックで確実に落ちる**（stop=1）
- python entrypoint の `OK: phase=end` 欠落が **チェックで確実に落ちる**
- 追加テストが通る
- CI green
- マージ後、S12-07 が 100% (Merged) へ更新される

## 7. Evidence（このPLANの根拠）
- discovery証拠: `.local/obs/s12-07_freeze_20260224T061833Z/impl_dir.txt`
- 作業中の観測ログは `.local/obs/s12-07_*` にUTC命名で残す

Last Updated: 2026-02-24 (UTC)
