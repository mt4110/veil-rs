use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

#[test]
fn html_report_contains_metadata_and_interactive_elements() {
    // Create a temp directory and file to ensure we scan something valid
    // without hitting .gitignore or veil.toml ignore rules (since it's in /tmp)
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("fake_secrets.py");
    let mut file = std::fs::File::create(&file_path).unwrap();
    writeln!(file, "aws_key = \"AKIA1234567890123456\"").unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.arg("scan")
        .arg(temp_dir.path())
        .arg("--format")
        .arg("html");

    let assert = cmd.assert();

    // Check for success and HTML content
    assert
        .success()
        // Header / Meta
        .stdout(predicate::str::contains("<!DOCTYPE html>"))
        .stdout(predicate::str::contains("Veil Security Report"))
        .stdout(predicate::str::contains("Scanned at:"))
        .stdout(predicate::str::contains("Command:"))
        // Summary Cards
        .stdout(predicate::str::contains("Findings Breakdown"))
        .stdout(predicate::str::contains("New"))
        .stdout(predicate::str::contains("Suppressed"))
        // Interactive Elements (Session 18)
        .stdout(predicate::str::contains("data-severity="))
        .stdout(predicate::str::contains(
            "data-rule-id=\"creds.aws.access_key_id\"",
        ))
        .stdout(predicate::str::contains("<div id=\"filters\">"))
        .stdout(predicate::str::contains("<script>"));
}
