use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn rules_list_shows_known_rule() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.arg("rules").arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("creds.aws.access_key_id"))
        .stdout(predicate::str::contains("HIGH"))
        .stdout(predicate::str::contains("Category"));
}

#[test]
fn rules_list_severity_filter() {
    // Should pass for HIGH
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.arg("rules")
        .arg("list")
        .arg("--severity")
        .arg("CRITICAL");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("creds.key.private_pem"))
        // Should NOT contain a LOW/MEDIUM rule if one exists (e.g. jp.phone.mobile is MEDIUM)
        .stdout(predicate::str::contains("jp.phone.mobile").not());
}

#[test]
fn rules_explain_shows_details() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.arg("rules")
        .arg("explain")
        .arg("creds.aws.access_key_id");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("AWS Access Key ID"))
        .stdout(predicate::str::contains("Pattern:"))
        .stdout(predicate::str::contains("Context:"));
}

#[test]
fn rules_explain_invalid_fails() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.arg("rules")
        .arg("explain")
        .arg("no_such_rule_exists_12345");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Rule not found"));
}
