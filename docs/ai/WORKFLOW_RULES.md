# AI Workflow Rules (Veil-RS) — ルール憲法

**Status:** Active / Living Document（運用しながら育てる）
**Owner:** Human
**Executors:** AI（Sora / Ambi / AntiGravityIDE）

---

## 0. Prime Directive

**Predictability > Convenience.**
AIは「推測」より「停止」を優先する。Humanは「認知と決定」を担う。

---

## 1. Roles（役割分担）

### 1.1 Human
- 目的の最終決定とレビュー（GO / NO-GO）
- 変更の認知（何が変わったか、何を次に作るか）
- **ルール改修の最終責任**
  - 同じ事故が2回起きたら「ルール不足確定」
  - 次のタイミングで追記する：
    - 事故が起きた直後（再発防止の1行追記が最強）
    - フェーズ切り替え時（目的が変わる）
    - AIが同じミスを2回した時（ルール不足が確定）

### 1.2 AI（Sora / Ambi / AntiGravityIDE）
- **relevant hunks だけ**で差分を出す（全文貼り禁止）
- テストは **最小 → 最後に全体** の順で実行
- コミット境界（Commit A/B/C）を最初に宣言
- STOP条件に当たったら止まる（推測で進めない）

---

## 2. Hard Constraints（絶対ルール）

### 2.1 “全文貼り”禁止
- AIの出力は **relevant hunks（必要な差分箇所）のみ**。
- ファイル丸ごと貼るのは禁止。
  例外：**新規ファイル**のみ（かつ短い場合）。

### 2.2 テスト規律（最小 → 全体）
- 変更点に直結する **最小テスト** を先に通す
- 最後に **full test** を通す
- “通った”と言う場合は **コマンドと実ログ**を必ず提示する（捏造禁止）

### 2.3 STOP条件（該当したら即停止）
以下が起きたら作業を止め、原因と必要情報を短く要求する（推測で進めない）：

- baseline failing（変更前からテスト落ち）
- 想定ファイルが存在しない / パスが違う
- テスト名・出力ラベルが想定と違う（＝ズレの兆候）
- 仕様（docs）と実装が矛盾し、どちらが正か確定できない
- 高カーディナリティ（例：`vuln_id`）を metrics/log に入れそう

---

## 3. Commit Boundary Rules（事故を減らす境界線）

作業開始時に **必ず Commit A/B/C を宣言**する。

- **Commit A: Docs/Spec**（仕様・憲法の更新）
- **Commit B: Code**（実装）
- **Commit C: Tests/Chore**（テスト、掃除、CI柵）

### 例
- Commit A: `cache_contract.md` の完全一致
- Commit B: Metrics 計測追加
- Commit C: `osv_retry_metrics.rs` 追加 + 全体テスト

**原則：1コミットで混ぜない。**
混ぜるとレビューも切り戻しも壊れる。

---

## 4. “AIに渡す素材”の作り方（コンテキスト節約）

### 4.1 Diff（地図 + relevant hunks）

#### 対象ファイル一覧（地図）
```bash
git diff --name-only
# relevant hunks（差分本体）
git diff -U5
```

#### 特定コミット確認（レビュー用）
```bash
git show -U5 <commit>
```

### 4.2 行番号付きの狙撃（レビュー・修正ポイント指定）

#### 行番号表示（抜粋）
```bash
nl -ba path/to/file.rs | sed -n '1,220p'
```

#### 文字列狙撃（検索）
```bash
grep -n "検索文字列" path/to/file.rs
```

---

## 5. “Mac怨霊”除霊（tarball生成の正規手順）

**[NEW]** Mac環境で tar を作る際は必ず環境変数をセットする。
目的: `._*` や `.DS_Store` などの混入を防ぐ。

### 5.1 正規手順（環境変数必須）
```bash
ts=$(date +"%Y%m%d_%H%M%S")
hash=$(git -C veil-rs rev-parse --short=12 HEAD 2>/dev/null || echo "nogit")

# 呪文: COPYFILE_DISABLE=1 が最重要
COPYFILE_DISABLE=1 tar --no-xattrs --no-mac-metadata -czf "veil-rs_${ts}_${hash}.tar.gz" \
  --exclude='veil-rs/.git' \
  --exclude='veil-rs/target' \
  --exclude='**/.DS_Store' \
  --exclude='**/._*' \
  veil-rs
```

### 5.2 それでも残る場合の“物理除霊”（事前掃除）
```bash
find veil-rs -name '.DS_Store' -delete
find veil-rs -name '._*' -delete
```

---

## 6. 狙撃grep（ズレやすい箇所を先に特定）

### 6.1 version を揃える（リリース境界）
```bash
grep -RIn '^version = "' Cargo.toml crates/*/Cargo.toml
```

### 6.2 “FIXEDコメント”の残骸を掃除（レビュー中だけ許可）
```bash
grep -n "<!-- FIXED" docs/guardian/cache_contract.md
```

### 6.3 quarantine reason の呼称統一（Docs / UX / metrics）
```bash
grep -RIn "corrupt_dirs_conflict\|conflict\|quarantine\|QuarantineFlags\|unsupported" \
  crates/veil-guardian/src/providers/osv
```

### 6.4 Metricsの二重定義の芽を潰す
```bash
grep -n "retries\|net_retry_attempts\|retry_attempts" crates/veil-guardian/src/metrics.rs
```

---

## 7. AI Output Requirements（提出物の必須構成）

AIは以下を必ず出す（詳細は docs/ai/OUTPUT_TEMPLATE.md）：

1. **Commit Plan（A/B/C）**
2. **Diff（relevant hunksのみ）**
3. **Test Log（最小 → 全体）**
4. **Summary（要点 3行）**
5. **Risk / Notes（危険箇所、互換性、次の一手）**

---

## 8. Rule Evolution（ルールは“修正していく前提”）

ルール変更はコミットで行い、理由を1行残す。

推奨コミットメッセージ：
`docs(ai): update workflow rules - <reason>`

例：
`docs(ai): update workflow rules - stop condition for missing tests`

---

必要なら、同じく **CONTEXT_MAP / OUTPUT_TEMPLATE も “Human表記に統一”**した版に揃えて貼り直す（表記ゆれは地味にAIの迷子要因になるので、やる価値ある）。

---

## 9. Automation & Workflow Rules (Phase 8)

### The Golden 3 Commands (Zero Hesitation)
To release a version (e.g., `v0.14.0`), run these 3 commands in order.

#### 1. Generate (Local)
Create all draft assets. Single entry point.
```bash
# Golden Command (Canonical)
./scripts/ai/gen.sh --version v0.14.0 --clean

# Alternative (Nix Direct)
nix run .#gen -- --version v0.14.0 --clean
```

#### 2. Verify Status
Confirm all 4 artifacts are ready and valid.
```bash
# Golden Command (Canonical)
./scripts/ai/status.sh --version v0.14.0

# Alternative (Nix Direct)
nix run .#status -- --version v0.14.0
```

#### 3. Review & Execute
View the release body, then paste it into GitHub Releases.
```bash
cat dist/publish/v0.14.0/RELEASE_BODY_v0.14.0.md
```

---

### Contract & Rules

#### 1. Dist Contract
- Output is ALWAYS `dist/publish/<VERSION>/`.
- Must produce 4 artifacts:
  - `PUBLISH_v*.md` (PR Body)
  - `RELEASE_BODY_v*.md` (Release Note)
  - `X_v*.md` (Social)
  - `AI_PACK_v*.txt` (LLM Context)

#### 2. CI/Local Parity
- **Local**: Generates everything.
- **CI Artifacts**: **Markdown Only (`**/*.md`)**.
  - `AI_PACK` must be `.txt` to physically prevent it from being uploaded as a release artifact in CI.
  - `scripts/ai/check.sh` enforces this extension rule.

#### 3. Guardrails
- `scripts/ai/check.sh` acts as the single source of truth for repository hygiene.
- It runs on every `gen.sh` execution and in CI.
