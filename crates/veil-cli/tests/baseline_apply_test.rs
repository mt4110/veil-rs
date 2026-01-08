use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_baseline_application_flow() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let scan_target = root.join("src");
    fs::create_dir(&scan_target).unwrap();

    let baseline_path = root.join("veil.baseline.json");
    let secret_file = scan_target.join("secret.txt");

    // 1. Create a secret and generate baseline
    fs::write(&secret_file, "aws_key = AKIA1234567890123456\n").unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(&scan_target)
        .arg("scan")
        .arg(".")
        .arg("--write-baseline")
        .arg(&baseline_path);
    cmd.assert().success();

    // 2. Run scan with baseline (Should be 0 new findings, exit 0)
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(&scan_target)
        .arg("scan")
        .arg(".")
        .arg("--baseline")
        .arg(&baseline_path);

    cmd.assert()
        .success() // Exit 0
        .stdout(predicate::str::contains("Baseline suppressed:"))
        .stdout(predicate::str::contains("No new secrets found.")); // No new findings

    // 3. Add a NEW secret (Use AWS key pattern again as it is reliably detected)
    let new_secret_file = scan_target.join("other.txt");
    fs::write(&new_secret_file, "aws_key_2 = AKIA9999999999999999\n").unwrap();

    // 4. Run scan again (Should be 1 new finding, exit 1 if fail-on-findings or severity matches)
    // By default veil fails on high severity? AWS and Github keys are usually High/Critical.
    // Let's ensure failure by passing fail-on-findings=1 just in case defaults change.

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(&scan_target)
        .arg("scan")
        .arg(".")
        .arg("--baseline")
        .arg(&baseline_path)
        .arg("--fail-on-findings")
        .arg("1");

    let assert = cmd
        .assert()
        .failure() // Exit 1 because of new finding
        .stdout(predicate::str::contains("Baseline suppressed:"))
        .stdout(predicate::str::contains("new secrets.")); // Some new finding

    // Output should contain the new finding content but NOT the suppressed one ideally?
    // The current logic prints only "final_findings" (new ones).
    // Let's verify "AKIA" is NOT in output, but "ghp_" IS.
    // Note: The formatter output might redact them, so we check rule IDs or paths if masking is on.
    // Default is usually Redact.

    // Check stdout content
    let output = assert.get_output();
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();

    assert!(stdout.contains("other.txt"), "Should show new finding file");
    assert!(
        !stdout.contains("secret.txt"),
        "Should NOT show suppressed finding file"
    );
}

#[test]
fn test_baseline_error_conditions() {
    let dir = tempdir().unwrap();
    let non_existent = dir.path().join("does_not_exist.json");
    let corrupt_path = dir.path().join("corrupt.json");

    // Setup corrupt file
    fs::write(&corrupt_path, "{ invalid json").unwrap();

    // 1. Missing baseline -> Exit 2
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path())
        .arg("scan")
        .arg("--baseline")
        .arg(&non_existent);

    cmd.assert()
        .failure()
        .code(2) // Explicitly check for Exit 2
        .stderr(predicate::str::contains("Error: Failed to load baseline"));

    // 2. Corrupt baseline -> Exit 2
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path())
        .arg("scan")
        .arg("--baseline")
        .arg(&corrupt_path);

    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("Error: Failed to load baseline"));
}
