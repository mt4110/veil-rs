use std::{
    fs,
    process::{Command, Output},
};
use tempfile::TempDir;

const PAGES: &[&str] = &[
    "",
    "scan",
    "filter",
    "init",
    "config",
    "config check",
    "config dump",
    "pre-commit",
    "pre-commit init",
    "git",
    "git scan",
    "doctor",
    "rules",
    "rules list",
    "rules explain",
    "rules promote-templates",
    "lsp",
    "update",
    "verify",
];

fn run(dir: &TempDir, args: &[&str], env: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_veil"));
    cmd.current_dir(dir.path()).args(args).env("LANG", "C");
    for key in [
        "VEIL_LANG",
        "LC_ALL",
        "LC_MESSAGES",
        "VEIL_REMOTE_RULES_URL",
        "VEIL_ORG_CONFIG",
        "VEIL_USER_CONFIG",
        "VEIL_ORG_RULES",
        "VEIL_FAIL_SCORE",
    ] {
        cmd.env_remove(key);
    }
    for (key, value) in env {
        cmd.env(key, value);
    }
    cmd.output().unwrap()
}

#[test]
fn all_public_help_paths_are_bilingual_and_side_effect_free() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("veil.toml"),
        "deliberately invalid TOML = [",
    )
    .unwrap();
    for lang in ["ja", "en"] {
        for page in PAGES {
            let mut args = vec!["--lang", lang];
            args.extend(page.split_whitespace());
            args.push("--help");
            let output = run(&dir, &args, &[]);
            assert!(output.status.success(), "{args:?}: {:?}", output.stderr);
            assert!(output.stderr.is_empty(), "{args:?}");
            let text = String::from_utf8(output.stdout).unwrap();
            assert!(
                text.contains(if lang == "ja" { "使い方:" } else { "Usage:" }),
                "{args:?}: {text}"
            );
            assert!(
                text.contains(if lang == "ja" {
                    "実行例:"
                } else {
                    "Examples:"
                }),
                "{args:?}"
            );
            let mut helper = vec!["--lang", lang, "help"];
            helper.extend(page.split_whitespace());
            assert_eq!(
                run(&dir, &helper, &[]).stdout,
                text.as_bytes(),
                "{helper:?}"
            );
            args.pop();
            args.push("-h");
            let short = run(&dir, &args, &[]);
            assert!(short.status.success());
            assert!(!String::from_utf8_lossy(&short.stdout).contains("Examples:"));
            assert!(!String::from_utf8_lossy(&short.stdout).contains("実行例:"));
        }
        let no_args = run(&dir, &["--lang", lang], &[]);
        assert!(no_args.status.success());
        for hidden in [
            "mask",
            "check-project",
            "ignore",
            "triage",
            "fix",
            "sot",
            "exceptions",
        ] {
            assert!(!String::from_utf8_lossy(&no_args.stdout)
                .lines()
                .any(|l| l.trim_start().starts_with(&format!("{hidden} "))));
        }
    }
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    assert_eq!(
        fs::read_to_string(dir.path().join("veil.toml")).unwrap(),
        "deliberately invalid TOML = ["
    );
}

#[test]
fn explicit_language_beats_environment_even_after_help() {
    let dir = TempDir::new().unwrap();
    for args in [
        vec!["scan", "--help", "--lang", "ja"],
        vec!["help", "scan", "--lang", "ja"],
        vec!["config", "help", "check", "--lang", "ja"],
    ] {
        let out = run(&dir, &args, &[("VEIL_LANG", "invalid"), ("LC_ALL", "C")]);
        assert!(out.status.success(), "{args:?}: {:?}", out.stderr);
        assert!(String::from_utf8_lossy(&out.stdout).contains("使い方:"));
    }
    for args in [
        vec!["--help", "--lang", "invalid"],
        vec!["help", "scan", "--lang", "invalid"],
        vec!["scan", "--lang=JA", "--help"],
    ] {
        let out = run(&dir, &args, &[]);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(out.stdout.is_empty());
    }
    assert_eq!(
        run(&dir, &["--help"], &[("VEIL_LANG", "invalid")])
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn locale_precedence_and_fallback_are_explicit() {
    let dir = TempDir::new().unwrap();
    for (values, japanese) in [
        (vec![("LANG", "ja_JP.UTF-8")], true),
        (vec![("VEIL_LANG", "ja"), ("LC_ALL", "C")], true),
        (vec![("LC_ALL", "C"), ("LANG", "ja_JP.UTF-8")], false),
        (vec![("LC_ALL", "fr_FR"), ("LANG", "ja")], false),
        (vec![("LC_ALL", ""), ("LC_MESSAGES", "ja-JP")], true),
        (vec![("LANG", "japanese")], false),
    ] {
        let out = run(&dir, &["help", "scan"], &values);
        assert!(out.status.success());
        assert_eq!(
            String::from_utf8_lossy(&out.stdout).contains("使い方:"),
            japanese
        );
    }
}

#[test]
fn language_like_values_and_paths_are_not_options() {
    let dir = TempDir::new().unwrap();
    let help = run(&dir, &["--config=--lang=ja", "--help"], &[]);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("Usage:"));
    for name in ["--lang", "ja"] {
        fs::write(dir.path().join(name), "setting=true\n").unwrap();
    }
    let out = run(
        &dir,
        &["scan", "--format", "json", "--", "--lang", "ja"],
        &[],
    );
    assert!(out.status.success(), "{:?}", out.stderr);
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["summary"]["scanned_files"], 2);
    let invalid = run(&dir, &["scan", "--lang", "ja", "--not-an-option"], &[]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
}

#[test]
fn language_does_not_change_execution_or_machine_contracts() {
    let dir = TempDir::new().unwrap();
    let secret = format!("AKIA{}", "Q".repeat(16));
    fs::write(dir.path().join("sample.txt"), format!("token={secret}\n")).unwrap();
    let mut reports = Vec::new();
    for lang in ["ja", "en"] {
        let out = run(
            &dir,
            &[
                "--lang",
                lang,
                "scan",
                "sample.txt",
                "--format",
                "json",
                "--mask-mode",
                "redact",
                "--fail-on-findings",
                "1",
            ],
            &[],
        );
        assert_eq!(out.status.code(), Some(1), "{:?}", out.stderr);
        assert!(!String::from_utf8_lossy(&out.stdout).contains(&secret));
        let mut report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(report["schemaVersion"], "veil-v1");
        // Timing and finding iteration order vary independently of help language.
        report["summary"]
            .as_object_mut()
            .unwrap()
            .remove("duration_ms");
        report["findings"]
            .as_array_mut()
            .unwrap()
            .sort_by_key(|finding| finding.to_string());
        reports.push(report);
        let version = run(&dir, &["--lang", lang, "--version"], &[]);
        assert!(version.status.success());
        assert_eq!(
            version.stdout,
            format!("veil {}\n", env!("CARGO_PKG_VERSION")).as_bytes()
        );
        assert_eq!(
            run(&dir, &["--lang", lang, "scan", "missing.txt"], &[])
                .status
                .code(),
            Some(2)
        );
    }
    assert_eq!(reports[0], reports[1]);
}
