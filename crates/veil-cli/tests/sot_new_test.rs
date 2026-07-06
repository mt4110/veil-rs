use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_sot_new_success() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let out_dir = temp.path().join("docs/pr");

    let mut cmd = cargo_bin_cmd!("veil");

    cmd.arg("sot")
        .arg("new")
        .arg("--release")
        .arg("v0.19.0")
        .arg("--epic")
        .arg("A")
        .arg("--out")
        .arg(out_dir.as_os_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created SOT:"));

    let expected_file = out_dir.join("PR-TBD-v0.19.0-epic-a.md");
    assert!(expected_file.exists());

    let content = fs::read_to_string(expected_file)?;
    assert!(content.contains("release: v0.19.0"));
    assert!(content.contains("epic: A"));
    assert!(content.contains("pr: TBD"));
    assert!(content.contains("status: Draft"));
    assert!(content.contains("created_at: TBD"));
    assert!(content.contains("title: TBD"));
    assert!(content.contains("---"));

    Ok(())
}

#[test]
fn test_sot_new_slug_only_success() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let out_dir = temp.path().join("docs/pr");

    let mut cmd = cargo_bin_cmd!("veil");

    cmd.arg("sot")
        .arg("new")
        .arg("--slug")
        .arg("SOT Template Helper")
        .arg("--title")
        .arg("Add PR SOT template helper")
        .arg("--status")
        .arg("Ready")
        .arg("--date")
        .arg("2026-07-06")
        .arg("--out")
        .arg(out_dir.as_os_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created SOT:"))
        .stdout(predicate::str::contains("PR-TBD-sot-template-helper.md"));

    let expected_file = out_dir.join("PR-TBD-sot-template-helper.md");
    assert!(expected_file.exists());

    let content = fs::read_to_string(expected_file)?;
    assert!(content.contains("release: TBD"));
    assert!(content.contains("epic: A"));
    assert!(content.contains("pr: TBD"));
    assert!(content.contains("status: Ready"));
    assert!(content.contains("created_at: 2026-07-06"));
    assert!(content.contains("title: Add PR SOT template helper"));
    assert!(content.contains("## SOT"));
    assert!(content.contains("## What"));
    assert!(content.contains("## Verification"));
    assert!(content.contains("## Evidence"));
    assert!(content.contains("## Non-goals"));
    assert!(content.contains("## Rollback"));

    Ok(())
}

#[test]
fn test_sot_new_pr_numbered_slug_success() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let out_dir = temp.path().join("docs/pr");

    let mut cmd = cargo_bin_cmd!("veil");

    cmd.arg("sot")
        .arg("new")
        .arg("--pr")
        .arg("123")
        .arg("--slug")
        .arg("sample")
        .arg("--title")
        .arg("Sample")
        .arg("--out")
        .arg(out_dir.as_os_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("PR-123-sample.md"));

    let expected_file = out_dir.join("PR-123-sample.md");
    assert!(expected_file.exists());

    let content = fs::read_to_string(expected_file)?;
    assert!(content.contains("pr: 123"));
    assert!(content.contains("- PR: 123"));

    Ok(())
}

#[test]
fn test_sot_new_slug_dry_run() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let out_dir = temp.path().join("docs/pr");

    let mut cmd = cargo_bin_cmd!("veil");

    cmd.arg("sot")
        .arg("new")
        .arg("--release")
        .arg("v0.19.0")
        .arg("--epic")
        .arg("C")
        .arg("--slug")
        .arg("Audit Log")
        .arg("--title")
        .arg("My Title")
        .arg("--dry-run")
        .arg("--out")
        .arg(out_dir.as_os_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run: would create"))
        .stdout(predicate::str::contains("title: My Title"))
        .stdout(predicate::str::contains("created_at: TBD"))
        .stdout(predicate::str::contains(
            "docs/pr/PR-TBD-v0.19.0-epic-c-audit-log.md",
        ));

    let expected_file = out_dir.join("PR-TBD-v0.19.0-epic-c-audit-log.md");
    assert!(!expected_file.exists());

    Ok(())
}

#[test]
fn test_sot_new_force() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let out_dir = temp.path().join("docs/pr");
    fs::create_dir_all(&out_dir)?;

    let expected_file = out_dir.join("PR-TBD-v0.19.0-epic-a.md");
    fs::write(&expected_file, "old content")?;

    let mut cmd = cargo_bin_cmd!("veil");

    // Fail without force
    cmd.arg("sot")
        .arg("new")
        .arg("--release")
        .arg("v0.19.0")
        .arg("--epic")
        .arg("A")
        .arg("--out")
        .arg(out_dir.as_os_str())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    // Success with force
    let mut cmd2 = cargo_bin_cmd!("veil");
    cmd2.arg("sot")
        .arg("new")
        .arg("--release")
        .arg("v0.19.0")
        .arg("--epic")
        .arg("A")
        .arg("--out")
        .arg(out_dir.as_os_str())
        .arg("--force")
        .assert()
        .success();

    let content = fs::read_to_string(expected_file)?;
    assert!(content.contains("release: v0.19.0"));

    Ok(())
}
