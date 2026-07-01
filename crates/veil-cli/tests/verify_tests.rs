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
    let report_json_content = b"{\"findings\": []}";
    let report_html_content = b"<html></html>";

    let artifacts = serde_json::json!({
        "effective_config": {
            "path": "effective_config.toml",
            "sha256": sha256_hex(effective_config_content),
            "size_bytes": effective_config_content.len()
        },
        "report_json": {
            "path": "report.json",
            "sha256": sha256_hex(report_json_content),
            "size_bytes": report_json_content.len()
        },
        "report_html": {
            "path": "report.html",
            "sha256": sha256_hex(report_html_content),
            "size_bytes": report_html_content.len()
        }
    });

    let run_meta_content = serde_json::json!({
        "schemaVersion": "veil-pro-run-meta-v1",
        "result": {
            "limit_reached": false,
            "summary": {
                "effectiveFindings": effective_findings,
                "totalFindings": effective_findings,
                "findings_count": effective_findings
            }
        },
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
    let harmless_report = b"{\"findings\": []}";

    let artifacts = serde_json::json!({
        "effective_config": {
            "path": "effective_config.toml",
            "sha256": sha256_hex(b"rules = []"),
        },
        "report_json": {
            "path": "report.json",
            "sha256": sha256_hex(harmless_report), // Hash doesn't match the malicious file injected!
        },
        "report_html": {
            "path": "report.html",
            "sha256": sha256_hex(b"<html></html>"),
        }
    });

    let run_meta_content = serde_json::json!({
        "schemaVersion": "veil-pro-run-meta-v1",
        "artifacts": artifacts
    })
    .to_string();

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
    let report_json_content = b"{\"findings\": []}";
    let report_html_content = b"<html></html>";
    let baseline_content = b"{\"schema\":\"veil.baseline.v1\",\"entries\":[]}";

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
        },
        "baseline": {
            "path": "veil.baseline.json",
            "sha256": sha256_hex(baseline_content),
        }
    });

    let run_meta_content = serde_json::json!({
        "schemaVersion": "veil-pro-run-meta-v1",
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
fn test_baseline_artifact_path_must_be_canonical() {
    let dir = TempDir::new().unwrap();
    let zip_path = dir.path().join("custom_baseline_path_evidence.zip");
    let file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

    let effective_config_content = b"rules = []";
    let report_json_content = b"{\"findings\": []}";
    let report_html_content = b"<html></html>";
    let baseline_content = b"{\"schema\":\"veil.baseline.v1\",\"entries\":[]}";

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
        },
        "baseline": {
            "path": "custom-baseline.json",
            "sha256": sha256_hex(baseline_content),
        }
    });

    let run_meta_content = serde_json::json!({
        "schemaVersion": "veil-pro-run-meta-v1",
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
    let run_meta =
        serde_json::json!({ "schemaVersion": "veil-pro-run-meta-v1", "artifacts": artifacts })
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
    let run_meta =
        serde_json::json!({ "schemaVersion": "veil-pro-run-meta-v1", "artifacts": artifacts })
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
    let json_rep = b"";
    let html_rep = b"";

    let artifacts = serde_json::json!({
        "effective_config": { "path": "effective_config.toml", "sha256": sha256_hex(rule_config) },
        "report_json": { "path": "report.json", "sha256": sha256_hex(json_rep) },
        "report_html": { "path": "report.html", "sha256": sha256_hex(html_rep) }
    });

    let run_meta = serde_json::json!({
        "schemaVersion": "veil-pro-run-meta-v1",
        "result": {
            "limit_reached": true, // <--- IMPORTANT FLAG
            "summary": {
                "findings_count": 5
            }
        },
        "artifacts": artifacts
    })
    .to_string();

    zip.start_file("run_meta.json", options).unwrap();
    zip.write_all(run_meta.as_bytes()).unwrap();
    zip.start_file("report.html", options).unwrap();
    zip.write_all(html_rep).unwrap();
    zip.start_file("report.json", options).unwrap();
    zip.write_all(json_rep).unwrap();
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
    let json_rep = b"";
    let html_rep = b"";

    let artifacts = serde_json::json!({
        "effective_config": { "path": "effective_config.toml", "sha256": sha256_hex(rule_config) },
        "report_json": { "path": "report.json", "sha256": sha256_hex(json_rep) },
        "report_html": { "path": "report.html", "sha256": sha256_hex(html_rep) }
    });

    let run_meta = serde_json::json!({
        "schemaVersion": "veil-pro-run-meta-v1",
        "result": {
            "limitReached": true,
            "summary": {
                "coverageComplete": true,
                "effectiveFindings": 0
            }
        },
        "artifacts": artifacts
    })
    .to_string();

    zip.start_file("run_meta.json", options).unwrap();
    zip.write_all(run_meta.as_bytes()).unwrap();
    zip.start_file("report.html", options).unwrap();
    zip.write_all(html_rep).unwrap();
    zip.start_file("report.json", options).unwrap();
    zip.write_all(json_rep).unwrap();
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
