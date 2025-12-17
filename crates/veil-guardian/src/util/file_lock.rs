use fs2::FileExt;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::Path;

/// Executes an operation with an exclusive lock on a separate lock file.
///
/// The lock file is created at `<path>.lock`.
/// Drops the lock automatically when the file handle goes out of scope.
pub fn with_file_lock<F, T>(path: &Path, op: F) -> io::Result<T>
where
    F: FnOnce() -> io::Result<T>,
{
    // Use .lock extension appended to the full file name/path
    // e.g. "cache.json" -> "cache.json.lock"
    let lock_path_noun = format!("{}.lock", path.display());
    let lock_path = Path::new(&lock_path_noun);

    // Ensure parent exists
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)?;

    file.lock_exclusive()?;

    let result = op();

    // Lock is released when `file` is dropped.
    drop(file);

    result
}
