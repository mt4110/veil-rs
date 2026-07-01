use assert_cmd::Command;
use tempfile::tempdir;

#[test]
#[allow(deprecated)]
fn scan_directory_terminates_ok() {
    let temp_dir = tempdir().unwrap();
    std::fs::write(temp_dir.path().join("sample.txt"), "hello\n").unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.arg("scan")
        .arg(temp_dir.path())
        .arg("--format")
        .arg("json")
        .arg("--limit")
        .arg("10")
        .assert()
        .success();
}
