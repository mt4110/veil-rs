# Reviewbundle Pack Contract v1（manifest/schema/verify）

## 目的
「bundle が存在する = 検証できる」を契約にする。
生成できるだけでは不十分。検証可能性 / 互換性 / 掃除可能性 を“仕様”として固定する。

## 1. 適用範囲
本契約は `reviewbundle` が出力する bundle（ディレクトリ）に適用する。
特に **strict bundle** は本契約の MUST を満たすこと。

## 2. MUST（必須）
### 2.1 manifest v1 の同梱
- bundle には `manifest v1` が存在すること
- manifest の場所（相対パス）は **固定であること**（後方互換の核）

### 2.2 閉世界（file set）
- manifest は bundle 内ファイル集合を閉じる（原則）
- manifest に載っていないファイルが bundle に存在したら `ERROR: file_extra`
- manifest に載っているファイルが無ければ `ERROR: file_missing`

### 2.3 path 安全性
- manifest の `files[].path` は相対パスのみ
- 絶対パス、`..`、OSドライブ表現を禁止する

### 2.4 verify は stopless
- verify は失敗しても終了コードで制御しない（exit禁止の思想）
- 出力で `stop=1` を示し、`ERROR:` を必ず出す（嘘をつかない）

## 3. SHOULD（推奨）
- `files[]` は path ソート（安定差分）
- `sha256` は lower-hex
- bundle 生成時は上書きしない（出力墓地化 or 新規ID）

## 4. manifest v1（スキーマ要点）
厳密な JSON Schema 化は任意だが、“フィールド意味”はここで固定する。

- `kind`: `"reviewbundle.manifest"`
- `version`: 1
- `created_utc`: RFC3339 UTC
- `bundle.format`: `"strict"`（まずは strict 必須）
- `bundle.id`: 生成ID（例: timestamp）
- `paths.evidence_report`: evidence_report の相対パス
- `files[]`:
  - `path`（相対）
  - `size`（bytes）
  - `sha256`（lower-hex）
  - `role`（任意）

## 5. verify 規範
verify は最低限以下を検証する：
- manifest の存在
- schema最小条件（kind/version等）
- file set の一致（missing/extra）
- size 一致
- sha256 一致（※端末保護の budget を持つ場合は、超過を `ERROR: budget_exceeded` として明示）
- evidence_report の禁止物検出（例：`「://」`、絶対パスっぽい文字列、`..`）

## 6. 出力墓地化（cemetery）/ 掃除（cleanup）
### 6.1 上書き禁止
writer は同一パスへの上書きを避けること。
衝突した場合：
- 既存出力を cemetery に退避してから新規生成、または新規IDを採る

### 6.2 cleanup は安全第一
原則：削除ではなく “候補列挙（dry-run）”
- 実削除は別フェーズで明示的に行う（誤爆防止）

## 7. 互換（compat）方針
manifest.version が上がる場合：
- verify は「理解できない version」は `ERROR: manifest_version_unsupported`
- ただし落ちない（stopless）
- deprecate を行う場合は docs に期限（UTC）と理由を残す
