# 5. 監査レポートUI用APIスキーマ設計

## 5.0 SOT方針

- API schema の正本は Rust DTO (`crates/veil-pro/src/api/dto.rs`) とする。schema出力先は repo root `schemas/`。
- OpenAPI (`schemas/openapi.local-api.yaml`) と JSON Schema は Rust DTO から生成する派生物。手編集は禁止。
- Local API は **camelCase** を使う。
- CLI JSON (`schemaVersion: veil-v1`) は既存互換のため **snake_case** を維持する。
- `SafeFindingApiV1` はUI/API/Evidence report用の raw-free schema。`FindingV1`（CLI/Core）と混同しない。

## 5.1 原則

- APIは `127.0.0.1` のみ。
- 認証は `Authorization: Bearer <token>`。
- tokenはURL fragment `#token=` でUIへ渡し、即 `history.replaceState` で消す。
- APIは raw secret / raw line_content / matched_content を返さない。
- UIは `maskedSnippet` のみ表示。
- SSOはEnterprise opt-in。通常起動の `/api/me` は `local_token` のみを返す。
- Enterprise opt-in時は `privacy.networkMode="enterprise-opt-in"` とし、通常時は `local-only`。

## 5.2 エンドポイント一覧

| Method | Path | 説明 | Response DTO |
|---|---|---|---|
| GET | `/api/me` | 認証状態 | `AuthContext` |
| GET | `/api/projects` | プロジェクト情報 | `ProjectsResponse` |
| POST | `/api/scan` | scan実行 | `ScanResponse` |
| GET | `/api/runs/{runId}` | run metadata取得 | `RunMetaResponse` |
| GET | `/api/runs/{runId}/evidence.zip` | Evidence ZIP取得 | `application/zip` |
| GET | `/api/policy` | effective policy概要 | `PolicyResponse` |
| POST | `/api/baseline` | baseline生成 | `BaselineResponse` |
| GET | `/api/doctor` | 診断情報 | `DoctorResponse` |

OpenAPIは上記全endpointを含む。設計書に列挙したendpointがOpenAPIに無い状態を禁止する。

## 5.3 共通エラー

```json
{
  "error": {
    "code": "RUN_EXPIRED",
    "message": "Run evidence has expired. Please run a new scan.",
    "nextAction": "RESCAN"
  }
}
```

| HTTP | code | UI動作 |
|---:|---|---|
| 400 | INVALID_REQUEST | 入力欄に戻す |
| 401 | UNAUTHORIZED | token再入力 |
| 403 | PATH_DENIED | path説明 |
| 404 | NOT_FOUND | runなし |
| 410 | RUN_EXPIRED | Re-Scan Now |
| 413 | RUN_TOO_LARGE | scope縮小 / CLI案内 |
| 500 | INTERNAL_ERROR | Doctor export |

## 5.4 DTO

### AuthContext
```ts
type AuthContext =
  | { authenticated: true; type: "local_token" }
  | { authenticated: true; type: "sso"; email: string; name?: string; enterpriseOptIn: true };
```

### ProjectsResponse
```ts
interface ProjectsResponse {
  schemaVersion: "veil-pro-local-api-v1";
  currentDir: string;
  projects: ProjectSummary[];
}

interface ProjectSummary {
  id: string;
  displayName: string;
  rootPath: string;
  isCurrent: boolean;
  hasRepoConfig: boolean;
}
```

### ScanRequest
```ts
interface ScanRequest {
  /** 省略または [] の場合は API が ["."] に正規化する。 */
  paths?: string[];
  preset?: "standard-jp" | "fintech-jp" | "gov-jp" | "si-vendor-jp" | "logs-jp";
  mode?: "full" | "staged" | "ci";
  baselineFile?: string;
  includeSuppressed?: boolean; // default false
  failOnScore?: number;
  failOnSeverity?: "Low" | "Medium" | "High" | "Critical";
  failOnFindings?: number; // >= 1. effectiveFindings >= N でviolation
}
```

### ScanResponse
```ts
interface ScanResponse {
  schemaVersion: "veil-pro-local-api-v1";
  runId: string;
  status: "success" | "violation" | "incomplete" | "error";
  scannedFiles: number;
  skippedFiles: number;

  /** baseline適用前の全finding数 */
  totalFindings: number;
  /** baselineで抑制されたfinding数 */
  suppressedFindings: number;
  /** baseline suppress後のCI判定対象finding数 */
  effectiveFindings: number;

  /** limit未到達ならtrue。max_findings/max_file_count/text oversize等のcoverage gapでfalse。 */
  coverageComplete: boolean;

  /** effective findingsのseverity集計。fail判定対象。 */
  severityCounts: Record<"Low" | "Medium" | "High" | "Critical", number>;
  /** baseline適用前の集計。監査表示用。 */
  allSeverityCounts: Record<"Low" | "Medium" | "High" | "Critical", number>;
  /** suppressed findingsの集計。 */
  suppressedSeverityCounts: Record<"Low" | "Medium" | "High" | "Critical", number>;

  limitReached: boolean;
  limitReasons: string[];
  builtinSkips: string[];
  /** default: effective findingsのみ。includeSuppressed=true時のみsuppressedも含める。 */
  findings: SafeFindingApiV1[];
  expiresAtUtc: string;
}
```

### SafeFindingApiV1
```ts
interface SafeFindingApiV1 {
  findingId: string;             // UI/API操作用ID。baseline照合には使わない。
  baselineFingerprint: string;   // baseline suppress照合キー。findingIdとは別物。
  path: string;
  lineNumber: number;
  ruleId: string;
  severity: "Low" | "Medium" | "High" | "Critical";
  score: number;              // 0-100 final score
  grade: "Low" | "Medium" | "High" | "Critical";
  maskedSnippet: string;
  category: string;
  tags: string[];
  baselineStatus: "new" | "suppressed" | "none";
}
```

### PolicyResponse
```ts
interface PolicyResponse {
  schemaVersion: "veil-pro-local-api-v1";
  hasOrgConfig: boolean;
  orgConfigPath?: string;
  repoConfigPath?: string;
  effectiveRulesCount: number;
  preset?: string;
  layers: ConfigLayerSummary[];
  conflicts: ConfigConflict[];
}

interface ConfigLayerSummary {
  name: "builtin" | "preset" | "org" | "repo" | "cli";
  path?: string;
  loaded: boolean;
  warnings: string[];
}

interface ConfigConflict {
  key: string;
  winner: "builtin" | "preset" | "org" | "repo" | "cli";
  shadowed: string[];
  explanation: string;
}
```

### BaselineRequest / BaselineResponse
```ts
interface BaselineRequest {
  paths?: string[]; // missing/empty -> ["."]
  outputPath?: string; // default veil.baseline.json
}

interface BaselineResponse {
  schemaVersion: "veil-pro-local-api-v1";
  filePath: string;
  findingsCount: number;
  written: boolean;
  nextAction: "COMMIT_BASELINE" | "REVIEW_FINDINGS";
}
```

### DoctorResponse
```ts
interface DoctorResponse {
  schemaVersion: "veil-pro-local-api-v1";
  productVersion: string;
  os: string;
  rustVersion?: string;
  config: PolicyResponse;
  bounds: Record<string, number | string | boolean>;
  rulePacks: { name: string; version?: string; source: "embedded" | "local" | "remote" }[];
  networkMode: "local-only" | "enterprise-opt-in";
  warnings: string[];
}
```

### RunMetaResponse
```ts
interface RunMetaResponse {
  schemaVersion: "veil-pro-run-meta-v1";
  runId: string;
  generatedAtUtc: string;
  product: ProductMeta;
  engine: EngineMeta;
  result: RunResultMeta;
  artifacts: EvidenceArtifacts;
  privacy: PrivacyMeta;
  extensions?: Record<string, unknown>; // reserved extension namespace only
}

interface ProductMeta {
  name: "veil-pro" | "veil";
  version: string;
  commit?: string;
  buildProfile?: "debug" | "release";
}

interface EngineMeta {
  name: "veil";
  schemaVersion: "veil-v1";
  rulePacks: Array<{
    name: string;
    source: "embedded" | "local" | "remote";
    contentSha256?: string;
    version?: string;
  }>;
}

interface RunResultMeta {
  status: "success" | "violation" | "incomplete" | "error";
  exitCode: 0 | 1 | 2;
  limitReached: boolean;
  limitReasons: string[];
  summary: EvidenceSummary;
}

`RunResultMeta` is strict. `limitReasons` is required and MUST be an empty array when there is no limit reason. Unknown keys under `result` are forbidden; use top-level `extensions` for future metadata.

interface EvidenceSummary {
  totalFindings: number;
  suppressedFindings: number;
  effectiveFindings: number;
  severityCounts: SeverityCounts;
  allSeverityCounts: SeverityCounts;
  suppressedSeverityCounts: SeverityCounts;
  coverageComplete: boolean;
}

type SeverityCounts = {
  Low: number;
  Medium: number;
  High: number;
  Critical: number;
};

interface EvidenceArtifacts {
  reportHtml: ArtifactMeta;
  reportJson: ArtifactMeta;
  effectiveConfig: ArtifactMeta;
  baseline?: BaselineArtifactMeta;
}

interface ArtifactMeta {
  path: string;
  sha256: string;
  sizeBytes?: number;
}

interface BaselineArtifactMeta extends ArtifactMeta {
  path: "veil.baseline.json";
}

interface PrivacyMeta {
  telemetry: "none";
  networkMode: "local-only" | "enterprise-opt-in";
  bind: "127.0.0.1";
}
```

`GET /api/runs/{runId}` は subset DTO を返さない。Evidence ZIP 内の `run_meta.json` と同じ full `RunMetaV1` 構造を返し、UIは必要なフィールドだけを読む。

## 5.5 baseline count contract

- `totalFindings` は baseline適用前の全件。
- `effectiveFindings` は baseline suppress後の件数。CI fail判定と画面の通常表示はこれを使う。
- `suppressedFindings` は baselineで抑制された件数。
- `findings` は default で effective findings のみ。
- `includeSuppressed=true` のときのみ suppressed finding を含め、`baselineStatus="suppressed"` とする。
- `--fail-on-findings N` / `failOnFindings` は `effectiveFindings >= N` で判定する。

## 5.6 Evidence ZIP endpoint

`GET /api/runs/{runId}/evidence.zip` は保存済みrun artifactsからZIPを返す。再スキャンは禁止。RunCacheの揮発時は `410 RUN_EXPIRED`。


## 5.8 v4 Response DTO契約

### RunMetaResponse

`GET /api/runs/{runId}` は subset ではなく **full RunMetaV1** を返す。Evidence ZIP内の `run_meta.json` と同じ構造であり、UIは必要部分だけを読む。

### ScanResponse counts

- `totalFindings`: baseline適用前の全finding数。ただし `coverageComplete=false` の場合は観測できた範囲の値。
- `suppressedFindings`: baselineで抑制された件数。
- `effectiveFindings`: CI fail判定対象件数。
- `severityCounts`: effective findings集計。
- `allSeverityCounts`: baseline適用前集計。
- `suppressedSeverityCounts`: 必須。抑制0件時も `Low` / `Medium` / `High` / `Critical` を0で持つ。
- `SeverityCounts` は全surfaceで zero-filled map。キー省略は禁止。
- `coverageComplete`: limit未到達ならtrue。`max_findings` / `max_file_count` / text oversize等でfalse。
- `findings`: defaultではeffective findingsのみ。`includeSuppressed=true` のときのみsuppressedを含める。

### failOnFindings

- `failOnFindings >= 1` のみ許可。
- `failOnFindings=0` は HTTP 400 `INVALID_REQUEST`。
- 違反判定は `effectiveFindings >= failOnFindings`。
