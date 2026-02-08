# PR44 SOT — Exception Registry Operator UX (v0.22.2 / Epic D)

## Status

* Draft (Kickoff)
* Target: v0.22.2 / Epic D
* Related: PR43 (Expiry Enforcement / Exception Registry v1)

---

## Background / Problem

Exception Registry（`ops/exceptions.toml`）は PR43 で「期限切れ例外を拒否できる」状態になった。
しかし運用目線では、**拒否された瞬間に “何が原因で / どう直し / 次に何を打つか” が1-scrollで分からない**と、例外レジストリはすぐ腐る（運用が迷子になる）。

PR44 は **“レジストリを点検できる read-only CLI”** と **“prverify の失敗UXの完成”**により、運用者が迷子にならない状態を作る。

---

## Goals (Objective)

1. **Read-only inspection commands** を提供し、手動ファイル読解なしでレジストリ状態を把握できる
2. `nix run .#prverify`（Drift / Registry）失敗時に、**原因・Fix・Next が1-scroll**で分かる
3. **決定論（時刻/順序/出力）**を固定し、テストで担保できる

---

## Non-goals (Out of Scope)

* 例外レジストリの編集（`add/remove/update` など書き込み系）
* 自動延長・自動修復
* インタラクティブ UI / TUI
* フォーマットの破壊的変更（v1互換を壊さない）

---

## Scope (In / Out)

### In

* `veil exceptions list` / `veil exceptions show <id>`（Read-only）
* Registry validation（必須項目/ID重複/期限フォーマット/スコープ書式）
* Status分類（active / expiring_soon / expired）
* prverify の Registry failure UX を **契約どおりに整形**（Fix/Nextを状況依存に）
* 出力安定化（ソート・日付・文言テンプレ）

### Out

* 書き込み系サブコマンド
* 自動修復
* 新しい概念の増殖（本PRの契約外の機能追加）

---

## Current State (as-is / a80688d)

既に存在する主要ファイル：

* Registry core: `internal/registry/*`（Loader / Validate / Status / tests）
* CLI: `cmd/veil/exceptions.go`（list/show + tests）
* prverify registry check: `cmd/prverify/registry.go`
* drift UX output: `cmd/prverify/main.go`（`driftError.Print()` が Fix/Next を表示）

PR44 は **“NEWで作る” ではなく “既存を契約仕様に締める”**PR。

---

## Spec

### Registry Source

* File: `ops/exceptions.toml`
* Loader: `internal/registry/loader.go`
* Validator: `internal/registry/validator.go`

### Time / “Today” definition (Determinism)

* CLI/prverify が扱う `utcToday` は **UTCの0:00**（日単位）とする

  * `cmd/veil`: `ctx.Now` は `time.Now().UTC()`、`utcToday` は `YYYY-MM-DD 00:00:00 UTC`
  * prverify: `utcToday` を上位から注入（既存設計を維持）
* Expiryの比較は「日付」単位（`YYYY-MM-DD`）

  * `now.After(expires)` のとき `expired`
  * **Expiry date は inclusive**（当日は expired ではない）

### Status definition (Contract)

* `active`: 期限なし、または期限が十分先
* `expiring_soon`: `expires - utcToday <= 7 days`（7日境界含む）
* `expired`: `utcToday > expires`（昨日まで）

> 注：現状 `CalculateStatus` は invalid expiry を status 判定上 `active` 扱いにフォールバックする。
> ただし validation で invalid expiry を必ずエラーにする（運用上は “壊れてる” なので）。

---

## CLI Contract — `veil exceptions ...` (Read-only)

### `veil exceptions list`

* Purpose: レジストリ全体の状態（status / expires / owner / reason）を安定表示する
* Flags:

  * `--status <active|expiring_soon|expired>`: statusフィルタ
  * `--format <table|json>`: 出力形式

#### Output (table)

* Header + rows を出す
* 表示列（現状に合わせる、必要ならSOTで固定）：

  * `ID / STATUS / EXPIRES / OWNER / REASON`
* 並び順は **Loaderの決定論ソート結果**（ID ASC, OriginalIndex ASC）を基準にする
* `REASON` の省略（例：40文字）を採用する場合は、**省略ルールを仕様として固定**する

#### Output (json)

* **encoding/json で正規に出力**（手組みJSONは禁止：壊れる）
* JSON配列の順序は table と同一（決定論）
* 少なくとも以下のキーを含める：

  * `id`, `status`, `expires`, `owner`, `reason`
  * 追加キーは許容するが、テストで固定するならSOTで宣言する

### `veil exceptions show <id>`

* Purpose: 単体エントリを key-value で安定表示
* Not found は exit=1 で「Exception not found: <id>」を出す（文言固定）

---

## prverify Failure UX Contract (1-scroll)

### Current issue

`driftError.Print()` の `Next` が常に `nix run .#prverify` 固定で、Registry系の復旧導線として弱い。

### Required output (Must)

Registry系 drift は、**Reason / Fix / Next が“状況に即して”**出ること。

* Reason:

  * 何が起きたか（missing / parse / validation / expired）
  * “どれが原因か” が分かる（IDまたは idx、期限・utc_today が見える）
  * 可能なら「最大表示件数 + 残件数」も出す（既存maxShow=10方針を踏襲）
* Fix:

  * 期限切れなら「延長 or 削除」+ runbook参照
  * それ以外は「壊れている箇所の修正」
* Next:

  * 期限切れ（expiredが含まれる）なら：

    * `veil exceptions list --status expired`
  * validation一般なら：

    * `veil exceptions list`
  * missing/parseなら：

    * `veil exceptions list`（ただしファイルがない/壊れてる場合は Reason/Fix で補う）

> Note: Next を状況依存にするため、`driftError` に `nextCmd`（または複数）を持たせ、未指定時は従来の `nix run .#prverify` をデフォルトにする。

### NO_COLOR compatibility

* NO_COLORでも同じ情報（Reason/Fix/Next）が出ること（色なしでも読める）

---

## Acceptance Criteria

1. `veil exceptions list` がレジストリを読み取り、status を含めて表示できる
2. `veil exceptions show <id>` が単体を安定表示できる
3. `--format json` が **正規JSON**で、決定論順序で出力される
4. `--status` フィルタが `active|expiring_soon|expired` で期待通り動作する
5. prverify の Registry drift が **1-scroll**で Reason/Fix/Next を提示し、Nextが状況依存になる
6. 出力・順序・日付が **決定論**で固定され、テストで担保できる
7. `nix run .#prverify` が最終 PASS（clean）し、Evidenceログ1本主義を満たす

---

## Failure Modes (Must Cover)

* `ops/exceptions.toml` missing
* TOML parse error
* Duplicate IDs
* Missing required fields (rule/scope/reason/owner/created_at/audit)
* Invalid scope grammar（`path:` / `fingerprint:` prefix）
* Invalid date format（created_at/expires_at）
* Expired entry（utc_today > expires）
* Expiring-soon boundary（<= 7 days）

---

## Tests (Golden-ish)

### Unit (internal/registry)

* `CalculateStatus` の境界（7日境界 / 当日 / 昨日）
* Validationの順序安定性（ID ASC + OriginalIndex ASC）
* Expired error string の内容（expires/now/status を含む）

### Integration (cmd/veil)

* `exceptions list` の出力（table）

  * “contains” ではなく **全文一致 or golden file** に寄せる（崩れを検知できる形）
* `exceptions list --format json` の出力

  * JSONとしてunmarshalできること
  * 配列順序が固定であること
* `exceptions show` の出力（全文一致 or golden）

### Integration (cmd/prverify)

* Registry drift failure の出力（NO_COLORを使い全文比較しやすくする）

  * expired を含むケースで Next が `veil exceptions list --status expired` になること
  * validation一般で Next が `veil exceptions list` になること

---

## Implementation Plan (C0〜C4 / break-safe)

### C0: SOT + Runbook skeleton

* Add: `docs/pr/PR-44-exception-registry-operator-ux.md`（本書）
* Add/Update: `docs/runbook/exception-registry.md`（章立てだけでも先に置く）
* 既存CI/テストを壊さない（doc only）

### C1: Core tightening (determinism / validation messaging)

* `internal/registry` の契約をSOTに合わせて明文化（必要ならコメント整備）
* validation error の安定化（順序・文字列・境界）を確認し、必要なら整える
* Unit tests を追加/強化（境界、error string）

### C2: CLI hardening (json correctness / golden-ish)

* `cmd/veil/exceptions.go`

  * `--format json` を **encoding/json** に置き換え
  * 必要なフィールドを確実に出す（id/status/expires/owner/reason）
  * 出力順序は loader の順序を維持
* `cmd/veil/exceptions_test.go`

  * “contains” 依存を減らし、全文一致 or golden 相当に寄せる
  * json output を unmarshal して構造検証 + 順序検証

### C3: prverify failure UX (contextual Next)

* `cmd/prverify/main.go`

  * `driftError` に `nextCmd`（または nextCmds）を追加
  * 未指定時のデフォルトは従来通り `nix run .#prverify`
* `cmd/prverify/registry.go`

  * expiredを含む場合:

    * `nextCmd = "veil exceptions list --status expired"`
  * validation一般:

    * `nextCmd = "veil exceptions list"`
  * missing/parse:

    * `nextCmd = "veil exceptions list"`（状況はReason/Fixで補う）
* prverify の出力テスト（NO_COLOR）を追加し、Reason/Fix/Nextが契約どおりで固定されることを確認

### C4: Runbook finalize + Evidence

* `docs/runbook/exception-registry.md` を完成

  * 期限切れ時の手順
  * validation NG時の直し方
  * list/show の使い方
  * 運用ルール（期限更新責任、ID命名）
* 最終 `nix run .#prverify` PASS を **1本** 保存（.local/prverify/…）

---

## Evidence Policy

* 最終状態の Evidence は **PASSログ1本**（.local/prverify/…）
* SOT は単独・重複禁止（PR-44はこの1枚が契約の根）

---

## Follow-ups (Future PR candidates, not this PR)

* Write commands（add/remove/update）
* `veil exceptions doctor`（より強い診断）
* Registry schema versioning（v2以降）
