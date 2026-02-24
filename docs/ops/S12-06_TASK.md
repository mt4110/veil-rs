# S12-06 TASK — Three Themes (B→C→A 推奨順)

**ステータス**: 0% (Kickoff前)
**推奨実施順**: B → C → A

---

## Phase B: pack contract を"法律"にする

**Branch**: `s12-06-pack-contract-law-v1`

### Setup
- [ ] `git switch -c s12-06-pack-contract-law-v1`
- [ ] Discovery: `ls -la cmd/reviewbundle cmd/prverify docs/ops docs/pr` をOBSへ
- [ ] `docs/ops/S12-06_PLAN.md` 存在確認（本ファイルと同梱）

### Contract 定義（軽い）
- [ ] `docs/ops/REVIEW_BUNDLE_CONTRACT_v1.md` 新規作成
  - `contract_version` フィールド定義（唯一の真実 = manifest のみ）
  - required files 一覧
  - required manifest fields
  - normalization rules
  - 互換ポリシー

### 実装（中）
- [ ] `cmd/reviewbundle/create.go` に `contract_version` を **manifest のみ** に追加（contract.json には入れない）
- [ ] verifier 実装 (`cmd/reviewbundle/` 内)
  - manifest から `contract_version` を読む（唯一の真実）
  - 違反 → `ERROR:` + `stop=1` (os.Exit/panic/log.Fatal/log.Panic禁止)
  - 必ず `OK: phase=end stop=<0|1>` を出す

### テスト（中）
- [ ] unit: manifest schema invariants（必須・最優先）
- [ ] testdata: manifest JSON のみの最小fixture（tar.gz は repo に入れない）
- [ ] (optional) bundle生成テスト内で決定論的に作って検証（巨大化禁止）

### SOT・仕上げ（軽い）
- [ ] `docs/ops/STATUS.md` の S12-06 を 99% (Review) に更新
- [ ] `docs/pr/PR-XX-s12-06-pack-contract-law.md` 作成
- [ ] `go build ./cmd/reviewbundle` clean
- [ ] CI green
- [ ] STATUS → 100% (Merged)

---

## Phase C: .local を"散らからない宇宙"にする

**Branch**: `s12-06-local-durability-gc-v1`

### Setup
- [ ] `git switch -c s12-06-local-durability-gc-v1`
- [ ] `.local` fast inventory（`ls .local/*/` 相当、件数+mtime のみ。サイズ深掘り禁止）をOBSへ

### ポリシー定義（軽い）
- [ ] `docs/ops/LOCAL_STORAGE_POLICY_v1.md` 新規作成
  - keep / archive / delete の定義
  - `"default: never delete without explicit flag"` を明文化

### GCツール実装（中）
- [ ] `cmd/localgc/main.go` 新規（**ハイフン無し**、import名衝突回避）
  - `--mode dry-run` (default): 削除候補を列挙のみ（何も消さない）
  - `--mode plan`: 上位ディレクトリ単位でサイズ計測 + 理由付き一覧
  - `--mode apply` **かつ** `--apply`: 両フラグ同時必須（二重ロック）
    - `--mode apply` だけでは削除しない
    - `--apply` だけでは削除しない
  - 失敗 → `ERROR:` + `stop=1` (panic/os.Exit/log.Fatal/log.Panic禁止)
  - `OK: phase=end stop=<0|1>` 必須

### テスト（軽い）
- [ ] unit: path selection rules + output format (FS副作用なし)

### SOT・仕上げ（軽い）
- [ ] `docs/ops/STATUS.md` 更新
- [ ] `docs/pr/PR-XX-s12-06-local-gc.md` 作成
- [ ] CI green → STATUS 100%

---

## Phase A: verify chain hardening（検証連鎖の完全整流）

**Branch**: `s12-06-verify-chain-hardening-v1`

### Setup
- [ ] `git switch -c s12-06-verify-chain-hardening-v1`
- [ ] 入口一覧棚卸し: prverify / flake.nix / reviewbundle / CI の stop解釈を記録

### 統一（中）
- [ ] `docs/ops/STDOUT_CONTRACT_v1.md` 新規
  - stdout canonical lines の完全仕様
  - stderr = UX only の明文化
- [ ] 残存 exit code 依存の発見 → `stop=1` parse へ移行

### テスト（中）
- [ ] chain test (hermetic, small):
  - 意図的な違反 → `stop=1` + `ERROR:` 確認
  - nix/cargo 本番起動なし

### SOT・仕上げ（軽い）
- [ ] `docs/ops/STATUS.md` 更新
- [ ] `docs/pr/PR-XX-s12-06-verify-chain.md` 作成
- [ ] CI green → STATUS 100%

---

## 共通チェック（全テーマ共通）

- [ ] `rg "os\.Exit|panic(|log\.Fatal|log\.Panic" cmd/ || true` → 0件を確認
- [ ] `go build ./...` clean
- [ ] `rg "file:" docs/ops/ || true` → 0件 (絶対パス混入防止)
- [ ] `OK: phase=end` が各コマンドの末尾に必ず存在することを確認
