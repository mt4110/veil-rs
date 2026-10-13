use ed25519_dalek::{Signer, SigningKey};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn signed_pack(root: &Path, expired: bool) {
    let key = SigningKey::from_bytes(&[7; 32]); // Synthetic test key, never a trust anchor.
    let expiry = chrono::Utc::now() + chrono::Duration::days(if expired { -1 } else { 1 });
    let manifest = format!(
        "[pack]\nid = 'test'\nversion = 1\nschema_version = 1\nexpires_at = '{}'\n",
        expiry.to_rfc3339()
    );
    let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    archive
        .start_file("00_manifest.toml", zip::write::SimpleFileOptions::default())
        .unwrap();
    archive.write_all(manifest.as_bytes()).unwrap();
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
