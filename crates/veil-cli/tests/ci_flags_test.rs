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
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(root)
        .arg("scan")
        .arg(".")
        .assert()
        .success(); // Exit 0

    // 2. Fail on findings count: Should exit 1 (threshold 1 <= 1 detected)
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(root)
        .arg("scan")
        .arg(".")
        .arg("--fail-on-findings")
        .arg("1")
        .assert()
        .failure(); // Exit 1

    // Fail on severity HIGH -> should FAIL (because AWS key yields sev:HIGH)
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(root)
        .arg("scan")
        .arg(".")
        .arg("--fail-on-findings")
        .arg("99")
        .arg("--fail-on-severity")
        .arg("HIGH")
        .assert()
        .failure();

    // Fail on severity CRITICAL is a score gate; this fixture reaches score 100.
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(root)
        .arg("scan")
        .arg(".")
        .arg("--fail-on-findings")
        .arg("99")
        .arg("--fail-on-severity")
        .arg("CRITICAL")
        .assert()
        .failure();
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
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(root)
        .arg("scan")
        .arg(".")
        .arg("--fail-on-score")
        .arg("1")
        .assert()
        .failure();

    // 2. Out-of-range thresholds are configuration errors.
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(root)
        .arg("scan")
        .arg(".")
        .arg("--fail-on-score")
        .arg("1000")
        .assert()
        .failure()
        .code(2);
}
