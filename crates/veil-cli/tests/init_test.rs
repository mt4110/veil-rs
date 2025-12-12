use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_init_ci_github() {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Run veil init --ci github inside temp dir
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(temp_path)
        .arg("init")
        .arg("--ci")
        .arg("github");

    cmd.assert().success().stdout(predicate::str::contains(
        "Generated GitHub Actions workflow",
    ));

    // Verify file existence
    let workflow_path = temp_path.join(".github/workflows/veil.yml");
    assert!(workflow_path.exists(), "workflow file should exist");

    // Verify content
    let content = fs::read_to_string(workflow_path).unwrap();
    assert!(content.contains("name: Veil Security Scan"));
    assert!(content.contains("veil scan . --format html"));
}

#[test]
fn test_init_ci_unsupported() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.arg("init").arg("--ci").arg("gitlab"); // Unsupported

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported CI provider"));
}
