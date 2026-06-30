# 0. Contract Decisions / Implementation-Blocking Ambiguity Resolution v4.4

この文書は、設計書群の実装解釈割れを防ぐための最上位SOTである。実装時に他章と矛盾する場合は、本章を優先し、該当章・schema・テストを同時に修正する。

## D-000 PR-0: Contract Alignment を最初に実装する

本設計書を実装に落とす最初のPRは **Contract Alignment PR** とする。機能追加ではなく、型・schema・生成・検証の正本を作る。

### D-000.1 正本ファイル

PR-0で以下を追加する。

- `crates/veil-pro/src/api/dto.rs`
  - Local API DTO の正本。
  - `#[serde(rename_all = "camelCase")]` を使う。
  - `schemars::JsonSchema` と `utoipa::ToSchema` をderiveできる型にする。
- `crates/veil-pro/src/bin/export_local_api_schema.rs`
  - Rust DTOから `schemas/openapi.local-api.yaml` と `schemas/json-schema.*.json` を生成する。
- `scripts/check_generated_schemas.py`
  - 生成結果とtracked schemaの差分を検査する。
  - **この名前を唯一のschema検証スクリプト名とする**。`validate_generated_schemas.py` は使用しない。
- `schemas/openapi.local-api.yaml`
- `schemas/json-schema.safe-finding-api.json`
- `schemas/json-schema.report.json`
- `schemas/json-schema.run-meta.json`
- `schemas/json-schema.finding.json`

### D-000.2 schema出力先

PR-0の出力先は **repo root の `schemas/`** とする。設計書パック内の `schemas/` は同じ内容の参照コピーであり、実装時の正規出力先ではない。

schema更新時の生成コマンド契約:

```bash
cargo run -p veil-pro --bin export_local_api_schema -- --out-dir schemas
```

schema検証コマンド契約:

```bash
python scripts/check_generated_schemas.py
```

`scripts/check_generated_schemas.py` は一時ディレクトリへ生成して tracked `schemas/` と比較する。acceptance gate では検証前に `schemas/` を上書きしてはならない。

実装者は OpenAPI / JSON Schema を手編集してはならない。

### D-000.3 schema生成crate

- JSON Schema: `schemars`
- OpenAPI: `utoipa`
- DTOは `serde`, `schemars::JsonSchema`, `utoipa::ToSchema` を同じRust型にderiveする。
- `utoipa` で表現しづらいschema制約（例: `baseline.path` const）は、生成後patchではなくDTO/Schema helperで生成できる形に寄せる。どうしても不可の場合は `export_local_api_schema` 内の deterministic post-process とし、`check_generated_schemas.py` が差分を検知する。

## D-001 API / Schema SOT

- **正本**: Rust DTO (`crates/veil-pro/src/api/dto.rs`)。
- `schemas/openapi.local-api.yaml` と JSON Schema は、Rust DTO から生成される派生物。
- Local API は **camelCase** を使う。
- CLI JSON (`schemaVersion: veil-v1`) は既存互換のため **snake_case** を維持する。
- `SafeFindingApiV1` は Local UI / Evidence preview / Evidence report 用の raw-free DTO。
- `FindingV1` は CLI/Core互換用。rawを含み得る内部/CLI用schemaであり、Local UIには返さない。

## D-002 ScanRequest.paths

- `paths` は省略可能。
- `paths` が省略または空配列の場合、APIは `['.']` に正規化する。
- 空配列をエラーにしない。UIの空入力は “repo root” の意図として扱う。

## D-003 Evidence ZIP baseline entry

- Evidence ZIP 内の baseline artifact 名は **`veil.baseline.json`** とする。
- baseline は任意。baseline未使用時は ZIP に入れず、`run_meta.artifacts.baseline` も省略する。
- `baseline.json` は v1契約では使用しない。
- on-disk の推奨 baseline 名も `veil.baseline.json` とする。既存 `.veil-baseline.json` は読み取り互換のみ許可する。
- `run_meta.artifacts.baseline.path` は schema/verify の両方で `veil.baseline.json` に固定する。

## D-004 run_meta self-hash 禁止

`run_meta.json` は **自分自身のsha256を内部に持たない**。

理由:

- `run_meta.json` 内に `run_meta.json` のhashを含めると自己参照になり、通常の一回生成では安定値を作れない。
- Evidence Packの外部アンカーは `veil verify --expect-run-meta-sha256 <hex>` で扱う。

契約:

- `run_meta.artifacts.runMeta` は存在しない。
- `run_meta.json` の raw bytes SHA256 は verifier / 監査台帳 / チケット側で外部アンカーとして扱う。
- `report.html`, `report.json`, `effective_config.toml`, optional `veil.baseline.json` のsha256は `run_meta.artifacts.*` に保持する。

## D-005 Local-first / SSO / Remote Rules

- デフォルトモードは完全ローカル・外部通信なし。
- SSO と remote rule download は v1の通常起動では無効。
- SSO は **Enterprise opt-in**。有効化には `VEIL_PRO_ENABLE_SSO=1` と明示設定が必要。
- Remote rules は **Enterprise opt-in**。`core.allow_remote_rules = true` と `VEIL_ALLOW_NETWORK=1` が両方必要。
- SSO/remote rules 有効時でも、ソース/PII/findings/Evidenceのアップロードは禁止。
- `privacy.networkMode` は `local-only | enterprise-opt-in`。
- デフォルトの `networkMode` は `local-only`。

## D-006 Presets / Precedence

- v1で提供する preset は5種: `standard-jp`, `fintech-jp`, `gov-jp`, `si-vendor-jp`, `logs-jp`。
- `minimal-ci` は preset ではない。`mode = ci` / `--staged` / fail flags / scope制限で表現する。
- `--preset` は “base layer” として適用し、`VEIL_ORG_CONFIG`, repo config, explicit CLI flags が上書きできる。
- この直感ズレは `veil config explain` で必ず説明する。
- `veil scan --preset ...` 実行時、repo config が preset由来値を上書きした場合は stderr に説明を出す。

## D-007 Score / Grade / Severity / Fail Conditions

- `score` は最終リスクスコア（0-100）。validator/context/baseline適用前後で最終値を確定する。
- `grade` は score から決定されるUI/レポート用バンド。
- `severity` は `grade` から `Safe` を除いた互換フィールド。public API / CLI に出るfindingは原則 `Low` 以上。

| score | grade | severity | CI threshold alias |
|---:|---|---|---|
| 0-19 | Safe | 出力しない（verbose時のみLow扱い） | - |
| 20-39 | Low | Low | `--fail-on-severity Low` |
| 40-69 | Medium | Medium | `--fail-on-severity Medium` |
| 70-89 | High | High | `--fail-on-severity High` |
| 90-100 | Critical | Critical | `--fail-on-severity Critical` |

Fail判定は baseline suppress 後の **effective findings** に対して行う。

- `--fail-on-score N`: `score >= N` の effective finding が1件以上あれば Exit 1。
- `--fail-on-severity S`: `score >= minScore(S)` の effective finding が1件以上あれば Exit 1。
- `--fail-on-findings N`: `effectiveFindings >= N` なら Exit 1。`N` は1以上。`N=0` は設定エラー。
- Local API の `failOnFindings=0` は HTTP 400 `INVALID_REQUEST`。
- 複数fail条件は OR。

## D-008 Baseline counting contract

Local API / Evidence report は件数の意味を以下に固定する。

- `totalFindings`: baseline適用前の全finding数。
- `suppressedFindings`: baselineにより抑制されたfinding数。
- `effectiveFindings`: baseline suppress 後に残るfinding数。CI fail判定の対象。
- `severityCounts`: effective findings のseverity集計。
- `allSeverityCounts`: baseline適用前の全finding集計。
- `suppressedSeverityCounts`: suppressed findings のseverity集計。**必須フィールド**。
- すべての `SeverityCounts` は zero-filled map とし、`Low` / `Medium` / `High` / `Critical` の4キーを必ず持つ。0件でもキー省略は禁止。
- Local API `findings`: デフォルトでは effective findings のみを返す。
- Local API `includeSuppressed=true` の場合のみ、`baselineStatus="suppressed"` のfindingを含めて返す。
- Evidence `report.json.findings`: 監査用提出物のため、raw-free な全finding（`new` と `suppressed`）を常に含める。

## D-009 Rule score contract

Rule側のリスク初期値は **`base_score`** のみを正とする。

- Rule定義に `score` は使わない。
- preset TOML も rule override は `base_score` のみを使う。
- 既存RulePackの `score` はmigrationで `base_score` へ移す。
- Finding側の `score` は validator/context/negative context 適用後の最終値。
- Rule側の `severity` はv1新規設計では使わない。互換読み込みが必要な場合のみ `severity` から `base_score` へ変換する。
- 互換入力としての `severity` は **migration専用** であり、新規presetや新規RulePackには出さない。

互換migrationの優先順位は以下に固定する。

| 入力 | 変換後 `base_score` | warning |
|---|---:|---|
| `base_score` のみ | `base_score` | なし |
| `base_score` + `score` | `base_score` | `score ignored because base_score is canonical` |
| `base_score` + `severity` | `base_score` | `severity ignored because base_score is canonical` |
| `score` のみ | `score` | `legacy score migrated to base_score` |
| `score` + `severity` | `score` | `legacy severity ignored because score is more precise` |
| `severity` のみ | 下表 | `legacy severity migrated to base_score` |

| legacy `severity` | `base_score` |
|---|---:|
| `Low` | 20 |
| `Medium` | 40 |
| `High` | 70 |
| `Critical` | 90 |

`Info` はv1では無効。読み込み時は設定エラーとして扱い、`Low` へ暗黙変換しない。

## D-010 LSP Span Contract

- `masked_snippet` は表示専用。編集範囲の復元には使わない。
- Coreのinternal findingは raw textを外に出さず、`FindingSpan { byte_start, byte_end }` と `Range { start, end }` を保持する。
- LSP Diagnostic range / CodeAction edit は `Range { start: Position, end: Position }` から生成する。
- `Position.character` はUTF-16 code units。
- DTO名は `Range` で統一し、`utf16_range` はフィールド名としてのみ使う。
- `veil:ignore` の挿入は言語別コメント構文で行う。コメント非対応形式（JSON等）は inline ignore code action を出さない。

## D-011 Skip / Incomplete / Exit 2

- user-configured ignore, `.gitignore`, built-in heavy dirs, unsupported binary skip は expected skip。scan complete として扱う。
- `max_file_count`, `max_findings`, text/log/source file `max_file_size` 超過、permission/read error, rule/config error は coverage gap。status=`incomplete` または `error`、Exit 2。
- oversized binary は expected skip。oversized text/log/source は incomplete。

## D-012 Evidence report.json schema

Evidence ZIP内 `report.json` は `EvidenceReportV1` schemaを使う。

- schema file: `schemas/json-schema.report.json`
- `schemaVersion`: `veil-evidence-report-v1`
- `summary.suppressedSeverityCounts` は必須。
- `findings`: `SafeFindingApiV1[]`。raw matched contentを含めない。
- `findings` は **raw-free な全finding** を含める。baseline使用時は `baselineStatus="new"` と `baselineStatus="suppressed"` の両方を含める。
- `summary`: total/effective/suppressed countsを含む。
- CLIの `veil scan --format json` は既存 `schemaVersion: veil-v1` であり、Evidence reportとは別契約。

## D-013 Repo hygiene

- `.gitignore` に `.codex/` を追加する。
- `.design/` と `.private/` はローカル設計/営業SOTであり、原則git管理しない。

## D-014 BaselineStatus rules

`SafeFindingApiV1.baselineStatus` は以下に固定する。

| 状態 | baselineStatus | findings配列への出現 |
|---|---|---|
| baseline未使用 | `none` | 通常返す |
| baseline使用・新規/未抑制 | `new` | 通常返す |
| baseline使用・抑制済み | `suppressed` | `includeSuppressed=true` の場合のみ |

## D-015 Limit / max_findings response contract

`max_findings` 等のcoverage limitに到達した場合:

- `status = "incomplete"`
- CLI Exit 2 / Local API HTTP 200 + status incomplete（API transport自体は成功）
- `limitReached = true`
- `limitReasons` に `max_findings` / `max_file_count` / `max_file_size_text` 等を入れる
- `coverageComplete = false`
- summary counts は **観測できた範囲の集計**。repo全体の推定値ではない。
- `findings` 配列は stable order の先頭から `max_findings` までの emitted findings。
- `Evidence ZIP` は生成可能だが `run_meta.result.status=incomplete` となる。`veil verify --require-complete` は Exit 1。

## D-016 Evidence generation order

Evidence ZIP生成順は以下に固定する。

1. `report.html` 生成
2. `report.json` 生成
3. `effective_config.toml` 生成
4. optional `veil.baseline.json` を確定
5. 上記artifactの sha256 / sizeBytes を計算
6. `run_meta.json` を生成（自分自身のsha256は入れない）
7. ZIP化
8. 必要に応じて、ZIP外で `run_meta.json` raw bytes SHA256 を外部アンカーとして表示/記録

## D-017 RunMetaResponse contract

`GET /api/runs/{runId}` は **full RunMetaV1** を返す。

- Evidence ZIP 内の `run_meta.json` と同じ構造を返す。
- `artifacts.*.sha256` は含める。
- `run_meta` 自身のsha256は含めない。
- UIが軽量表示をしたい場合は、クライアント側で必要フィールドを抜き出す。別subset DTOはv1では作らない。

## D-018 Compatibility / migration policy

- 既存 Evidence ZIP が `baseline.json` を含む場合、v1 verifier は `INVALID_EVIDENCE_SCHEMA` 相当の Exit 2 とし、メッセージで `veil.baseline.json` への再生成を促す。
- 既存 `.veil-baseline.json` は入力として読み取り互換を許可するが、Evidence ZIP 出力名は `veil.baseline.json` のみ。
- 既存 RulePack の `score` / `severity` は D-009 の優先順位表で `base_score` へ変換する。
- `base_score` がある場合は常に `base_score` を優先する。`score + severity` のみの場合は `score` を優先する。
- migration warning はstderrまたは `veil config explain` に出す。
- 旧 `severity` rule override はmigration専用。新規presetには書かない。
- 旧 schemaVersion の Evidence report / run_meta は strict に Exit 2。forward compatibility は v2設計時に明示実装する。

## D-019 Bulk implementation safety SOT

全機能を同一ブランチでまとめて実装する場合でも、内部順序・ON/OFF・rollback条件は固定する。詳細は `14_bulk_implementation_safety.md` を最優先で参照する。

要約:

```text
DTO/schema生成
→ Core model移行
→ Evidence契約
→ CLI exit契約
→ Local API
→ UI
→ LSP
```

契約は最初からON、既存挙動を壊す機能は段階ONにする。


## D-023 Finding ID / baseline fingerprint contract

`findingId` と `baselineFingerprint` は別物として維持する。

- `findingId`: Local API / Evidence / UI操作で使う表示・相関ID。
- `baselineFingerprint`: baseline suppress照合に使う長期安定キー。
- `SafeFindingApiV1` は `baselineFingerprint` をrequired fieldとして公開する。baseline未使用時も計算して返す。
- `veil.baseline.json` 内のJSON field名は既存互換のため `fingerprint` とし、API field名 `baselineFingerprint` とは分ける。
- baseline照合で `findingId` を使ってはならない。
- `findingId == baselineFingerprint` を仮定してはならない。
- どちらも raw secret を含まない opaque identifier とする。

この契約は現行実装の `baseline fingerprint` と `FindingId` が別に存在する前提を維持し、移行時に統合しない。

## D-024 RunMeta strict extension contract

RunMeta v1 は audit contract として strict schema を採用する。

- root `additionalProperties=false`。
- 将来拡張は `extensions: Record<string, unknown>` の専用namespaceにのみ入れる。
- v1 core fields に未知keyを混入させない。


### Product metadata

`RunMetaV1.product.name` は v1 では `"veil-pro" | "veil"` の列挙値に固定する。Local Audit UI から生成される Evidence は通常 `"veil-pro"`、将来 CLI 生成Evidenceを許可する場合のみ `"veil"` を使う。自由な string にはしない。


## D-021 RunMeta.result required fields and strictness

`RunMetaV1.result` is a strict v1 object. It MUST include `limitReasons` even when the array is empty. Unknown keys directly under `result` are forbidden. Future result-level extensions MUST use the top-level `extensions` namespace, not ad-hoc keys under `result`.

Canonical Rust DTO contract:

```rust
pub struct RunResultMeta {
    pub status: RunStatus,
    pub exit_code: u8,
    pub limit_reached: bool,
    pub limit_reasons: Vec<String>, // required, may be []
    pub summary: EvidenceSummary,
}
```

Schema/OpenAPI contract:

- `result.required` includes `status`, `exitCode`, `limitReached`, `limitReasons`, `summary`.
- `result.additionalProperties = false`.
- `limitReasons=[]` is required for complete non-limited runs.
