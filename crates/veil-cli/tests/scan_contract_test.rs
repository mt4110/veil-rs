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
        .env("XDG_CONFIG_HOME", root.join("settings"));
    command
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
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
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn all_scan_commands_reject_explicit_missing_configs() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("input.txt"), "unchanged\n").unwrap();
    git(dir.path(), &["init", "-q"]);
    for args in [
        vec!["scan", "input.txt"],
        vec!["filter"],
        vec!["mask", "input.txt", "--backup-suffix", ".bak"],
        vec!["git", "scan"],
    ] {
        let output = cli(dir.path())
            .args(["--config", "missing.toml"])
            .args(&args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("Config file not found"));
        assert_eq!(
            fs::read_to_string(dir.path().join("input.txt")).unwrap(),
            "unchanged\n"
        );
        assert!(!dir.path().join("input.txt.bak").exists());
    }
}

#[test]
fn scan_filter_mask_and_git_share_layered_custom_and_pack_rules() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir(root.join("packs")).unwrap();
    fs::write(
        root.join("user.toml"),
        "[rules.\"audit.custom\"]\npattern = 'AUDIT_ONLY_TOKEN'\nenabled = true\n",
    )
    .unwrap();
    fs::write(
        root.join("packs/rules.toml"),
        "[[rules]]\nid = 'audit.pack'\npattern = 'PACK_ONLY_TOKEN'\ndescription = 'Fixture'\n",
    )
    .unwrap();
    fs::write(root.join("custom.toml"), "[core]\nrules_dir = 'packs'\n").unwrap();
    let body = "first=AUDIT_ONLY_TOKEN second=PACK_ONLY_TOKEN\r\n";
    fs::write(root.join("input.txt"), body).unwrap();
    git(root, &["init", "-q"]);
    git(root, &["add", "input.txt"]);
    git(root, &["commit", "-qm", "fixture"]);
    let configured = || {
        let mut command = cli(root);
        command
            .env("VEIL_USER_CONFIG", root.join("user.toml"))
            .args(["--config", "custom.toml"]);
        command
    };
    let scan = configured()
        .args([
            "scan",
            "input.txt",
            "--format",
            "json",
            "--fail-on-findings",
            "1",
        ])
        .output()
        .unwrap();
    assert_eq!(scan.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&scan.stdout).unwrap();
    assert_eq!(value["summary"]["new_findings"], 2);
    let history = configured().args(["git", "scan", "HEAD"]).output().unwrap();
    assert!(history.status.success());
    let output = String::from_utf8(history.stdout).unwrap();
    assert!(output.contains("audit.custom") && output.contains("audit.pack"));
    assert!(!output.contains("AUDIT_ONLY_TOKEN") && !output.contains("PACK_ONLY_TOKEN"));
    let mut filter = configured()
        .arg("filter")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    filter
        .stdin
        .take()
        .unwrap()
        .write_all(body.as_bytes())
        .unwrap();
    let filtered = filter.wait_with_output().unwrap();
    assert!(filtered.status.success());
    assert_eq!(
        String::from_utf8(filtered.stdout).unwrap(),
        "first=<REDACTED> second=<REDACTED>\n"
    );
    let dry = configured()
        .args(["mask", "input.txt", "--dry-run"])
        .output()
        .unwrap();
    assert!(dry.status.success());
    assert!(String::from_utf8_lossy(&dry.stdout).contains("Would mask 2 secrets"));
    assert_eq!(fs::read_to_string(root.join("input.txt")).unwrap(), body);
    let masked = configured()
        .args(["mask", "input.txt", "--backup-suffix", ".bak"])
        .output()
        .unwrap();
    assert!(
        masked.status.success(),
        "{}",
        String::from_utf8_lossy(&masked.stderr)
    );
    assert_eq!(
        fs::read_to_string(root.join("input.txt")).unwrap(),
        "first=<REDACTED> second=<REDACTED>\r\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("input.txt.bak")).unwrap(),
        body
    );
}

#[test]
fn mask_preserves_mixed_endings_final_newline_permissions_and_all_matches() {
    let dir = tempdir().unwrap();
    let token = format!("AKIA{}", "A".repeat(16));
    for ending in ["", "\n", "\r\n"] {
        let body = format!("safe\r\nfirst={token} repeat={token}\nlast={token}{ending}");
        let path = dir.path().join("input.txt");
        fs::write(&path, body).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        }
        let output = cli(dir.path())
            .args(["mask", "input.txt"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            format!("safe\r\nfirst=<REDACTED> repeat=<REDACTED>\nlast=<REDACTED>{ending}")
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o640
            );
        }
    }
}

#[test]
fn mask_preflights_every_input_and_refuses_incomplete_scans() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let body = format!("key=AKIA{}\n", "A".repeat(16));
    fs::write(root.join("first.txt"), &body).unwrap();
    for (config, invalid) in [
        ("", vec![0xff]),
        ("[core]\nmax_file_size = 30\n", vec![b'x'; 31]),
        ("[core]\nmax_file_count = 1\n", b"safe".to_vec()),
        ("[output]\nmax_findings = 1\n", b"safe".to_vec()),
    ] {
        fs::write(root.join("veil.toml"), config).unwrap();
        fs::write(root.join("second.txt"), &invalid).unwrap();
        for extra in [vec!["--dry-run"], vec!["--backup-suffix", ".bak"]] {
            let output = cli(root)
                .args(["mask", "first.txt", "second.txt"])
                .args(extra)
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(2));
            assert_eq!(fs::read_to_string(root.join("first.txt")).unwrap(), body);
            assert_eq!(fs::read(root.join("second.txt")).unwrap(), invalid);
            assert!(!root.join("first.txt.bak").exists());
        }
    }
}

#[test]
fn mask_refuses_existing_backups_collisions_and_duplicate_inputs() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let body = format!("key=AKIA{}\n", "A".repeat(16));
    fs::write(root.join("input.txt"), &body).unwrap();
    fs::write(root.join("input.txt.bak"), "retain backup").unwrap();
    for args in [
        vec!["mask", "input.txt", "--backup-suffix", ".bak"],
        vec![
            "mask",
            "input.txt",
            "input.txt.bak",
            "--backup-suffix",
            ".bak",
        ],
        vec!["mask", "input.txt", "./input.txt"],
        vec!["mask", "input.txt", "--backup-suffix", ""],
        vec!["mask", "input.txt", "--backup-suffix", "/other"],
        vec!["mask", "."],
    ] {
        let output = cli(root).args(&args).output().unwrap();
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert_eq!(fs::read_to_string(root.join("input.txt")).unwrap(), body);
        assert_eq!(
            fs::read_to_string(root.join("input.txt.bak")).unwrap(),
            "retain backup"
        );
    }
}

#[cfg(unix)]
#[test]
fn mask_refuses_symlink_inputs_and_backup_destinations() {
    use std::os::unix::fs::symlink;
    let dir = tempdir().unwrap();
    let root = dir.path();
    let body = format!("key=AKIA{}\n", "A".repeat(16));
    fs::write(root.join("input.txt"), &body).unwrap();
    symlink("input.txt", root.join("link.txt")).unwrap();
    symlink("missing.txt", root.join("input.txt.bak")).unwrap();
    for args in [
        vec!["mask", "link.txt"],
        vec!["mask", "input.txt", "--backup-suffix", ".bak"],
    ] {
        assert_eq!(
            cli(root).args(args).output().unwrap().status.code(),
            Some(2)
        );
        assert_eq!(fs::read_to_string(root.join("input.txt")).unwrap(), body);
    }
    assert!(fs::symlink_metadata(root.join("input.txt.bak"))
        .unwrap()
        .file_type()
        .is_symlink());
}

#[test]
fn git_scan_rejects_incomplete_blobs_and_limits_but_keeps_binary_exclusions() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    git(root, &["init", "-q"]);
    fs::write(root.join("input.txt"), [b'x', 0xff, b'\n']).unwrap();
    git(root, &["add", "input.txt"]);
    git(root, &["commit", "-qm", "invalid fixture"]);
    for config in ["", "[core]\nmax_file_size = 1\n"] {
        fs::write(root.join("veil.toml"), config).unwrap();
        assert_eq!(
            cli(root)
                .args(["git", "scan", "HEAD"])
                .output()
                .unwrap()
                .status
                .code(),
            Some(2)
        );
    }
    fs::write(root.join("input.txt"), [0_u8, 0xff]).unwrap();
    git(root, &["add", "input.txt"]);
    git(root, &["commit", "-qm", "binary fixture"]);
    // Range selects only the new commit; the older invalid blob remains an error in full history.
    assert!(cli(root)
        .args(["git", "scan", "HEAD~1..HEAD"])
        .output()
        .unwrap()
        .status
        .success());
    fs::write(root.join("one.txt"), format!("AKIA{}\n", "A".repeat(16))).unwrap();
    fs::write(root.join("two.txt"), "safe\n").unwrap();
    git(root, &["add", "one.txt", "two.txt"]);
    git(root, &["commit", "-qm", "text fixture"]);
    for config in [
        "[core]\nmax_file_count = 1\n",
        "[output]\nmax_findings = 1\n",
    ] {
        fs::write(root.join("veil.toml"), config).unwrap();
        assert_eq!(
            cli(root)
                .args(["git", "scan", "HEAD~1..HEAD"])
                .output()
                .unwrap()
                .status
                .code(),
            Some(2)
        );
    }
}

#[test]
fn malformed_rule_pack_is_an_error_for_all_scanning_commands() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir(root.join("packs")).unwrap();
    fs::write(root.join("packs/rules.toml"), "invalid TOML").unwrap();
    fs::write(root.join("veil.toml"), "[core]\nrules_dir = 'packs'\n").unwrap();
    fs::write(root.join("input.txt"), "unchanged\n").unwrap();
    git(root, &["init", "-q"]);
    for args in [
        vec!["scan", "input.txt"],
        vec!["filter"],
        vec!["mask", "input.txt", "--backup-suffix", ".bak"],
        vec!["git", "scan"],
    ] {
        assert_eq!(
            cli(root).args(args).output().unwrap().status.code(),
            Some(2)
        );
        assert_eq!(
            fs::read_to_string(root.join("input.txt")).unwrap(),
            "unchanged\n"
        );
        assert!(!root.join("input.txt.bak").exists());
    }
}

#[cfg(unix)]
#[test]
fn mask_keeps_readonly_and_explicitly_ignored_files_unchanged() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempdir().unwrap();
    let root = dir.path();
    let body = format!("key=AKIA{}\n", "A".repeat(16));
    fs::write(root.join("input.txt"), &body).unwrap();
    fs::set_permissions(root.join("input.txt"), fs::Permissions::from_mode(0o440)).unwrap();
    assert_eq!(
        cli(root)
            .args(["mask", "input.txt"])
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
    assert_eq!(fs::read_to_string(root.join("input.txt")).unwrap(), body);
    fs::write(root.join("veil.toml"), "[core]\nignore = ['input.txt']\n").unwrap();
    assert!(cli(root)
        .args(["mask", "input.txt"])
        .output()
        .unwrap()
        .status
        .success());
    assert_eq!(fs::read_to_string(root.join("input.txt")).unwrap(), body);
}

#[test]
fn regular_git_scan_modes_apply_file_count_limits_and_preserve_baselines() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    git(root, &["init", "-q"]);
    fs::write(root.join("first.txt"), "safe\n").unwrap();
    fs::write(root.join("second.txt"), "safe\n").unwrap();
    fs::write(root.join("veil.toml"), "[core]\nmax_file_count = 1\n").unwrap();
    git(root, &["add", "first.txt", "second.txt"]);
    let baseline = root.join("baseline.json");
    fs::write(&baseline, "retain existing baseline").unwrap();
    let staged = cli(root)
        .args(["scan", "--staged", "--write-baseline", "baseline.json"])
        .output()
        .unwrap();
    assert_eq!(staged.status.code(), Some(2));
    assert_eq!(
        fs::read_to_string(&baseline).unwrap(),
        "retain existing baseline"
    );
    git(root, &["commit", "-qm", "fixture"]);
    for mode in [vec!["--commit", "HEAD"], vec!["--since", "2000-01-01"]] {
        let output = cli(root)
            .arg("scan")
            .args(mode)
            .args(["--format", "json", "--write-baseline", "baseline.json"])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(
            fs::read_to_string(&baseline).unwrap(),
            "retain existing baseline"
        );
    }
}

#[test]
fn ignored_mask_inputs_do_not_consume_the_file_budget() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let body = format!("key=AKIA{}\n", "A".repeat(16));
    fs::write(root.join("secret.txt"), &body).unwrap();
    let ignored = [b'x', 0xff, b'\n'];
    fs::write(root.join("ignored.txt"), ignored).unwrap();
    fs::write(
        root.join("veil.toml"),
        "[core]\nmax_file_count = 1\nignore = ['ignored.txt']\n",
    )
    .unwrap();
    let dry_run = cli(root)
        .args(["mask", "secret.txt", "ignored.txt", "--dry-run"])
        .output()
        .unwrap();
    assert!(
        dry_run.status.success(),
        "{}",
        String::from_utf8_lossy(&dry_run.stderr)
    );
    assert!(String::from_utf8_lossy(&dry_run.stdout).contains("Would mask 1 secrets"));
    assert_eq!(fs::read_to_string(root.join("secret.txt")).unwrap(), body);
    let masked = cli(root)
        .args([
            "mask",
            "secret.txt",
            "ignored.txt",
            "--backup-suffix",
            ".bak",
        ])
        .output()
        .unwrap();
    assert!(
        masked.status.success(),
        "{}",
        String::from_utf8_lossy(&masked.stderr)
    );
    assert_eq!(
        fs::read_to_string(root.join("secret.txt")).unwrap(),
        "key=<REDACTED>\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("secret.txt.bak")).unwrap(),
        body
    );
    assert_eq!(fs::read(root.join("ignored.txt")).unwrap(), ignored);
    assert!(!root.join("ignored.txt.bak").exists());
}
