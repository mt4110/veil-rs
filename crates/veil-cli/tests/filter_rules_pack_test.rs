use assert_cmd::Command;
use predicates::prelude::*;

use std::path::PathBuf;

#[test]
fn test_filter_load_rules_pack() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    
    // We assume the test runs from the workspace root or crate root.
    // We point to examples/veil-log.toml
    // Relative path depends on where cargo test runs.
    // Usually, for workspace, it runs in crate dir.
    // ../../examples/veil-log.toml?
    // Let's resolve absolute path to be safe.
    
    let root = env!("CARGO_MANIFEST_DIR");
    let config_path = PathBuf::from(root).parent().unwrap().parent().unwrap().join("examples/veil-log.toml");
    
    // Input containing an email (masked by pii.email from pack)
    let input = "Contact: test@example.com for support.";
    
    cmd.arg("filter")
        .arg("--config")
        .arg(config_path)
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Contact: <REDACTED:PII> for support."));
}

#[test]
fn test_filter_load_rules_pack_jp() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    let root = env!("CARGO_MANIFEST_DIR");
    let config_path = PathBuf::from(root).parent().unwrap().parent().unwrap().join("examples/veil-log.toml");
    
    // Test JP Postal Code
    let input = "Address: 123-4567 Tokyo";
    
    cmd.arg("filter")
        .arg("--config")
        .arg(config_path)
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Address: <REDACTED:JP:POSTAL> Tokyo"));
}
