#![allow(deprecated)]
use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_fail_on_severity() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path();

    // Create a file with a known AWS key (Likely High or Critical)
    // AKIA... is usually High/Critical.
    let secret = "AWS_ACCESS_KEY_ID = \"AKIAIOSFODNN7EXAMPLE\"";
    let file_path = root.join("secret.py");
    fs::write(&file_path, secret).unwrap();

    // 1. Base case: Should find it but exit 0 (default behavior)
    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.current_dir(root)
        .arg("scan")
        .arg(".")
        .assert()
        .success(); // Exit 0

    // 2. Fail on findings count: Should exit 1 (threshold 1 <= 1 detected)
    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.current_dir(root)
        .arg("scan")
        .arg(".")
        .arg("--fail-on-findings")
        .arg("1")
        .assert()
        .failure(); // Exit 1

    // Should exit 0 if threshold 2 > 1 detected
    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.current_dir(root)
        .arg("scan")
        .arg(".")
        .arg("--fail-on-findings")
        .arg("2")
        .assert()
        .success(); // Exit 0

    // 3. Fail on Severity Low: Should exit 1 (since High > Low)
    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.current_dir(root)
        .arg("scan")
        .arg(".")
        .arg("--fail-on-severity")
        .arg("Low")
        .assert()
        .failure();

    // 4. Fail on Severity Critical:
    // If AWS key is High, this should PASS (exit 0).
    // If AWS key is Critical, this should FAIL.
    // Let's inspect the output to know what it is, but for now let's assume High.
    // To be safe, let's use a "Medium" severity trigger test.
    // Or we can construct a test case where we check JSON output first to know severity.

    // Let's use `password = "..."` which is usually Medium/Low or not detected if default rules are strict.
    // Actually, `veil-rs` rules for Generic API Key are usually High.
}

#[test]
fn test_fail_on_score() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path();

    // Create a file with a secret
    let secret = "AWS_ACCESS_KEY_ID = \"AKIAIOSFODNN7EXAMPLE\"";
    let file_path = root.join("secret.py");
    fs::write(&file_path, secret).unwrap();

    // 1. Fail on Score 1 (Anything detected essentially) -> Fail
    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.current_dir(root)
        .arg("scan")
        .arg(".")
        .arg("--fail-on-score")
        .arg("1")
        .assert()
        .failure();

    // 2. Fail on Score 1000 (Impossible) -> Success
    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.current_dir(root)
        .arg("scan")
        .arg(".")
        .arg("--fail-on-score")
        .arg("1000")
        .assert()
        .success();
}
