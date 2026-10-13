use assert_cmd::Command;
use tempfile::tempdir;

#[test]
#[allow(deprecated)]
fn scan_directory_terminates_ok() {
    let temp_dir = tempdir().unwrap();
    std::fs::write(temp_dir.path().join("sample.txt"), "hello\n").unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.timeout(std::time::Duration::from_secs(30));
    let output = cmd
        .arg("scan")
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cmd/fixtures/clean"))
        .arg("--format")
        .arg("json")
        .arg("--limit")
        .arg("10")
        .assert()
        .success();
    let report: serde_json::Value = serde_json::from_slice(&output.get_output().stdout).unwrap();
    assert_eq!(report["summary"]["scanned_files"], 1);
}
