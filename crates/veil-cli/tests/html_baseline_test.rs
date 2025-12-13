use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_html_report_baseline_integration() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let path = temp.path();

    // 1. Create a dummy secret file
    let secret_file = path.join("secrets.txt");
    fs::write(
        &secret_file,
        "
        API_KEY=AKIA1234567890123456
        OTHER_KEY=AKIA9999999999999999
        ",
    )?;

    // 2. Generate baseline
    // Step 2a: Write file with ONLY the first secret
    fs::write(&secret_file, "API_KEY=AKIA1234567890123456\n")?;

    // Step 2b: Generate baseline
    let baseline_path = path.join("baseline.toml");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(path)
        .arg("scan")
        .arg("--write-baseline")
        .arg(&baseline_path)
        .assert()
        .success();

    // Step 3: Modify file to add a NEW secret (mixed state)
    // AKIA1234567890123456 is in baseline
    // AKIA9999999999999999 is NEW
    fs::write(
        &secret_file,
        "API_KEY=AKIA1234567890123456\nNEW_KEY=AKIA9999999999999999\n",
    )?;

    // Step 4: Run veil scan with --baseline and --format html
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    let assert = cmd
        .current_dir(path)
        .arg("scan")
        .arg("--baseline")
        .arg(&baseline_path)
        .arg("--format")
        .arg("html")
        // Default execution might succeed if no fail conditions are met
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Step 5: Verify HTML content
    // a) Check for summary stats
    assert!(stdout.contains("New"), "Should mention New findings");
    assert!(
        stdout.contains("Suppressed"),
        "Should mention Suppressed findings"
    );

    // b) Check for data-status attributes
    // One row should be suppressed
    assert!(
        stdout.contains(r#"data-status="suppressed""#),
        "Should have a row with data-status='suppressed'"
    );

    // One row should be new
    assert!(
        stdout.contains(r#"data-status="new""#),
        "Should have a row with data-status='new'"
    );

    // c) Check visibility of secrets (masked) - verify filename exists in table
    assert!(
        stdout.contains("secrets.txt"),
        "Should show secrets.txt in the report"
    );

    Ok(())
}
