use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_staged_scan() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup temp repo
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // Init git
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(repo_path)
        .output()?;

    // Config git user (needed for commits if we were committing, but we are just staging)
    // But helpful to be safe
    std::process::Command::new("git")
        .args(&["config", "user.email", "you@example.com"])
        .current_dir(repo_path)
        .output()?;
    std::process::Command::new("git")
        .args(&["config", "user.name", "Your Name"])
        .current_dir(repo_path)
        .output()?;

    // 2. Create a file with a secret (AWS Key)
    let secret = "AKIA1234567890AVCDEF"; // 20 chars, all CAPS for rule match
    let staged_file = repo_path.join("staged_secret.txt");
    let mut file = File::create(&staged_file)?;
    writeln!(file, "aws_key = \"{}\"", secret)?;

    // 3. Stage the file
    std::process::Command::new("git")
        .args(&["add", "staged_secret.txt"])
        .current_dir(repo_path)
        .output()?;

    // 4. Create another file but DO NOT stage it
    let unstaged_file = repo_path.join("unstaged_secret.txt");
    let mut file2 = File::create(&unstaged_file)?;
    writeln!(file2, "other_key = \"{}\"", secret)?;

    // 5. Run veil scan --staged
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("veil-cli")?;
    cmd.current_dir(repo_path)
        .arg("scan")
        .arg("--staged")
        .assert()
        .failure() // Should fail because secret is found
        .stdout(predicate::str::contains("staged_secret.txt"))
        .stdout(predicate::str::contains("unstaged_secret.txt").not()); // Should NOT find unstaged

    Ok(())
}

#[test]
fn test_fail_score_env() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // Create a file with a High severity secret (Score 80-100)
    let secret = "AKIA1234567890AVCDEF";
    let file_path = repo_path.join("secret.txt");
    let mut file = File::create(&file_path)?;
    writeln!(file, "key = '{}'", secret)?;

    // Case 1: Fail score 200 (Should Pass, exit 0)
    #[allow(deprecated)]
    let mut cmd_pass = Command::cargo_bin("veil-cli")?;
    cmd_pass
        .current_dir(repo_path)
        .arg("scan")
        .env("VEIL_FAIL_SCORE", "200")
        .assert()
        .success(); // Exit code 0

    // Case 2: Fail score 50 (Should Fail, exit 1)
    #[allow(deprecated)]
    let mut cmd_fail = Command::cargo_bin("veil-cli")?;
    cmd_fail
        .current_dir(repo_path)
        .arg("scan")
        .env("VEIL_FAIL_SCORE", "50")
        .assert()
        .failure(); // Exit code 1

    Ok(())
}
