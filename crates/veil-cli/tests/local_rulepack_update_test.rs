use ed25519_dalek::{Signer, SigningKey};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn signed_pack_entries(root: &Path, expired: bool, prefix: &str, entries: &[(&str, &str)]) {
    let key = SigningKey::from_bytes(&[7; 32]); // Synthetic test key, never a trust anchor.
    let expiry = chrono::Utc::now() + chrono::Duration::days(if expired { -1 } else { 1 });
    let manifest = format!(
        "{prefix}[pack]\nid = 'test'\nversion = 1\nschema_version = 1\nexpires_at = '{}'\n",
        expiry.to_rfc3339()
    );
    let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    archive
        .start_file("00_manifest.toml", zip::write::SimpleFileOptions::default())
        .unwrap();
    archive.write_all(manifest.as_bytes()).unwrap();
    for (name, contents) in entries {
        archive
            .start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        archive.write_all(contents.as_bytes()).unwrap();
    }
    let bytes = archive.finish().unwrap().into_inner();
    fs::write(root.join("rules.zip"), &bytes).unwrap();
    fs::write(
        root.join("rules.sig"),
        hex::encode(key.sign(&bytes).to_bytes()),
    )
    .unwrap();
    fs::write(
        root.join("publisher.hex"),
        hex::encode(key.verifying_key().to_bytes()),
    )
    .unwrap();
}

fn signed_pack(root: &Path, expired: bool) {
    signed_pack_entries(root, expired, "", &[]);
}

fn update(root: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_veil"))
        .current_dir(root)
        .args([
            "update",
            "--local-pack",
            "rules.zip",
            "--public-key",
            "publisher.hex",
        ])
        .output()
        .unwrap()
}

#[test]
fn verified_local_pack_installs_in_a_fresh_directory() {
    let dir = tempdir().unwrap();
    signed_pack(dir.path(), false);
    let output = update(dir.path());
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.path().join(".veil/rules/00_manifest.toml").exists());
}

#[test]
fn existing_rules_are_preserved() {
    let dir = tempdir().unwrap();
    signed_pack(dir.path(), false);
    fs::create_dir_all(dir.path().join(".veil/rules")).unwrap();
    fs::write(
        dir.path().join(".veil/rules/keep.toml"),
        "keep-existing-work",
    )
    .unwrap();
    let output = update(dir.path());
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("was preserved"));
    assert_eq!(
        fs::read_to_string(dir.path().join(".veil/rules/keep.toml")).unwrap(),
        "keep-existing-work"
    );
    assert!(!dir.path().join(".veil/rules/00_manifest.toml").exists());
}

#[test]
fn expired_and_tampered_packs_are_rejected_before_installation() {
    let dir = tempdir().unwrap();
    signed_pack(dir.path(), true);
    let output = update(dir.path());
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("EXPIRED"));
    assert!(!dir.path().join(".veil").exists());

    let tampered = tempdir().unwrap();
    signed_pack(tampered.path(), false);
    fs::write(tampered.path().join("rules.zip"), "tampered").unwrap();
    let output = update(tampered.path());
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Signature verification FAILED"));
    assert!(!tampered.path().join(".veil").exists());
}

const CUSTOM_RULE: &str = "[[rules]]\nid='synthetic.custom'\ndescription='synthetic test'\npattern='ONLY_SYNTHETIC_TEST_123'\nseverity='high'\n";

fn assert_no_installation(root: &Path) {
    assert!(!root.join(".veil/rules").exists());
    if root.join(".veil").exists() {
        assert_eq!(fs::read_dir(root.join(".veil")).unwrap().count(), 0);
    }
}

#[test]
fn installed_rules_require_explicit_activation_without_rewriting_configuration() {
    let dir = tempdir().unwrap();
    let config = "[core]\nmax_file_count = 100\n";
    fs::write(dir.path().join("veil.toml"), config).unwrap();
    signed_pack_entries(dir.path(), false, "", &[("custom.toml", CUSTOM_RULE)]);
    let output = update(dir.path());
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("Not activated automatically"));
    assert_eq!(
        fs::read_to_string(dir.path().join("veil.toml")).unwrap(),
        config
    );
    let list = Command::new(env!("CARGO_BIN_EXE_veil"))
        .current_dir(dir.path())
        .args(["rules", "list"])
        .output()
        .unwrap();
    assert!(list.status.success());
    assert!(!String::from_utf8_lossy(&list.stdout).contains("synthetic.custom"));
    fs::write(
        dir.path().join("veil.toml"),
        "[core]\nrules_dir = '.veil/rules'\n",
    )
    .unwrap();
    let list = Command::new(env!("CARGO_BIN_EXE_veil"))
        .current_dir(dir.path())
        .args(["rules", "list"])
        .output()
        .unwrap();
    assert!(
        list.status.success(),
        "{}",
        String::from_utf8_lossy(&list.stderr)
    );
    assert!(String::from_utf8_lossy(&list.stdout).contains("synthetic.custom"));
    fs::write(dir.path().join("input.txt"), "ONLY_SYNTHETIC_TEST_123").unwrap();
    let scan = Command::new(env!("CARGO_BIN_EXE_veil"))
        .current_dir(dir.path())
        .args(["scan", "input.txt", "--format", "json"])
        .output()
        .unwrap();
    assert!(
        scan.status.success(),
        "{}",
        String::from_utf8_lossy(&scan.stderr)
    );
    assert!(String::from_utf8_lossy(&scan.stdout).contains("synthetic.custom"));
}

#[test]
fn invalid_signed_rule_contents_are_rejected_before_publication() {
    for (prefix, rule) in [
        ("files = ['missing.toml']\n", CUSTOM_RULE.to_string()),
        ("", CUSTOM_RULE.replace("ONLY_SYNTHETIC_TEST_123", "[")),
        ("", format!("{CUSTOM_RULE}{CUSTOM_RULE}")),
    ] {
        let dir = tempdir().unwrap();
        signed_pack_entries(dir.path(), false, prefix, &[("custom.toml", &rule)]);
        let output = update(dir.path());
        assert_eq!(
            output.status.code(),
            Some(2),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("validation failed"));
        assert_no_installation(dir.path());
    }
}

#[test]
fn invalid_manifest_digest_is_rejected_before_publication() {
    let dir = tempdir().unwrap();
    let prefix = "[signature]\nrequired = true\npinned_digests = ['sha256:0000000000000000000000000000000000000000000000000000000000000000']\n";
    signed_pack_entries(dir.path(), false, prefix, &[("custom.toml", CUSTOM_RULE)]);
    let output = update(dir.path());
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("digest mismatch"));
    assert_no_installation(dir.path());
}

#[test]
fn extraction_failure_leaves_no_target_and_allows_retry() {
    let dir = tempdir().unwrap();
    signed_pack_entries(
        dir.path(),
        false,
        "",
        &[("file", "first"), ("file/child", "second")],
    );
    assert_eq!(update(dir.path()).status.code(), Some(2));
    assert_no_installation(dir.path());
    signed_pack_entries(dir.path(), false, "", &[("custom.toml", CUSTOM_RULE)]);
    assert!(update(dir.path()).status.success());
    assert!(dir.path().join(".veil/rules/custom.toml").is_file());
}

#[test]
fn escaping_manifest_paths_are_rejected_before_installation() {
    let dir = tempdir().unwrap();
    signed_pack_entries(dir.path(), false, "files = ['../outside.toml']\n", &[]);
    assert_eq!(update(dir.path()).status.code(), Some(2));
    assert_no_installation(dir.path());
}

#[test]
fn configured_expired_rules_fail_closed_without_mutating_the_pack() {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join(".veil/rules")).unwrap();
    let expiry = chrono::Utc::now() - chrono::Duration::days(1);
    let manifest =
        format!("[pack]\nid='expired-local'\nversion=1\nschema_version=1\nexpires_at='{expiry}'\n");
    fs::write(dir.path().join(".veil/rules/00_manifest.toml"), &manifest).unwrap();
    fs::write(dir.path().join(".veil/rules/custom.toml"), CUSTOM_RULE).unwrap();
    fs::write(
        dir.path().join("veil.toml"),
        "[core]\nrules_dir='.veil/rules'\n",
    )
    .unwrap();
    fs::write(dir.path().join("input.txt"), "ONLY_SYNTHETIC_TEST_123").unwrap();
    for args in [
        vec!["rules", "list"],
        vec!["scan", "input.txt", "--format", "json"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_veil"))
            .current_dir(dir.path())
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("has expired"));
    }
    assert_eq!(
        fs::read_to_string(dir.path().join(".veil/rules/00_manifest.toml")).unwrap(),
        manifest
    );
    assert_eq!(
        fs::read_to_string(dir.path().join(".veil/rules/custom.toml")).unwrap(),
        CUSTOM_RULE
    );
}

#[cfg(unix)]
#[test]
fn symlinked_installation_parent_is_rejected_without_writing_outside_project() {
    for dangling in [false, true] {
        let dir = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let destination = if dangling {
            outside.path().join("missing")
        } else {
            outside.path().to_path_buf()
        };
        fs::write(outside.path().join("keep.txt"), "preserve-outside").unwrap();
        std::os::unix::fs::symlink(&destination, dir.path().join(".veil")).unwrap();
        signed_pack_entries(dir.path(), false, "", &[("custom.toml", CUSTOM_RULE)]);
        let output = update(dir.path());
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("must be a real directory"));
        assert!(!destination.join("rules").exists());
        assert_eq!(
            fs::read_to_string(outside.path().join("keep.txt")).unwrap(),
            "preserve-outside"
        );
        assert!(dir
            .path()
            .join(".veil")
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 1);
    }
}
