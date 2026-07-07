#![allow(deprecated)]
use assert_cmd::Command;
use predicates::prelude::*;
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

    // 2. Zero threshold is a configuration error.
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.arg("scan")
        .arg(temp_dir.path())
        .arg("--fail-on-findings")
        .arg("0")
        .assert()
        .failure()
        .code(2);

    // 3. Explicit threshold behavior
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.arg("scan")
        .arg(temp_dir.path())
        .arg("--fail-on-findings")
        .arg("1")
        .assert()
        .failure()
        .code(1);

    // 4. Clean scan should succeed
    let clean_dir = tempdir().unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.arg("scan").arg(clean_dir.path()).assert().success();
}

#[test]
fn scan_interactive_accepts_scripted_quit() {
    let temp_dir = tempdir().unwrap();
    fs::write(
        temp_dir.path().join("secret.txt"),
        "AWS_ACCESS_KEY_ID = \"AKIAIOSFODNN7EXAMPLE\"",
    )
    .unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.arg("scan")
        .arg("--interactive")
        .arg(temp_dir.path())
        .write_stdin("q\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Finding 1/")
                .and(predicate::str::contains("Snippet:"))
                .and(predicate::str::contains("Mask preview:"))
                .and(predicate::str::contains("<REDACTED>")),
        )
        .stderr(predicate::str::is_empty());
}

#[test]
fn scan_interactive_masks_with_scripted_input() {
    let temp_dir = tempdir().unwrap();
    let secret_path = temp_dir.path().join("secret.txt");
    fs::write(
        &secret_path,
        "AWS_ACCESS_KEY_ID = \"AKIAIOSFODNN7EXAMPLE\"\n",
    )
    .unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.arg("scan")
        .arg("--interactive")
        .arg(temp_dir.path())
        .write_stdin("mask\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Masked:"))
        .stderr(predicate::str::is_empty());

    let masked = fs::read_to_string(secret_path).unwrap();
    assert!(masked.contains("<REDACTED>"));
    assert!(!masked.contains("AKIAIOSFODNN7EXAMPLE"));
}
