# S12-11 PLAN: reviewbundle verify SSOT v1

## 0. 現状とゴール

### 現状
- S12-10 は SSOT 契約（docs）を確立済み（contract v1）。
- 実装（verify/create）とテストは SSOT と完全同期しているとは限らない。

### S12-11 ゴール（このPRの目的）
SSOT（以下の3点セット）を **docsだけでなく実装の verify/create と軽量テストに完全同期**させる。

- `review/meta/contract.json`
- `review/meta/SHA256SUMS`
- `review/meta/SHA256SUMS.sha256`（SHA256SUMS の seal）

狙いは「bundle が存在する＝検証できる」を **壊せない**状態にする：
- 改ざん検出（missing/extra/seal broken/sha mismatch）
- 閉世界（closed world）
- budget safety valve（重くなる前に止める、ただし exit しない）
- evidence 部分スキャン（狙い撃ち：file スキーム／絶対パス臭／../ を検出、https は殺さない）

## 1. 成果物（S12-11 の固定スコープ）
- `cmd/reviewbundle/verify.go`
  - SSOT準拠の verify 強化
  - 閉世界・seal検証・budget・evidence部分スキャンを実装に降ろす
- `cmd/reviewbundle/create.go`
  - strict/wip 生成物が契約を満たすことを担保（必要なら軽い内部検証）
- `cmd/reviewbundle/errors.go`
  - ErrorCode と出力整合（`E_SHA256` 等 + `budget_exceeded` 等）
- テスト（軽量 fixture）
  - `cmd/reviewbundle` 配下で完結
  - 数KB〜数十KBで改ざん系を cover
- docs
  - `docs/ops/S12-11_PLAN.md`（本ファイル）
  - `docs/ops/S12-11_TASK.md`
  - `docs/ops/STATUS.md`（S12-11=1% WIP）

## 2. 非ゴール（このPRでやらない）
- 大容量bundleの完全性能最適化（budget を入れるので「重くなる前に止める」が先）
- 外部ネットワークアクセスや PR 取得の改善（verify/create の SSOT 同期が主）
- 既存の CLI UI を大幅に破壊する変更（互換性優先）

## 3. “止まらない” 実装・運用ルール（S12-11）
- exit / return 非ゼロ / set -e / trap EXIT 等で **強制終了させない**
- 成否は「終了コード」ではなく **出力の真実（`OK:` / `ERROR:` / `SKIP:` と `stop=`）**で表現する
- 失敗したら：
  - `ERROR: <reason> ... stop=1` を残す
  - 以降の工程は **進めない**（ただしプロセスは落とさない）
- 重い処理は 1ステップ1本に分割（OBS に観測結果を残す）

## 4. SSOT：固定パス（実パス固定）
本PRでは以下を “固定の真実” として扱う（bundle root からの相対パス）：

- Contract: `review/meta/contract.json`
- Sums:     `review/meta/SHA256SUMS`
- Seal:     `review/meta/SHA256SUMS.sha256`
- Evidence root（最低限ここはスキャン対象にする）:
  - `review/evidence/prverify`

※ contract が evidence の追加パスを持つ場合は「追加で」対象にできるが、
本PRの最小保証は上記固定パス。

## 5. verify の期待仕様（DoD を実装仕様へ落とす）

### 5-1. 出力（終了コード依存なし）
- 成功時：最終的に `OK: verify stop=0` を必ず出す
- 失敗時：最終的に `ERROR: <reason> stop=1` を必ず出す
- 途中ログは増えて良いが、最後の1行で判定できること

### 5-2. strict bundle の verify（stop=0）
- meta 3点セットが存在し、相互整合している
- SHA256SUMS.sha256 が SHA256SUMS の sha256 と一致（seal OK）
- SHA256SUMS に載っている各ファイルの sha256 が一致（改ざんなし）
- 閉世界：実ファイル集合が SSOT と一致（extra/missing なし）
- evidence スキャンで禁止物が出ない（https は許可）

### 5-3. 改ざん・欠落・余計なファイル
以下は stop=1：
- missing（SSOTにあるが実体がない）
- extra（SSOTにないが実体がある）
- seal broken（SHA256SUMS の内容が seal と一致しない）
- sha mismatch（ファイルが SHA256SUMS と一致しない）

### 5-4. budget safety valve
- budget 超過時：`ERROR: budget_exceeded ... stop=1`
- budget は verify 全体（walk / read / hash / scan）に跨って消費
- budget 超過後に無理に続けない（止める＝進めない）が、exit はしない

### 5-5. evidence 部分スキャン（狙い撃ち）
禁止（stop=1）にする候補（最小セット）：
- file スキーム：`file: + //` / `file: + ///` / `file:/` / `file:\`（大文字小文字無視）
- 親ディレクトリ：`../`（ただし https を殺さない＝URL内は基本スルーできる形にする）
- 絶対パス臭：
  - 先頭 or 空白/引用符の直後に `/Users/` `/home/` `/etc/` `/var/` `/private/` `/Volumes/` `/mnt/` 等
  - Windows：`C:\` `D:\` / `C:/` 等（先頭 or 空白/引用符の直後）

許可：
- `https://`（これ自体は絶対に止めない）
- `http://`（同上）

スキャン範囲：
- 基本は `review/evidence/prverify/**`（存在するファイルのみ）
- バイナリっぽいものは SKIP しても良い（ただし OBS に残す）

## 6. create の期待仕様（SSOT 同期）
- strict 生成物は SSOT を満たす（contract + sums + seal が正しい）
- 可能なら create の最後に **軽量な内部 verify** を実行して、
  - NGなら `ERROR: create_generated_invalid_bundle stop=1` を出す
  - OKなら `OK: create stop=0` を出す
- wip は「SSOT未完」でも作れて良いが、strict は必ず固める

## 7. errors.go：ErrorCode と標準出力の整合

### 7-1. ErrorCode 方針
- “分類” は `E_*` で揃える（例：`E_SHA256`, `E_SEAL`, `E_MISSING`, `E_EXTRA`, `E_BUDGET`, `E_EVIDENCE`）
- “表示上の reason” は読みやすい固定語も併記できる（特に budget は `budget_exceeded` を優先）

### 7-2. 最低限ほしい reason（出力先頭語）
- `budget_exceeded`（最重要：DoDで固定）
- `missing_file`
- `extra_file`
- `seal_broken`
- `sha_mismatch`
- `evidence_forbidden`

## 8. 実装アーキテクチャ（軽量・閉世界・予算）

### 8-1. 内部データ構造（案）
- VerifyOptions
  - BundleRoot string
  - Mode string（strict/wip。既存があるならそれに合わせる）
  - BudgetBytes int64（既定値あり）
  - BudgetFiles int（既定値あり）
  - EvidenceScan bool（既定 true）
- VerifyReport
  - Stop int（0/1）
  - Errors []RBError（複数収集しても良いが、budget超過は即止めでもOK）
  - Observations map[string]string（任意：OBS用の要点）
- RBError
  - Code ErrorCode
  - Reason string（出力先頭語に対応）
  - Path string（関係パス）
  - Detail string

### 8-2. budget の扱い（疑似コード）
try:
  budget = {maxBytes, maxFiles, usedBytes=0, usedFiles=0}

  for each file in walk(bundleRoot):
    if usedFiles+1 > maxFiles:
      error budget_exceeded; stop=1
    usedFiles++

  for each path in sha256sums:
    size = stat(path)
    if usedBytes+size > maxBytes:
      error budget_exceeded; stop=1
    hash(path)  # 読んだ分だけ usedBytes を増やす

  for each evidence file:
    read chunked with limit
    if usedBytes+chunk > maxBytes:
      error budget_exceeded; stop=1

catch:
  error panic_recovered; stop=1

finally:
  print final OK/ERROR + stop

※ “budget超過” は即 stop=1 にして以降を進めない（止まらない＝exitしない）。

### 8-3. 閉世界（closed world）
最も安全で単純な閉世界：
- 「実ファイル集合」と「SSOT集合（contract + sums）」が一致していること
- ただし SSOT集合の定義は discovery で確定する

最小保証（S12-11 の段階）：
- `SHA256SUMS` に載っているファイル群（+ meta 3点）は “許可集合”
- bundle から walk したファイルが許可集合に含まれない => extra
- 許可集合にあるが実体がない => missing
- symlink を見つけたら stop=1（閉世界を壊せるため）

### 8-4. seal 検証（最小の法）
- `SHA256SUMS.sha256` から期待 hash を読む
- 実 `SHA256SUMS` の sha256 を計算
- 不一致 => `seal_broken` stop=1

## 9. テスト戦略（軽量 fixture）
- fixture は数KB〜数十KBで完結
- テストケース（最低限）：
  - OK（strict 正常）
  - missing
  - extra
  - seal broken（SHA256SUMS を改変）
  - sha mismatch（対象ファイルを改変）
  - budget exceeded（超小budget）
  - evidence forbidden（file: + // / ../ / 絶対パス臭）
  - evidence allow（https を含むが stop=0）

実装方針：
- `t.TempDir()` + 固定内容で bundle を生成してもOK（再現性は “固定データ” で担保）
- ただし “fixture固定” を重視するなら `cmd/reviewbundle/testdata/` に最小bundleを置いて
  そこからコピーする方式もOK

## 10. OBS（観測点）
- `.local/obs/s12-11_*_YYYYmmddTHHMMSSZ` を各ステップで作る
- “軽い rg / ls / sed” で事実だけを残す
- OK/ERROR/SKIP と stop を必ず残す
