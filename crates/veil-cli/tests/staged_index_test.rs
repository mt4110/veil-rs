#![cfg(unix)]
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::{tempdir, TempDir};

fn command(root: &Path, program: &str) -> Command {
    let mut c = Command::new(program);
    for (key, _) in std::env::vars_os() {
        let name = key.to_string_lossy();
        if name.starts_with("VEIL_") || name.starts_with("GIT_") {
            c.env_remove(key);
        }
    }
    c.current_dir(root)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("XDG_CONFIG_HOME", root.join("settings"))
        .env("VEIL_USER_CONFIG", root.join("empty.toml"))
        .env("VEIL_ORG_CONFIG", root.join("empty.toml"));
    c
}
fn git(root: &Path, args: &[&str]) {
    assert!(command(root, "git")
        .args(args)
        .output()
        .unwrap()
        .status
        .success());
}
fn fixture() -> TempDir {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join("empty.toml"), "").unwrap();
    git(root, &["init", "-q"]);
    fs::write(root.join("input.txt"), "safe\n").unwrap();
    git(root, &["add", "input.txt"]);
    dir
}
fn scan(root: &Path) -> Command {
    let mut c = command(root, env!("CARGO_BIN_EXE_veil"));
    c.args([
        "scan",
        "--staged",
        "--format",
        "json",
        "--fail-on-findings",
        "1",
    ]);
    c
}

#[test]
fn alternate_index_matches_git_for_relative_and_absolute_paths_without_mutation() {
    for relative in [false, true] {
        let dir = fixture();
        let root = dir.path();
        let ordinary = fs::read(root.join(".git/index")).unwrap();
        let alternate = root.join("alternate-index");
        fs::write(&alternate, &ordinary).unwrap();
        let token = format!("AKIA{}", "A".repeat(16));
        fs::write(root.join("input.txt"), format!("key={token}\n")).unwrap();
        assert!(command(root, "git")
            .env("GIT_INDEX_FILE", &alternate)
            .args(["add", "input.txt"])
            .output()
            .unwrap()
            .status
            .success());
        let alternate_bytes = fs::read(&alternate).unwrap();
        // Working tree and default index are both clean; only the selected index contains the marker.
        fs::write(root.join("input.txt"), "safe\n").unwrap();
        let selected = if relative {
            Path::new("alternate-index")
        } else {
            alternate.as_path()
        };
        let output = scan(root).env("GIT_INDEX_FILE", selected).output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["summary"]["scanned_files"], 1);
        assert_eq!(report["summary"]["new_findings"], 1);
        assert!(!String::from_utf8_lossy(&output.stdout).contains(&token));
        assert_eq!(fs::read(root.join(".git/index")).unwrap(), ordinary);
        assert_eq!(fs::read(&alternate).unwrap(), alternate_bytes);
        assert_eq!(
            fs::read_to_string(root.join("input.txt")).unwrap(),
            "safe\n"
        );
        assert!(scan(root).output().unwrap().status.success());
    }
}

#[test]
fn clean_alternate_index_does_not_scan_default_index_secret() {
    let dir = fixture();
    let root = dir.path();
    let alternate = root.join("alternate-index");
    fs::copy(root.join(".git/index"), &alternate).unwrap();
    fs::write(
        root.join("input.txt"),
        format!("key=AKIA{}\n", "A".repeat(16)),
    )
    .unwrap();
    git(root, &["add", "input.txt"]);
    assert_eq!(scan(root).output().unwrap().status.code(), Some(1));
    let output = scan(root)
        .env("GIT_INDEX_FILE", alternate)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["summary"]["scanned_files"], 1);
    assert_eq!(report["summary"]["new_findings"], 0);
}

#[test]
fn invalid_explicit_index_fails_closed_and_preserves_baselines() {
    let dir = fixture();
    let root = dir.path();
    fs::write(root.join("corrupt-index"), "not a Git index").unwrap();
    fs::create_dir(root.join("directory-index")).unwrap();
    let ordinary = fs::read(root.join(".git/index")).unwrap();
    for index in ["missing-index", "corrupt-index", "directory-index", ""] {
        for existing in [false, true] {
            let name = format!("baseline-{index}-{existing}.json");
            let baseline = root.join(&name);
            if existing {
                fs::write(&baseline, "preserve baseline").unwrap();
            }
            let output = scan(root)
                .env("GIT_INDEX_FILE", index)
                .args(["--write-baseline", &name])
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(2), "{index:?}");
            assert!(output.stdout.is_empty());
            if existing {
                assert_eq!(fs::read_to_string(&baseline).unwrap(), "preserve baseline");
            } else {
                assert!(!baseline.exists());
            }
        }
    }
    assert_eq!(fs::read(root.join(".git/index")).unwrap(), ordinary);
    assert!(!root.join("missing-index").exists());
}
