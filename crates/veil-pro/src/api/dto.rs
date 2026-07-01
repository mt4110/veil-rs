use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum LocalApiSchemaVersion {
    VeilProLocalApiV1,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceReportSchemaVersion {
    VeilEvidenceReportV1,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RunMetaSchemaVersion {
    VeilProRunMetaV1,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "PascalCase")]
pub enum SeverityName {
    Low,
    Medium,
    High,
    Critical,
}

impl From<&veil_core::Severity> for SeverityName {
    fn from(value: &veil_core::Severity) -> Self {
        match value {
            veil_core::Severity::Low => Self::Low,
            veil_core::Severity::Medium => Self::Medium,
            veil_core::Severity::High => Self::High,
            veil_core::Severity::Critical => Self::Critical,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "PascalCase")]
pub enum GradeName {
    Low,
    Medium,
    High,
    Critical,
}

impl From<&veil_core::Grade> for GradeName {
    fn from(value: &veil_core::Grade) -> Self {
        match value {
            veil_core::Grade::Critical => Self::Critical,
            veil_core::Grade::High => Self::High,
            veil_core::Grade::Medium => Self::Medium,
            veil_core::Grade::Low | veil_core::Grade::Safe => Self::Low,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "PascalCase")]
pub enum FindingGradeName {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum BaselineStatus {
    None,
    New,
    Suppressed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Success,
    Violation,
    Incomplete,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "kebab-case")]
#[allow(clippy::enum_variant_names)]
pub enum PresetName {
    StandardJp,
    FintechJp,
    GovJp,
    SiVendorJp,
    LogsJp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ScanMode {
    Full,
    Staged,
    Ci,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidRequest,
    Unauthorized,
    PathDenied,
    NotFound,
    RunExpired,
    RunTooLarge,
    InternalError,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NextAction {
    Rescan,
    CheckToken,
    NarrowScope,
    OpenDoctor,
    CommitBaseline,
    ReviewFindings,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ErrorEnvelope {
    pub error: ErrorBody,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_action: Option<NextAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(untagged)]
pub enum AuthContext {
    LocalToken(LocalTokenAuthContext),
    Sso(SsoAuthContext),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalTokenAuthContext {
    pub authenticated: bool,
    #[serde(rename = "type")]
    pub kind: AuthContextType,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SsoAuthContext {
    pub authenticated: bool,
    #[serde(rename = "type")]
    pub kind: AuthContextType,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub enterprise_opt_in: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthContextType {
    LocalToken,
    Sso,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectsResponse {
    pub schema_version: LocalApiSchemaVersion,
    pub current_dir: String,
    pub projects: Vec<ProjectSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectSummary {
    pub id: String,
    pub display_name: String,
    pub root_path: String,
    pub is_current: bool,
    pub has_repo_config: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScanRequest {
    #[serde(default)]
    pub paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<PresetName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ScanMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_file: Option<String>,
    #[serde(default)]
    pub include_suppressed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 0, max = 100))]
    #[schema(minimum = 0, maximum = 100)]
    pub fail_on_score: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_on_severity: Option<SeverityName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 1))]
    #[schema(minimum = 1)]
    pub fail_on_findings: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SafeFindingApiV1 {
    pub finding_id: String,
    pub baseline_fingerprint: String,
    pub path: String,
    #[schemars(range(min = 1))]
    #[schema(minimum = 1)]
    pub line_number: usize,
    pub rule_id: String,
    pub severity: SeverityName,
    #[schemars(range(min = 0, max = 100))]
    #[schema(minimum = 0, maximum = 100)]
    pub score: u32,
    pub grade: GradeName,
    pub masked_snippet: String,
    pub category: String,
    pub tags: Vec<String>,
    pub baseline_status: BaselineStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct SeverityCounts {
    pub low: usize,
    pub medium: usize,
    pub high: usize,
    pub critical: usize,
}

impl SeverityCounts {
    pub fn zero() -> Self {
        Self {
            low: 0,
            medium: 0,
            high: 0,
            critical: 0,
        }
    }

    pub fn increment(&mut self, severity: SeverityName) {
        match severity {
            SeverityName::Low => self.low += 1,
            SeverityName::Medium => self.medium += 1,
            SeverityName::High => self.high += 1,
            SeverityName::Critical => self.critical += 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScanResponse {
    pub schema_version: LocalApiSchemaVersion,
    pub run_id: String,
    pub status: RunStatus,
    pub scanned_files: usize,
    pub skipped_files: usize,
    pub total_findings: usize,
    pub suppressed_findings: usize,
    pub effective_findings: usize,
    pub coverage_complete: bool,
    pub severity_counts: SeverityCounts,
    pub all_severity_counts: SeverityCounts,
    pub suppressed_severity_counts: SeverityCounts,
    pub limit_reached: bool,
    pub limit_reasons: Vec<String>,
    pub builtin_skips: Vec<String>,
    pub findings: Vec<SafeFindingApiV1>,
    pub expires_at_utc: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ConfigLayerName {
    Builtin,
    Preset,
    Org,
    Repo,
    Cli,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfigLayerSummary {
    pub name: ConfigLayerName,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub loaded: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfigConflict {
    pub key: String,
    pub winner: ConfigLayerName,
    pub shadowed: Vec<String>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PolicyResponse {
    pub schema_version: LocalApiSchemaVersion,
    pub has_org_config: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_config_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_config_path: Option<String>,
    pub effective_rules_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
    pub layers: Vec<ConfigLayerSummary>,
    pub conflicts: Vec<ConfigConflict>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BaselineRequest {
    #[serde(default)]
    pub paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BaselineResponse {
    pub schema_version: LocalApiSchemaVersion,
    pub file_path: String,
    pub findings_count: usize,
    pub written: bool,
    pub next_action: NextAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(untagged)]
pub enum BoundValue {
    Number(u64),
    Text(String),
    Flag(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DoctorResponse {
    pub schema_version: LocalApiSchemaVersion,
    pub product_version: String,
    pub os: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_version: Option<String>,
    pub config: PolicyResponse,
    pub bounds: BTreeMap<String, BoundValue>,
    pub rule_packs: Vec<DoctorRulePack>,
    pub network_mode: NetworkMode,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DoctorRulePack {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub source: RulePackSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceReportV1 {
    pub schema_version: EvidenceReportSchemaVersion,
    pub run_id: String,
    pub generated_at_utc: String,
    pub summary: EvidenceSummary,
    pub findings: Vec<SafeFindingApiV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceSummary {
    pub total_findings: usize,
    pub suppressed_findings: usize,
    pub effective_findings: usize,
    pub severity_counts: SeverityCounts,
    pub all_severity_counts: SeverityCounts,
    pub suppressed_severity_counts: SeverityCounts,
    pub coverage_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunMetaV1 {
    pub schema_version: RunMetaSchemaVersion,
    pub run_id: String,
    pub generated_at_utc: String,
    pub product: ProductMeta,
    pub engine: EngineMeta,
    pub result: RunResultMeta,
    pub artifacts: EvidenceArtifacts,
    pub privacy: PrivacyMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object)]
    pub extensions: Option<BTreeMap<String, serde_json::Value>>,
}

pub type RunMetaResponse = RunMetaV1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ProductName {
    VeilPro,
    Veil,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum BuildProfile {
    Debug,
    Release,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProductMeta {
    pub name: ProductName,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_profile: Option<BuildProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EngineMeta {
    pub name: EngineName,
    pub schema_version: EngineSchemaVersion,
    pub rule_packs: Vec<RulePackMeta>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum EngineName {
    Veil,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum EngineSchemaVersion {
    VeilV1,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum RulePackSource {
    Embedded,
    Local,
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RulePackMeta {
    pub name: String,
    pub source: RulePackSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunResultMeta {
    pub status: RunStatus,
    #[schemars(range(min = 0, max = 2))]
    #[schema(minimum = 0, maximum = 2)]
    pub exit_code: u8,
    pub limit_reached: bool,
    pub limit_reasons: Vec<String>,
    pub summary: EvidenceSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceArtifacts {
    pub report_html: ArtifactMeta,
    pub report_json: ArtifactMeta,
    pub effective_config: ArtifactMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<BaselineArtifactMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtifactMeta {
    pub path: String,
    pub sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BaselineArtifactMeta {
    pub path: BaselineArtifactPath,
    pub sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<usize>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
pub enum BaselineArtifactPath {
    #[serde(rename = "veil.baseline.json")]
    VeilBaselineJson,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkMode {
    LocalOnly,
    EnterpriseOptIn,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum TelemetryMode {
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrivacyMeta {
    pub telemetry: TelemetryMode,
    pub network_mode: NetworkMode,
    pub bind: BindAddress,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
pub enum BindAddress {
    #[serde(rename = "127.0.0.1")]
    LocalhostV4,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct FindingV1 {
    pub path: String,
    #[schemars(range(min = 1))]
    #[schema(minimum = 1)]
    pub line_number: usize,
    pub line_content: String,
    pub rule_id: String,
    pub matched_content: String,
    pub severity: SeverityName,
    #[schemars(range(min = 0, max = 100))]
    #[schema(minimum = 0, maximum = 100)]
    pub score: u32,
    pub grade: FindingGradeName,
    pub masked_snippet: String,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_meta_json(result_extra: serde_json::Value) -> serde_json::Value {
        let mut result = serde_json::json!({
            "status": "success",
            "exitCode": 0,
            "limitReached": false,
            "limitReasons": [],
            "summary": {
                "totalFindings": 0,
                "suppressedFindings": 0,
                "effectiveFindings": 0,
                "severityCounts": {"Low": 0, "Medium": 0, "High": 0, "Critical": 0},
                "allSeverityCounts": {"Low": 0, "Medium": 0, "High": 0, "Critical": 0},
                "suppressedSeverityCounts": {"Low": 0, "Medium": 0, "High": 0, "Critical": 0},
                "coverageComplete": true
            }
        });
        if let Some(extra) = result_extra.as_object() {
            result.as_object_mut().unwrap().extend(extra.clone());
        }

        serde_json::json!({
            "schemaVersion": "veil-pro-run-meta-v1",
            "runId": "run-test",
            "generatedAtUtc": "2026-06-29T00:00:00Z",
            "product": {
                "name": "veil-pro",
                "version": "0.17.0"
            },
            "engine": {
                "name": "veil",
                "schemaVersion": "veil-v1",
                "rulePacks": [{"name": "default", "source": "embedded"}]
            },
            "result": result,
            "artifacts": {
                "reportHtml": {"path": "report.html", "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
                "reportJson": {"path": "report.json", "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"},
                "effectiveConfig": {"path": "effective_config.toml", "sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"}
            },
            "privacy": {
                "telemetry": "none",
                "networkMode": "local-only",
                "bind": "127.0.0.1"
            }
        })
    }

    #[test]
    fn run_meta_result_requires_limit_reasons() {
        let mut value = run_meta_json(serde_json::json!({}));
        value["result"]
            .as_object_mut()
            .unwrap()
            .remove("limitReasons");

        let decoded = serde_json::from_value::<RunMetaV1>(value);

        assert!(decoded.is_err());
    }

    #[test]
    fn run_meta_result_rejects_unknown_keys() {
        let value = run_meta_json(serde_json::json!({"unexpected": true}));

        let decoded = serde_json::from_value::<RunMetaV1>(value);

        assert!(decoded.is_err());
    }
}
