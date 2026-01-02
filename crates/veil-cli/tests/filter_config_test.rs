use assert_cmd::Command;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_filter_config_default_placeholder() {
    // case: default placeholder from config is applied (<REDACTED>)
    // Config not provided -> should use default, but we can provide an empty one to check loading
    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.arg("filter");
    
    let input = "aws_key=AKIA1234567890123456";
    cmd.write_stdin(input);

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("aws_key=<REDACTED>"));
}

#[test]
fn test_filter_config_custom_placeholder() {
    // case: custom placeholder in config
    let mut config_file = NamedTempFile::new().unwrap();
    writeln!(config_file, r#"
[masking]
placeholder = "[SECRET]"
"#).unwrap();

    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.arg("filter")
       .arg("--config")
       .arg(config_file.path());

    let input = "aws_key=AKIA1234567890123456";
    cmd.write_stdin(input);

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("aws_key=[SECRET]"));
}

#[test]
fn test_filter_config_rule_override() {
    // case: override rule pattern in config changes masking
    // Using a rule that doesn't match by default, or matching something else
    // Let's create a custom rule in config
    let mut config_file = NamedTempFile::new().unwrap();
    writeln!(config_file, r#"
[rules.custom-foo]
pattern = "foo"
enabled = true
"#).unwrap();

    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.arg("filter")
       .arg("--config")
       .arg(config_file.path());

    let input = "some foo bar";
    cmd.write_stdin(input);

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("some <REDACTED> bar"));
}

#[test]
fn test_filter_exit_code_zero() {
    // case: exit code is 0 even when masked output occurs
    let mut cmd = Command::cargo_bin("veil").unwrap();
    cmd.arg("filter");
    
    let input = "aws_key=AKIA1234567890123456";
    cmd.write_stdin(input);

    cmd.assert()
        .success(); // Means exit code 0
}
