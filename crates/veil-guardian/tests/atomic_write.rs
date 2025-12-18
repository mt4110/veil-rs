use std::fs;
use tempfile::tempdir;
use veil_guardian::util::atomic_write::atomic_write_bytes;

#[test]
fn test_atomic_write_creates_file() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.txt");
    let content = b"Hello world";

    atomic_write_bytes(&file, content).unwrap();

    let read_back = fs::read(&file).unwrap();
    assert_eq!(read_back, content);
}

#[test]
fn test_atomic_write_overwrites_existing() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("overwrite.txt");

    atomic_write_bytes(&file, b"First").unwrap();
    assert_eq!(fs::read(&file).unwrap(), b"First");

    atomic_write_bytes(&file, b"Second").unwrap();
    assert_eq!(fs::read(&file).unwrap(), b"Second");
}

#[test]
fn test_atomic_write_creates_parent_dir() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("nested").join("deep").join("test.txt");
    let content = b"Deep";

    atomic_write_bytes(&file, content).unwrap();

    let read_back = fs::read(&file).unwrap();
    assert_eq!(read_back, content);
}

#[test]
fn test_no_tmp_leftovers() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("clean.txt");

    atomic_write_bytes(&file, b"clean").unwrap();

    // Check for any files starting with .clean.txt.tmp
    let entries = fs::read_dir(dir.path()).unwrap();
    let count = entries
        .filter_map(Result::ok)
        .filter(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            name.contains(".clean.txt.tmp")
        })
        .count();

    assert_eq!(count, 0, "Should have 0 tmp files left");
}
