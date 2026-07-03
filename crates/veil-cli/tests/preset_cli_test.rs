use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn scan_preset_applies_rule_base_score() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("card.txt"), "card: 4111222233334448\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_veil"))
        .current_dir(dir.path())
        .arg("--quiet")
        .arg("scan")
        .arg(".")
        .arg("--preset")
        .arg("fintech-jp")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        json["findings"][0]["rule_id"],
        "pii.fin.credit_card.keyword"
    );
    assert_eq!(json["findings"][0]["score"], 85);
}

#[test]
fn scan_unknown_preset_fails_fast() {
    let dir = tempdir().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_veil"))
        .current_dir(dir.path())
        .arg("scan")
        .arg(".")
        .arg("--preset")
        .arg("minimal-ci")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown preset 'minimal-ci'"));
}

#[test]
fn scan_logs_preset_loads_log_rule_pack() {
    let dir = tempdir().unwrap();
    Command::new(env!("CARGO_BIN_EXE_veil"))
        .current_dir(dir.path())
        .arg("init")
        .arg("--preset")
        .arg("logs-jp")
        .assert()
        .success();

    fs::write(dir.path().join("app.log"), "payment=4111222233334448\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_veil"))
        .current_dir(dir.path())
        .arg("--quiet")
        .arg("scan")
        .arg(".")
        .arg("--preset")
        .arg("logs-jp")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = json["findings"].as_array().unwrap();
    let log_card = findings
        .iter()
        .find(|finding| finding["rule_id"] == "log.pii.credit_card")
        .unwrap();
    assert!(log_card["score"].as_u64().unwrap() >= 88);
}

#[test]
fn scan_logs_preset_without_rule_pack_fails_with_guidance() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("app.log"), "payment=4111222233334448\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_veil"))
        .current_dir(dir.path())
        .arg("scan")
        .arg(".")
        .arg("--preset")
        .arg("logs-jp")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Preset 'logs-jp' requires the log rule pack"));
    assert!(stderr.contains("veil init --preset logs-jp"));
}
