use assert_cmd::cargo::cargo_bin_cmd;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;
use zip::write::FileOptions;
use zip::ZipWriter;

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn severity_counts() -> serde_json::Value {
    serde_json::json!({
        "Low": 0,
        "Medium": 0,
        "High": 0,
        "Critical": 0
    })
}

fn run_result(
    effective_findings: usize,
    limit_reached: bool,
    coverage_complete: bool,
) -> serde_json::Value {
    serde_json::json!({
        "status": if limit_reached { "incomplete" } else { "success" },
        "exitCode": if limit_reached { 2 } else { 0 },
        "limitReached": limit_reached,
        "limitReasons": if limit_reached {
            serde_json::json!(["result-limit"])
        } else {
            serde_json::json!([])
        },
        "summary": {
            "totalFindings": effective_findings,
            "suppressedFindings": 0,
            "effectiveFindings": effective_findings,
            "severityCounts": severity_counts(),
            "allSeverityCounts": severity_counts(),
            "suppressedSeverityCounts": severity_counts(),
            "coverageComplete": coverage_complete
        }
    })
}

fn evidence_report(effective_findings: usize, coverage_complete: bool) -> Vec<u8> {
    serde_json::json!({
        "schemaVersion": "veil-evidence-report-v1",
        "runId": "test-run",
        "generatedAtUtc": "2026-06-29T00:00:00Z",
        "summary": {
            "totalFindings": effective_findings,
            "suppressedFindings": 0,
            "effectiveFindings": effective_findings,
            "severityCounts": severity_counts(),
            "allSeverityCounts": severity_counts(),
            "suppressedSeverityCounts": severity_counts(),
            "coverageComplete": coverage_complete
        },
        "findings": []
    })
    .to_string()
    .into_bytes()
}

fn artifacts_json(
    effective_config_content: &[u8],
    report_json_content: &[u8],
    report_html_content: &[u8],
) -> serde_json::Value {
    serde_json::json!({
        "effectiveConfig": {
            "path": "effective_config.toml",
            "sha256": sha256_hex(effective_config_content),
            "sizeBytes": effective_config_content.len()
        },
        "reportJson": {
            "path": "report.json",
            "sha256": sha256_hex(report_json_content),
            "sizeBytes": report_json_content.len()
        },
        "reportHtml": {
            "path": "report.html",
            "sha256": sha256_hex(report_html_content),
            "sizeBytes": report_html_content.len()
        }
    })
}

fn run_meta_json(artifacts: serde_json::Value, result: serde_json::Value) -> String {
    serde_json::json!({
        "schemaVersion": "veil-pro-run-meta-v1",
        "runId": "test-run",
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
        "artifacts": artifacts,
        "privacy": {
            "telemetry": "none",
            "networkMode": "local-only",
            "bind": "127.0.0.1"
        }
    })
    .to_string()
}

fn create_golden_zip(dir: &TempDir) -> std::path::PathBuf {
    create_zip_with_effective_findings(dir, 0)
}

fn create_zip_with_effective_findings(
    dir: &TempDir,
    effective_findings: usize,
) -> std::path::PathBuf {
    let zip_path = dir.path().join("golden_evidence.zip");
    let file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);

    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // Create artifacts
    let effective_config_content = b"rules = []";
    let report_json_content = evidence_report(effective_findings, true);
    let report_html_content = b"<html></html>";

    let artifacts = artifacts_json(
        effective_config_content,
        &report_json_content,
        report_html_content,
    );
    let run_meta_content = run_meta_json(artifacts, run_result(effective_findings, false, true));

    zip.start_file("run_meta.json", options).unwrap();
    zip.write_all(run_meta_content.as_bytes()).unwrap();

    zip.start_file("effective_config.toml", options).unwrap();
    zip.write_all(effective_config_content).unwrap();

    zip.start_file("report.json", options).unwrap();
    zip.write_all(&report_json_content).unwrap();

    zip.start_file("report.html", options).unwrap();
    zip.write_all(report_html_content).unwrap();

    zip.finish().unwrap();
    zip_path
}

#[test]
fn test_golden_zip() {
    let dir = TempDir::new().unwrap();
    let zip_path = create_golden_zip(&dir);

    let mut cmd = cargo_bin_cmd!("veil");
    cmd.arg("verify").arg(&zip_path);

    cmd.assert()
        .success()
        .code(0) // Exit 0 for valid
        .stdout(predicates::str::contains("PASSED"));
}

#[test]
fn test_fail_on_findings_zero_is_rejected() {
    let dir = TempDir::new().unwrap();
    let zip_path = create_golden_zip(&dir);

    let mut cmd = cargo_bin_cmd!("veil");
    cmd.arg("verify")
        .arg(&zip_path)
        .arg("--fail-on-findings")
        .arg("0");

    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("--fail-on-findings must be >= 1"));
}

#[test]
fn test_fail_on_findings_fails_at_threshold() {
    let dir = TempDir::new().unwrap();
    let zip_path = create_zip_with_effective_findings(&dir, 1);

    let mut cmd = cargo_bin_cmd!("veil");
    cmd.arg("verify")
        .arg(&zip_path)
        .arg("--fail-on-findings")
        .arg("1");

    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicates::str::contains("meeting or exceeding"));
}

#[test]
fn test_hash_mismatch() {
    let dir = TempDir::new().unwrap();
    let zip_path = dir.path().join("mismatch_evidence.zip");
    let file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);

    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

    let malicious_report = b"{\"findings\": [\"hacked!\"]}";

    // We register the hash for harmless report to simulate tampering
    let harmless_report = evidence_report(0, true);

    let artifacts = artifacts_json(b"rules = []", &harmless_report, b"<html></html>");
    let run_meta_content = run_meta_json(artifacts, run_result(0, false, true));

    zip.start_file("run_meta.json", options).unwrap();
    zip.write_all(run_meta_content.as_bytes()).unwrap();
    zip.start_file("effective_config.toml", options).unwrap();
    zip.write_all(b"rules = []").unwrap();
    zip.start_file("report.html", options).unwrap();
    zip.write_all(b"<html></html>").unwrap();

    // Write MALICIOUS report
    zip.start_file("report.json", options).unwrap();
    zip.write_all(malicious_report).unwrap();
    zip.finish().unwrap();

    let mut cmd = cargo_bin_cmd!("veil");
    cmd.arg("verify").arg(&zip_path);

    cmd.assert()
        .failure()
        .code(2) // Exit 2 for dangerous/corrupt
        .stdout(predicates::str::contains("Hash mismatch"));
}

#[test]
fn test_declared_baseline_must_exist_in_zip() {
    let dir = TempDir::new().unwrap();
    let zip_path = dir.path().join("missing_baseline_evidence.zip");
    let file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

    let effective_config_content = b"rules = []";
    let report_json_content = evidence_report(0, true);
    let report_html_content = b"<html></html>";
    let baseline_content = b"{\"schema\":\"veil.baseline.v1\",\"entries\":[]}";

    let mut artifacts = artifacts_json(
        effective_config_content,
        &report_json_content,
        report_html_content,
    );
    artifacts.as_object_mut().unwrap().insert(
        "baseline".to_string(),
        serde_json::json!({
            "path": "veil.baseline.json",
            "sha256": sha256_hex(baseline_content),
        }),
    );

    let run_meta_content = run_meta_json(artifacts, run_result(0, false, true));

    zip.start_file("run_meta.json", options).unwrap();
    zip.write_all(run_meta_content.as_bytes()).unwrap();
    zip.start_file("effective_config.toml", options).unwrap();
    zip.write_all(effective_config_content).unwrap();
    zip.start_file("report.json", options).unwrap();
    zip.write_all(&report_json_content).unwrap();
    zip.start_file("report.html", options).unwrap();
    zip.write_all(report_html_content).unwrap();
    zip.finish().unwrap();

    let mut cmd = cargo_bin_cmd!("veil");
    cmd.arg("verify").arg(&zip_path);

    cmd.assert()
        .failure()
        .code(2)
        .stdout(predicates::str::contains(
            "Missing required file: veil.baseline.json",
        ));
}

#[test]
fn test_legacy_run_meta_schema_is_rejected() {
    let dir = TempDir::new().unwrap();
    let zip_path = dir.path().join("legacy_schema_evidence.zip");
    let file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

    let effective_config_content = b"rules = []";
    let report_json_content = b"{\"findings\": []}";
    let report_html_content = b"<html></html>";

    let artifacts = serde_json::json!({
        "effective_config": {
            "path": "effective_config.toml",
            "sha256": sha256_hex(effective_config_content),
        },
        "report_json": {
            "path": "report.json",
            "sha256": sha256_hex(report_json_content),
        },
        "report_html": {
            "path": "report.html",
            "sha256": sha256_hex(report_html_content),
        }
    });

    let run_meta_content = serde_json::json!({
        "schemaVersion": "veil-v1",
        "artifacts": artifacts
    })
    .to_string();

    zip.start_file("run_meta.json", options).unwrap();
    zip.write_all(run_meta_content.as_bytes()).unwrap();
    zip.start_file("effective_config.toml", options).unwrap();
    zip.write_all(effective_config_content).unwrap();
    zip.start_file("report.json", options).unwrap();
    zip.write_all(report_json_content).unwrap();
    zip.start_file("report.html", options).unwrap();
    zip.write_all(report_html_content).unwrap();
    zip.finish().unwrap();

    let mut cmd = cargo_bin_cmd!("veil");
    cmd.arg("verify").arg(&zip_path);

    cmd.assert()
        .failure()
        .code(2)
        .stdout(predicates::str::contains(
            "Unsupported run_meta.json schema: veil-v1",
        ));
}

#[test]
fn test_v1_run_meta_missing_result_is_rejected() {
    let dir = TempDir::new().unwrap();
    let zip_path = dir.path().join("missing_result_evidence.zip");
    let file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

    let effective_config_content = b"rules = []";
    let report_json_content = evidence_report(0, true);
    let report_html_content = b"<html></html>";
    let artifacts = artifacts_json(
        effective_config_content,
        &report_json_content,
        report_html_content,
    );
    let mut run_meta_content: serde_json::Value =
        serde_json::from_str(&run_meta_json(artifacts, run_result(0, false, true))).unwrap();
    run_meta_content.as_object_mut().unwrap().remove("result");
    let run_meta_content = run_meta_content.to_string();

    zip.start_file("run_meta.json", options).unwrap();
    zip.write_all(run_meta_content.as_bytes()).unwrap();
    zip.start_file("effective_config.toml", options).unwrap();
    zip.write_all(effective_config_content).unwrap();
    zip.start_file("report.json", options).unwrap();
    zip.write_all(&report_json_content).unwrap();
    zip.start_file("report.html", options).unwrap();
    zip.write_all(report_html_content).unwrap();
    zip.finish().unwrap();

    let mut cmd = cargo_bin_cmd!("veil");
    cmd.arg("verify").arg(&zip_path);

    cmd.assert()
        .failure()
        .code(2)
        .stdout(predicates::str::contains(
            "run_meta.json missing required field: result",
        ));
}

#[test]
fn test_v1_run_meta_missing_limit_reasons_is_rejected() {
    let dir = TempDir::new().unwrap();
    let zip_path = dir.path().join("missing_limit_reasons_evidence.zip");
    let file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

    let effective_config_content = b"rules = []";
    let report_json_content = evidence_report(0, true);
    let report_html_content = b"<html></html>";
    let artifacts = artifacts_json(
        effective_config_content,
        &report_json_content,
        report_html_content,
    );
    let mut result = run_result(0, false, true);
    result.as_object_mut().unwrap().remove("limitReasons");
    let run_meta_content = run_meta_json(artifacts, result);

    zip.start_file("run_meta.json", options).unwrap();
    zip.write_all(run_meta_content.as_bytes()).unwrap();
    zip.start_file("effective_config.toml", options).unwrap();
    zip.write_all(effective_config_content).unwrap();
    zip.start_file("report.json", options).unwrap();
    zip.write_all(&report_json_content).unwrap();
    zip.start_file("report.html", options).unwrap();
    zip.write_all(report_html_content).unwrap();
    zip.finish().unwrap();

    let mut cmd = cargo_bin_cmd!("veil");
    cmd.arg("verify").arg(&zip_path);

    cmd.assert()
        .failure()
        .code(2)
        .stdout(predicates::str::contains(
            "run_meta.json result missing required field: limitReasons",
        ));
}

#[test]
fn test_v1_run_meta_invalid_status_is_rejected() {
    let dir = TempDir::new().unwrap();
    let zip_path = dir.path().join("invalid_status_evidence.zip");
    let file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

    let effective_config_content = b"rules = []";
    let report_json_content = evidence_report(0, true);
    let report_html_content = b"<html></html>";
    let artifacts = artifacts_json(
        effective_config_content,
        &report_json_content,
        report_html_content,
    );
    let mut result = run_result(0, false, true);
    result.as_object_mut().unwrap().insert(
        "status".to_string(),
        serde_json::Value::String("passed".to_string()),
    );
    let run_meta_content = run_meta_json(artifacts, result);

    zip.start_file("run_meta.json", options).unwrap();
    zip.write_all(run_meta_content.as_bytes()).unwrap();
    zip.start_file("effective_config.toml", options).unwrap();
    zip.write_all(effective_config_content).unwrap();
    zip.start_file("report.json", options).unwrap();
    zip.write_all(&report_json_content).unwrap();
    zip.start_file("report.html", options).unwrap();
    zip.write_all(report_html_content).unwrap();
    zip.finish().unwrap();

    let mut cmd = cargo_bin_cmd!("veil");
    cmd.arg("verify").arg(&zip_path);

    cmd.assert()
        .failure()
        .code(2)
        .stdout(predicates::str::contains(
            "run_meta.json result.status must be one of success, violation, incomplete, error",
        ));
}

#[test]
fn test_v1_run_meta_missing_top_level_required_is_rejected() {
    let dir = TempDir::new().unwrap();
    let zip_path = dir.path().join("missing_run_id_evidence.zip");
    let file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

    let effective_config_content = b"rules = []";
    let report_json_content = evidence_report(0, true);
    let report_html_content = b"<html></html>";
    let artifacts = artifacts_json(
        effective_config_content,
        &report_json_content,
        report_html_content,
    );
    let mut run_meta_content: serde_json::Value =
        serde_json::from_str(&run_meta_json(artifacts, run_result(0, false, true))).unwrap();
    run_meta_content.as_object_mut().unwrap().remove("runId");
    let run_meta_content = run_meta_content.to_string();

    zip.start_file("run_meta.json", options).unwrap();
    zip.write_all(run_meta_content.as_bytes()).unwrap();
    zip.start_file("effective_config.toml", options).unwrap();
    zip.write_all(effective_config_content).unwrap();
    zip.start_file("report.json", options).unwrap();
    zip.write_all(&report_json_content).unwrap();
    zip.start_file("report.html", options).unwrap();
    zip.write_all(report_html_content).unwrap();
    zip.finish().unwrap();

    let mut cmd = cargo_bin_cmd!("veil");
    cmd.arg("verify").arg(&zip_path);

    cmd.assert()
        .failure()
        .code(2)
        .stdout(predicates::str::contains(
            "run_meta.json missing required field: runId",
        ));
}

#[test]
fn test_report_json_schema_is_rejected() {
    let dir = TempDir::new().unwrap();
    let zip_path = dir.path().join("invalid_report_evidence.zip");
    let file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

    let effective_config_content = b"rules = []";
    let report_json_content = b"{\"findings\": []}";
    let report_html_content = b"<html></html>";
    let artifacts = artifacts_json(
        effective_config_content,
        report_json_content,
        report_html_content,
    );
    let run_meta_content = run_meta_json(artifacts, run_result(0, false, true));

    zip.start_file("run_meta.json", options).unwrap();
    zip.write_all(run_meta_content.as_bytes()).unwrap();
    zip.start_file("effective_config.toml", options).unwrap();
    zip.write_all(effective_config_content).unwrap();
    zip.start_file("report.json", options).unwrap();
    zip.write_all(report_json_content).unwrap();
    zip.start_file("report.html", options).unwrap();
    zip.write_all(report_html_content).unwrap();
    zip.finish().unwrap();

    let mut cmd = cargo_bin_cmd!("veil");
    cmd.arg("verify").arg(&zip_path);

    cmd.assert()
        .failure()
        .code(2)
        .stdout(predicates::str::contains(
            "report.json missing required field: schemaVersion",
        ));
}

#[test]
fn test_baseline_artifact_path_must_be_canonical() {
    let dir = TempDir::new().unwrap();
    let zip_path = dir.path().join("custom_baseline_path_evidence.zip");
    let file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

    let effective_config_content = b"rules = []";
    let report_json_content = evidence_report(0, true);
    let report_html_content = b"<html></html>";
    let baseline_content = b"{\"schema\":\"veil.baseline.v1\",\"entries\":[]}";

    let mut artifacts = artifacts_json(
        effective_config_content,
        &report_json_content,
        report_html_content,
    );
    artifacts.as_object_mut().unwrap().insert(
        "baseline".to_string(),
        serde_json::json!({
            "path": "custom-baseline.json",
            "sha256": sha256_hex(baseline_content),
        }),
    );

    let run_meta_content = run_meta_json(artifacts, run_result(0, false, true));

    zip.start_file("run_meta.json", options).unwrap();
    zip.write_all(run_meta_content.as_bytes()).unwrap();
    zip.start_file("effective_config.toml", options).unwrap();
    zip.write_all(effective_config_content).unwrap();
    zip.start_file("report.json", options).unwrap();
    zip.write_all(&report_json_content).unwrap();
    zip.start_file("report.html", options).unwrap();
    zip.write_all(report_html_content).unwrap();
    zip.start_file("custom-baseline.json", options).unwrap();
    zip.write_all(baseline_content).unwrap();
    zip.finish().unwrap();

    let mut cmd = cargo_bin_cmd!("veil");
    cmd.arg("verify").arg(&zip_path);

    cmd.assert()
        .failure()
        .code(2)
        .stdout(predicates::str::contains(
            "artifact baseline path must be veil.baseline.json",
        ));
}

#[test]
fn test_external_anchor_mismatch() {
    let dir = TempDir::new().unwrap();
    let zip_path = create_golden_zip(&dir);

    let mut cmd = cargo_bin_cmd!("veil");
    cmd.arg("verify")
        .arg(&zip_path)
        .arg("--expect-run-meta-sha256")
        .arg("badf00dhash12345");

    cmd.assert()
        .failure()
        .code(2)
        .stdout(predicates::str::contains("Hash mismatch"));
}

#[test]
fn test_zip_slip() {
    let dir = TempDir::new().unwrap();
    let zip_path = dir.path().join("slip_evidence.zip");
    let file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

    // Write harmless valid structure
    let artifacts = serde_json::json!({
        "report_html": { "path": "report.html", "sha256": ".." },
        "report_json": { "path": "report.json", "sha256": ".." },
        "effective_config": { "path": "effective_config.toml", "sha256": ".." },
    });
    let run_meta = serde_json::json!({
        "schemaVersion": "veil-pro-run-meta-v1",
        "result": run_result(0, false, true),
        "artifacts": artifacts
    })
    .to_string();

    zip.start_file("run_meta.json", options).unwrap();
    zip.write_all(run_meta.as_bytes()).unwrap();
    zip.start_file("report.html", options).unwrap();
    zip.write_all(b"").unwrap();
    zip.start_file("report.json", options).unwrap();
    zip.write_all(b"").unwrap();
    zip.start_file("effective_config.toml", options).unwrap();
    zip.write_all(b"").unwrap();

    // DANGEROUS PATH INJECTION
    zip.start_file("../evil.txt", options).unwrap();
    zip.write_all(b"I overwrite your system_keys").unwrap();
    zip.finish().unwrap();

    let mut cmd = cargo_bin_cmd!("veil");
    cmd.arg("verify").arg(&zip_path);

    cmd.assert()
        .failure()
        .code(2)
        .stdout(predicates::str::contains("ZipSlip"));
}

#[test]
fn test_token_leakage() {
    let dir = TempDir::new().unwrap();
    let zip_path = dir.path().join("leak_evidence.zip");
    let file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

    let artifacts = serde_json::json!({
        "report_html": { "path": "report.html", "sha256": ".." },
        "report_json": { "path": "report.json", "sha256": ".." },
        "effective_config": { "path": "effective_config.toml", "sha256": ".." },
    });
    let run_meta = serde_json::json!({
        "schemaVersion": "veil-pro-run-meta-v1",
        "result": run_result(0, false, true),
        "artifacts": artifacts
    })
    .to_string();

    zip.start_file("run_meta.json", options).unwrap();
    zip.write_all(run_meta.as_bytes()).unwrap();
    zip.start_file("report.html", options).unwrap();
    // Inject leakage here
    zip.write_all(b"<html><script>window.location.hash='#token=secret123';</script></html>")
        .unwrap();
    zip.start_file("report.json", options).unwrap();
    zip.write_all(b"").unwrap();
    zip.start_file("effective_config.toml", options).unwrap();
    zip.write_all(b"").unwrap();
    zip.finish().unwrap();

    let mut cmd = cargo_bin_cmd!("veil");
    cmd.arg("verify").arg(&zip_path);

    cmd.assert()
        .failure()
        .code(2)
        .stdout(predicates::str::contains("Token leakage detected"));
}

#[test]
fn test_require_complete_fail() {
    let dir = TempDir::new().unwrap();
    let zip_path = dir.path().join("inc_evidence.zip");
    let file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

    let rule_config = b"";
    let json_rep = evidence_report(5, false);
    let html_rep = b"";

    let artifacts = artifacts_json(rule_config, &json_rep, html_rep);

    let run_meta = run_meta_json(artifacts, run_result(5, true, false));

    zip.start_file("run_meta.json", options).unwrap();
    zip.write_all(run_meta.as_bytes()).unwrap();
    zip.start_file("report.html", options).unwrap();
    zip.write_all(html_rep).unwrap();
    zip.start_file("report.json", options).unwrap();
    zip.write_all(&json_rep).unwrap();
    zip.start_file("effective_config.toml", options).unwrap();
    zip.write_all(rule_config).unwrap();
    zip.finish().unwrap();

    let mut cmd = cargo_bin_cmd!("veil");
    cmd.arg("verify").arg(&zip_path).arg("--require-complete");

    cmd.assert()
        .failure()
        .code(1) // EXIT 1 for Policy Violation
        .stdout(predicates::str::contains("POLICY VIOLATION"));
}

#[test]
fn test_require_complete_fails_when_limit_reached_contradicts_summary() {
    let dir = TempDir::new().unwrap();
    let zip_path = dir.path().join("contradictory_evidence.zip");
    let file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

    let rule_config = b"";
    let json_rep = evidence_report(0, true);
    let html_rep = b"";

    let artifacts = artifacts_json(rule_config, &json_rep, html_rep);

    let run_meta = run_meta_json(artifacts, run_result(0, true, true));

    zip.start_file("run_meta.json", options).unwrap();
    zip.write_all(run_meta.as_bytes()).unwrap();
    zip.start_file("report.html", options).unwrap();
    zip.write_all(html_rep).unwrap();
    zip.start_file("report.json", options).unwrap();
    zip.write_all(&json_rep).unwrap();
    zip.start_file("effective_config.toml", options).unwrap();
    zip.write_all(rule_config).unwrap();
    zip.finish().unwrap();

    let mut cmd = cargo_bin_cmd!("veil");
    cmd.arg("verify").arg(&zip_path).arg("--require-complete");

    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicates::str::contains("POLICY VIOLATION"));
}

// Note: test_duplicate_entries is intentionally omitted.
// The zip 5.x writer API structurally prevents writing duplicate filenames
// (it panics before the ZIP is created). The duplicate-entry runtime guard
// in verify.rs remains as defense-in-depth for hand-crafted or legacy ZIPs.
