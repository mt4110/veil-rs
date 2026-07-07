// UI-facing types for schemas/openapi.local-api.yaml.
// Keep these names and casing aligned with crates/veil-pro/src/api/dto.rs.
export type LocalApiSchemaVersion = 'veil-pro-local-api-v1';
export type EvidenceReportSchemaVersion = 'veil-evidence-report-v1';
export type RunMetaSchemaVersion = 'veil-pro-run-meta-v1';
export type SeverityName = 'Low' | 'Medium' | 'High' | 'Critical';
export type GradeName = 'Low' | 'Medium' | 'High' | 'Critical';
export type BaselineStatus = 'none' | 'new' | 'suppressed';
export type PresetName = 'standard-jp' | 'fintech-jp' | 'gov-jp' | 'si-vendor-jp' | 'logs-jp';
export type ScanMode = 'full' | 'staged' | 'ci';
export type ConfigLayerName = 'builtin' | 'preset' | 'org' | 'repo' | 'cli';
export type RunStatus = 'success' | 'violation' | 'incomplete' | 'error';
export type AuthContextType = 'local_token' | 'sso';
export type ProductName = 'veil-pro' | 'veil';
export type BuildProfile = 'debug' | 'release';
export type EngineName = 'veil';
export type EngineSchemaVersion = 'veil-v1';
export type RulePackSource = 'embedded' | 'local' | 'remote';
export type NetworkMode = 'local-only' | 'enterprise-opt-in';
export type TelemetryMode = 'none';
export type BindAddress = '127.0.0.1';
export type ErrorCode =
  | 'INVALID_REQUEST'
  | 'UNAUTHORIZED'
  | 'PATH_DENIED'
  | 'NOT_FOUND'
  | 'RUN_EXPIRED'
  | 'RUN_TOO_LARGE'
  | 'INTERNAL_ERROR';
export type NextAction =
  | 'RESCAN'
  | 'CHECK_TOKEN'
  | 'NARROW_SCOPE'
  | 'OPEN_DOCTOR'
  | 'COMMIT_BASELINE'
  | 'REVIEW_FINDINGS';

export type SeverityCounts = {
  Low: number;
  Medium: number;
  High: number;
  Critical: number;
};

export type LocalTokenAuthContext = {
  authenticated: boolean;
  type: 'local_token';
};

export type SsoAuthContext = {
  authenticated: boolean;
  type: 'sso';
  email: string;
  name?: string | null;
  enterpriseOptIn: boolean;
};

export type AuthContext = LocalTokenAuthContext | SsoAuthContext;

export type ProjectsResponse = {
  schemaVersion: LocalApiSchemaVersion;
  currentDir: string;
  projects: ProjectSummary[];
};

export type ProjectSummary = {
  id: string;
  displayName: string;
  rootPath: string;
  isCurrent: boolean;
  hasRepoConfig: boolean;
};

export type ScanRequest = {
  paths?: string[] | null;
  preset?: PresetName | null;
  mode?: ScanMode | null;
  baselineFile?: string | null;
  includeSuppressed?: boolean;
  failOnScore?: number | null;
  failOnSeverity?: SeverityName | null;
  failOnFindings?: number | null;
};

export type SafeFindingApiV1 = {
  findingId: string;
  baselineFingerprint: string;
  path: string;
  lineNumber: number;
  ruleId: string;
  severity: SeverityName;
  score: number;
  grade: GradeName;
  maskedSnippet: string;
  category: string;
  tags: string[];
  baselineStatus: BaselineStatus;
};

export type ScanResponse = {
  schemaVersion: LocalApiSchemaVersion;
  runId: string;
  status: RunStatus;
  scannedFiles: number;
  skippedFiles: number;
  totalFindings: number;
  suppressedFindings: number;
  effectiveFindings: number;
  coverageComplete: boolean;
  severityCounts: SeverityCounts;
  allSeverityCounts: SeverityCounts;
  suppressedSeverityCounts: SeverityCounts;
  limitReached: boolean;
  limitReasons: string[];
  builtinSkips: string[];
  findings: SafeFindingApiV1[];
  expiresAtUtc: string;
};

export type ConfigLayerSummary = {
  name: ConfigLayerName;
  path?: string | null;
  loaded: boolean;
  warnings: string[];
};

export type ConfigConflict = {
  key: string;
  winner: ConfigLayerName;
  shadowed: string[];
  explanation: string;
};

export type PolicyResponse = {
  schemaVersion: LocalApiSchemaVersion;
  hasOrgConfig: boolean;
  orgConfigPath?: string | null;
  repoConfigPath?: string | null;
  effectiveRulesCount: number;
  preset?: string | null;
  layers: ConfigLayerSummary[];
  conflicts: ConfigConflict[];
};

export type BaselineRequest = {
  paths?: string[] | null;
  outputPath?: string | null;
};

export type BaselineResponse = {
  schemaVersion: LocalApiSchemaVersion;
  filePath: string;
  findingsCount: number;
  written: boolean;
  nextAction: NextAction;
};

export type BoundValue = number | string | boolean;

export type DoctorRulePack = {
  name: string;
  version?: string | null;
  source: RulePackSource;
};

export type DoctorResponse = {
  schemaVersion: LocalApiSchemaVersion;
  productVersion: string;
  os: string;
  rustVersion?: string | null;
  config: PolicyResponse;
  bounds: Record<string, BoundValue>;
  rulePacks: DoctorRulePack[];
  networkMode: NetworkMode;
  warnings: string[];
};

export type EvidenceSummary = {
  totalFindings: number;
  suppressedFindings: number;
  effectiveFindings: number;
  severityCounts: SeverityCounts;
  allSeverityCounts: SeverityCounts;
  suppressedSeverityCounts: SeverityCounts;
  coverageComplete: boolean;
};

export type ProductMeta = {
  name: ProductName;
  version: string;
  commit?: string | null;
  buildProfile?: BuildProfile | null;
};

export type RulePackMeta = {
  name: string;
  source: RulePackSource;
  contentSha256?: string | null;
  version?: string | null;
};

export type EngineMeta = {
  name: EngineName;
  schemaVersion: EngineSchemaVersion;
  rulePacks: RulePackMeta[];
};

export type RunResultMeta = {
  status: RunStatus;
  exitCode: 0 | 1 | 2;
  limitReached: boolean;
  limitReasons: string[];
  summary: EvidenceSummary;
};

export type ArtifactMeta = {
  path: string;
  sha256: string;
  sizeBytes?: number | null;
};

export type BaselineArtifactMeta = {
  path: 'veil.baseline.json';
  sha256: string;
  sizeBytes?: number | null;
};

export type EvidenceArtifacts = {
  reportHtml: ArtifactMeta;
  reportJson: ArtifactMeta;
  effectiveConfig: ArtifactMeta;
  baseline?: BaselineArtifactMeta | null;
};

export type PrivacyMeta = {
  telemetry: TelemetryMode;
  networkMode: NetworkMode;
  bind: BindAddress;
};

export type RunMetaResponse = {
  schemaVersion: RunMetaSchemaVersion;
  runId: string;
  generatedAtUtc: string;
  product: ProductMeta;
  engine: EngineMeta;
  result: RunResultMeta;
  artifacts: EvidenceArtifacts;
  privacy: PrivacyMeta;
  extensions?: Record<string, unknown> | null;
};

export type EvidenceReportV1 = {
  schemaVersion: EvidenceReportSchemaVersion;
  runId: string;
  generatedAtUtc: string;
  summary: EvidenceSummary;
  findings: SafeFindingApiV1[];
};

export type ErrorEnvelope = {
  error: {
    code: ErrorCode;
    message: string;
    nextAction?: NextAction | null;
  };
};
