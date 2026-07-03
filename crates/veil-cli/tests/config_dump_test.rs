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
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path())
        .arg("config")
        .arg("dump")
        .arg("--layer")
        .arg("repo");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"fail_on_score\": 88"));

    // effective layer (should contain same value)
    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd2.current_dir(dir.path()).arg("config").arg("dump");
    cmd2.assert()
        .success()
        .stdout(predicate::str::contains("\"fail_on_score\": 88"));
}

#[test]
fn config_dump_preset_layer_json() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.arg("config")
        .arg("dump")
        .arg("--preset")
        .arg("fintech-jp")
        .arg("--layer")
        .arg("preset");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("pii.fin.credit_card.keyword"))
        .stdout(predicate::str::contains("\"base_score\": 85"));
}

#[test]
fn config_dump_preset_layer_ignores_external_config_errors() {
    let dir = tempdir().unwrap();
    let missing_org_config = dir.path().join("missing-org.toml");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.env("VEIL_ORG_CONFIG", missing_org_config)
        .arg("config")
        .arg("dump")
        .arg("--preset")
        .arg("fintech-jp")
        .arg("--layer")
        .arg("preset");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("pii.fin.credit_card.keyword"));
}

#[test]
fn config_dump_logs_preset_effective_skips_rule_pack_validation() {
    let dir = tempdir().unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path())
        .arg("config")
        .arg("dump")
        .arg("--preset")
        .arg("logs-jp")
        .arg("--layer")
        .arg("effective");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("log.pii.credit_card"));
}

#[test]
fn config_dump_effective_shows_repo_override_over_preset() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("veil.toml");

    let config_toml = r#"
[rules."pii.fin.credit_card.keyword"]
enabled = true
base_score = 99
"#;
    let mut file = File::create(&config_path).unwrap();
    writeln!(file, "{}", config_toml).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path())
        .arg("config")
        .arg("dump")
        .arg("--preset")
        .arg("fintech-jp")
        .arg("--layer")
        .arg("effective");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"base_score\": 99"))
        .stdout(predicate::str::contains("\"base_score\": 85").not());
}

#[test]
fn config_dump_org_is_empty_by_default() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
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

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
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

#[test]
fn config_dump_org_env_explicit() {
    let dir = tempdir().unwrap();
    let org_config_path = dir.path().join("org.toml");

    let config_toml = r#"
[core]
fail_on_score = 10
"#;
    let mut file = File::create(&org_config_path).unwrap();
    writeln!(file, "{}", config_toml).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.env("VEIL_ORG_CONFIG", &org_config_path)
        .arg("config")
        .arg("dump")
        .arg("--layer")
        .arg("org");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"fail_on_score\": 10"));
}

#[test]
fn config_dump_user_xdg_implicit() {
    let dir = tempdir().unwrap();
    let xdg_dir = dir.path().join("veil");
    std::fs::create_dir_all(&xdg_dir).unwrap();
    let user_config_path = xdg_dir.join("veil.toml");

    let config_toml = r#"
[core]
fail_on_score = 20
"#;
    let mut file = File::create(&user_config_path).unwrap();
    writeln!(file, "{}", config_toml).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.env("XDG_CONFIG_HOME", dir.path())
        // Ensure explicit overrides are unset so fallbacks run
        .env_remove("VEIL_USER_CONFIG")
        .arg("config")
        .arg("dump")
        .arg("--layer")
        .arg("user");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"fail_on_score\": 20"));
}

#[test]
fn config_layer_precedence() {
    // Repo > Org > User
    let dir = tempdir().unwrap();

    // User Layer (20)
    let xdg_dir = dir.path().join("config_home/veil");
    std::fs::create_dir_all(&xdg_dir).unwrap();
    let user_path = xdg_dir.join("veil.toml");
    writeln!(
        File::create(&user_path).unwrap(),
        "[core]\nfail_on_score = 20"
    )
    .unwrap();

    // Org Layer (10)
    let org_path = dir.path().join("org.toml");
    writeln!(
        File::create(&org_path).unwrap(),
        "[core]\nfail_on_score = 10"
    )
    .unwrap();

    // Repo Layer (99)
    let repo_dir = dir.path().join("repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    let repo_path = repo_dir.join("veil.toml");
    writeln!(
        File::create(&repo_path).unwrap(),
        "[core]\nfail_on_score = 99"
    )
    .unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(&repo_dir)
        .env("XDG_CONFIG_HOME", dir.path().join("config_home"))
        .env("VEIL_ORG_CONFIG", &org_path)
        .arg("config")
        .arg("dump")
        .arg("--layer")
        .arg("effective");

    // Repo should win (99)
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"fail_on_score\": 99"));

    // Test Org > User (without repo)
    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_veil"));
    // Run in a dir without veil.toml
    let empty_dir = dir.path().join("empty");
    std::fs::create_dir_all(&empty_dir).unwrap();

    cmd2.current_dir(&empty_dir)
        .env("XDG_CONFIG_HOME", dir.path().join("config_home"))
        .env("VEIL_ORG_CONFIG", &org_path)
        .arg("config")
        .arg("dump")
        .arg("--layer")
        .arg("effective");

    // Org should win over User (10)
    cmd2.assert()
        .success()
        .stdout(predicate::str::contains("\"fail_on_score\": 10"));
}
