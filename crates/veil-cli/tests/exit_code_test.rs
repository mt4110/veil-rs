#![allow(deprecated)]
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

    // 1. Default behavior (should SUCCEED now, unless flag is passed)
    // We changed policy to be "safe by default" unless --fail-on-findings is used.
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.arg("scan").arg(temp_dir.path()).assert().success(); // Expecting exit code 0

    // 2. Explicit flag behavior
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.arg("scan")
        .arg(temp_dir.path())
        .arg("--fail-on-findings")
        .arg("0")
        .assert()
        .failure()
        .code(1);

    // 3. Clean scan should succeed
    let clean_dir = tempdir().unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.arg("scan").arg(clean_dir.path()).assert().success();
}
