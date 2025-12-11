use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn config_dump_repo_and_effective_json() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("veil.toml");

    let config_toml = r#"
[core]
fail_on_score = 88
"#;
    let mut file = File::create(&config_path).unwrap();
    writeln!(file, "{}", config_toml).unwrap();

    // repo layer (via default veil.toml)
    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.current_dir(dir.path())
        .arg("config")
        .arg("dump")
        .arg("--layer")
        .arg("repo");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"fail_on_score\": 88"));

    // effective layer (should contain same value)
    let mut cmd2 = Command::cargo_bin("veil").unwrap();
    cmd2.current_dir(dir.path()).arg("config").arg("dump");
    cmd2.assert()
        .success()
        .stdout(predicate::str::contains("\"fail_on_score\": 88"));
}

#[test]
fn config_dump_org_is_empty_by_default() {
    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.arg("config").arg("dump").arg("--layer").arg("org");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("(no config for this layer)"));
}

#[test]
fn config_dump_toml_format() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("veil.toml");

    let config_toml = r#"
[core]
fail_on_score = 77
"#;
    let mut file = File::create(&config_path).unwrap();
    writeln!(file, "{}", config_toml).unwrap();

    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.current_dir(dir.path())
        .arg("config")
        .arg("dump")
        .arg("--format")
        .arg("toml");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[core]"))
        .stdout(predicate::str::contains("fail_on_score = 77"));
}
