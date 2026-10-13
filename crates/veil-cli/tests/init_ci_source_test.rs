use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::tempdir;

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

fn generate(root: &Path, args: &[&str]) -> Output {
    veil(root)
        .args(["init", "--ci", "github"])
        .args(args)
        .output()
        .unwrap()
}

fn workflow(root: &Path) -> String {
    fs::read_to_string(root.join(".github/workflows/veil.yml")).unwrap()
}

fn rejects_without_writing(args: &[&str]) {
    let dir = tempdir().unwrap();
    let output = generate(dir.path(), args);
    assert_eq!(output.status.code(), Some(2));
    assert!(!dir.path().join(".github").exists());
    assert!(output.stdout.is_empty());
}

#[test]
fn no_source_is_not_inferred_from_manifest_version() {
    rejects_without_writing(&[]);
}

#[test]
fn automatic_and_mutable_tags_are_rejected() {
    for source in ["auto", "none"] {
        rejects_without_writing(&["--pin-tag", source]);
    }
}

#[test]
fn checkout_installs_only_the_checked_out_source() {
    let dir = tempdir().unwrap();
    let output = generate(dir.path(), &["--ci-source", "checkout"]);
    assert!(output.status.success());
    let content = workflow(dir.path());
    assert!(content.contains("--path \"$GITHUB_WORKSPACE/crates/veil-cli\""));
    assert!(!content.contains("--git"));
    assert!(!content.contains("--tag"));
    assert!(content.contains("does not include uncommitted local work in GitHub CI"));
    assert!(String::from_utf8_lossy(&output.stdout).contains("generation does not verify it"));
}

#[test]
fn full_commit_is_pinned_without_using_package_version() {
    let dir = tempdir().unwrap();
    let revision = "AB".repeat(20);
    let output = generate(dir.path(), &["--ci-source", &revision]);
    assert!(output.status.success());
    let content = workflow(dir.path());
    assert!(content.contains(&format!("--rev {}", revision.to_ascii_lowercase())));
    assert!(!content.contains("--tag"));
    assert!(!content.contains("__VEIL_"));
}

#[test]
fn invalid_sources_and_expression_or_shell_injection_are_rejected() {
    for source in [
        "main",
        "abcd",
        "v0.17.0",
        "${{ secrets.VALUE }}",
        "$(touch injected)",
        "éééééééééééééééééééé",
    ] {
        rejects_without_writing(&["--ci-source", source]);
    }
    for tag in [
        "v1.0.0;touch injected",
        "v${{ secrets.VALUE }}",
        "v`touch injected`",
        "v1\nrun: unsafe",
    ] {
        rejects_without_writing(&["--pin-tag", tag]);
    }
}

#[test]
fn conflicting_sources_are_rejected() {
    rejects_without_writing(&["--ci-source", "checkout", "--pin-tag", "v1.0.0"]);
}

#[test]
fn source_options_require_ci_generation() {
    let dir = tempdir().unwrap();
    for args in [
        vec!["init", "--ci-source", "checkout"],
        vec!["init", "--pin-tag", "v1.0.0"],
    ] {
        let output = veil(dir.path()).args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(!dir.path().join("veil.toml").exists());
        assert!(!dir.path().join(".github").exists());
    }
}

#[test]
fn removed_legacy_init_does_not_generate_files() {
    let dir = tempdir().unwrap();
    for args in [
        vec!["legacy", "init", "--ci", "github"],
        vec![
            "legacy",
            "init",
            "--ci",
            "github",
            "--ci-source",
            "checkout",
        ],
    ] {
        let output = veil(dir.path()).args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("unrecognized subcommand"));
        assert!(!dir.path().join(".github").exists());
        assert!(!dir.path().join("veil.toml").exists());
    }
}

#[test]
fn existing_workflow_is_preserved() {
    let dir = tempdir().unwrap();
    let output = generate(dir.path(), &["--ci-source", "checkout"]);
    assert!(output.status.success());
    let original = workflow(dir.path());
    let output = generate(dir.path(), &["--pin-tag", "v1.0.0"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(workflow(dir.path()), original);
}

#[test]
fn generated_report_and_build_paths_are_outside_workspace() {
    let dir = tempdir().unwrap();
    assert!(generate(dir.path(), &["--ci-source", "checkout"])
        .status
        .success());
    let content = workflow(dir.path());
    assert!(content.contains("VEIL_REPORT_DIR: ${{ runner.temp }}/veil-report"));
    assert!(content.contains("VEIL_BUILD_DIR: ${{ runner.temp }}/veil-build"));
    assert!(content.contains("VEIL_INSTALL_DIR: ${{ runner.temp }}/veil-cli"));
    assert!(content.contains("--target-dir \"$VEIL_BUILD_DIR\""));
    assert!(content.contains("path: ${{ env.VEIL_REPORT_DIR }}/veil-report.html"));
    assert!(!content.contains("> veil-report.html"));
    assert_eq!(content.matches("scan . --format html").count(), 1);
}

#[cfg(unix)]
fn shell_block(content: &str, step: &str) -> String {
    let marker = format!("      - name: {step}\n");
    let section = content.split_once(&marker).unwrap().1;
    let body = section.split_once("        run: |\n").unwrap().1;
    body.lines()
        .take_while(|line| line.starts_with("          "))
        .map(|line| format!("{}\n", &line[10..]))
        .collect()
}

#[cfg(unix)]
#[test]
fn generated_scan_uses_external_report_even_on_repeat_or_failure() {
    let dir = tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    fs::create_dir(&workspace).unwrap();
    assert!(generate(&workspace, &["--ci-source", "checkout"])
        .status
        .success());
    let content = workflow(&workspace);
    let runner = dir.path().join("runner");
    let install = runner.join("veil-cli/bin");
    let report_dir = runner.join("veil-report");
    fs::create_dir_all(&install).unwrap();
    fs::create_dir(&report_dir).unwrap();
    fs::copy(env!("CARGO_BIN_EXE_veil"), install.join("veil")).unwrap();
    let scan = shell_block(&content, "Veil Scan (report and enforce)");
    let run = || {
        let mut command = Command::new("bash");
        command.current_dir(&workspace).args(["-e", "-c", &scan]);
        for name in [
            "VEIL_REMOTE_RULES_URL",
            "VEIL_ORG_CONFIG",
            "VEIL_USER_CONFIG",
            "VEIL_ORG_RULES",
            "VEIL_FAIL_SCORE",
        ] {
            command.env_remove(name);
        }
        command
            .env("VEIL_INSTALL_DIR", runner.join("veil-cli"))
            .env("VEIL_REPORT_DIR", &report_dir)
            .env("XDG_CONFIG_HOME", dir.path().join("settings"));
        command.output().unwrap()
    };
    // An external old report containing a synthetic finding must never be scanned.
    let token = format!("AKIA{}", "A".repeat(16));
    let report = report_dir.join("veil-report.html");
    fs::write(&report, format!("aws_key={token}\n")).unwrap();
    for _ in 0..2 {
        assert!(run().status.success());
        assert!(fs::read_to_string(&report)
            .unwrap()
            .starts_with("<!DOCTYPE html>"));
        assert!(!workspace.join("veil-report.html").exists());
    }
    fs::write(workspace.join("sample.txt"), format!("aws_key={token}\n")).unwrap();
    assert_eq!(run().status.code(), Some(1));
    let html = fs::read_to_string(&report).unwrap();
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(!html.contains(&token));
    assert_eq!(run().status.code(), Some(1));
}

#[cfg(unix)]
#[test]
fn path_guard_rejects_reports_or_runner_temp_inside_workspace() {
    let dir = tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    fs::create_dir(&workspace).unwrap();
    assert!(generate(&workspace, &["--ci-source", "checkout"])
        .status
        .success());
    let guard = shell_block(
        &workflow(&workspace),
        "Prepare paths outside scan workspace",
    );
    let outside = dir.path().join("runner");
    fs::create_dir(&outside).unwrap();
    std::os::unix::fs::symlink(&workspace, outside.join("link")).unwrap();
    for (runner, report) in [
        (&workspace, workspace.join("report")),
        (&outside, workspace.join("report")),
        (&outside, outside.join("link/report")),
    ] {
        let output = Command::new("bash")
            .args(["-c", &guard])
            .env("GITHUB_WORKSPACE", &workspace)
            .env("RUNNER_TEMP", runner)
            .env("VEIL_REPORT_DIR", &report)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(!report.join("veil-report.html").exists());
    }
}
