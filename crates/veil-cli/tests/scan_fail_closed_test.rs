use git2::{IndexEntry, IndexTime, ObjectType, Oid, Repository, Signature};
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::process::{Command, Output, Stdio};
use tempfile::tempdir;

const KNOWN: &str = "aws_key = AKIA1234567890123456\n";
const NEW: &str = "other_key = AKIA9999999999999999\n";
const PREVIOUS: &[u8] = br#"{"schema":"veil.baseline.v1","generated_at":"2026-10-12T00:00:00Z","tool":"previous-baseline","entries":[]}"#;

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

fn config(root: &Path, contents: &str) {
    fs::write(root.join("veil.toml"), contents).unwrap();
}

fn quit_interactive(mut command: Command) -> Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    if let Err(error) = input.write_all(b"q\n") {
        // Rejected scans may close stdin before any interactive input is needed.
        assert_eq!(error.kind(), ErrorKind::BrokenPipe);
    }
    drop(input);
    child.wait_with_output().unwrap()
}

fn rejected(output: &Output, diagnostic: &str) {
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(
        output.stdout.is_empty(),
        "Review or partial report was shown"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(diagnostic),
        "{output:?}"
    );
}

fn interactive_baseline_limit(configured: bool) {
    let dir = tempdir().unwrap();
    let root = dir.path();
    config(root, "[core]\nmax_file_count = 100\n");
    fs::write(root.join("known.txt"), KNOWN).unwrap();
    let output = veil(root)
        .args(["scan", "known.txt", "--write-baseline", "baseline.json"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let previous = fs::read(root.join("baseline.json")).unwrap();
    let contents = format!("{KNOWN}{NEW}");
    fs::write(root.join("known.txt"), &contents).unwrap();
    if configured {
        config(
            root,
            "[core]\nmax_file_count = 100\n[output]\nmax_findings = 1\n",
        );
    }
    let mut command = veil(root);
    command.args([
        "scan",
        "known.txt",
        "--baseline",
        "baseline.json",
        "--interactive",
    ]);
    if !configured {
        command.args(["--limit", "1"]);
    }
    rejected(&quit_interactive(command), "scan incomplete");
    assert_eq!(fs::read(root.join("baseline.json")).unwrap(), previous);
    assert_eq!(
        fs::read_to_string(root.join("known.txt")).unwrap(),
        contents
    );
}

#[test]
fn interactive_baseline_keeps_cli_limit_exit_two() {
    interactive_baseline_limit(false);
}

#[test]
fn interactive_baseline_keeps_configured_limit_exit_two() {
    interactive_baseline_limit(true);
}

fn interactive_incomplete(configuration: &str, invalid_utf8: bool, finding_limit: bool) {
    let dir = tempdir().unwrap();
    let root = dir.path();
    config(root, configuration);
    fs::create_dir(root.join("input")).unwrap();
    fs::write(root.join("input/known.txt"), KNOWN).unwrap();
    let other = if invalid_utf8 {
        vec![0xff, 0xfe, b'x']
    } else {
        b"safe=true\n".to_vec()
    };
    fs::write(root.join("input/other.txt"), other).unwrap();
    let mut command = veil(root);
    command.args(["scan", "input", "--interactive"]);
    if finding_limit {
        command.args(["--limit", "1"]);
    }
    rejected(&quit_interactive(command), "scan incomplete");
    assert_eq!(
        fs::read_to_string(root.join("input/known.txt")).unwrap(),
        KNOWN
    );
}

#[test]
fn interactive_finding_limit_rejects_before_review() {
    interactive_incomplete("[core]\nmax_file_count = 100\n", false, true);
}

#[test]
fn interactive_file_limit_rejects_before_review() {
    interactive_incomplete("[core]\nmax_file_count = 1\n", false, false);
}

#[test]
fn interactive_size_limit_rejects_before_review() {
    interactive_incomplete("[core]\nmax_file_size = 8\n", false, false);
}

#[test]
fn interactive_read_error_rejects_before_review() {
    interactive_incomplete("[core]\nmax_file_count = 100\n", true, false);
}

fn interactive_write_conflict(existing: bool) {
    let dir = tempdir().unwrap();
    let root = dir.path();
    config(root, "[core]\nmax_file_count = 100\n");
    fs::write(root.join("known.txt"), KNOWN).unwrap();
    if existing {
        fs::write(root.join("baseline.json"), PREVIOUS).unwrap();
    }
    let mut command = veil(root);
    command.args([
        "scan",
        "known.txt",
        "--interactive",
        "--write-baseline",
        "baseline.json",
    ]);
    rejected(&quit_interactive(command), "cannot be used with");
    if existing {
        assert_eq!(fs::read(root.join("baseline.json")).unwrap(), PREVIOUS);
    } else {
        assert!(!root.join("baseline.json").exists());
    }
}

#[test]
fn interactive_write_baseline_conflict_does_not_create_file() {
    interactive_write_conflict(false);
}

#[test]
fn interactive_write_baseline_conflict_preserves_file() {
    interactive_write_conflict(true);
}

#[test]
fn complete_interactive_scan_can_quit_without_modification() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    config(root, "[core]\nmax_file_count = 100\n");
    fs::write(root.join("known.txt"), KNOWN).unwrap();
    let mut command = veil(root);
    command.args(["scan", "known.txt", "--interactive"]);
    assert_eq!(quit_interactive(command).status.code(), Some(0));
    assert_eq!(fs::read_to_string(root.join("known.txt")).unwrap(), KNOWN);
}

#[derive(Clone, Copy)]
enum Source {
    Commit,
    History,
    Staged,
}

fn index_entry(path: &str, id: Oid, mode: u32) -> IndexEntry {
    IndexEntry {
        ctime: IndexTime::new(0, 0),
        mtime: IndexTime::new(0, 0),
        dev: 0,
        ino: 0,
        mode,
        uid: 0,
        gid: 0,
        file_size: 0,
        id,
        flags: 0,
        flags_extended: 0,
        path: path.as_bytes().to_vec(),
    }
}

fn fixture(root: &Path, source: Source, gitlink: bool) -> Option<Oid> {
    let repo = Repository::init(root).unwrap();
    let good = repo.blob(KNOWN.as_bytes()).unwrap();
    let missing = Oid::from_bytes(&[0x11; 20]).unwrap();
    assert!(repo.find_object(missing, None).is_err());
    let (name, mode) = if gitlink {
        ("module", 0o160000)
    } else {
        ("missing.txt", 0o100644)
    };
    if matches!(source, Source::Staged) {
        if gitlink {
            let tree_id = repo.treebuilder(None).unwrap().write().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();
            let signature = Signature::now("Fixture", "fixture@example.invalid").unwrap();
            repo.commit(Some("HEAD"), &signature, &signature, "Fixture", &tree, &[])
                .unwrap();
        }
        // A detached index permits a synthetic dangling entry without deleting objects.
        let mut index = git2::Index::open(&repo.path().join("index")).unwrap();
        index
            .add(&index_entry("healthy.txt", good, 0o100644))
            .unwrap();
        index.add(&index_entry(name, missing, mode)).unwrap();
        index.write().unwrap();
        return None;
    }
    // Construct the missing reference without deleting any Git objects.
    let mut raw = b"100644 healthy.txt\0".to_vec();
    raw.extend_from_slice(good.as_bytes());
    raw.extend_from_slice(format!("{mode:o} {name}\0").as_bytes());
    raw.extend_from_slice(missing.as_bytes());
    let tree_id = repo.odb().unwrap().write(ObjectType::Tree, &raw).unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let signature = Signature::now("Fixture", "fixture@example.invalid").unwrap();
    Some(
        repo.commit(Some("HEAD"), &signature, &signature, "Fixture", &tree, &[])
            .unwrap(),
    )
}

fn source_args(command: &mut Command, source: Source, oid: Option<Oid>) {
    command.arg("scan");
    match source {
        Source::Commit => {
            command.args(["--commit", &oid.unwrap().to_string()]);
        }
        Source::History => {
            command.args(["--since", "1990-01-01"]);
        }
        Source::Staged => {
            command.arg("--staged");
        }
    }
}

fn missing_blob(source: Source, baseline: Option<bool>) {
    let dir = tempdir().unwrap();
    let root = dir.path();
    config(root, "[core]\nmax_file_count = 100\n");
    let oid = fixture(root, source, false);
    if baseline == Some(true) {
        fs::write(root.join("baseline.json"), PREVIOUS).unwrap();
    }
    let mut command = veil(root);
    source_args(&mut command, source, oid);
    if baseline.is_some() {
        command.args(["--write-baseline", "baseline.json"]);
    } else {
        command.args(["--format", "json", "--fail-on-findings", "1"]);
    }
    rejected(&command.output().unwrap(), "Cannot read Git blob");
    if baseline == Some(true) {
        assert_eq!(fs::read(root.join("baseline.json")).unwrap(), PREVIOUS);
    } else {
        assert!(!root.join("baseline.json").exists());
    }
}

macro_rules! missing_cases {
    ($scan:ident, $new:ident, $existing:ident, $source:expr) => {
        #[test]
        fn $scan() {
            missing_blob($source, None);
        }
        #[test]
        fn $new() {
            missing_blob($source, Some(false));
        }
        #[test]
        fn $existing() {
            missing_blob($source, Some(true));
        }
    };
}

missing_cases!(
    missing_commit_blob_returns_two,
    missing_commit_blob_does_not_save,
    missing_commit_blob_preserves_baseline,
    Source::Commit
);
missing_cases!(
    missing_history_blob_returns_two,
    missing_history_blob_does_not_save,
    missing_history_blob_preserves_baseline,
    Source::History
);
missing_cases!(
    missing_staged_blob_returns_two,
    missing_staged_blob_does_not_save,
    missing_staged_blob_preserves_baseline,
    Source::Staged
);

fn gitlink_is_explicitly_skipped(source: Source) {
    let dir = tempdir().unwrap();
    let root = dir.path();
    config(root, "[core]\nmax_file_count = 100\n");
    let oid = fixture(root, source, true);
    let mut command = veil(root);
    source_args(&mut command, source, oid);
    command.args(["--format", "json", "--fail-on-findings", "1"]);
    let output = command.output().unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["summary"]["scanned_files"], 1);
    assert_eq!(report["summary"]["skipped_files"], 1);
    assert_eq!(report["summary"]["new_findings"], 1);
}

#[test]
fn commit_gitlink_is_skipped_without_reading_submodule_object() {
    gitlink_is_explicitly_skipped(Source::Commit);
}

#[test]
fn history_gitlink_is_skipped_without_reading_submodule_object() {
    gitlink_is_explicitly_skipped(Source::History);
}

#[test]
fn staged_gitlink_is_skipped_without_reading_submodule_object() {
    gitlink_is_explicitly_skipped(Source::Staged);
}

fn staged_missing_head_tree(existing: bool) {
    let dir = tempdir().unwrap();
    let root = dir.path();
    config(root, "[core]\nmax_file_count = 100\n");
    let repo = Repository::init(root).unwrap();
    let raw = format!(
        "tree {}\nauthor Fixture <fixture@example.invalid> 1700000000 +0000\ncommitter Fixture <fixture@example.invalid> 1700000000 +0000\n\nFixture\n",
        "11".repeat(20)
    );
    let oid = repo
        .odb()
        .unwrap()
        .write(ObjectType::Commit, raw.as_bytes())
        .unwrap();
    repo.reference("refs/heads/main", oid, true, "Fixture")
        .unwrap();
    repo.set_head("refs/heads/main").unwrap();
    if existing {
        fs::write(root.join("baseline.json"), PREVIOUS).unwrap();
    }
    let output = veil(root)
        .args(["scan", "--staged", "--write-baseline", "baseline.json"])
        .output()
        .unwrap();
    rejected(&output, "Cannot read Git HEAD tree");
    if existing {
        assert_eq!(fs::read(root.join("baseline.json")).unwrap(), PREVIOUS);
    } else {
        assert!(!root.join("baseline.json").exists());
    }
}

#[test]
fn staged_missing_head_tree_does_not_create_baseline() {
    staged_missing_head_tree(false);
}

#[test]
fn staged_missing_head_tree_preserves_baseline() {
    staged_missing_head_tree(true);
}

#[test]
fn unborn_staged_complete_baseline_still_succeeds() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    config(root, "[core]\nmax_file_count = 100\n");
    let repo = Repository::init(root).unwrap();
    let blob = repo.blob(KNOWN.as_bytes()).unwrap();
    let mut index = repo.index().unwrap();
    index
        .add(&index_entry("healthy.txt", blob, 0o100644))
        .unwrap();
    index.write().unwrap();
    let output = veil(root)
        .args(["scan", "--staged", "--write-baseline", "baseline.json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("baseline.json")).unwrap()).unwrap();
    assert_eq!(saved["entries"].as_array().unwrap().len(), 1);
    assert_eq!(saved["entries"][0]["path"], "healthy.txt");
}
