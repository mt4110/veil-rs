use assert_cmd::Command;

use std::fs;
use tempfile::tempdir;

#[test]
fn test_init_logs_profile_generates_pack() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir_path)
        .arg("init")
        .arg("--profile")
        .arg("Logs")
        .assert()
        .success();

    // Verify veil.toml
    let veil_toml = dir_path.join("veil.toml");
    assert!(veil_toml.exists());
    let config_content = fs::read_to_string(&veil_toml).unwrap();
    assert!(config_content.contains("rules_dir = \"rules/log\""));
    assert!(config_content.contains("placeholder = \"<REDACTED:PII>\""));
    // Verify fail_on_score is NOT present (None) for Logs
    assert!(!config_content.contains("fail_on_score"));

    // Verify rules/log files
    let rules_dir = dir_path.join("rules/log");
    assert!(rules_dir.exists());
    assert!(rules_dir.join("00_manifest.toml").exists());
    assert!(rules_dir.join("secrets.toml").exists());
    assert!(rules_dir.join("pii.toml").exists());
    assert!(rules_dir.join("observability_services.toml").exists());
    assert!(rules_dir.join("README.md").exists());

    // Debug: Check if generated secrets.toml has the placeholder
    let secrets_content = fs::read_to_string(rules_dir.join("secrets.toml")).unwrap();
    if !secrets_content.contains("placeholder = \"<REDACTED:SECRET>\"") {
        panic!(
            "Generated secrets.toml MISSING placeholder! Content snippet:\n{}",
            &secrets_content[0..300]
        );
    } else {
        println!("Generated secrets.toml HAS placeholder.");
    }

    // Verify filter works with generated pack
    // Secrets
    let input_secret = "GH Token: ghp_123456789012345678901234567890123456";
    let mut cmd_filter = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd_filter
        .current_dir(dir_path)
        .arg("filter")
        .write_stdin(input_secret);
    let output = cmd_filter.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains("GH Token: <REDACTED:SECRET>") {
        panic!("Secret masking failed. Output: '{}'", stdout);
    }

    // Observability Key
    let input_obs = "OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317";
    let mut cmd_filter_obs = Command::new(env!("CARGO_BIN_EXE_veil"));
    // Use output() instead of assert() to debug
    let output_obs = cmd_filter_obs
        .current_dir(dir_path)
        .arg("filter")
        .write_stdin(input_obs)
        .output()
        .unwrap();

    let stdout_obs = String::from_utf8_lossy(&output_obs.stdout);
    if !stdout_obs.contains("<REDACTED:OBSERVABILITY>=http://localhost:4317") {
        panic!("Obs masking failed. Output: '{}'", stdout_obs);
    }
}

#[test]
fn test_init_app_profile_defaults() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir_path).arg("init").assert().success();

    let veil_toml = dir_path.join("veil.toml");
    let config_content = fs::read_to_string(&veil_toml).unwrap();
    // Default app profile should have fail_on_score = 80
    assert!(config_content.contains("fail_on_score = 80"));
}

#[test]
fn test_init_fintech_preset_writes_rule_overrides() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir_path)
        .arg("init")
        .arg("--preset")
        .arg("fintech-jp")
        .assert()
        .success();

    let config_content = fs::read_to_string(dir_path.join("veil.toml")).unwrap();
    assert!(config_content.contains("[rules.\"pii.fin.credit_card.keyword\"]"));
    assert!(config_content.contains("base_score = 85"));
    assert!(!config_content.contains("severity ="));
}

#[test]
fn test_init_logs_preset_generates_log_pack() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir_path)
        .arg("init")
        .arg("--preset")
        .arg("logs-jp")
        .assert()
        .success();

    let config_content = fs::read_to_string(dir_path.join("veil.toml")).unwrap();
    assert!(config_content.contains("rules_dir = \"rules/log\""));
    assert!(config_content.contains("[rules.\"log.pii.credit_card\"]"));
    assert!(config_content.contains("base_score = 88"));
    assert!(dir_path.join("rules/log/00_manifest.toml").exists());
}

#[test]
fn test_init_logs_preset_with_application_profile_generates_log_pack() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir_path)
        .arg("init")
        .arg("--profile")
        .arg("application")
        .arg("--preset")
        .arg("logs-jp")
        .assert()
        .success();

    let config_content = fs::read_to_string(dir_path.join("veil.toml")).unwrap();
    assert!(config_content.contains("rules_dir = \"rules/log\""));
    assert!(config_content.contains("fail_on_score = 80"));
    assert!(config_content.contains("[rules.\"log.pii.credit_card\"]"));
    assert!(dir_path.join("rules/log/00_manifest.toml").exists());
}

#[test]
fn test_init_logs_preset_fills_existing_empty_log_pack_dir() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();
    fs::create_dir_all(dir_path.join("rules/log")).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir_path)
        .arg("init")
        .arg("--preset")
        .arg("logs-jp")
        .assert()
        .success();

    assert!(dir_path.join("rules/log/00_manifest.toml").exists());
    assert!(dir_path.join("rules/log/pii.toml").exists());
}

#[test]
fn test_init_logs_preset_fails_existing_stale_log_pack() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();
    let rules_dir = dir_path.join("rules/log");
    fs::create_dir_all(&rules_dir).unwrap();
    fs::write(
        rules_dir.join("pii.toml"),
        r#"
[[rules]]
id = "log.pii.email"
description = "Email in logs"
pattern = '''[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}'''
severity = "LOW"
score = 40
category = "log_pii"
tags = ["log", "pii"]
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_veil"))
        .current_dir(dir_path)
        .arg("init")
        .arg("--preset")
        .arg("logs-jp")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(!dir_path.join("veil.toml").exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("missing required logs-jp rules"));
    assert!(stderr.contains("log.pii.credit_card"));
}
