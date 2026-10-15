#![cfg(unix)]
use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::Path;
use std::process::{Command, Output};
use tempfile::{tempdir, TempDir};

fn command(root: &Path, program: &str) -> Command {
    let mut c = Command::new(program);
    for (key, _) in std::env::vars_os() {
        let name = key.to_string_lossy();
        if name.starts_with("VEIL_") || name.starts_with("GIT_") {
            c.env_remove(key);
        }
    }
    let mut paths = vec![root.join("bin")];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    c.current_dir(root)
        .env("PATH", std::env::join_paths(paths).unwrap())
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("XDG_CONFIG_HOME", root.join("settings"))
        .env("VEIL_USER_CONFIG", root.join("empty.toml"))
        .env("VEIL_ORG_CONFIG", root.join("empty.toml"));
    c
}
fn git(root: &Path, args: &[&str]) -> Output {
    command(root, "git")
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
        ])
        .args(args)
        .output()
        .unwrap()
}
fn fixture() -> TempDir {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join("empty.toml"), "").unwrap();
    fs::create_dir(root.join("bin")).unwrap();
    symlink(env!("CARGO_BIN_EXE_veil"), root.join("bin/veil")).unwrap();
    assert!(git(root, &["init", "-q"]).status.success());
    dir
}
fn install(root: &Path) -> Output {
    command(root, env!("CARGO_BIN_EXE_veil"))
        .args(["pre-commit", "init"])
        .output()
        .unwrap()
}

#[test]
fn installation_is_executable_and_refuses_to_replace_existing_hook() {
    let dir = fixture();
    let root = dir.path();
    assert!(install(root).status.success());
    let hook = root.join(".git/hooks/pre-commit");
    assert_eq!(
        fs::metadata(&hook).unwrap().permissions().mode() & 0o111,
        0o111
    );
    let bytes = fs::read(&hook).unwrap();
    assert_eq!(install(root).status.code(), Some(2));
    assert_eq!(fs::read(&hook).unwrap(), bytes);
    assert!(!root.join(".git/hooks/pre-commit.veil.bak").exists());
}

#[test]
fn generated_hook_distinguishes_clean_findings_and_scan_errors() {
    let dir = fixture();
    let root = dir.path();
    assert!(install(root).status.success());
    fs::write(root.join("input.txt"), "safe\n").unwrap();
    assert!(git(root, &["add", "input.txt"]).status.success());
    let run = || {
        command(root, "sh")
            .arg(".git/hooks/pre-commit")
            .output()
            .unwrap()
    };
    assert!(run().status.success());
    let token = format!("AKIA{}", "A".repeat(16));
    fs::write(root.join("input.txt"), format!("key={token}\n")).unwrap();
    assert!(git(root, &["add", "input.txt"]).status.success());
    let finding = run();
    assert_eq!(finding.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&finding.stderr).contains("findings"));
    assert!(!String::from_utf8_lossy(&finding.stdout).contains(&token));
    fs::write(root.join("veil.toml"), "invalid = [").unwrap();
    let error = run();
    assert_eq!(error.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&error.stderr).contains("scan failed"));
    assert!(!String::from_utf8_lossy(&error.stderr).contains("Secrets were detected"));
}

#[test]
fn existing_files_and_symlinks_are_not_changed() {
    for link in [false, true] {
        let dir = fixture();
        let root = dir.path();
        let hook = root.join(".git/hooks/pre-commit");
        let target = root.join("original-hook");
        fs::write(&target, "#!/bin/sh\nexit 7\n").unwrap();
        if link {
            symlink(&target, &hook).unwrap();
        } else {
            fs::copy(&target, &hook).unwrap();
        }
        let config = fs::read(root.join(".git/config")).unwrap();
        assert_eq!(install(root).status.code(), Some(2));
        assert_eq!(fs::read(&hook).unwrap(), fs::read(&target).unwrap());
        assert_eq!(
            fs::symlink_metadata(&hook)
                .unwrap()
                .file_type()
                .is_symlink(),
            link
        );
        assert_eq!(fs::read(root.join(".git/config")).unwrap(), config);
    }
    let dir = fixture();
    let hook = dir.path().join(".git/hooks/pre-commit");
    symlink("absent-hook", &hook).unwrap();
    assert_eq!(install(dir.path()).status.code(), Some(2));
    assert_eq!(fs::read_link(hook).unwrap(), Path::new("absent-hook"));
}

#[test]
fn configured_hook_paths_preserve_other_checks_and_configuration() {
    for (global, absolute) in [(false, false), (false, true), (true, false), (true, true)] {
        let dir = fixture();
        let root = dir.path();
        let hooks = root.join("configured hooks");
        let setting = if absolute {
            hooks.to_str().unwrap()
        } else {
            "configured hooks"
        };
        let global_config = root.join("global.gitconfig");
        if global {
            fs::write(&global_config, format!("[core]\n hooksPath = {setting}\n")).unwrap();
        } else {
            assert!(git(root, &["config", "core.hooksPath", setting])
                .status
                .success());
        }
        fs::create_dir(&hooks).unwrap();
        let other = hooks.join("commit-msg");
        fs::write(&other, "#!/bin/sh\nexit 7\n").unwrap();
        fs::set_permissions(&other, fs::Permissions::from_mode(0o755)).unwrap();
        let config = fs::read(root.join(".git/config")).unwrap();
        let sub = root.join("sub");
        fs::create_dir(&sub).unwrap();
        let mut c = command(&sub, env!("CARGO_BIN_EXE_veil"));
        if global {
            c.env("GIT_CONFIG_GLOBAL", &global_config);
        }
        assert!(c
            .args(["pre-commit", "init"])
            .output()
            .unwrap()
            .status
            .success());
        assert!(hooks.join("pre-commit").is_file());
        assert!(!root.join(".git/hooks/pre-commit").exists());
        assert_eq!(fs::read_to_string(&other).unwrap(), "#!/bin/sh\nexit 7\n");
        assert_eq!(
            fs::metadata(&other).unwrap().permissions().mode() & 0o777,
            0o755
        );
        assert_eq!(fs::read(root.join(".git/config")).unwrap(), config);
        let mut c = command(root, env!("CARGO_BIN_EXE_veil"));
        if global {
            c.env("GIT_CONFIG_GLOBAL", &global_config);
        }
        assert_eq!(
            c.args(["pre-commit", "init"])
                .output()
                .unwrap()
                .status
                .code(),
            Some(2)
        );
        if global {
            assert_eq!(
                fs::read_to_string(global_config).unwrap(),
                format!("[core]\n hooksPath = {setting}\n")
            );
        }
    }
}

#[test]
fn installation_from_linked_worktree_subdirectory_uses_effective_hooks() {
    let dir = fixture();
    let root = dir.path();
    fs::write(root.join("input.txt"), "safe\n").unwrap();
    assert!(git(root, &["add", "input.txt"]).status.success());
    assert!(git(root, &["commit", "-qm", "fixture"]).status.success());
    let linked = root.join("linked");
    assert!(git(
        root,
        &[
            "worktree",
            "add",
            "-qb",
            "linked-test",
            linked.to_str().unwrap()
        ]
    )
    .status
    .success());
    let sub = linked.join("sub");
    fs::create_dir(&sub).unwrap();
    let installed = command(&sub, env!("CARGO_BIN_EXE_veil"))
        .args(["pre-commit", "init"])
        .output()
        .unwrap();
    assert!(
        installed.status.success(),
        "{}",
        String::from_utf8_lossy(&installed.stderr)
    );
    assert!(root.join(".git/hooks/pre-commit").is_file());
    assert!(fs::read_to_string(linked.join(".git"))
        .unwrap()
        .starts_with("gitdir:"));
}

#[test]
fn partial_commit_scans_selected_content_and_preserves_regular_index() {
    let dir = fixture();
    let root = dir.path();
    fs::write(root.join("input.txt"), "safe\n").unwrap();
    assert!(git(root, &["add", "input.txt"]).status.success());
    assert!(git(root, &["commit", "-qm", "fixture"]).status.success());
    assert!(install(root).status.success());
    let index = fs::read(root.join(".git/index")).unwrap();
    let head = git(root, &["rev-parse", "HEAD"]).stdout;
    fs::write(
        root.join("input.txt"),
        format!("key=AKIA{}\n", "A".repeat(16)),
    )
    .unwrap();
    assert!(
        !git(root, &["commit", "--only", "input.txt", "-m", "must block"])
            .status
            .success()
    );
    assert_eq!(git(root, &["rev-parse", "HEAD"]).stdout, head);
    assert_eq!(fs::read(root.join(".git/index")).unwrap(), index);
    fs::write(root.join("input.txt"), "safe change\n").unwrap();
    assert!(git(
        root,
        &["commit", "--only", "input.txt", "-m", "safe change"]
    )
    .status
    .success());
}

#[test]
fn disabled_hooks_path_is_not_overridden() {
    let dir = fixture();
    let root = dir.path();
    assert!(git(root, &["config", "core.hooksPath", "/dev/null"])
        .status
        .success());
    let config = fs::read(root.join(".git/config")).unwrap();
    assert_eq!(install(root).status.code(), Some(2));
    assert_eq!(fs::read(root.join(".git/config")).unwrap(), config);
    assert!(!root.join(".git/hooks/pre-commit").exists());
}

#[test]
fn missing_cli_is_a_scan_error_and_blocks_the_hook() {
    let dir = fixture();
    let root = dir.path();
    assert!(install(root).status.success());
    let empty = root.join("empty-bin");
    fs::create_dir(&empty).unwrap();
    let output = command(root, "/bin/sh")
        .env("PATH", empty)
        .arg(".git/hooks/pre-commit")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("scan failed (exit 127)"));
}
