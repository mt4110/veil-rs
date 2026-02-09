use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn test_exceptions_flag_exclusivity() {
    // Test that --system-registry and --registry-path cannot be used together
    let mut cmd = cargo_bin_cmd!("veil");
    let result = cmd
        .args(&[
            "exceptions",
            "--system-registry",
            "--registry-path",
            "foo.toml",
            "list",
        ])
        .assert()
        .failure();

    // Verify error message contains contract text
    let output = String::from_utf8_lossy(&result.get_output().stderr);
    assert!(
        output.contains("cannot be used with"),
        "Error message should indicate mutual exclusivity. Got: {}",
        output
    );
    assert!(
        output.contains("--system-registry") || output.contains("system-registry"),
        "Error should mention --system-registry. Got: {}",
        output
    );
    assert!(
        output.contains("--registry-path") || output.contains("registry-path"),
        "Error should mention --registry-path. Got: {}",
        output
    );
}

#[test]
fn test_exceptions_help_shows_all_flags() {
    let mut cmd = cargo_bin_cmd!("veil");
    let result = cmd.args(&["exceptions", "--help"]).assert().success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    
    // Verify all three key flags are documented
    assert!(
        output.contains("--system-registry"),
        "Help should show --system-registry"
    );
    assert!(
        output.contains("--registry-path"),
        "Help should show --registry-path"
    );
    assert!(
        output.contains("--strict-exceptions"),
        "Help should show --strict-exceptions"
    );
}

#[test]
fn test_system_registry_flag_is_boolean() {
    // Verify that --system-registry doesn't take a value
    let mut cmd = cargo_bin_cmd!("veil");
    let result = cmd
        .args(&["exceptions", "--system-registry", "list"])
        .assert();
    
    // Should succeed (list command will fail due to missing registry, but arg parsing succeeds)
    // We're just verifying the flag doesn't expect a value
    let output = String::from_utf8_lossy(&result.get_output().stderr);
    assert!(
        !output.contains("requires a value") && !output.contains("expected a value"),
        "Flag should not require a value. Got: {}",
        output
    );
}
