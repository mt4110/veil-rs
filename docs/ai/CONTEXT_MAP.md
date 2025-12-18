# AI Context Map (Veil-RS) — どこを見れば何が分かるか

**目的:** AIに「読むべき場所」だけを渡して、全文読みによる劣化を止める。  
**原則:** *地図 + 差分 + 最小テスト* だけで作業できる状態を作る。

---

## 1. Repo Topology（ざっくり）

- `crates/veil-guardian/`
  - Guardian本体（OSV, cache, metrics, net retry/concurrency が集中）
- `crates/veil-cli/`
  - CLI表示・出力（人間向けUXが絡む時）
- `docs/guardian/`
  - Guardian仕様ドキュメント（憲法・規約）
- `docs/ai/`
  - AI運用ルール（この地図 / ルール / 出力テンプレ）

---

## 2. Hot Paths（触りがちな“中枢”）

### 2.1 OSV Provider（最重要）
- `crates/veil-guardian/src/providers/osv/client.rs`
  - `OsvClient` の状態機械（cache→legacy→fetch、offline、UX文言）
- `crates/veil-guardian/src/providers/osv/details_store.rs`
  - `DetailsStore`（v1/legacy、Envelope、quarantine、migrate-on-read）

### 2.2 Network（観測性・信頼性）
- `crates/veil-guardian/src/providers/osv/net/retry.rs`
  - Retry/Backoff/Retry-After/TimeBudget
- `crates/veil-guardian/src/providers/osv/net/`（*concurrency gate* が居る）
  - ConcurrencyGate / wait計測（場所は `mod.rs` で辿る）

### 2.3 Metrics（可観測性）
- `crates/veil-guardian/src/metrics.rs`
  - Atomic counters / snapshot / display（高カーディナリティ禁止）

---

## 3. Spec & Constitution（“正”はここ）

- `docs/guardian/cache_contract.md`
  - Cache Constitution（NormKey, Envelope schema, The Law, quarantine）
- `docs/ai/WORKFLOW_RULES.md`
  - AIの作業手順（STOP条件、Commit A/B/C、最小→全体テスト）
- `docs/ai/OUTPUT_TEMPLATE.md`
  - AIが返すべき提出物テンプレ

---

## 4. Tests（最小→全体の“最小”側）

### 4.1 Cache / Key / Migration
- `crates/veil-guardian/tests/key_versioning.rs`
  - NormKey / v1 path / legacy migration / conflict quarantine

### 4.2 Retry / Metrics proof
- `crates/veil-guardian/tests/osv_retry_metrics.rs`
  - 429 → retry → 200 のカウンタ検証

### 4.3 Operator UX / Resilience
- `crates/veil-guardian/tests/osv_operator_ux.rs`
  - quarantine表示、offline remediation hint など

---

## 5. “触る前に”チェック（AI向け）

AIは作業開始前に、次を短く出す：

- 変更対象ファイル（`git diff --name-only`）
- 影響する最小テスト（例：`key_versioning`, `osv_retry_metrics` など）
- Commit A/B/C 境界宣言

---

## 6. “ここだけ grep すれば迷子が減る”索引

### version境界
```bash
grep -RIn '^version = "' Cargo.toml crates/*/Cargo.toml
```

### quarantine reason / flags
```bash
grep -RIn "quarantine\|QuarantineFlags\|conflict\|unsupported\|corrupt" \
  crates/veil-guardian/src/providers/osv
```

### metricsカウンタ名の揺れ
```bash
grep -n "net_retry_attempts\|retry_attempts\|retries\|budget_exceeded\|not_modified" \
  crates/veil-guardian/src/metrics.rs
```
