use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_exit_code_behavior() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("secret.txt");
    // Write a fake secret that matches default rules (e.g. AWS key pattern)
    // AKIA... is usually detected.
    fs::write(&file_path, "AKIA1234567890ABCDEF").unwrap();

    // 1. Default behavior (should fail if threshold is 0/default)
    // Since we didn't set threshold in config, default is 0, so any finding triggers failure.
    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.arg("scan")
        .arg(temp_dir.path())
        .assert()
        .failure() // Expecting exit code 1
        .code(1);

    // 2. Explicit flag behavior
    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.arg("scan")
        .arg(temp_dir.path())
        .arg("--fail-on-findings")
        .assert()
        .failure()
        .code(1);

    // 3. Clean scan should succeed
    let clean_dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.arg("scan").arg(clean_dir.path()).assert().success();
}
