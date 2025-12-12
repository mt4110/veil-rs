use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Veil Summary Schema v1
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SummaryV1 {
    pub schema: String,
    pub generated_at: String,
    pub tool: ToolInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<RunInfo>,
    pub target: TargetInfo,
    pub scan: ScanInfo,
    pub counts: Counts,
    pub breakdown: Breakdown,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_new_findings: Option<Vec<TopFinding>>,
    pub limits: Limits,
    pub extensions: Value,
}

impl Default for SummaryV1 {
    fn default() -> Self {
        Self {
            schema: "veil.summary.v1".to_string(),
            generated_at: String::new(),
            tool: ToolInfo::default(),
            run: None,
            target: TargetInfo::default(),
            scan: ScanInfo::default(),
            counts: Counts::default(),
            breakdown: Breakdown::default(),
            top_new_findings: None,
            limits: Limits::default(),
            extensions: Value::Object(serde_json::Map::new()),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    pub ruleset_digest: String,
    pub config_digest: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint_key_digest: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunInfo {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attempt: Option<u64>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetInfo {
    pub kind: TargetKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subpath: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetKind {
    Filesystem,
    Git,
}

impl Default for TargetKind {
    fn default() -> Self {
        Self::Filesystem
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScanInfo {
    pub mode: ScanMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<BaselineInfo>,
    pub duration_ms: u64,
    pub files_scanned: u64,
    pub bytes_scanned: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanMode {
    Standard,
    Baseline,
    History,
}

impl Default for ScanMode {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaselineInfo {
    pub path: String,
    pub digest: String,
    pub new_findings: u64,
    pub baselined_findings: u64,
    pub resolved_findings: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Counts {
    pub total: u64,
    pub blocking: u64,
    pub by_severity: SeverityCounts,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeverityCounts {
    pub low: u64,
    pub medium: u64,
    pub high: u64,
    pub critical: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Breakdown {
    pub by_rule: Vec<RuleBreakdown>,
    pub by_path_prefix: Vec<PathBreakdown>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleBreakdown {
    pub rule_id: String,
    pub severity: SummarySeverity,
    pub total: u64,
    pub new: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathBreakdown {
    pub prefix: String,
    pub total: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SummarySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for SummarySeverity {
    fn default() -> Self {
        Self::Low
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopFinding {
    pub fingerprint: String,
    pub rule_id: String,
    pub severity: SummarySeverity,
    pub path: String,
    pub line_start: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<FindingOrigin>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FindingOrigin {
    pub commit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub top_new_findings_truncated: bool,
    pub max_top_new_findings: u64,
    pub top_new_findings_bytes: u64,
    pub max_top_new_findings_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_summary_serialization() {
        // Create a summary matching the example
        let summary = SummaryV1 {
            schema: "veil.summary.v1".to_string(),
            generated_at: "2023-10-27T10:00:00Z".to_string(),
            tool: ToolInfo {
                name: "veil".to_string(),
                version: "0.9.3".to_string(),
                git_commit: Some("1234567890abcdef".to_string()),
                ruleset_digest:
                    "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                        .to_string(),
                config_digest:
                    "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                        .to_string(),
                fingerprint_key_digest: Some(
                    "sha256:a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e"
                        .to_string(),
                ),
            },
            run: Some(RunInfo {
                id: "123456789".to_string(),
                ci_provider: Some("github-actions".to_string()),
                workflow: Some("security-scan".to_string()),
                job: Some("scan".to_string()),
                attempt: Some(1),
            }),
            target: TargetInfo {
                kind: TargetKind::Filesystem,
                repo: Some("https://github.com/mt4110/veil-rs".to_string()),
                r#ref: Some("refs/heads/main".to_string()),
                commit: Some("1234567890abcdef".to_string()),
                range: None,
                subpath: None,
            },
            scan: ScanInfo {
                mode: ScanMode::Baseline,
                duration_ms: 150,
                files_scanned: 120,
                bytes_scanned: 1024000,
                baseline: Some(BaselineInfo {
                    path: "veil.baseline.json".to_string(),
                    digest:
                        "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                            .to_string(),
                    new_findings: 1,
                    baselined_findings: 5,
                    resolved_findings: 0,
                }),
            },
            counts: Counts {
                total: 6,
                blocking: 1,
                by_severity: SeverityCounts {
                    low: 0,
                    medium: 0,
                    high: 1,
                    critical: 0,
                },
            },
            breakdown: Breakdown {
                by_rule: vec![
                    RuleBreakdown {
                        rule_id: "aws_access_key".to_string(),
                        severity: SummarySeverity::High,
                        total: 1,
                        new: 1,
                    },
                    RuleBreakdown {
                        rule_id: "slack_token".to_string(),
                        severity: SummarySeverity::Medium,
                        total: 5,
                        new: 0,
                    },
                ],
                by_path_prefix: vec![
                    PathBreakdown {
                        prefix: "src/".to_string(),
                        total: 5,
                        new: Some(1),
                    },
                    PathBreakdown {
                        prefix: "tests/".to_string(),
                        total: 1,
                        new: Some(0),
                    },
                ],
            },
            top_new_findings: Some(vec![TopFinding {
                fingerprint:
                    "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                        .to_string(),
                rule_id: "aws_access_key".to_string(),
                severity: SummarySeverity::High,
                path: "src/main.rs".to_string(),
                line_start: 42,
                origin: Some(FindingOrigin {
                    commit: "1234567890abcdef".to_string(),
                    author: Some("dev@example.com".to_string()),
                }),
            }]),
            limits: Limits {
                top_new_findings_truncated: false,
                max_top_new_findings: 20,
                top_new_findings_bytes: 512,
                max_top_new_findings_bytes: 32768,
            },
            extensions: json!({
                "security": {
                    "fingerprint": {
                        "mode": "keyed",
                        "key_present": true,
                        "key_digest": "sha256:a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e"
                    }
                }
            }),
        };

        // Serialize and check against something we know is valid
        let json_output = serde_json::to_string_pretty(&summary).expect("Serialization failed");

        // In a real scenario, we might want to validate this against the schema file itself
        // for now just ensuring it serializes is a good first step.
        assert!(json_output.contains("veil.summary.v1"));
    }
}
