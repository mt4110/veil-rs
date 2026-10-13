use std::process::Command;
use tempfile::tempdir;

#[test]
fn explicit_lsp_reports_missing_standalone_server() {
    let directory = tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_veil"))
        .current_dir(directory.path())
        .env("PATH", directory.path())
        .env("XDG_CONFIG_HOME", directory.path())
        .env_remove("VEIL_ORG_CONFIG")
        .env_remove("VEIL_USER_CONFIG")
        .args(["lsp", "--preset", "fintech-jp"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Cannot start standalone veil-lsp"));
}

#[cfg(unix)]
#[test]
fn explicit_lsp_forwards_preset_and_keeps_stdout_for_protocol() {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempdir().unwrap();
    let executable = directory.path().join("veil-lsp");
    std::fs::write(
        &executable,
        "#!/bin/sh\nprintf '%s' \"$VEIL_LSP_CONFIG_JSON\"\n",
    )
    .unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_veil"))
        .current_dir(directory.path())
        .env("PATH", directory.path())
        .env("XDG_CONFIG_HOME", directory.path())
        .env_remove("VEIL_ORG_CONFIG")
        .env_remove("VEIL_USER_CONFIG")
        .args(["lsp", "--preset", "fintech-jp"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let received: veil_config::Config = serde_json::from_slice(&output.stdout).unwrap();
    let expected = veil_config::presets::builtin_preset_config("fintech-jp").unwrap();
    assert_eq!(
        serde_json::to_value(received.rules).unwrap(),
        serde_json::to_value(expected.rules).unwrap()
    );
    assert_eq!(received.output.max_findings, expected.output.max_findings);
    assert!(String::from_utf8_lossy(&output.stderr).contains("Using preset 'fintech-jp'"));
}
