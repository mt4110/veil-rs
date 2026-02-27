use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_json_output_purity_with_limit() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let scan_target = root.join("src");
    fs::create_dir(&scan_target).unwrap();

    let secret_file_1 = scan_target.join("secret1.txt");
    let secret_file_2 = scan_target.join("secret2.txt");

    fs::write(&secret_file_1, "aws_key = AKIA1234567890123456\n").unwrap();
    fs::write(&secret_file_2, "aws_key2 = AKIA9999999999999999\n").unwrap();

    // Run scan with json format and --limit 1.
    // It should hit the finding limit, output a warning, and exit code 0 or 1.
    // The important part is that stdout MUST be valid JSON.
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(&scan_target)
        .arg("scan")
        .arg(".")
        .arg("--format")
        .arg("json")
        .arg("--limit")
        .arg("1");

    let assert = cmd.assert();

    // Depending on fail-on-findings or fail-score defaults, it might exit 0 or 1.
    // We mainly care about stdout and stderr.
    let assert = assert.stderr(predicate::str::contains("Reached finding limit (1)"));

    let output = assert.get_output();
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    // Try parsing the stdout as JSON. If it prints anything other than the JSON, this will panic.
    let _parsed: serde_json::Value = serde_json::from_str(&stdout_str)
        .expect("stdout was not valid JSON! Output purity is compromised.");
}

#[test]
fn test_html_output_purity_with_file_limit() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let scan_target = root.join("src");
    fs::create_dir(&scan_target).unwrap();

    // Write a config file to lower max_file_count to 1
    let config_file = scan_target.join("veil.toml");
    let config_content = r#"
[core]
max_file_count = 1
"#;
    fs::write(&config_file, config_content).unwrap();

    let secret_file_1 = scan_target.join("secret1.txt");
    let secret_file_2 = scan_target.join("secret2.txt");

    fs::write(&secret_file_1, "aws_key = AKIA1234567890123456\n").unwrap();
    fs::write(&secret_file_2, "aws_key2 = AKIA9999999999999999\n").unwrap();

    // Run scan with html format
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(&scan_target)
        .arg("scan")
        .arg(".")
        .arg("--format")
        .arg("html");

    // It should hit the file limit, output a warning to stderr, and exit code 2.
    // The stdout MUST be valid HTML (or at least start with <!DOCTYPE html> without prior garbage).
    let assert = cmd.assert().code(2);

    let assert = assert.stderr(predicate::str::contains(
        "The scan was truncated due to max_file_count limit",
    ));

    let output = assert.get_output();
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout_str.starts_with("<!DOCTYPE html>"),
        "stdout did not start with HTML DOCTYPE! Output purity is compromised. Output: {}",
        stdout_str.chars().take(100).collect::<String>()
    );
}
