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
- [x] evidence_report の実体（ファイル名/パス/形式）を特定
- [x] 既存の “verify/manifest/sha” 相当処理があるか確認

**成果物（OBS）**
- `.local/obs/s12-10_discovery_*/rg_cmd_reviewbundle_contract.txt`
- `.local/obs/s12-10_discovery_*/rg_*_reviewbundle.txt`

## フェーズ2：ドキュメント固定（このPRの核）
- [x] `docs/ops/S12-10_PLAN.md` 作成（疑似コードつき）
- [x] `docs/ops/S12-10_TASK.md` 作成（順序固定チェックリスト）
- [x] `docs/ops/REVIEWBUNDLE_PACK_CONTRACT_v1.md` 作成（契約本文）
- [x] `docs/ops/STATUS.md` 更新（S12-09=100% / S12-10=1% WIP）

## フェーズ3：SSOT 固定（manifest=SHA256SUMS）と閉世界の法律化（実装：小さく）

> 目的：docs の真実に合わせて、実装側の “固定定数/閉世界/予算(budget)” を **一切の嘘なく**確定する。  
> 注意：ここで「新しい JSON manifest を追加」はしない。SSOT は既に存在する（contract.json + SHA256SUMS + SHA256SUMS.sha256）。

### 3-1) 固定パス定数の確定（実装/テスト/ドキュメント共通）
- [ ] `BUNDLE_ROOT = review/` を固定（逸脱は契約違反）
- [ ] `CONTRACT_PATH = review/meta/contract.json` を固定
- [ ] `HASH_MANIFEST_PATH = review/meta/SHA256SUMS` を固定
- [ ] `HASH_MANIFEST_DIGEST_PATH = review/meta/SHA256SUMS.sha256` を固定
- [ ] `EVIDENCE_PREFIX = review/evidence/` を固定
- [ ] `EVIDENCE_PRVERIFY_DIR = review/evidence/prverify/` を固定
- [ ] `INDEX_PATH = review/INDEX.md` / `PATCH_PATH = review/patch/series.patch` を固定（既に存在するなら契約に明記）

### 3-2) “閉世界（file set）” の定義を実装に落とす
- [ ] `S_hash = paths_in(SHA256SUMS)` を定義（SHA256SUMSのpath列）
- [ ] `S_meta = { review/meta/SHA256SUMS, review/meta/SHA256SUMS.sha256 }` を定義
- [ ] `S_allow = S_hash ∪ S_meta` を定義
- [ ] `S_actual`（bundle内の実ファイル）と `S_allow` の一致を仕様化
  - missing → `ERROR: file_missing` + `stop=1`
  - extra → `ERROR: file_extra` + `stop=1`

### 3-3) path 安全性（SHA256SUMSのpath列に適用）
- [ ] 絶対パス禁止（`/` で始まる等）
- [ ] `..` 禁止（parent traversal）
- [ ] OSドライブ表現禁止（例：`C:\`）
- [ ] すべて `review/` 配下であること（bundle root 逸脱禁止）

### 3-4) budget（端末保護）を “契約として” 定数化
- [ ] `MAX_FILES` を固定（例：5000）
- [ ] `HASH_BUDGET_BYTES` を固定（例：256MiB）
- [ ] 超過時は黙ってPASSしない：`ERROR: budget_exceeded` + `stop=1`
- [ ] evidence 部分スキャンの上限（例：`EVIDENCE_SCAN_BYTES=64KiB`）を固定（任意だが強い）

---

## フェーズ4：既存 verify の SSOT 準拠強化（stopless）

> 目的：verify を “追加” するのではなく、既存 verify の挙動を **Pack Contract v1（SSOT）に一致**させる。  
> 重要：verify は bundle 単体で自己完結（repo全体スキャン禁止）。重い処理は走らせない。

### 4-1) verify の対象を SSOT に統一
- [ ] `contract.json` の存在確認（無ければ `ERROR: contract_missing` + `stop=1`）
- [ ] `SHA256SUMS` / `SHA256SUMS.sha256` の存在確認（無ければ `ERROR: manifest_missing` + `stop=1`）
- [ ] seal 検証（`SHA256SUMS.sha256` と `SHA256SUMS` の整合）
  - 不整合 → `ERROR: manifest_seal_broken` + `stop=1`

### 4-2) contract.json の最低限チェック（過剰に schema 強制しない）
- [ ] `contract_version` が空でない（空なら `ERROR: contract_version_missing`）
- [ ] `mode` が `strict|wip`
- [ ] `mode=strict` の場合、`evidence.required=true` を必須（違反は `ERROR: strict_requires_evidence_required_true`）

### 4-3) 閉世界（file set）検証
- [ ] `S_allow = S_hash ∪ S_meta` を構成
- [ ] `S_actual` と比較し missing/extra を検出（`stop=1`）

### 4-4) sha256 検証（budget 付き）
- [ ] `SHA256SUMS` の entries を path ソートで処理（安定差分）
- [ ] `MAX_FILES` / `HASH_BUDGET_BYTES` を超えたら `ERROR: budget_exceeded` + `stop=1`
- [ ] sha256 不一致 → `ERROR: sha256_mismatch:<path>` + `stop=1`

### 4-5) evidence 禁止物検出（狙い撃ち／https を禁止しない）
- [ ] file スキーム検出（docs ガードの都合で表記は分割するが、実装では `file:` の直後に `//` が隣接する形を検出）
  - 検出 → `ERROR: forbidden_uri_scheme:file:<path>` + `stop=1`
- [ ] 絶対パス臭（例：`/Users/` や `C:\`）→ `ERROR: forbidden_absolute_path`
- [ ] `../` 等 → `ERROR: forbidden_parent_traversal`
- [ ] heavy 回避：evidence 全文は読まず先頭 `EVIDENCE_SCAN_BYTES` の部分スキャンでよい（任意だが推奨）

### 4-6) stopless（絶対）
- [ ] 例外で落ちない：`ERROR: unexpected_exception` + `stop=1`
- [ ] 終了コードに依存しない：最終行に `OK: phase=end stop=<0|1>` を出す

## フェーズ5：テスト（小さく・軽く / SSOT前提）

- [ ] 小さい fixture bundle（数KB〜数十KB）を `testdata/` 等に用意（tarでも展開済みdirでもOK）
- [ ] verify OK をテスト（stop=0）
- [ ] 改ざん系（stop=1）をテスト：
  - [ ] `review/meta/SHA256SUMS` 欠落 → `ERROR: manifest_missing`
  - [ ] `review/meta/SHA256SUMS.sha256` 欠落 → `ERROR: manifest_missing`
  - [ ] `SHA256SUMS` と seal 不整合 → `ERROR: manifest_seal_broken`
  - [ ] `SHA256SUMS` に存在しないファイルを 1つ混入 → `ERROR: file_extra`
  - [ ] `SHA256SUMS` にあるファイルを 1つ削除 → `ERROR: file_missing`
  - [ ] 1ファイルの内容改ざん → `ERROR: sha256_mismatch:<path>`
- [ ] budget 超過（意図的に大きめfixture）→ `ERROR: budget_exceeded`
- [ ] ローカルで重い場合は実行を強制しない（CIで担保）※ただし fixture は小さく保つ

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
- strict bundle:
  - `review/meta/contract.json` + `review/meta/SHA256SUMS` + `review/meta/SHA256SUMS.sha256` が入る
  - verify が `OK: phase=end stop=0` を出す
- 改ざんケース（欠落/extra/sha不一致/budget超過）で verify が `ERROR:` + `stop=1` を出す
- docs（PLAN/TASK/CONTRACT/STATUS）が SSOT と一致し、嘘がない
