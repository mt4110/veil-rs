use git2::{Oid, Repository, Signature};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::tempdir;

const KNOWN: &str = "aws_key = AKIA1234567890123456\n";
const NEW: &str = "other_key = AKIA9999999999999999\n";
const PREVIOUS_BASELINE: &[u8] = br#"{"schema":"veil.baseline.v1","generated_at":"2026-10-12T00:00:00Z","tool":"previous-baseline","entries":[]}"#;

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
    command.args(["--config", "veil.toml"]);
    command
}

fn config(root: &Path, extra: &str) {
    fs::write(
        root.join("veil.toml"),
        format!("[core]\nmax_file_count = 100\n{extra}"),
    )
    .unwrap();
}

fn report(output: &Output, expected_code: i32) -> Value {
    assert_eq!(
        output.status.code(),
        Some(expected_code),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schemaVersion"], "veil-v1");
    report
}

fn make_baseline(root: &Path) -> Vec<u8> {
    let output = veil(root)
        .args(["scan", "known.txt", "--write-baseline", "baseline.json"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    fs::read(root.join("baseline.json")).unwrap()
}

fn baseline_limit(configured: bool, multiple_targets: bool) {
    let dir = tempdir().unwrap();
    let root = dir.path();
    config(root, "");
    fs::write(root.join("known.txt"), KNOWN).unwrap();
    let baseline = make_baseline(root);
    if multiple_targets {
        fs::write(root.join("new.txt"), NEW).unwrap();
    } else {
        fs::write(root.join("known.txt"), format!("{KNOWN}{NEW}")).unwrap();
    }
    if configured {
        config(root, "[output]\nmax_findings = 1\n");
    }
    let mut command = veil(root);
    command.args(["scan", "known.txt"]);
    if multiple_targets {
        command.arg("new.txt");
    }
    command.args(["--baseline", "baseline.json", "--format", "json"]);
    if !configured {
        command.args(["--limit", "1"]);
    }
    let result = report(&command.output().unwrap(), 2);
    assert_eq!(result["summary"]["limit_reached"], true);
    assert_eq!(result["summary"]["baseline_suppressed"], 1);
    assert_eq!(result["summary"]["new_findings"], 0);
    assert_eq!(result["findings"].as_array().unwrap().len(), 0);
    assert_eq!(fs::read(root.join("baseline.json")).unwrap(), baseline);
}

#[test]
fn baseline_suppression_preserves_cli_finding_limit() {
    baseline_limit(false, false);
}

#[test]
fn baseline_suppression_preserves_configured_finding_limit() {
    baseline_limit(true, false);
}

#[test]
fn baseline_suppression_preserves_limit_across_targets() {
    baseline_limit(false, true);
}

#[test]
fn complete_baseline_scan_still_succeeds() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    config(root, "");
    fs::write(root.join("known.txt"), KNOWN).unwrap();
    let baseline = make_baseline(root);
    let output = veil(root)
        .args([
            "scan",
            "known.txt",
            "--baseline",
            "baseline.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let result = report(&output, 0);
    assert_eq!(result["summary"]["limit_reached"], false);
    assert_eq!(result["summary"]["baseline_suppressed"], 1);
    assert_eq!(fs::read(root.join("baseline.json")).unwrap(), baseline);
}

#[test]
fn unlimited_cli_override_keeps_new_findings_after_baseline() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    config(root, "");
    fs::write(root.join("known.txt"), KNOWN).unwrap();
    make_baseline(root);
    fs::write(root.join("known.txt"), format!("{KNOWN}{NEW}")).unwrap();
    config(root, "[output]\nmax_findings = 1\n");
    let output = veil(root)
        .args([
            "scan",
            "known.txt",
            "--baseline",
            "baseline.json",
            "--limit",
            "0",
            "--format",
            "json",
            "--fail-on-findings",
            "1",
        ])
        .output()
        .unwrap();
    let result = report(&output, 1);
    assert_eq!(result["summary"]["limit_reached"], false);
    assert_eq!(result["summary"]["baseline_suppressed"], 1);
    assert_eq!(result["summary"]["new_findings"], 1);
}

fn commit_file(repo: &Repository, content: &str) -> Oid {
    fs::write(repo.workdir().unwrap().join("secret.txt"), content).unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("secret.txt")).unwrap();
    index.write().unwrap();
    let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
    let signature = Signature::now("Fixture", "fixture@example.invalid").unwrap();
    let parent = repo.head().ok().map(|head| head.peel_to_commit().unwrap());
    let parents: Vec<_> = parent.iter().collect();
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Fixture",
        &tree,
        &parents,
    )
    .unwrap()
}

fn scan_commit(root: &Path, oid: Oid) -> Output {
    veil(root)
        .args([
            "scan",
            "--commit",
            &oid.to_string(),
            "--format",
            "json",
            "--fail-on-findings",
            "1",
        ])
        .output()
        .unwrap()
}

#[test]
fn first_commit_is_scanned() {
    let dir = tempdir().unwrap();
    config(dir.path(), "");
    let repo = Repository::init(dir.path()).unwrap();
    let oid = commit_file(&repo, KNOWN);
    assert_eq!(repo.find_commit(oid).unwrap().parent_count(), 0);
    let result = report(&scan_commit(dir.path(), oid), 1);
    assert_eq!(result["summary"]["scanned_files"], 1);
    assert_eq!(result["summary"]["new_findings"], 1);
    assert_eq!(result["summary"]["limit_reached"], false);
    assert_eq!(result["findings"][0]["path"], "secret.txt");
}

#[test]
fn parented_commit_still_scans_its_changes() {
    let dir = tempdir().unwrap();
    config(dir.path(), "");
    let repo = Repository::init(dir.path()).unwrap();
    commit_file(&repo, "safe=true\n");
    let oid = commit_file(&repo, NEW);
    assert_eq!(repo.find_commit(oid).unwrap().parent_count(), 1);
    let result = report(&scan_commit(dir.path(), oid), 1);
    assert_eq!(result["summary"]["scanned_files"], 1);
    assert_eq!(result["summary"]["new_findings"], 1);
}

#[test]
fn complete_first_commit_baseline_contains_findings() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    config(root, "");
    let repo = Repository::init(root).unwrap();
    let oid = commit_file(&repo, KNOWN);
    let output = veil(root)
        .args([
            "scan",
            "--commit",
            &oid.to_string(),
            "--write-baseline",
            "baseline.json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let baseline: Value =
        serde_json::from_slice(&fs::read(root.join("baseline.json")).unwrap()).unwrap();
    assert_eq!(baseline["entries"].as_array().unwrap().len(), 1);
    assert_eq!(baseline["entries"][0]["path"], "secret.txt");
}

fn incomplete_first_commit_baseline(existing: bool, oversized: bool) {
    let dir = tempdir().unwrap();
    let root = dir.path();
    config(root, if oversized { "max_file_size = 8\n" } else { "" });
    let repo = Repository::init(root).unwrap();
    let oid = commit_file(&repo, KNOWN);
    if existing {
        fs::write(root.join("baseline.json"), PREVIOUS_BASELINE).unwrap();
    }
    let mut command = veil(root);
    command.args([
        "scan",
        "--commit",
        &oid.to_string(),
        "--write-baseline",
        "baseline.json",
    ]);
    if !oversized {
        command.args(["--limit", "1"]);
    }
    let output = command.output().unwrap();
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Cannot write baseline: scan incomplete")
    );
    if existing {
        assert_eq!(
            fs::read(root.join("baseline.json")).unwrap(),
            PREVIOUS_BASELINE
        );
    } else {
        assert!(!root.join("baseline.json").exists());
    }
}

#[test]
fn first_commit_finding_limit_does_not_create_baseline() {
    incomplete_first_commit_baseline(false, false);
}

#[test]
fn first_commit_finding_limit_preserves_existing_baseline() {
    incomplete_first_commit_baseline(true, false);
}

#[test]
fn first_commit_size_limit_does_not_create_baseline() {
    incomplete_first_commit_baseline(false, true);
}

#[test]
fn first_commit_size_limit_preserves_existing_baseline() {
    incomplete_first_commit_baseline(true, true);
}

#[test]
fn history_scan_includes_first_commit() {
    let dir = tempdir().unwrap();
    config(dir.path(), "");
    let repo = Repository::init(dir.path()).unwrap();
    commit_file(&repo, KNOWN);
    let output = veil(dir.path())
        .args([
            "scan",
            "--since",
            "1990-01-01",
            "--format",
            "json",
            "--fail-on-findings",
            "1",
        ])
        .output()
        .unwrap();
    let result = report(&output, 1);
    assert_eq!(result["summary"]["scanned_files"], 1);
    assert_eq!(result["summary"]["new_findings"], 1);
}
