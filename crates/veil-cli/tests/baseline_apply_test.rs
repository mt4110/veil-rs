use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_baseline_application_flow() {
    let dir = tempdir().unwrap();
    let baseline_path = dir.path().join("veil.baseline.json");
    let secret_file = dir.path().join("secret.txt");

    // 1. Create a secret and generate baseline
    fs::write(&secret_file, "aws_key = AKIA1234567890123456\n").unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path())
        .arg("scan")
        .arg("--write-baseline")
        .arg(&baseline_path);
    cmd.assert().success();

    // 2. Run scan with baseline (Should be 0 new findings, exit 0)
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path())
        .arg("scan")
        .arg("--baseline")
        .arg(&baseline_path);

    cmd.assert()
        .success() // Exit 0
        .stdout(predicate::str::contains("(Baseline suppressed: 1)"))
        .stdout(predicate::str::contains("No new secrets found.")); // No new findings

    // 3. Add a NEW secret (Use AWS key pattern again as it is reliably detected)
    let new_secret_file = dir.path().join("other.txt");
    fs::write(&new_secret_file, "aws_key_2 = AKIA9999999999999999\n").unwrap();

    // 4. Run scan again (Should be 1 new finding, exit 1 if fail-on-findings or severity matches)
    // By default veil fails on high severity? AWS and Github keys are usually High/Critical.
    // Let's ensure failure by passing fail-on-findings=1 just in case defaults change.

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path())
        .arg("scan")
        .arg("--baseline")
        .arg(&baseline_path)
        .arg("--fail-on-findings")
        .arg("1");

    cmd.assert()
        .failure() // Exit 1 because of new finding
        .stdout(predicate::str::contains("(Baseline suppressed: 1)"))
        .stdout(predicate::str::contains("Found 1 new secrets.")); // 1 new finding

    // Output should contain the new finding content but NOT the suppressed one ideally?
    // The current logic prints only "final_findings" (new ones).
    // Let's verify "AKIA" is NOT in output, but "ghp_" IS.
    // Note: The formatter output might redact them, so we check rule IDs or paths if masking is on.
    // Default is usually Redact.

    // Check stdout content
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("other.txt"), "Should show new finding file");
    assert!(
        !stdout.contains("secret.txt"),
        "Should NOT show suppressed finding file"
    );
}
