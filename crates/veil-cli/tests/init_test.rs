use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_init_ci_github() {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Run veil init --ci github --pin-tag v0.17.0 inside temp dir
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(temp_path)
        .arg("init")
        .arg("--ci")
        .arg("github")
        .arg("--pin-tag")
        .arg("v0.17.0");

    cmd.assert().success().stdout(predicate::str::contains(
        "Generated GitHub Actions workflow",
    ));

    // Verify file existence
    let workflow_path = temp_path.join(".github/workflows/veil.yml");
    assert!(workflow_path.exists(), "workflow file should exist");

    // Verify content
    let content = fs::read_to_string(workflow_path).unwrap();
    assert!(content.contains("name: Veil Security Scan"));
    assert!(
        content.contains("--tag v0.17.0"),
        "workflow should contain pinned tag v0.17.0"
    );
    assert!(content.contains("veil scan . --format html"));
}

#[test]
fn test_init_ci_github_pinned_none() {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Run veil init --ci github --pin-tag none inside temp dir
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(temp_path)
        .arg("init")
        .arg("--ci")
        .arg("github")
        .arg("--pin-tag")
        .arg("none");

    cmd.assert().success().stdout(predicate::str::contains(
        "Generated GitHub Actions workflow",
    ));

    let workflow_path = temp_path.join(".github/workflows/veil.yml");
    let content = fs::read_to_string(workflow_path).unwrap();
    assert!(
        !content.contains("--tag"),
        "workflow should NOT contain --tag when pinned=none"
    );
}

#[test]
fn test_init_ci_unsupported() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.arg("init").arg("--ci").arg("gitlab"); // Unsupported

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported CI provider"));
}
