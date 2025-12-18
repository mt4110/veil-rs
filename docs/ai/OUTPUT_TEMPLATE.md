# AI Output Template (Veil-RS) — diff / test / summary テンプレ

**目的:** 余計な全文貼りを禁止し、レビューと安全性を最大化する。  
**ルール:** relevant hunks だけ。テストは最小→最後に全体。STOP条件で止まる。

---

## 0. Commit Plan（最初に宣言）

- Commit A（Docs/Spec）: <やること>
- Commit B（Code）: <やること>
- Commit C（Tests/Chore）: <やること>

---

## 1. Scope（対象と意図）

- Why: <なぜ必要か>
- What: <何を変えるか（要点3つまで）>
- Not doing: <今回はやらないこと>

---

## 2. STOP条件チェック（該当ならここで停止）

- [ ] baseline failing（変更前から落ちてないか）
- [ ] 想定ファイル missing / path違い
- [ ] テスト名・出力ラベルの不一致
- [ ] docs と実装の矛盾で正が不明
- [ ] 高カーディナリティ（例：vuln_id）を metrics/log に入れそう

**該当した場合:**  
→「何が起きたか（1行）」+「次に必要な情報（箇条書き）」だけ出して停止。

---

## 3. Diff（relevant hunks only）

### 3.1 変更ファイル一覧
```bash
git diff --name-only
```

### 3.2 差分（必要箇所だけ）
> **注意:** ファイル丸ごと貼り禁止。該当 hunk のみ。

```diff
<ここに relevant hunks の diff を貼る>
```

---

## 4. Tests（最小 → 全体）

### 4.1 最小テスト（今回の変更に直結）
```bash
# 例：変更内容に応じて必要なものだけ実行
cargo test --package veil-guardian --test key_versioning
cargo test --package veil-guardian --test osv_retry_metrics
cargo test --package veil-guardian --test osv_operator_ux
```

#### 実ログ（省略せず貼る / 長い場合は末尾を優先）
```text
<コマンド出力を貼る>
```

### 4.2 全体テスト（最後）
```bash
cargo test --workspace
```

#### 実ログ
```text
<コマンド出力を貼る>
```

---

## 5. Summary（要点3行）

- <変更点1>
- <変更点2>
- <変更点3>

---

## 6. Risks / Notes（未来の自分を救う欄）

- Backward compatibility: <互換性の注意>
- UX impact: <表示や挙動の変化>
- Follow-ups: <次の一手（Task 3 など）>
