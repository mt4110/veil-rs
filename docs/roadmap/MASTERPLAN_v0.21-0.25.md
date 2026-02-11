# v0.21.x〜v0.25.x — MASTER PLAN（Always Run 前提）

## 0. Purpose
この範囲は「機能」よりも先に **開発儀式の骨格**を固める帯域。
設計→実行→証拠→PR を定型化し、停止点を “PRマージ判断だけ” に寄せる。

---

## 1. Always Run（契約）
PRごとに必ず満たす。満たせないなら PR を作らない。

### 1.1 必須成果物（PRごと）
- `docs/pr/PR-<num>-<slug>.md`（SOT）
- `docs/evidence/prverify/prverify_<UTC>_<sha7>.md`（証拠の永続化）
- `nix run .#prverify` PASS（SOTに evidence パスを記載）
- docs のリンク規約違反ゼロ：
  - docs に `file:` と `//` の生連結（= `file:` + `//`）を置かない

### 1.2 停止点
- 停止して良いのは **PRマージ判断の瞬間だけ**
- 設計/実装/検証/証拠/PR作成は “手順化して止めない”

---

## 2. Version Intent（v0.21〜v0.25）
### v0.21.x — Foundation Determinism
- 証拠の永続化、SOT、検証手順の土台
- “脳内ルール” を repo の規約へ

### v0.22.x — Governance（失敗が説明可能で復旧しやすい）
- 例外レジストリ、期限、カテゴリ、復旧UX

### v0.23.x — Distribution（第三者検証）
- pack 単体で検証できる配布物（review pack など）
- 改ざん検知と verify-only の固定

### v0.24.x — Supply Chain（依存・ポリシー）
- dep-guard、禁止依存、feature 統制
- 例外はレジストリへ一本化

### v0.25.x — PR Ritual Automation（PR Kit）
- SOT作成・evidence退避・prverify実行・SOT更新を機械化
- 人間の仕事を “判断” へ圧縮する

---

## 3. Tracks
- Track A: Ritual（SOT/evidence/prverify/doc-links）
- Track B: Governance（例外/復旧UX）
- Track C: Supply Chain（依存/ポリシー）
- Track D: Distribution（第三者検証）

---

## 4. DoD（Definition of Done）
### Hard（必須）
- Always Run 契約を満たす
- SOT の Scope と実装が一致（ズレたら作り直し）
- 失敗時の復旧が 1スクロールで分かる

### Soft（望ましい）
- 変更が小さい、ログが読みやすい、次に壊れる場所へ予防線がある
