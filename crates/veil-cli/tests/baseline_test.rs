use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn write_baseline_creates_file_with_schema() {
    let dir = tempdir().unwrap();
    let baseline_path = dir.path().join("veil.baseline.json");

    // Create a dummy secret file to ensure we have findings
    let secret_file = dir.path().join("secret.txt");
    fs::write(&secret_file, "aws_key = AKIA1234567890123456").unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path())
        .arg("scan")
        .arg("--write-baseline")
        .arg(&baseline_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Baseline written to"));

    assert!(baseline_path.exists());

    let content = fs::read_to_string(&baseline_path).unwrap();
    assert!(content.contains("\"schema\": \"veil.baseline.v1\""));
    assert!(content.contains("\"fingerprint\":"));
    assert!(content.contains("creds.aws.access_key")); // Rule ID check
}

#[test]
fn baseline_argument_conflicts_with_write_baseline() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.arg("scan")
        .arg("--baseline")
        .arg("foo.json")
        .arg("--write-baseline")
        .arg("bar.json");

    cmd.assert().failure().stderr(predicate::str::contains(
        "argument '--baseline <PATH>' cannot be used with '--write-baseline <PATH>'",
    ));
}
