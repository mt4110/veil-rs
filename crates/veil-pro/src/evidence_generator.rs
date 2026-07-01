use crate::api::dto::{
    ArtifactMeta, BaselineArtifactMeta, BaselineArtifactPath, BindAddress, BuildProfile,
    EngineMeta, EngineName, EngineSchemaVersion, EvidenceArtifacts, EvidenceReportSchemaVersion,
    EvidenceReportV1, EvidenceSummary, NetworkMode, PrivacyMeta, ProductMeta, ProductName,
    RulePackMeta, RulePackSource, RunMetaSchemaVersion, RunMetaV1, RunResultMeta, RunStatus,
    SafeFindingApiV1, TelemetryMode,
};
use crate::evidence::CachedRun;
use chrono::Utc;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub fn generate_evidence_pack(
    config: &veil_config::Config,
    findings: &[SafeFindingApiV1],
    summary: EvidenceSummary,
    status: RunStatus,
    limit_reached: bool,
    limit_reasons: Vec<String>,
    scanned_files: usize,
    skipped_files: usize,
    _duration_ms: u64,
    baseline_content: Option<String>,
) -> (RunMetaV1, CachedRun) {
    let run_id = Uuid::new_v4().to_string();
    let generated_at = Utc::now().to_rfc3339();

    let html_content = generate_html_report(findings, scanned_files, skipped_files);
    let html_sha = sha256_str(&html_content);

    let report = EvidenceReportV1 {
        schema_version: EvidenceReportSchemaVersion::VeilEvidenceReportV1,
        run_id: run_id.clone(),
        generated_at_utc: generated_at.clone(),
        summary: summary.clone(),
        findings: findings.to_vec(),
    };
    let json_content =
        serde_json::to_string_pretty(&report).expect("failed to serialize evidence report.json");
    let json_sha = sha256_str(&json_content);

    let config_content = sanitized_effective_config_toml(config);
    let config_sha = sha256_str(&config_content);

    let baseline_meta = baseline_content.as_ref().map(|text| BaselineArtifactMeta {
        path: BaselineArtifactPath::VeilBaselineJson,
        sha256: sha256_str(text),
        size_bytes: Some(text.len()),
    });

    let meta = RunMetaV1 {
        schema_version: RunMetaSchemaVersion::VeilProRunMetaV1,
        run_id: run_id.clone(),
        generated_at_utc: generated_at,
        product: ProductMeta {
            name: ProductName::VeilPro,
            version: env!("CARGO_PKG_VERSION").to_string(),
            commit: None,
            build_profile: Some(if cfg!(debug_assertions) {
                BuildProfile::Debug
            } else {
                BuildProfile::Release
            }),
        },
        engine: EngineMeta {
            name: EngineName::Veil,
            schema_version: EngineSchemaVersion::VeilV1,
            rule_packs: vec![RulePackMeta {
                name: "default".to_string(),
                source: RulePackSource::Embedded,
                content_sha256: None,
                version: None,
            }],
        },
        result: RunResultMeta {
            status,
            exit_code: match status {
                RunStatus::Success => 0,
                RunStatus::Violation => 1,
                RunStatus::Incomplete | RunStatus::Error => 2,
            },
            limit_reached,
            limit_reasons,
            summary,
        },
        artifacts: EvidenceArtifacts {
            report_html: ArtifactMeta {
                path: "report.html".to_string(),
                sha256: html_sha,
                size_bytes: Some(html_content.len()),
            },
            report_json: ArtifactMeta {
                path: "report.json".to_string(),
                sha256: json_sha,
                size_bytes: Some(json_content.len()),
            },
            effective_config: ArtifactMeta {
                path: "effective_config.toml".to_string(),
                sha256: config_sha,
                size_bytes: Some(config_content.len()),
            },
            baseline: baseline_meta,
        },
        privacy: PrivacyMeta {
            telemetry: TelemetryMode::None,
            network_mode: NetworkMode::LocalOnly,
            bind: BindAddress::LocalhostV4,
        },
        extensions: None,
    };

    let cached = CachedRun {
        meta: meta.clone(),
        report_html: html_content,
        report_json: json_content,
        effective_config: config_content,
        baseline_json: baseline_content,
        timestamp: std::time::Instant::now(),
    };

    (meta, cached)
}

fn sha256_str(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn sanitized_effective_config_toml(config: &veil_config::Config) -> String {
    let mut value =
        toml::Value::try_from(config).expect("failed to convert effective config to TOML value");
    redact_toml_value(None, &mut value);
    toml::to_string_pretty(&value).expect("failed to serialize effective_config.toml")
}

fn redact_toml_value(key: Option<&str>, value: &mut toml::Value) {
    match value {
        toml::Value::String(text)
            if key.is_some_and(is_sensitive_config_key) || contains_secret_marker(text) =>
        {
            *text = "<REDACTED>".to_string();
        }
        toml::Value::Array(values) => {
            for value in values {
                redact_toml_value(key, value);
            }
        }
        toml::Value::Table(table) => {
            for (key, value) in table.iter_mut() {
                redact_toml_value(Some(key), value);
            }
        }
        _ => {}
    }
}

fn is_sensitive_config_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.contains("password")
        || key.contains("secret")
        || key.contains("token")
        || key.contains("api_key")
        || key.contains("apikey")
        || key.contains("authorization")
        || key.contains("private_key")
        || key.contains("access_key")
}

fn contains_secret_marker(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    [
        "?token=",
        "&token=",
        "#token=",
        "access_token=",
        "api_key=",
        "apikey=",
        "client_secret=",
        "password=",
        "secret=",
        "authorization: bearer ",
    ]
    .iter()
    .any(|marker| value.contains(marker))
        || value.trim_start().starts_with("bearer ")
}

fn generate_html_report(
    findings: &[SafeFindingApiV1],
    _scanned_files: usize,
    _skipped_files: usize,
) -> String {
    let rows = findings
        .iter()
        .map(|finding| {
            format!(
                r#"<tr class="finding-row">
            <td><span class="badge">{}</span></td>
            <td>{}</td>
            <td>{}</td>
            <td class="mono">{}</td>
            <td>{}</td>
        </tr>"#,
                html_escape(&format!("{:?}", finding.severity)),
                html_escape(&finding.rule_id),
                html_escape(&finding.path),
                html_escape(&finding.masked_snippet),
                html_escape(&finding.line_number.to_string())
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Veil Security Report</title>
    <style>
        body {{ font-family: sans-serif; background: #f8f9fa; color: #333; padding: 2rem; }}
        table {{ width: 100%; border-collapse: collapse; background: white; margin-top: 2rem; }}
        th, td {{ padding: 1rem; text-align: left; border-bottom: 1px solid #ddd; }}
        .badge {{ background: #4c51bf; color: white; padding: 0.2rem 0.5rem; border-radius: 9999px; font-size: 0.8em; }}
        .mono {{ font-family: monospace; background: #eee; padding: 0.2rem; border-radius: 4px; }}
    </style>
</head>
<body>
    <h1>Veil Security Report</h1>
    <p>Total Findings: {}</p>
    <table>
        <thead><tr><th>Severity</th><th>Rule ID</th><th>File</th><th>Match Content</th><th>Line</th></tr></thead>
        <tbody>{}</tbody>
    </table>
</body>
</html>"#,
        html_escape(&findings.len().to_string()),
        rows
    )
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::dto::{BaselineStatus, GradeName, SeverityCounts, SeverityName};

    fn finding(id: &str, status: BaselineStatus) -> SafeFindingApiV1 {
        SafeFindingApiV1 {
            finding_id: id.to_string(),
            baseline_fingerprint: format!("sha256:{:0>64}", id.len()),
            path: "src/config.rs".to_string(),
            line_number: 1,
            rule_id: "creds.test".to_string(),
            severity: SeverityName::High,
            score: 80,
            grade: GradeName::High,
            masked_snippet: "token = <REDACTED>".to_string(),
            category: "credentials".to_string(),
            tags: vec!["secret".to_string()],
            baseline_status: status,
        }
    }

    #[test]
    fn evidence_report_contains_raw_free_all_findings() {
        let findings = vec![
            finding("new", BaselineStatus::New),
            finding("suppressed", BaselineStatus::Suppressed),
        ];
        let summary = EvidenceSummary {
            total_findings: 2,
            suppressed_findings: 1,
            effective_findings: 1,
            severity_counts: SeverityCounts {
                low: 0,
                medium: 0,
                high: 1,
                critical: 0,
            },
            all_severity_counts: SeverityCounts {
                low: 0,
                medium: 0,
                high: 2,
                critical: 0,
            },
            suppressed_severity_counts: SeverityCounts {
                low: 0,
                medium: 0,
                high: 1,
                critical: 0,
            },
            coverage_complete: true,
        };

        let (_meta, cached) = generate_evidence_pack(
            &veil_config::Config::default(),
            &findings,
            summary,
            RunStatus::Violation,
            false,
            Vec::new(),
            1,
            0,
            1,
            None,
        );
        let report: serde_json::Value = serde_json::from_str(&cached.report_json).unwrap();

        assert_eq!(report["schemaVersion"], "veil-evidence-report-v1");
        assert_eq!(report["findings"].as_array().unwrap().len(), 2);
        assert_eq!(report["findings"][0]["baselineStatus"], "new");
        assert_eq!(report["findings"][1]["baselineStatus"], "suppressed");
        assert!(!report.to_string().contains("matched_content"));
        assert!(!report.to_string().contains("line_content"));
    }

    #[test]
    fn effective_config_redacts_token_bearing_values() {
        let token = "sensitive_token_leakage_test_123456789";
        let mut config = veil_config::Config::default();
        config.core.remote_rules_url = Some(format!("https://example.test/rules?token={token}"));
        let summary = EvidenceSummary {
            total_findings: 0,
            suppressed_findings: 0,
            effective_findings: 0,
            severity_counts: SeverityCounts::zero(),
            all_severity_counts: SeverityCounts::zero(),
            suppressed_severity_counts: SeverityCounts::zero(),
            coverage_complete: true,
        };

        let (meta, cached) = generate_evidence_pack(
            &config,
            &[],
            summary,
            RunStatus::Success,
            false,
            Vec::new(),
            1,
            0,
            1,
            None,
        );

        assert!(!cached.effective_config.contains(token));
        assert!(!cached.effective_config.contains("?token="));
        assert!(cached.effective_config.contains("remote_rules_url"));
        assert!(cached.effective_config.contains("<REDACTED>"));
        assert_eq!(
            meta.artifacts.effective_config.sha256,
            sha256_str(&cached.effective_config)
        );
    }
}
