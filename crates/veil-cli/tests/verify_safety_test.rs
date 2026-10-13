use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn missing_evidence_returns_invalid_json_and_exit_two() {
    let directory = tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_veil"))
        .current_dir(directory.path())
        .args(["verify", "missing.zip", "--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "INVALID");
    assert_eq!(report["code"], 2);
}

#[test]
fn explicit_zip_budget_reaches_the_core_verifier() {
    let directory = tempdir().unwrap();
    fs::write(directory.path().join("oversized.zip"), [0; 16]).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_veil"))
        .current_dir(directory.path())
        .args([
            "verify",
            "oversized.zip",
            "--format",
            "json",
            "--max-zip-bytes",
            "8",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "INVALID");
    assert!(report["error"]
        .as_str()
        .unwrap()
        .contains("ZIP size 16 exceeds limit 8"));
}

#[test]
fn default_verify_help_preserves_v1_controls() {
    let output = Command::new(env!("CARGO_BIN_EXE_veil"))
        .args(["verify", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for flag in [
        "--require-complete",
        "--expect-run-meta-sha256",
        "--fail-on-findings",
        "--max-zip-bytes",
        "--max-entry-bytes",
        "--max-total-bytes",
        "--max-files",
    ] {
        assert!(help.contains(flag), "missing existing v1 option {flag}");
    }
    assert!(!help.contains("--public-key"));
}

#[test]
fn deferred_commands_are_rejected_without_creating_state() {
    for command in ["detect", "prove", "decide", "case", "legacy"] {
        let directory = tempdir().unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_veil"))
            .current_dir(directory.path())
            .arg(command)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("unrecognized subcommand"));
        assert!(!directory.path().join(".veil").exists());
    }
}

fn zip_with_file(path: &std::path::Path, name: &str, content: &[u8]) {
    use std::io::Write;
    let mut archive = zip::ZipWriter::new(fs::File::create(path).unwrap());
    archive
        .start_file(name, zip::write::SimpleFileOptions::default())
        .unwrap();
    archive.write_all(content).unwrap();
    archive.finish().unwrap();
}

#[test]
fn v2_contract_is_rejected_by_existing_v1_verifier() {
    let directory = tempdir().unwrap();
    zip_with_file(
        &directory.path().join("new-format.zip"),
        "attestation_v2.json",
        b"{}",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_veil"))
        .current_dir(directory.path())
        .args(["verify", "new-format.zip", "--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "INVALID");
    assert!(report["error"]
        .as_str()
        .unwrap()
        .contains("attestation_v2.json"));
}

#[test]
fn leakage_failure_does_not_print_matched_token() {
    let directory = tempdir().unwrap();
    let token = "A".repeat(20);
    let text = format!("Authorization: Bearer {token}");
    zip_with_file(
        &directory.path().join("unsafe.zip"),
        "report.html",
        text.as_bytes(),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_veil"))
        .current_dir(directory.path())
        .args(["verify", "unsafe.zip", "--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["error"]
        .as_str()
        .unwrap()
        .contains("Token leakage detected"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains(&token));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(&token));
}
