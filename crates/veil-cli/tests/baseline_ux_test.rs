use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_ux_case_a_clean() {
    let dir = tempdir().unwrap();
    // No secrets, no baseline

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path()).arg("scan");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No secrets found."))
        .stdout(predicate::str::contains("Baseline suppressed").not());
}

#[test]
fn test_ux_case_b_suppressed_clean() {
    let dir = tempdir().unwrap();
    let baseline_path = dir.path().join("veil.baseline.json");
    let secret = dir.path().join("secret.txt");
    fs::write(&secret, "aws_key = AKIA1234567890123456\n").unwrap();

    // Generate baseline
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path())
        .arg("scan")
        .arg("--write-baseline")
        .arg(&baseline_path);
    cmd.assert().success();

    // Scan with baseline -> Should say "No new secrets found"
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path())
        .arg("scan")
        .arg("--baseline")
        .arg(&baseline_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No new secrets found."))
        .stdout(predicate::str::contains("Baseline suppressed: 1"));
}

#[test]
fn test_ux_case_c_dirty() {
    let dir = tempdir().unwrap();
    let baseline_path = dir.path().join("veil.baseline.json");
    let secret = dir.path().join("secret.txt");
    fs::write(&secret, "aws_key = AKIA1234567890123456\n").unwrap();

    // Generate baseline
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path())
        .arg("scan")
        .arg("--write-baseline")
        .arg(&baseline_path);
    cmd.assert().success();

    // Add new secret
    let new_secret = dir.path().join("new.txt");
    fs::write(&new_secret, "aws_key_2 = AKIA9999999999999999\n").unwrap();

    // Scan -> "Found 1 new secrets"
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path())
        .arg("scan")
        .arg("--baseline")
        .arg(&baseline_path)
        .arg("--fail-on-findings")
        .arg("1");

    cmd.assert()
        .failure() // New findings
        .stdout(predicate::str::contains("Found 1 new secrets."))
        .stdout(predicate::str::contains("Baseline suppressed: 1"));
}

#[test]
fn test_ux_case_d_json_schema() {
    let dir = tempdir().unwrap();
    let baseline_path = dir.path().join("veil.baseline.json");
    let secret = dir.path().join("secret.txt");
    fs::write(&secret, "aws_key = AKIA1234567890123456\n").unwrap();

    // Generate baseline
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path())
        .arg("scan")
        .arg("--write-baseline")
        .arg(&baseline_path);
    cmd.assert().success();

    // Scan json
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path())
        .arg("scan")
        .arg("--baseline")
        .arg(&baseline_path)
        .arg("--format")
        .arg("json");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Simple string checks for keys
    assert!(stdout.contains("\"total_findings\":"));
    assert!(stdout.contains("\"new_findings\":"));
    assert!(stdout.contains("\"baseline_suppressed\":"));
}
