// UI-facing subset of schemas/openapi.local-api.yaml.
// Keep these names and casing aligned with crates/veil-pro/src/api/dto.rs.
export type LocalApiSchemaVersion = 'veil-pro-local-api-v1';
export type SeverityName = 'Low' | 'Medium' | 'High' | 'Critical';
export type GradeName = 'Low' | 'Medium' | 'High' | 'Critical';
export type BaselineStatus = 'none' | 'new' | 'suppressed';
export type PresetName = 'standard-jp' | 'fintech-jp' | 'gov-jp' | 'si-vendor-jp' | 'logs-jp';
export type ScanMode = 'full' | 'staged' | 'ci';
export type RunStatus = 'success' | 'violation' | 'incomplete' | 'error';
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

export type ErrorEnvelope = {
  error: {
    code: ErrorCode;
    message: string;
    nextAction?: NextAction | null;
  };
};
