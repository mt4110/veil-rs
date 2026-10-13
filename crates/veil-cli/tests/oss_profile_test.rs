use std::fs;
use std::net::TcpListener;
use std::process::Command;
use tempfile::tempdir;

fn veil() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_veil"));
    command.env_remove("VEIL_REMOTE_RULES_URL");
    command
}

#[test]
fn help_exposes_local_entry_points() {
    let output = veil().arg("--help").output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    for name in [
        "scan",
        "filter",
        "init",
        "config",
        "rules",
        "doctor",
        "git",
        "pre-commit",
    ] {
        assert!(text
            .lines()
            .any(|line| line.trim_start().starts_with(&format!("{name} "))));
    }
    assert!(!text
        .lines()
        .any(|line| line.trim_start().starts_with("guardian ")));
    assert!(text
        .lines()
        .any(|line| line.trim_start().starts_with("verify ")));
    assert!(!text
        .lines()
        .any(|line| line.trim_start().starts_with("detect ")));
}

#[test]
fn json_scan_is_masked_and_does_not_create_state() {
    let dir = tempdir().unwrap();
    let token = format!("AKIA{}", "A".repeat(16));
    fs::write(
        dir.path().join("sample.txt"),
        format!("private-neighbor-marker\naws_key={token}\n"),
    )
    .unwrap();
    let output = veil()
        .current_dir(dir.path())
        .args(["scan", "sample.txt", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schemaVersion"], "veil-v1");
    assert!(!report["findings"].as_array().unwrap().is_empty());
    assert!(!String::from_utf8_lossy(&output.stdout).contains(&token));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("private-neighbor-marker"));
    assert!(!dir.path().join(".veil").exists());
    assert!(report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| !finding["context_before"].as_array().unwrap().is_empty()));
    for finding in report["findings"].as_array().unwrap() {
        assert_eq!(finding["matched_content"], "<REDACTED>");
        assert_eq!(finding["line_content"], finding["masked_snippet"]);
        for field in ["context_before", "context_after"] {
            assert!(finding[field]
                .as_array()
                .unwrap()
                .iter()
                .all(|line| line == "<REDACTED>"));
        }
    }
    let raw = veil()
        .current_dir(dir.path())
        .args(["scan", "sample.txt", "--format", "json", "--unsafe"])
        .output()
        .unwrap();
    assert!(raw.status.success());
    assert!(String::from_utf8_lossy(&raw.stdout).contains(&token));
}

#[test]
fn retired_remote_configuration_fails_without_contacting_the_endpoint() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}/private-marker", listener.local_addr().unwrap());
    for args in [
        vec!["scan", "."],
        vec!["rules", "list"],
        vec!["config", "check"],
    ] {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("veil.toml"),
            format!("[core]\nremote_rules_url = {url:?}\n"),
        )
        .unwrap();
        let output = veil().current_dir(dir.path()).args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("Remote rules are disabled"));
        assert!(!error.contains("private-marker"));
        assert!(matches!(listener.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock));
    }
    let dir = tempdir().unwrap();
    let output = veil()
        .current_dir(dir.path())
        .env("VEIL_REMOTE_RULES_URL", &url)
        .args(["scan", "."])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(matches!(listener.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock));
}

#[test]
fn update_requires_a_local_signed_pack() {
    let dir = tempdir().unwrap();
    for args in [
        vec!["update"],
        vec!["update", "--url", "https://127.0.0.1/unused"],
    ] {
        let output = veil().current_dir(dir.path()).args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("--local-pack"));
        assert!(!dir.path().join(".veil").exists());
    }
}

#[test]
fn missing_paths_and_unknown_formats_are_errors() {
    let dir = tempdir().unwrap();
    let output = veil()
        .current_dir(dir.path())
        .args(["scan", "missing", "--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Cannot access scan path"));
    let output = veil()
        .current_dir(dir.path())
        .args(["scan", ".", "--format", "jsno"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid value"));
}
