use serde_json::Value;
use std::fs;
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::tempdir;
use veil_guardian::util::atomic_write::atomic_write_bytes;
use veil_guardian::util::file_lock::with_file_lock;

#[test]
fn test_concurrent_writes_are_safe() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("concurrent.json");
    let file_arc = Arc::new(file);

    let n = 20;
    let barrier = Arc::new(Barrier::new(n));
    let mut handles = Vec::with_capacity(n);

    for i in 0..n {
        let f = file_arc.clone();
        let b = barrier.clone();
        handles.push(thread::spawn(move || {
            b.wait();
            with_file_lock(&f, || {
                // Write valid JSON with an index, enough content to risk tearing if not atomic/locked
                let content = serde_json::json!({
                    "writer": i,
                    "payload": "x".repeat(1000)
                });
                let bytes = serde_json::to_vec(&content).unwrap();
                // We use atomic write inside lock, matching Stores
                atomic_write_bytes(&f, &bytes)
            })
            .unwrap();
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    // Verify content is valid JSON (no partial/corrupt write)
    let content = fs::read_to_string(&*file_arc).unwrap();
    let json: Value = serde_json::from_str(&content).expect("JSON should be valid");

    // Check that writer is one of 0..n
    let writer = json["writer"].as_u64().unwrap();
    assert!(writer < n as u64, "Writer index {} invalid", writer);
    assert_eq!(json["payload"].as_str().unwrap().len(), 1000);
}

#[test]
fn test_lock_file_creation() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("data.json");

    with_file_lock(&file, || Ok(())).unwrap();

    assert!(dir.path().join("data.json.lock").exists());
}
