#![allow(deprecated)]
use assert_cmd::Command;
use serde_json::Value;

#[test]
fn test_json_output_has_schema_version() {
    let mut cmd = Command::cargo_bin("veil").unwrap();
    let assert = cmd
        .arg("scan")
        .arg("crates/veil-core/tests") // Scan a small directory
        .arg("--format")
        .arg("json")
        .arg("--limit")
        .arg("5")
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = std::str::from_utf8(&output.stdout).expect("Valid UTF-8 output");

    let json: Value = serde_json::from_str(stdout).expect("Valid JSON output");

    assert_eq!(
        json["schemaVersion"], "veil-v1",
        "JSON output must contain schemaVersion: veil-v1"
    );

    // Also check that summary and findings exist
    assert!(json.get("summary").is_some(), "Summary missing");
    assert!(json.get("findings").is_some(), "Findings missing");
}
