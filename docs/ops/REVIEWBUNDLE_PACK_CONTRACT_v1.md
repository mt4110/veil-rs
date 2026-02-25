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

### 2.3 path 安全性（SHA256SUMS の path 列に適用）

本契約における “path 安全性” は、`review/meta/SHA256SUMS` に列挙される **path 列**に対して適用する。

禁止（MUST）：
- 絶対パス（例：`/` で始まるもの）
- 親ディレクトリ参照（`..` を含むもの、`../` など）
- OS ドライブ表現（例：`C:\` など）

追加制約（MUST）：
- すべての path は `review/` 配下であること（bundle root 逸脱禁止）

### 2.4 verify は stopless
- verify は失敗しても終了コードで制御しない（exit禁止の思想）
- 出力で `stop=1` を示し、`ERROR:` を必ず出す（嘘をつかない）

## 3. SHOULD（推奨）
- `SHA256SUMS` のエントリは path ソート（安定差分・再現性）
- sha256 は lower-hex（`[0-9a-f]`）
- bundle 生成時は上書きしない（出力墓地化 or 新規ID）
- verify は “重い処理を勝手に走らせない”（bundle 単体で自己完結）

## 4. SSOT（contract.json + SHA256SUMS）

本契約の SSOT（Source of Truth）は以下の 3 点セットである：

- `review/meta/contract.json`
  - 契約メタ（`contract_version`, `mode`, `evidence.required`, `evidence.path_prefix` など）
- `review/meta/SHA256SUMS`
  - 対象ファイルのハッシュ一覧（事実上の manifest 本体）
  - 形式：`<sha256_hex><spaces><path>`
- `review/meta/SHA256SUMS.sha256`
  - `SHA256SUMS` 自身のシール（改ざん検知）
  - 形式：`SHA256SUMS` の sha256 を 1 行で保持（再帰を回避）

閉世界（file set）の定義：
- `S_hash` = `SHA256SUMS` に列挙された path 集合
- `S_meta` = `{ review/meta/SHA256SUMS, review/meta/SHA256SUMS.sha256 }`
- `S_allow` = `S_hash ∪ S_meta`
- `S_actual`（bundle 内実ファイル）と `S_allow` が一致しなければ `ERROR: file_missing` または `ERROR: file_extra`（+ `stop=1`）

注記：
- JSON 形式の “manifest v1” は本契約では採用しない（SSOT に一本化する）

## 5. verify 規範（stopless / SSOT 準拠）

verify は bundle 単体で自己完結し、以下を最低限検証する（MUST）。

### 5.1 stopless
- verify は終了コードで制御しない（exit 禁止の思想）
- 失敗時は `ERROR:` を出力し、`stop=1` を明示する
- 例外・想定外も “落ちない” で `ERROR: unexpected_exception` とする

### 5.2 SSOT の存在と整合
- `review/meta/contract.json` が存在する
- `review/meta/SHA256SUMS` が存在する
- `review/meta/SHA256SUMS.sha256` が存在する
- `SHA256SUMS.sha256` と `SHA256SUMS` の整合（seal 検証）が取れる  
  - 不整合は `ERROR: manifest_seal_broken`

### 5.3 contract.json の最低限検証（過剰な schema 強制はしない）
- `contract_version` が空でない
- `mode` が `strict` または `wip`
- `mode=strict` の場合、`evidence.required=true` を必須とする

### 5.4 閉世界（file set）
- `S_allow = S_hash ∪ S_meta` を定義し、
  - `S_actual` に不足があれば `ERROR: file_missing`
  - `S_actual` に余計があれば `ERROR: file_extra`

### 5.5 sha256 一致（budget safety valve）
- `SHA256SUMS` の各エントリについて、実ファイルの sha256 が一致する
- CPU/端末保護のため、budget を持ってよい（MAY）：
  - 上限超過時は `ERROR: budget_exceeded` を明示し `stop=1`（黙って PASS しない）

### 5.6 path 安全性
- `SHA256SUMS` の path 列に対して `2.3 path 安全性` を適用する
- 違反は `ERROR: path_invalid:<path>`

### 5.7 evidence の禁止物検出（狙い撃ち）
verify は evidence を “過剰に” 禁止しない（例：`https` を禁止しない）。  
検出は狙い撃ちで行う（MUST）。

- 禁止：file スキーム  
  - docs ガード安全のため表記は分割するが、実装としては `file:` の直後に `//` が隣接する形を検出する
- 禁止：絶対パス臭（例：`/Users/` や `C:\` など）
- 禁止：親ディレクトリ参照（`../` など）

※ evidence 全文を読むと重くなり得るため、verify は先頭 N バイトの部分スキャンで良い（MAY）。

## 6. 出力墓地化（cemetery）/ 掃除（cleanup）
### 6.1 上書き禁止
writer は同一パスへの上書きを避けること。
衝突した場合：
- 既存出力を cemetery に退避してから新規生成、または新規IDを採る

### 6.2 cleanup は安全第一
原則：削除ではなく “候補列挙（dry-run）”
- 実削除は別フェーズで明示的に行う（誤爆防止）

## 7. 互換（compat）方針（SSOT ベース）

互換の核は `review/meta/contract.json` の `contract_version` である。

- verify は未知の `contract_version` を検出した場合：
  - `ERROR: contract_version_unsupported` を出し `stop=1`
  - ただし落ちない（stopless）

`SHA256SUMS` の形式互換：
- 形式変更が必要になった場合は `contract_version` を上げて明示する
- 旧形式の継続サポート可否は docs に期限（UTC）と理由を残す（MUST）
