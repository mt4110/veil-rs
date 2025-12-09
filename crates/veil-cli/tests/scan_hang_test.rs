use assert_cmd::Command;

#[test]
fn scan_directory_terminates_ok() {
    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.arg("scan")
        .arg("crates/veil-core/tests")
        .arg("--format")
        .arg("json")
        .arg("--limit")
        .arg("10")
        .assert()
        .success();
}
