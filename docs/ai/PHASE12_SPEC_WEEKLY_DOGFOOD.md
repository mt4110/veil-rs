# Phase 12 Spec v1.0 — Weekly Dogfood “事故らず回る” 仕様

## 1. 用語定義（固定）

- **WEEK_ID**：YYYY-Www 形式（ISO week）。例：2025-W52
- **WEEK_DIR**：`docs/dogfood/{WEEK_ID}-Tokyo/`（例：`docs/dogfood/2025-W52-Tokyo/`）
- **raw events**：dogfood 実行中に出る生ログ/イベント列（運用の証拠だが repo には入れない）
- **weekly report**：週次レポ（追跡対象 / 差分レビュー対象）
- **cockpit build out-link**：Nix build の生成物リンク。固定名で統一する（事故防止）

## 2. Git 管理ポリシー（追跡/無視の境界線）

### 2.1 .gitignore MUST
- `result/dogfood/` は **完全無視（MUST）**
  - raw events / 中間生成物 / 一時ファイルが混ざる可能性があるため
- `docs/dogfood/**` は **追跡（MUST）**
  - 週次レポは「監査証跡」なので Git に残す
  - ただし例外として `docs/dogfood/README.md` は常に追跡（MUST）

### 2.2 .gitignore 推奨最終形（提案・強い）
（既に近い形がある前提で、最終形を定義）

```
ignore:
result/
result/**
result/dogfood/**

track:
docs/dogfood/**
!docs/dogfood/README.md
```

**ルールの魂**：“rawは捨てる、reportは残す”。
ここが曖昧だと、未来の自分が死ぬ。

## 3. コミット分割ルール（絶対）

### 3.1 コミット種別
- **A（実装）コミット**：Go/Rust/Schema/Workflow/Docs(仕様) の変更のみ
- **B（生成物）コミット**：`docs/dogfood/{WEEK_ID}-Tokyo/` 配下のみ（MUST）
  - **B は 単独コミット（MUST）**
  - B にはコード・設定・workflow・schema変更を混ぜない（MUST）

### 3.2 Bコミットの安全弁（MUST）
- Bコミット前に、以下を満たさないなら fail（CIでもローカルでも）
- `git status --porcelain` の差分が `docs/dogfood/{WEEK_ID}-Tokyo/**` と `docs/dogfood/README.md` だけ
- **それ以外が 1行でも出たら 即中止（MUST）**

## 4. Nix out-link 規約（再発防止の中核）

### 4.1 out-link 名
- workflow/ローカルともに 常に `--out-link dist/cockpit-build`（MUST）
- `result` など既存の一般名は **使用禁止（MUST）**

### 4.2 事前クリーン
- `dist/cockpit-build` が既に存在する場合は **削除してから build（MUST）**
  - 例：`rm -f dist/cockpit-build`（symlink前提）
  - ディレクトリで残ってたら `rm -rf`（MUST）

## 5. dogfood CLI 契約（固定）

### 5.1 許可コマンド（唯一）
- MUST：`cockpit dogfood weekly`
- MUST：CI は `nix run .#cockpit -- dogfood weekly` を使用
- SHOULD：ローカルも可能な限り同じ経路（再現性のため）

### 5.2 入力（環境変数 / 引数）
- MUST：`WEEK_ID` は CLI が内部計算してもよいが、CI 側でも算出して注入する（2重化）
- MUST：`WEEK_ID` が空/不正なら fail-fast（後述）
- SHOULD：タイムゾーンは `Asia/Tokyo` を規範とする

### 5.3 出力（生成物の契約）
`cockpit dogfood weekly` 成功時、以下を **必ず生成する（MUST）**

- `docs/dogfood/{WEEK_ID}-Tokyo/metrics_v1.json`
- `docs/dogfood/{WEEK_ID}-Tokyo/worklist_v1.json`（or .md でもいいが固定する）
- `docs/dogfood/{WEEK_ID}-Tokyo/top3.json`（Top3 があるなら）
- `docs/dogfood/{WEEK_ID}-Tokyo/scorecard.txt`（スコア出すなら）
- `docs/dogfood/{WEEK_ID}-Tokyo/weekly.md`（人間用まとめ。最低限でも）

※ファイル名は例。重要なのは「毎週同じセット」。増減させるなら schema と docs を同時更新（Aコミット側）に寄せる。

### 5.4 Exit code（契約・固定）
- `0`：成功（生成物が揃っている）
- `2`：Usage error（引数不正など）
- `3`：Precondition error（WEEK_ID 未設定/不正、前週参照不能など）
- `10`：Internal error（panic/予期しない例外）

※CIの分岐は exit code に依存する。ここが揺れると CI が幻覚を見る。

## 6. Weekly Dogfood Workflow 契約（GitHub Actions）

### 6.1 トリガ
- `schedule`（週1回）
- `workflow_dispatch`（手動）

### 6.2 Fail-fast（最優先）
- **WEEK_ID が空なら 即 fail（MUST）**
  - 例：workflow 内で算出→ `test -n "$WEEK_ID"` → 空なら `exit 3`

### 6.3 permissions 最小化（MUST）
PR を作成/更新する方式なら、原則:
- `contents: write`
- `pull-requests: write`
- それ以外は `none` を基本（MUST）

PR を作らず artifact だけなら `contents: read` に落とせる（SHOULD）
「本当に必要か？」は方式で決まる。方式が変わっても最小化が正義。

### 6.4 concurrency（意図の固定）
- concurrency group：weekly dogfood は常に1本（MUST）
  - 例：`group: weekly-dogfood`
- `cancel-in-progress: true`（MUST）
- “週次生成物”は最新が正。並列はノイズ。

## 7. PR create/update の安全設計

### 7.1 ブランチ命名（固定）
- `automation/dogfood/{WEEK_ID}`（MUST）
  - 例：`automation/dogfood/2025-W52`

### 7.2 PR の一意性（MUST）
- PR は週ごとに **1つだけ**
- 既に同 WEEK_ID の PR がある場合は update（push）するだけ（MUST）
- PR タイトルも固定：
  - `dogfood: {WEEK_ID} (Tokyo)`

### 7.3 差分安全チェック（MUST）
- PR 作成/更新前に、コミット対象が以下だけであることを検証：
  - `docs/dogfood/{WEEK_ID}-Tokyo/**`
  - `docs/dogfood/README.md`（必要なら）
- **それ以外が混じったら fail（exit 10でも 3でもいいが契約で決める）**

## 8. Worklist / metrics 仕様（ドキュメント化対象）

### 8.1 除外ルール（MUST）
- `dogfood.*` は Top3 / Worklist の集計対象から除外（MUST）
- ルールは `docs/dogfood/README.md` か専用 docs に明記
- 実装側にも同ルール（コード）を持つ（MUST）

### 8.2 スコア計算式（固定）
- `score = Σ (CountWeight * count + DeltaWeight * delta)` 的なものをやるなら、式を docs に書く
- count / delta の定義（増減の基準・前週比較の参照元）も書く
- **tie-breaker（同点時の並び）**を明記（MUST）
  - 例：`score desc` → `metric_key asc`（完全決定的に）

## 9. テスト最小安全網（落ちないための最短距離）

### 9.1 Go（MUST）
- WEEK_ID の解決（Tokyo基準）テスト
- prev metrics の探索規約テスト（前週が無い場合の挙動含む）
- Top3 determinism（同入力なら同出力）テスト
  - 並び順・tie-breaker の固定がここで守られる

### 9.2 Rust（MUST）
- schema/golden が落ちない
- `metrics_v1.schema.json` / `reason_event_v1.schema.json` との整合
- golden は「互換性の番犬」

## 10. 受け入れ条件（Definition of Done）
- [ ] 週次 workflow が 2回連続で成功する（手動 dispatch でも可）
- [ ] out-link 事故が構造的に再発しない（固定名＋クリーン＋禁止名）
- [ ] PR が「作成/更新」で安定し、WEEK_ID 空で暴走しない
- [ ] docs に以下が明文化されている：
  - `dogfood.*` 除外
  - score式
  - 出力ファイルセット（週次レポの契約）
