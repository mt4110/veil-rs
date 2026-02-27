use crate::api::SafeFinding;
use crate::evidence::*;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

pub fn generate_evidence_pack(
    config: &veil_config::Config,
    findings: &[SafeFinding],
    scanned_files: usize,
    skipped_files: usize,
    duration_ms: u64,
    has_baseline: bool,
    baseline_content: Option<String>,
) -> (RunMeta, CachedRun) {
    let run_id = Uuid::new_v4().to_string();
    let generated_at = Utc::now().to_rfc3339();

    // 1. Generate report.html
    let html_content = generate_html_report(findings, scanned_files, skipped_files);
    let html_sha = sha256_str(&html_content);

    // 2. Generate report.json
    let json_content = generate_json_report(findings, scanned_files, skipped_files);
    let json_sha = sha256_str(&json_content);

    // 3. Generate effective_config.toml
    let config_content = toml::to_string_pretty(config).unwrap_or_default();
    let config_sha = sha256_str(&config_content);

    // 4. Handle Baseline
    let baseline_meta = if has_baseline && baseline_content.is_some() {
        let text = baseline_content.clone().unwrap();
        Some(ArtifactFileMeta {
            path: "baseline.json".to_string(),
            sha256: sha256_str(&text),
        })
    } else {
        None
    };

    let mut severity_counts = HashMap::new();
    severity_counts.insert("Critical".to_string(), 0);
    severity_counts.insert("High".to_string(), 0);
    severity_counts.insert("Medium".to_string(), 0);
    severity_counts.insert("Low".to_string(), 0);

    for f in findings {
        let label = match f.severity {
            veil_core::Severity::Critical => "Critical",
            veil_core::Severity::High => "High",
            veil_core::Severity::Medium => "Medium",
            veil_core::Severity::Low => "Low",
        };
        *severity_counts.entry(label.to_string()).or_insert(0) += 1;
    }

    let meta = RunMeta {
        schema_version: "veil-pro-run-meta-v1".to_string(),
        run_id: run_id.clone(),
        generated_at_utc: generated_at.clone(),
        product: ProductMeta {
            name: "veil-pro".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            commit: "unknown".to_string(), // could parse git
            build_profile: "release".to_string(),
        },
        engine: EngineMeta {
            name: "veil".to_string(),
            schema_version: "veil-v1".to_string(),
            rule_packs: vec![RulePackMeta {
                name: "default".to_string(),
                source: "embedded".to_string(),
                content_sha256: "unknown".to_string(),
            }],
        },
        project: ProjectMeta {
            display_name: std::env::current_dir()
                .map(|p| {
                    p.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                })
                .unwrap_or_default(),
            root_hint: ".".to_string(),
            git: ProjectGitMeta {
                head: "unknown".to_string(),
                dirty: false,
            },
        },
        invocation: InvocationMeta {
            mode: "ui".to_string(),
            targets: vec![".".to_string()],
            config: InvocationConfigMeta {
                repo_config: "veil.toml".to_string(),
                ci_config: "veil.ci.toml".to_string(),
                org_config_env: "VEIL_ORG_CONFIG".to_string(),
                org_config_path: std::env::var("VEIL_ORG_CONFIG").unwrap_or_default(),
            },
            baseline: InvocationBaselineMeta {
                enabled: has_baseline,
                file: if has_baseline {
                    ".veil-baseline.json".to_string()
                } else {
                    "".to_string()
                },
            },
        },
        limits: LimitsMeta {
            max_file_count: config.core.max_file_count.unwrap_or(20000),
            max_findings: config.output.max_findings.unwrap_or(1000),
            max_file_size_bytes: config.core.max_file_size.unwrap_or(10_000_000) as usize,
        },
        scope: ScopeMeta {
            gitignore_respected: true,
            built_in_excluded_dirs: config.core.ignore.clone(),
            binary_skip: true,
            oversize_skip: true,
        },
        result: ResultMeta {
            status: if findings.is_empty() {
                "success".to_string()
            } else {
                "violation".to_string()
            },
            exit_code: if findings.is_empty() { 0 } else { 1 },
            limit_reached: false, // Need actual status
            limit_reasons: vec![],
            summary: ResultSummaryMeta {
                scanned_files,
                skipped_files,
                findings_count: findings.len(),
                severity_counts,
            },
            timing: ResultTimingMeta {
                started_at_utc: generated_at,
                finished_at_utc: Utc::now().to_rfc3339(),
                duration_ms,
            },
        },
        artifacts: ArtifactsMeta {
            report_html: ArtifactFileMeta {
                path: "report.html".to_string(),
                sha256: html_sha,
            },
            report_json: ArtifactFileMeta {
                path: "report.json".to_string(),
                sha256: json_sha,
            },
            effective_config: ArtifactFileMeta {
                path: "effective_config.toml".to_string(),
                sha256: config_sha,
            },
            baseline: baseline_meta,
        },
        privacy: PrivacyMeta {
            telemetry: "none".to_string(),
            network: "local-only".to_string(),
            bind: "127.0.0.1".to_string(),
        },
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

#[derive(serde::Serialize)]
struct JsonReport<'a> {
    #[serde(rename = "schemaVersion")]
    schema_version: &'a str,
    summary: serde_json::Value,
    findings: &'a [SafeFinding],
}

fn generate_json_report(
    findings: &[SafeFinding],
    scanned_files: usize,
    skipped_files: usize,
) -> String {
    let report = JsonReport {
        schema_version: "veil-v1",
        summary: serde_json::json!({
            "scanned_files": scanned_files,
            "skipped_files": skipped_files,
            "total_findings": findings.len(),
        }),
        findings,
    };
    serde_json::to_string_pretty(&report).unwrap_or_default()
}

fn generate_html_report(
    findings: &[SafeFinding],
    _scanned_files: usize,
    _skipped_files: usize,
) -> String {
    // simplified beautiful html generator similar to veil-cli
    let mut top_rules = HashMap::new();
    for f in findings {
        *top_rules.entry(f.rule_id.clone()).or_insert(0) += 1;
    }

    let rows = findings
        .iter()
        .map(|f| {
            format!(
                r#"<tr class="finding-row">
            <td><span class="badge">{}</span></td>
            <td>{}</td>
            <td>{}</td>
            <td class="mono">{}</td>
            <td>{}</td>
        </tr>"#,
                html_escape(&format!("{:?}", f.severity)),
                html_escape(&f.rule_id),
                html_escape(&f.path),
                html_escape(&f.masked_snippet),
                html_escape(&f.line_number.to_string())
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let template = format!(
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
    );

    template
}

fn html_escape(input: &str) -> String {
    input
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}
