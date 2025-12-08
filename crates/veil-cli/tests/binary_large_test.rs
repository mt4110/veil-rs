use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_binary_and_large_files() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // 1. Create a Large File (1.1MB)
    // Default limit is 1MB
    let large_file_path = repo_path.join("large.log");
    let mut f = File::create(&large_file_path)?;
    // Write 1.1MB of dummy data
    let chunk = [b'a'; 1024];
    for _ in 0..1100 {
        f.write_all(&chunk)?;
    }

    // 2. Create a Binary File
    let binary_file_path = repo_path.join("app.bin");
    let mut f_bin = File::create(&binary_file_path)?;
    f_bin.write_all(b"Hello World \0 Binary Content")?;

    // 3. Run veil scan
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("veil-cli")?;
    cmd.current_dir(repo_path)
        .arg("scan")
        .assert()
        // We expect it to FAIL (exit 1) because the legacy behavior (threshold 0) fails on any finding.
        .failure()
        // Check for findings in output
        .stdout(predicate::str::contains("MAX_FILE_SIZE"))
        .stdout(predicate::str::contains("BINARY_FILE"));

    Ok(())
}
