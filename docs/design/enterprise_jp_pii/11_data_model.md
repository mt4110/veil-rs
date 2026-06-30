# 11. Data Model 設計

## 11.1 Finding（Core internal）

```rust
pub struct Finding {
    pub path: PathBuf,
    pub line_number: usize,
    pub line_content: String,      // 内部のみ。UI/Evidenceには出さない
    pub rule_id: String,
    pub matched_content: String,   // 内部のみ。UI/Evidenceには出さない
    pub masked_snippet: String,
    pub severity: Severity,        // scoreから導出された最終severity
    pub score: u32,                // 0-100 final score
    pub grade: Grade,              // score band
    pub span: FindingSpan,         // raw textを外へ出さない範囲情報
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
    pub commit_sha: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
}
```

## 11.2 SafeFindingApiV1（Local API / Evidence）

```rust
pub struct SafeFindingApiV1 {
    pub finding_id: String,
    pub baseline_fingerprint: String,
    pub path: String,
    pub line_number: usize,
    pub rule_id: String,
    pub severity: Severity,
    pub score: u32,
    pub grade: Grade,
    pub masked_snippet: String,
    pub category: String,
    pub tags: Vec<String>,
    pub baseline_status: BaselineStatus, // new | suppressed | none
}
```

`SafeFindingApiV1` は raw `matched_content` / `line_content` を絶対に含めない。


## 11.2.2 Finding ID / baseline fingerprint contract

`finding_id` と `baseline_fingerprint` は **別物**として維持する。

| Field | 用途 | 安定性 | 外部公開 |
|---|---|---|---|
| `finding_id` | UI行ID、deep link、ユーザー操作対象 | 同一run内と同一入力で安定 | Local API / Evidenceに出す |
| `baseline_fingerprint` | baseline suppress照合キー | baseline互換性のため長期安定 | opaque hashとしてLocal API / Evidenceに出す |

- baseline照合は `baseline_fingerprint` のみで行う。
- `finding_id == baseline_fingerprint` を仮定してはならない。
- `veil.baseline.json` 内のJSON field名は既存互換のため `fingerprint` とし、API/Evidence field名 `baselineFingerprint` とは分ける。
- `finding_id` は表示/API操作用であり、baseline互換性の正本ではない。
- どちらも raw secret / matched content を含まない opaque identifier とする。

## 11.2.1 Score / Grade / Severity contract

`score` を唯一のCI判定SOTとし、`grade` と `severity` は score から導出する。Rule側に `score` は存在しない。

| score | grade | severity | 備考 |
|---:|---|---|---|
| 0-19 | Safe | 出力しない | verbose/debugのみ |
| 20-39 | Low | Low | 低信頼または弱文脈 |
| 40-69 | Medium | Medium | 通常検知 |
| 70-89 | High | High | CI fail候補 |
| 90-100 | Critical | Critical | 強文脈/validator確定 |

`--fail-on-severity S` は `score >= minScore(S)` の別名として評価する。

| failOnSeverity | minScore |
|---|---:|
| Low | 20 |
| Medium | 40 |
| High | 70 |
| Critical | 90 |

`--fail-on-findings N` は baseline suppress 後の `effectiveFindings >= N` で判定する。`N=0` は設定エラー。

## 11.3 Rule

```rust
pub struct Rule {
    pub id: String,
    pub pattern: Regex,
    pub description: String,
    pub base_score: u32,
    pub category: String,
    pub tags: Vec<String>,
    pub context_lines_before: u8,
    pub context_lines_after: u8,
    pub validator_id: Option<String>,
    pub placeholder: Option<String>,
}
```

Rule定義の `base_score` は初期値。Findingの最終 `score` は validator/context/negative context で調整後に決まる。既存RulePackに `score` / `severity` が残る場合は `00_contract_decisions.md` D-009 の優先順位表で `base_score` へ移す。

## 11.4 Config

```rust
pub struct Config {
    pub core: CoreConfig,
    pub masking: MaskingConfig,
    pub output: OutputConfig,
    pub rules: HashMap<String, RuleConfig>,
}

pub struct CoreConfig {
    pub include: Vec<String>,
    pub ignore: Vec<String>,
    pub max_file_size: Option<u64>,
    pub max_file_count: Option<usize>,
    pub fail_on_score: Option<u32>,
    pub rules_dir: Option<String>,
    pub preset: Option<String>,
    // Enterprise opt-in only. Ignored unless allow_remote_rules && VEIL_ALLOW_NETWORK=1.
    pub allow_remote_rules: bool,
    pub remote_rules_url: Option<String>,
}
```

## 11.5 ScanResult

```rust
pub struct ScanResult {
    pub findings_all: Vec<Finding>,
    pub findings_effective: Vec<Finding>,
    pub findings_suppressed: Vec<Finding>,
    pub total_files: usize,
    pub scanned_files: usize,
    pub skipped_files: usize,
    pub limit_reached: bool,
    pub file_limit_reached: bool,
    pub builtin_skips: HashSet<String>,
    pub coverage_complete: bool,
}
```

## 11.6 LSP Span Model

`masked_snippet` から編集範囲を復元してはならない。Coreは内部用に raw text を外へ出さず、以下のspanを保持する。

```rust
pub struct FindingSpan {
    pub byte_start: usize,
    pub byte_end: usize,
}

pub struct Position {
    pub line: u32,
    pub character: u32, // UTF-16 code units
}

pub struct Range {
    pub start: Position,
    pub end: Position,
}

pub struct LspFinding {
    pub safe: SafeFindingApiV1,
    pub original_span: FindingSpan, // internal only
    pub utf16_range: Range,
}
```

LSP Diagnostic range と CodeAction edit は `utf16_range` から生成する。raw matched_content はLSP `data` に載せない。

## 11.7 LSP Diagnostic Data

```json
{
  "ruleId": "pii.jp.mynumber.keyword",
  "score": 92,
  "grade": "Critical",
  "maskedSnippet": "個人番号: <REDACTED>",
  "actions": ["mask", "ignore"]
}
```

## 11.8 EvidenceReportV1 / EvidenceSummary

`EvidenceSummary` は Evidence report と RunMeta の両方で使う唯一のcanonical定義である。別定義を作らない。

```rust
pub struct EvidenceReportV1 {
    pub schema_version: String, // veil-evidence-report-v1
    pub run_id: String,
    pub generated_at_utc: String,
    pub summary: EvidenceSummary,
    pub findings: Vec<SafeFindingApiV1>,
}

pub struct EvidenceSummary {
    pub total_findings: usize,
    pub suppressed_findings: usize,
    pub effective_findings: usize,
    pub severity_counts: BTreeMap<Severity, usize>,
    pub all_severity_counts: BTreeMap<Severity, usize>,
    pub suppressed_severity_counts: BTreeMap<Severity, usize>,
    pub coverage_complete: bool,
}
```

`severity_counts`, `all_severity_counts`, `suppressed_severity_counts` は zero-filled とし、`Low` / `Medium` / `High` / `Critical` の4キーを必ず持つ。

Evidence `report.json` は raw-free な全 `SafeFindingApiV1[]` を含み、raw secretを含まない。baseline使用時は `new` と `suppressed` の両方を含める。

## 11.9 JSON Schema互換方針

- CLI JSONは `schemaVersion: veil-v1`。
- Evidence `report.json` は `schemaVersion: veil-evidence-report-v1`。
- 破壊的変更はそれぞれ新しいschemaVersionで行う。
- UI APIはcamelCase、CLI JSONは既存互換を維持。
- `schemas/json-schema.safe-finding-api.json` は UI/API用。
- `schemas/json-schema.finding.json` は CLI/Core用。
- `schemas/json-schema.report.json` は Evidence ZIP用。


## 11.10 RunMetaV1 / RunMetaResponse

`RunMetaResponse` は `RunMetaV1` と同一構造を返す。

```rust
pub struct RunMetaV1 {
    pub schema_version: String, // veil-pro-run-meta-v1
    pub run_id: String,
    pub generated_at_utc: String,
    pub product: ProductMeta,
    pub engine: EngineMeta,
    pub result: RunResultMeta,
    pub artifacts: EvidenceArtifacts,
    pub privacy: PrivacyMeta,
    pub extensions: Option<BTreeMap<String, serde_json::Value>>, // reserved extension namespace only
}

pub struct RunResultMeta {
    pub status: RunStatus,
    pub exit_code: u8,
    pub limit_reached: bool,
    pub limit_reasons: Vec<String>,
    pub summary: EvidenceSummary,
}

`limit_reasons` is required and may be empty. `RunResultMeta` has no ad-hoc extension fields; schema/OpenAPI set `additionalProperties=false` for the `result` object.

pub struct EvidenceArtifacts {
    pub report_html: ArtifactMeta,
    pub report_json: ArtifactMeta,
    pub effective_config: ArtifactMeta,
    pub baseline: Option<BaselineArtifactMeta>, // path const: veil.baseline.json
}
```

```rust
pub struct ProductMeta {
    pub name: ProductName, // enum: VeilPro | Veil
    pub version: String,
    pub commit: Option<String>,
    pub build_profile: Option<BuildProfile>, // debug | release
}
```

`product.name` は `veil-pro | veil` のenumであり、`veil-pro` 固定literalではない。Local UIは通常 `veil-pro` を返すが、CLI/OSS由来のEvidence互換のため `veil` も許容する。

`RunMetaV1` は自分自身のsha256を内部に持たない。未知の将来拡張は `extensions` namespace にのみ格納し、既知v1フィールドの `additionalProperties` は禁止する。

## 11.11 BaselineStatus rules

```rust
pub enum BaselineStatus {
    None,
    New,
    Suppressed,
}
```

- baseline未使用時: `None`
- baseline使用時かつ未抑制: `New`
- baseline使用時かつ抑制済み: `Suppressed`
