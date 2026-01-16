use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_sot_new_success() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let out_dir = temp.path().join("docs/pr");
    
    let mut cmd = Command::cargo_bin("veil")?;
    
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
    assert!(content.contains("title: TBD"));
    assert!(content.contains("---")); 

    Ok(())
}

#[test]
fn test_sot_new_slug_dry_run() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let out_dir = temp.path().join("docs/pr");
    
    let mut cmd = Command::cargo_bin("veil")?;
    
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
        .stdout(predicate::str::contains("docs/pr/PR-TBD-v0.19.0-epic-c-audit-log.md"));

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

    let mut cmd = Command::cargo_bin("veil")?;
    
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
    let mut cmd2 = Command::cargo_bin("veil")?;
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
