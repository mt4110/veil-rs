use assert_cmd::Command;
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
                "findings_count": 0
            }
        },
        "artifacts": artifacts
    })
    .to_string();

    zip.start_file("run_meta.json", options.clone()).unwrap();
    zip.write_all(run_meta_content.as_bytes()).unwrap();

    zip.start_file("effective_config.toml", options.clone())
        .unwrap();
    zip.write_all(effective_config_content).unwrap();

    zip.start_file("report.json", options.clone()).unwrap();
    zip.write_all(report_json_content).unwrap();

    zip.start_file("report.html", options.clone()).unwrap();
    zip.write_all(report_html_content).unwrap();

    zip.finish().unwrap();
    zip_path
}

#[test]
fn test_golden_zip() {
    let dir = TempDir::new().unwrap();
    let zip_path = create_golden_zip(&dir);

    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.arg("verify").arg(&zip_path);

    cmd.assert()
        .success()
        .code(0) // Exit 0 for valid
        .stdout(predicates::str::contains("PASSED"));
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

    zip.start_file("run_meta.json", options.clone()).unwrap();
    zip.write_all(run_meta_content.as_bytes()).unwrap();
    zip.start_file("effective_config.toml", options.clone())
        .unwrap();
    zip.write_all(b"rules = []").unwrap();
    zip.start_file("report.html", options.clone()).unwrap();
    zip.write_all(b"<html></html>").unwrap();

    // Write MALICIOUS report
    zip.start_file("report.json", options.clone()).unwrap();
    zip.write_all(malicious_report).unwrap();
    zip.finish().unwrap();

    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.arg("verify").arg(&zip_path);

    cmd.assert()
        .failure()
        .code(2) // Exit 2 for dangerous/corrupt
        .stdout(predicates::str::contains("Hash mismatch"));
}

#[test]
fn test_external_anchor_mismatch() {
    let dir = TempDir::new().unwrap();
    let zip_path = create_golden_zip(&dir);

    let mut cmd = Command::cargo_bin("veil").unwrap();
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

    zip.start_file("run_meta.json", options.clone()).unwrap();
    zip.write_all(run_meta.as_bytes()).unwrap();
    zip.start_file("report.html", options.clone()).unwrap();
    zip.write_all(b"").unwrap();
    zip.start_file("report.json", options.clone()).unwrap();
    zip.write_all(b"").unwrap();
    zip.start_file("effective_config.toml", options.clone())
        .unwrap();
    zip.write_all(b"").unwrap();

    // DANGEROUS PATH INJECTION
    zip.start_file("../evil.txt", options.clone()).unwrap();
    zip.write_all(b"I overwrite your system_keys").unwrap();
    zip.finish().unwrap();

    let mut cmd = Command::cargo_bin("veil").unwrap();
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

    zip.start_file("run_meta.json", options.clone()).unwrap();
    zip.write_all(run_meta.as_bytes()).unwrap();
    zip.start_file("report.html", options.clone()).unwrap();
    // Inject leakage here
    zip.write_all(b"<html><script>window.location.hash='#token=secret123';</script></html>")
        .unwrap();
    zip.start_file("report.json", options.clone()).unwrap();
    zip.write_all(b"").unwrap();
    zip.start_file("effective_config.toml", options.clone())
        .unwrap();
    zip.write_all(b"").unwrap();
    zip.finish().unwrap();

    let mut cmd = Command::cargo_bin("veil").unwrap();
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

    zip.start_file("run_meta.json", options.clone()).unwrap();
    zip.write_all(run_meta.as_bytes()).unwrap();
    zip.start_file("report.html", options.clone()).unwrap();
    zip.write_all(html_rep).unwrap();
    zip.start_file("report.json", options.clone()).unwrap();
    zip.write_all(json_rep).unwrap();
    zip.start_file("effective_config.toml", options.clone())
        .unwrap();
    zip.write_all(rule_config).unwrap();
    zip.finish().unwrap();

    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.arg("verify").arg(&zip_path).arg("--require-complete");

    cmd.assert()
        .failure()
        .code(1) // EXIT 1 for Policy Violation
        .stdout(predicates::str::contains("POLICY VIOLATION"));
}

// Note: test_duplicate_entries is intentionally omitted.
// The zip 5.x writer API structurally prevents writing duplicate filenames
// (it panics before the ZIP is created). The duplicate-entry runtime guard
// in verify.rs remains as defense-in-depth for hand-crafted or legacy ZIPs.
