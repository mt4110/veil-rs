/// Test to PROVE that lock busy scenario is non-blocking and fails fast
/// This test ensures we never introduce blocking locks or hangs in CI/local
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;
use veil_core::registry::{Registry, RegistryError};

#[test]
fn test_lock_busy_is_non_blocking() {
    let temp_dir = TempDir::new().unwrap();
    let registry_path = temp_dir.path().join("test_registry.toml");

    // Create a registry file
    let mut registry = Registry::new();
    registry.save(&registry_path).unwrap();

    // Hold exclusive lock in main thread
    let lock_path = registry_path.with_extension("lock");
    let _lock_file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .unwrap();

    // Use fs2 explicitly to match the impl
    fs2::FileExt::try_lock_exclusive(&_lock_file).unwrap();
    // Spawn thread that tries to load (which needs shared lock)
    let (tx, rx) = channel();
    let registry_path_clone = registry_path.clone();

    thread::spawn(move || {
        let result = Registry::load(&registry_path_clone);
        tx.send(result).expect("channel send failed");
    });

    // CRITICAL: Timeout-based hang detection
    // If lock is blocking, this will timeout → test FAILS
    // If lock is non-blocking, result comes back immediately with LockBusy→ test PASSES
    let timeout = Duration::from_secs(2);
    let result = match rx.recv_timeout(timeout) {
        Ok(res) => res,
        Err(RecvTimeoutError::Timeout) => {
            panic!("HANG DETECTED: Lock operation did not return within {}s. Implementation is BLOCKING!", timeout.as_secs());
        }
        Err(RecvTimeoutError::Disconnected) => {
            panic!("Thread crashed unexpectedly");
        }
    };

    // Verify it returned LockBusy (not hanging)
    match result {
        Err(RegistryError::LockBusy(path)) => {
            // SUCCESS: Non-blocking confirmed
            assert_eq!(path, lock_path);
        }
        other => {
            panic!("Expected LockBusy error, got: {:?}", other);
        }
    }
}

#[test]
fn test_lock_busy_error_message_contract() {
    // Verify error message contains recovery information
    let lock_path = PathBuf::from("/fake/path.lock");
    let err = RegistryError::LockBusy(lock_path.clone());
    let msg = err.to_string();

    // Contract: Must indicate lock status and path
    assert!(
        msg.contains("locked") || msg.contains("busy"),
        "Message should indicate lock busy. Got: {}",
        msg
    );
    assert!(
        msg.contains(&lock_path.to_string_lossy().to_string()),
        "Message should show lock path. Got: {}",
        msg
    );
    assert!(
        msg.contains("another process") || msg.contains("process"),
        "Message should indicate another process holds the lock. Got: {}",
        msg
    );
}

#[test]
fn test_save_lock_busy_is_non_blocking() {
    let temp_dir = TempDir::new().unwrap();
    let registry_path = temp_dir.path().join("test_registry_save.toml");

    // Create initial registry
    let mut registry = Registry::new();
    registry.save(&registry_path).unwrap();

    // Hold exclusive lock
    let lock_path = registry_path.with_extension("lock");
    let _lock_file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .unwrap();

    fs2::FileExt::try_lock_exclusive(&_lock_file).unwrap();

    // Try to save (which needs exclusive lock)
    let (tx, rx) = channel();
    let registry_path_clone = registry_path.clone();

    thread::spawn(move || {
        let mut reg = Registry::new();
        let result = reg.save(&registry_path_clone);
        tx.send(result).expect("channel send failed");
    });

    // Hang detection
    let timeout = Duration::from_secs(2);
    let result = match rx.recv_timeout(timeout) {
        Ok(res) => res,
        Err(RecvTimeoutError::Timeout) => {
            panic!("HANG DETECTED: Save operation did not return within {}s. Implementation is BLOCKING!", timeout.as_secs());
        }
        Err(RecvTimeoutError::Disconnected) => {
            panic!("Thread crashed unexpectedly");
        }
    };

    // Verify non-blocking LockBusy
    match result {
        Err(RegistryError::LockBusy(path)) => {
            assert_eq!(path, lock_path);
        }
        other => {
            panic!("Expected LockBusy error, got: {:?}", other);
        }
    }
}
