use serde_json::json;
use std::fs;
use std::time::SystemTime;
use tempfile::tempdir;
use veil_guardian::providers::osv::details::CachedVuln;
use veil_guardian::providers::osv::details_store::DetailsStore;
use veil_guardian::util::key::normalize_key;

#[test]
fn test_normalize_key_safe_chars() {
    let key = "GHSA-aaaa-bbbb";
    // safe chars are unchanged
    assert_eq!(normalize_key(key), "GHSA-aaaa-bbbb");
}

#[test]
fn test_normalize_key_collision_avoidance() {
    let key1 = "foo:bar";
    let key2 = "foo_bar";

    let n1 = normalize_key(key1);
    let n2 = normalize_key(key2);

    // key1 converts to foo_bar but has modified=true, so it gets hash suffix
    // key2 is safe, so it stays foo_bar
    // Therefore n1 != n2
    assert_ne!(n1, n2);
    assert!(n1.starts_with("foo_bar-"));
    assert_eq!(n2, "foo_bar");
}

#[test]
fn test_normalize_key_truncation() {
    let long_key = "a".repeat(200);
    let normalized = normalize_key(&long_key);

    assert!(normalized.len() < 200);
    assert!(normalized.len() <= 128 + 20); // rough upper bound
                                           // should be 64 char prefix + '-' + 16 char hash = 81 chars
    assert_eq!(normalized.len(), 64 + 1 + 16);
    assert!(normalized.starts_with(&"a".repeat(64)));
}

#[test]
fn test_legacy_fallback() {
    let dir = tempdir().unwrap();
    let store = DetailsStore::with_dir(dir.path()).unwrap();

    let id = "GHSA-legacy-123";
    let entry = CachedVuln::new(id, SystemTime::now(), json!({"id": id}), None);
    let body = serde_json::to_string(&entry).unwrap();

    // 1. Manually write to legacy path
    // Legacy path uses simple sanitize (replace : with _ etc), but GHSA-legacy-123 is safe
    let legacy_path = dir.path().join(format!("{}.json", id));
    fs::write(&legacy_path, body).unwrap();

    // 2. Load via store - should find it in legacy
    let loaded = store.load(id).expect("Should fallback to legacy");
    assert_eq!(loaded.vuln_id, id);

    // 3. Save via store - should write to v1
    store.save(&loaded).unwrap();

    // 4. Verify v1 file exists
    // GHSA-legacy-123 is safe, so v1 name is same
    // But it's in v1 subdirectory
    let v1_path = dir.path().join("v1").join(format!("{}.json", id));
    assert!(v1_path.exists());

    // 5. Load again - assumes v1 is read
    // Modify v1 to prove it's being read
    let mut modified = entry.clone();
    modified.vuln = json!({"id": "modified"});
    let mod_body = serde_json::to_string(&modified).unwrap();
    fs::write(&v1_path, mod_body).unwrap();

    let reloaded = store.load(id).unwrap();
    assert_eq!(reloaded.vuln["id"], "modified");
}

#[test]
fn test_complex_id_roundtrip() {
    let dir = tempdir().unwrap();
    let store = DetailsStore::with_dir(dir.path()).unwrap();

    let id = "GHSA/complex:id";
    let entry = CachedVuln::new(id, SystemTime::now(), json!({"id": id}), None);

    // Save
    store.save(&entry).unwrap();

    // Check file exists in v1 with hashed name
    // normalized: GHSA_complex_id-<hash>
    let v1_entries: Vec<_> = fs::read_dir(dir.path().join("v1"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".json") && !name.ends_with(".lock"))
        .collect();

    assert!(!v1_entries.is_empty(), "Should have found json file");
    assert!(v1_entries[0].starts_with("GHSA_complex_id-"));
    assert!(v1_entries[0].ends_with(".json"));

    // Load
    let loaded = store.load(id).unwrap();
    assert_eq!(loaded.vuln_id, id);
}
