use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn output_formats_hide_matches_unless_raw_output_is_requested() {
    let root = tempdir().unwrap();
    let home = root.path().join("home");
    fs::create_dir(&home).unwrap();
    let token = format!("AKIA{}", "A".repeat(16));
    fs::write(root.path().join("sample.txt"), format!("aws_key={token}\n")).unwrap();
    fs::write(root.path().join("safe.toml"), "").unwrap();
    fs::write(
        root.path().join("plain.toml"),
        "[output]\nmask_mode = \"plain\"\n",
    )
    .unwrap();

    for format in ["text", "json", "html", "md", "markdown", "table"] {
        for (config, options, raw) in [
            ("safe.toml", vec![], false),
            ("safe.toml", vec!["--mask-mode", "redact"], false),
            ("safe.toml", vec!["--mask-mode", "partial"], false),
            ("safe.toml", vec!["--unsafe"], true),
            ("safe.toml", vec!["--mask-mode", "plain"], true),
            ("plain.toml", vec![], false),
            ("plain.toml", vec!["--mask-mode", "redact"], false),
        ] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_veil"));
            for (key, _) in std::env::vars_os() {
                if key.to_string_lossy().starts_with("VEIL_") {
                    command.env_remove(key);
                }
            }
            let output = command
                .current_dir(root.path())
                .env("HOME", &home)
                .env("XDG_CONFIG_HOME", home.join(".config"))
                .args([
                    "--config",
                    config,
                    "scan",
                    "sample.txt",
                    "--format",
                    format,
                    "--no-color",
                    "--fail-on-findings",
                    "1",
                ])
                .args(&options)
                .output()
                .unwrap();
            let case = format!("format={format}, config={config}, options={options:?}");
            let expected_exit = if config == "plain.toml" { 2 } else { 1 };
            assert_eq!(
                output.status.code(),
                Some(expected_exit),
                "{case}: exit code"
            );
            let stdout = String::from_utf8(output.stdout).unwrap();
            let stderr = String::from_utf8(output.stderr).unwrap();
            assert_eq!(stdout.contains(&token), raw, "{case}: raw match visibility");
            assert!(!stderr.contains(&token), "{case}: raw match in diagnostics");
            if expected_exit == 1 {
                assert!(
                    stdout.contains("creds.aws.access_key_id"),
                    "{case}: missing finding"
                );
            } else {
                assert!(
                    stdout.is_empty(),
                    "{case}: rejected config produced a report"
                );
            }
            assert!(
                !root.path().join(".veil").exists(),
                "{case}: unexpected state"
            );
        }
    }
}
