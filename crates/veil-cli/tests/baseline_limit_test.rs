use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

const EXISTING_BASELINE: &[u8] = br#"{"schema":"veil.baseline.v1","generated_at":"2026-10-12T00:00:00Z","tool":"previous-baseline","entries":[]}"#;

fn veil(root: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_veil"));
    command.current_dir(root);
    for name in [
        "VEIL_REMOTE_RULES_URL",
        "VEIL_ORG_CONFIG",
        "VEIL_USER_CONFIG",
        "VEIL_ORG_RULES",
        "VEIL_FAIL_SCORE",
    ] {
        command.env_remove(name);
    }
    command.env("XDG_CONFIG_HOME", root.join("settings"));
    command
}

fn write_inputs(root: &Path) {
    fs::create_dir(root.join("input")).unwrap();
    for (name, suffix) in [("first.txt", "A"), ("second.txt", "C")] {
        let token = format!("AKIA{}", suffix.repeat(16));
        fs::write(root.join("input").join(name), format!("aws_key={token}\n")).unwrap();
    }
}

enum Limit {
    FindingsArgument,
    FindingsConfig,
    Files,
    Oversized,
    InvalidUtf8,
}

fn rejects_incomplete_baseline(limit: Limit, existing: bool) {
    let dir = tempdir().unwrap();
    let root = dir.path();
    write_inputs(root);
    if matches!(limit, Limit::InvalidUtf8) {
        fs::write(root.join("input/invalid.txt"), [0xff, 0xfe, b'x']).unwrap();
    }
    let destination = root.join("baseline.json");
    if existing {
        fs::write(&destination, EXISTING_BASELINE).unwrap();
    }
    let config = match limit {
        Limit::FindingsArgument => "[core]\nmax_file_count = 10\n",
        Limit::FindingsConfig => "[core]\nmax_file_count = 10\n[output]\nmax_findings = 1\n",
        Limit::Files => "[core]\nmax_file_count = 1\n",
        Limit::Oversized => "[core]\nmax_file_size = 8\n",
        Limit::InvalidUtf8 => "[core]\nmax_file_count = 10\n",
    };
    fs::write(root.join("veil.toml"), config).unwrap();
    let mut command = veil(root);
    command.args([
        "scan",
        "input",
        "--write-baseline",
        "baseline.json",
        "--format",
        "json",
    ]);
    if matches!(limit, Limit::FindingsArgument) {
        command.args(["--limit", "1"]);
    }
    let output = command.output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Cannot write baseline: scan incomplete"));
    assert!(!stderr.contains("Baseline written to"));
    if existing {
        assert_eq!(fs::read(&destination).unwrap(), EXISTING_BASELINE);
    } else {
        assert!(!destination.exists());
    }
}

#[test]
fn finding_argument_limit_does_not_create_baseline() {
    rejects_incomplete_baseline(Limit::FindingsArgument, false);
}

#[test]
fn finding_argument_limit_preserves_existing_baseline() {
    rejects_incomplete_baseline(Limit::FindingsArgument, true);
}

#[test]
fn configured_finding_limit_does_not_create_baseline() {
    rejects_incomplete_baseline(Limit::FindingsConfig, false);
}

#[test]
fn configured_finding_limit_preserves_existing_baseline() {
    rejects_incomplete_baseline(Limit::FindingsConfig, true);
}

#[test]
fn file_limit_does_not_create_baseline() {
    rejects_incomplete_baseline(Limit::Files, false);
}

#[test]
fn file_limit_preserves_existing_baseline() {
    rejects_incomplete_baseline(Limit::Files, true);
}

#[test]
fn complete_scan_writes_baseline_even_with_finding_failure_policy() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    write_inputs(root);
    fs::write(
        root.join("veil.toml"),
        "[core]\nmax_file_count = 3\n[output]\nmax_findings = 100\n",
    )
    .unwrap();
    let scan = veil(root)
        .args(["scan", "input", "--format", "json"])
        .output()
        .unwrap();
    assert!(scan.status.success());
    let report: serde_json::Value = serde_json::from_slice(&scan.stdout).unwrap();
    assert_eq!(report["summary"]["scanned_files"], 2);
    assert_eq!(report["summary"]["limit_reached"], false);
    assert_eq!(report["summary"]["file_limit_reached"], false);
    let count = report["summary"]["total_findings"].as_u64().unwrap();
    assert!(count > 0);

    let output = veil(root)
        .args([
            "scan",
            "input",
            "--write-baseline",
            "baseline.json",
            "--fail-on-findings",
            "1",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Baseline written to"));
    assert!(output.stdout.is_empty());
    let content = fs::read_to_string(root.join("baseline.json")).unwrap();
    let snapshot: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(snapshot["schema"], "veil.baseline.v1");
    assert_eq!(snapshot["entries"].as_array().unwrap().len() as u64, count);
    for suffix in ["A", "C"] {
        assert!(!content.contains(&format!("AKIA{}", suffix.repeat(16))));
    }
}

#[test]
fn complete_empty_scan_can_write_empty_baseline() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir(root.join("input")).unwrap();
    let output = veil(root)
        .args(["scan", "input", "--write-baseline", "baseline.json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let snapshot: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("baseline.json")).unwrap()).unwrap();
    assert_eq!(snapshot["entries"], serde_json::json!([]));
}

#[test]
fn oversized_file_does_not_create_baseline() {
    rejects_incomplete_baseline(Limit::Oversized, false);
}
#[test]
fn oversized_file_preserves_existing_baseline() {
    rejects_incomplete_baseline(Limit::Oversized, true);
}
#[test]
fn unreadable_utf8_does_not_create_baseline() {
    rejects_incomplete_baseline(Limit::InvalidUtf8, false);
}
#[test]
fn unreadable_utf8_preserves_existing_baseline() {
    rejects_incomplete_baseline(Limit::InvalidUtf8, true);
}
