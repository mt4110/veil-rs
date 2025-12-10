#![allow(deprecated)]
use assert_cmd::Command;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;

#[test]
fn test_pre_commit_init() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path();

    // 1. Initialize git repo
    std::process::Command::new("git")
        .arg("init")
        .current_dir(root)
        .output()
        .expect("Failed to git init");

    // 2. Run `veil pre-commit init`
    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.current_dir(root)
        .arg("pre-commit")
        .arg("init")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Veil pre-commit hook installed successfully",
        ));

    // 3. Verify hook exists
    let hook_path = root.join(".git").join("hooks").join("pre-commit");
    assert!(hook_path.exists(), "Hook file should be created");

    // 4. Verify executable permissions
    let metadata = fs::metadata(&hook_path).unwrap();
    let mode = metadata.permissions().mode();
    assert_eq!(mode & 0o111, 0o111, "Hook must be executable");

    // 5. Verify content
    let content = fs::read_to_string(&hook_path).unwrap();
    assert!(content.contains("veil scan --staged"));
    assert!(content.contains("Commit blocked by Veil Security Check"));

    // 6. Test Idempotency / Backup
    // Run it again
    let mut cmd2 = Command::cargo_bin("veil").unwrap();
    cmd2.current_dir(root)
        .arg("pre-commit")
        .arg("init")
        .assert()
        .success()
        .stdout(predicates::str::contains("Backed up existing hook"));

    // Verify backup exists
    let backup_path = root.join(".git").join("hooks").join("pre-commit.veil.bak");
    assert!(backup_path.exists(), "Backup should be created");
}
