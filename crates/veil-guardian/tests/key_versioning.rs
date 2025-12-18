use serde_json::json;
use std::fs;
use std::time::SystemTime;
use tempfile::tempdir;
use veil_guardian::providers::osv::details::CachedVuln;
use veil_guardian::providers::osv::details_store::{DetailsStore, StoreLoad, StoreSource};
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

    // 1. Manually write to legacy path (which is ROOT/<clean>.json)
    // Legacy path uses simple sanitize (replace : with _ etc), but GHSA-legacy-123 is safe
    let legacy_path = dir.path().join(format!("{}.json", id));
    fs::write(&legacy_path, body).unwrap();

    // 2. Load via store - should find it in legacy and MIGRATE
    // 2. Load via store - should find it in legacy and MIGRATE
    match store.load(id) {
        StoreLoad::Hit {
            entry: loaded,
            source,
            migrated,
            quarantined: _,
        } => {
            assert_eq!(loaded.vuln_id, id);
            assert_eq!(source, StoreSource::Legacy);
            assert!(migrated, "Should indicate migration happened");

            // 3. Verify v1 file exists immediately after load
            let name = veil_guardian::util::key::normalize_key(id);
            // Unified Path: <root>/vulns/v1
            let v1_path = dir
                .path()
                .join("vulns")
                .join("v1")
                .join(format!("{}.json", name));
            assert!(
                v1_path.exists(),
                "v1 normalized path should exist after load (migrate-on-read)"
            );
        }
        StoreLoad::Miss { .. } => panic!("Should have hit legacy"),
    }

    // 4. Verify content is v1 Envelope
    let name = veil_guardian::util::key::normalize_key(id);
    let v1_path = dir
        .path()
        .join("vulns")
        .join("v1")
        .join(format!("{}.json", name));

    let content = std::fs::read_to_string(&v1_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["key"], id);
    assert!(json.get("payload").is_some());
    assert_eq!(json["payload"]["id"], id);
    assert_eq!(json["source"], "legacy_migration");
}

#[test]
fn test_complex_id_roundtrip_with_etag() {
    let dir = tempdir().unwrap();
    let store = DetailsStore::with_dir(dir.path()).unwrap();

    let id = "GHSA/complex:id";
    let etag = Some("W/\"test-etag\"".to_string());
    let entry = CachedVuln::new(id, SystemTime::now(), json!({"id": id}), etag.clone());

    // Save
    store.save(&entry).unwrap();

    // Check file exists in v1 with hashed name
    // normalized: GHSA_complex_id-<hash>
    // Unified Path: <root>/vulns/v1
    let v1_entries: Vec<_> = fs::read_dir(dir.path().join("vulns").join("v1"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".json") && !name.ends_with(".lock"))
        .collect();

    assert!(!v1_entries.is_empty(), "Should have found json file");
    assert!(v1_entries[0].starts_with("GHSA_complex_id-"));
    assert!(v1_entries[0].ends_with(".json"));

    // Load
    match store.load(id) {
        StoreLoad::Hit {
            entry: loaded,
            source,
            migrated,
            quarantined: _,
        } => {
            assert_eq!(loaded.vuln_id, id);
            assert_eq!(source, StoreSource::V1);
            assert!(!migrated);
            assert_eq!(loaded.etag, etag);
        }
        StoreLoad::Miss { .. } => panic!("Expected Hit"),
    }
}

#[test]
fn test_directory_conflict() {
    let dir = tempdir().unwrap();
    let store = DetailsStore::with_dir(dir.path()).unwrap();
    // v1 is now deep: <root>/vulns/v1
    let v1_dir = dir.path().join("vulns").join("v1");

    // 1. Sabotage: Create "v1" as a file
    // Note: with_dir might have created it as dir. Remove it first.
    if v1_dir.exists() {
        fs::remove_dir_all(&v1_dir).unwrap();
    }
    // Ensure parent "vulns" exists
    fs::create_dir_all(v1_dir.parent().unwrap()).unwrap();
    fs::write(&v1_dir, "I am a file blockage").unwrap();

    // 2. Load something (triggers ensure_v1_dir)
    // We use OsvClient to verify Metrics integration
    // We use OsvClient to verify Metrics integration
    let metrics = std::sync::Arc::new(veil_guardian::Metrics::new());

    // Create client using custom store
    let net = veil_guardian::providers::osv::net::NetConfig::default();
    let sleeper = std::sync::Arc::new(veil_guardian::providers::osv::net::TokioSleeper);
    let client = veil_guardian::providers::osv::client::OsvClient::new_custom_with_net_and_metrics(
        true,
        Some(store.clone()), // Use our instrumented store
        None,
        net,
        sleeper,
        Some(metrics.clone()),
    );

    // Call check_packages or fetch_vuln_details to trigger load
    let _ = client.fetch_vuln_details("GHSA-conflict");

    // 2. Load something (metrics should update)
    // StoreLoad::Miss happens because conflict flagged.

    // 3. Verify Metrics
    let snap = metrics.snapshot();
    assert_eq!(snap.cache_quarantine_conflict, 1, "Should count 1 conflict");
    assert_eq!(snap.cache_miss, 1, "Should also count as 1 miss");

    // 3. Verify "v1" is now a directory
    assert!(v1_dir.is_dir(), "v1 should be recovered to a directory");

    // 4. Verify blockage was quarantined
    let quarantined: Vec<_> = fs::read_dir(v1_dir.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with("v1.corrupt_dirs_conflict"))
        .collect();

    assert_eq!(
        quarantined.len(),
        1,
        "Should have quarantined the conflict file"
    );
}
