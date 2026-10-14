use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::tempdir;

fn cli(root: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_veil"));
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("VEIL_") {
            command.env_remove(key);
        }
    }
    command
        .current_dir(root)
        .env("HOME", root.join("home"))
        .env("XDG_CONFIG_HOME", root.join("settings"))
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_CONFIG_NOSYSTEM", "1");
    command
}

#[test]
fn interactive_preview_hides_all_matches_on_the_source_line_without_writing() {
    let dir = tempdir().unwrap();
    let first = format!("AKIA{}", "A".repeat(16));
    let second = format!("AKIA{}", "C".repeat(16));
    let body = format!("first={first} second={second} repeated={first}\n");
    fs::write(dir.path().join("input.txt"), &body).unwrap();
    let mut child = cli(dir.path())
        .args([
            "scan",
            "input.txt",
            "--interactive",
            "--mask-mode",
            "redact",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"quit\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("Mask preview:"));
    assert!(text.contains("- <sensitive content hidden>"));
    assert!(text.contains("<REDACTED>"));
    for value in [&first, &second] {
        assert!(!text.contains(value));
        assert!(!String::from_utf8_lossy(&output.stderr).contains(value));
    }
    assert_eq!(
        fs::read_to_string(dir.path().join("input.txt")).unwrap(),
        body
    );
}

#[test]
fn inline_ignore_requires_an_exact_directive_in_an_unquoted_comment() {
    let dir = tempdir().unwrap();
    let token = format!("AKIA{}", "A".repeat(16));
    for (suffix, expected) in [
        ("// veil:ignore", 0),
        ("# veil:ignore=creds.aws.access_key_id", 0),
        ("// veil:ignore=creds.aws.access_key_id reason", 0),
        ("// veil:ignore=creds.aws.access_key_id_extra", 1),
        ("// veil:ignore=other", 1),
        ("// veil:ignore_extra", 1),
        ("note=\"// veil:ignore\"", 1),
        ("note='# veil:ignore'", 1),
        ("note=`// veil:ignore`", 1),
        ("note=\"escaped \\\" // veil:ignore\"", 1),
        ("note=\"# veil:ignore\" // veil:ignore", 0),
        ("// unrelated veil:ignore", 1),
    ] {
        fs::write(
            dir.path().join("input.txt"),
            format!("key={token} {suffix}\n"),
        )
        .unwrap();
        let output = cli(dir.path())
            .args([
                "scan",
                "input.txt",
                "--format",
                "json",
                "--mask-mode",
                "redact",
                "--fail-on-findings",
                "1",
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(expected), "{suffix}");
        assert!(!String::from_utf8_lossy(&output.stdout).contains(&token));
    }
}

#[test]
fn malformed_config_diagnostics_hide_source_and_mask_never_falls_back() {
    let dir = tempdir().unwrap();
    let token = format!("AKIA{}", "A".repeat(16));
    let body = format!("key={token}\n");
    fs::write(dir.path().join("input.txt"), &body).unwrap();
    // Include non-ASCII text to exercise byte-span to line/column conversion.
    fs::write(
        dir.path().join("bad.toml"),
        format!("# 日本語\nsecret=\"{token}\" trailing\n"),
    )
    .unwrap();
    for args in [
        vec!["scan", "input.txt"],
        vec!["mask", "input.txt", "--dry-run"],
        vec!["mask", "input.txt", "--backup-suffix", ".bak"],
    ] {
        let output = cli(dir.path())
            .args(["--config", "bad.toml"])
            .args(&args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(!error.contains(&token));
        assert!(!error.contains("trailing"));
        assert!(error.contains("line 2"));
        assert!(error.contains("column"));
        assert_eq!(
            fs::read_to_string(dir.path().join("input.txt")).unwrap(),
            body
        );
        assert!(!dir.path().join("input.txt.bak").exists());
    }
}

#[test]
fn invalid_utf8_git_inputs_are_incomplete_and_preserve_baselines() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let git = |args: &[&str]| {
        let status = Command::new("git")
            .current_dir(root)
            .env(
                "GIT_CONFIG_GLOBAL",
                if cfg!(windows) { "NUL" } else { "/dev/null" },
            )
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .args([
                "-c",
                "core.hooksPath=/dev/null",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(status.status.success(), "git {args:?}");
    };
    git(&["init", "-q"]);
    fs::write(root.join("clean.txt"), "clean\n").unwrap();
    git(&["add", "clean.txt"]);
    git(&["commit", "-qm", "clean"]);
    let token = format!("AKIA{}", "A".repeat(16));
    let mut body = format!("key={token}\n").into_bytes();
    body.push(0xff);
    fs::write(root.join("bad.txt"), &body).unwrap();
    git(&["add", "bad.txt"]);
    let index = fs::read(root.join(".git/index")).unwrap();
    let staged = cli(root)
        .args(["scan", "--staged", "--format", "json"])
        .output()
        .unwrap();
    assert_eq!(staged.status.code(), Some(2));
    assert_eq!(fs::read(root.join(".git/index")).unwrap(), index);
    git(&["commit", "-qm", "invalid utf8"]);
    for args in [
        vec!["--staged"],
        vec!["--commit", "HEAD"],
        vec!["--since", "HEAD~1"],
    ] {
        // Make the staged delta point at the same invalid text again.
        if args == ["--staged"] {
            body.push(0xfe);
            fs::write(root.join("bad.txt"), &body).unwrap();
            git(&["add", "bad.txt"]);
        }
        for existing in [false, true] {
            let name = if existing {
                "existing.json"
            } else {
                "new.json"
            };
            if existing {
                fs::write(root.join(name), "preserved bytes").unwrap();
            }
            let output = cli(root)
                .args(["scan"])
                .args(&args)
                .args(["--write-baseline", name])
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(2), "{args:?}");
            if existing {
                assert_eq!(
                    fs::read_to_string(root.join(name)).unwrap(),
                    "preserved bytes"
                );
            } else {
                assert!(!root.join(name).exists());
            }
        }
    }
}
