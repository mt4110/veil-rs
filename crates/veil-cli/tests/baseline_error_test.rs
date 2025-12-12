use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_missing_baseline_file() {
    let dir = tempdir().unwrap();
    let missing_path = dir.path().join("does_not_exist.json");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path())
        .arg("scan")
        .arg("--baseline")
        .arg(&missing_path);

    cmd.assert()
        .failure() // Should fail
        .stderr(predicate::str::contains("Failed to load baseline")); // Context message we added/expect
}

#[test]
fn test_corrupt_baseline_file() {
    let dir = tempdir().unwrap();
    let corrupt_path = dir.path().join("corrupt.json");
    fs::write(&corrupt_path, "{ this is not valid json }").unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path())
        .arg("scan")
        .arg("--baseline")
        .arg(&corrupt_path);

    cmd.assert()
        .failure() // Should fail
        .stderr(predicate::str::contains("Failed to load baseline"))
        .stderr(predicate::str::contains("Caused by:")) // Verify we show causes
        .stderr(predicate::str::contains("line 1")); // Verify it's a parse error location
}

#[test]
fn test_empty_baseline_file() {
    let dir = tempdir().unwrap();
    let empty_path = dir.path().join("empty.json");
    fs::write(&empty_path, "").unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path())
        .arg("scan")
        .arg("--baseline")
        .arg(&empty_path);

    cmd.assert()
        .failure() // Should fail
        .stderr(predicate::str::contains("Failed to load baseline"));
}
