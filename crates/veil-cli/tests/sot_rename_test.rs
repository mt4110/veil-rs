use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn write_min_sot(path: &std::path::Path) -> std::io::Result<()> {
    let content = r#"---
release: v0.19.0
epic: A
pr: TBD
status: draft
created_at: 2026-01-16
branch: feat/v0.19.0-sot-autopilot
commit: deadbeef
title: TBD
---

# Why
- TBD
"#;
    fs::write(path, content)
}

#[test]
fn test_sot_rename_success_autodetect() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let dir = temp.path().join("docs/pr");
    fs::create_dir_all(&dir)?;

    let src = dir.join("PR-TBD-v0.19.0-epic-a.md");
    write_min_sot(&src)?;

    let mut cmd = Command::cargo_bin("veil")?;
    cmd.arg("sot")
        .arg("rename")
        .arg("--pr")
        .arg("123")
        .arg("--dir")
        .arg(dir.as_os_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("Renamed SOT:"))
        .stdout(predicate::str::contains("PR-123-v0.19.0-epic-a.md"));

    let dst = dir.join("PR-123-v0.19.0-epic-a.md");
    assert!(dst.exists());
    assert!(!src.exists());

    let content = fs::read_to_string(dst)?;
    assert!(content.contains("pr: 123"));

    Ok(())
}

#[test]
fn test_sot_rename_dry_run() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let dir = temp.path().join("docs/pr");
    fs::create_dir_all(&dir)?;

    let src = dir.join("PR-TBD-v0.19.0-epic-a.md");
    write_min_sot(&src)?;

    let mut cmd = Command::cargo_bin("veil")?;
    cmd.arg("sot")
        .arg("rename")
        .arg("--pr")
        .arg("123")
        .arg("--dir")
        .arg(dir.as_os_str())
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run: would rename"));

    // No changes
    assert!(src.exists());
    let dst = dir.join("PR-123-v0.19.0-epic-a.md");
    assert!(!dst.exists());

    Ok(())
}

#[test]
fn test_sot_rename_multiple_candidates_requires_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let dir = temp.path().join("docs/pr");
    fs::create_dir_all(&dir)?;

    let a = dir.join("PR-TBD-v0.19.0-epic-a.md");
    let b = dir.join("PR-TBD-v0.19.0-epic-b-sot-ux.md");
    write_min_sot(&a)?;
    write_min_sot(&b)?;

    let mut cmd = Command::cargo_bin("veil")?;
    cmd.arg("sot")
        .arg("rename")
        .arg("--pr")
        .arg("123")
        .arg("--dir")
        .arg(dir.as_os_str())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Multiple PR-TBD SOT files found"));

    Ok(())
}
