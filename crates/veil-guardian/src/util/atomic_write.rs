use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Writes bytes to a file atomically.
///
/// 1. Creates a unique temporary file in the same directory.
/// 2. Writes data, flushes, and syncs.
/// 3. Renames the temporary file to the target path.
///    On Windows, if the target exists, it attempts to remove it first before renaming to simulate atomic overwrite.
/// 4. Cleans up the temporary file on failure.
pub fn atomic_write_bytes(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Try a few times to create a unique temporary file
    for _ in 0..3 {
        let tmp_path = unique_tmp_path(path);
        match write_and_rename(&tmp_path, path, bytes) {
            Ok(_) => return Ok(()),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue, // Collision, retry with new name
            Err(e) => {
                // Best-effort cleanup
                let _ = fs::remove_file(&tmp_path);
                return Err(e);
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "failed to create unique temporary file after retries",
    ))
}

fn unique_tmp_path(path: &Path) -> PathBuf {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);

    // .<filename>.tmp.<pid>.<nanos>
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let tmp_name = format!(".{}.tmp.{}.{}", file_name, pid, nanos);
    path.with_file_name(tmp_name)
}

fn write_and_rename(tmp_path: &Path, target_path: &Path, bytes: &[u8]) -> io::Result<()> {
    // Open with create_new(true) to ensure uniqueness/no-clobber of existing tmp
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(tmp_path)?;

    file.write_all(bytes)?;
    file.flush()?;
    file.sync_all()?;

    // Rename
    // On Unix, rename is atomic replacing destination.
    // On Windows, rename fails if destination exists.
    #[cfg(unix)]
    {
        fs::rename(tmp_path, target_path)?;
    }

    #[cfg(windows)]
    {
        // Windows atomic-ish replacement: remove dest then rename.
        // There is a race window here, but it's better than failing.
        // v0.11.7 lock will fix the race.
        if target_path.exists() {
            let _ = fs::remove_file(target_path);
        }
        fs::rename(tmp_path, target_path)?;
    }

    // Non-unix/windows fallback (e.g. wasm? unlikely for this crate but safe default)
    #[cfg(not(any(unix, windows)))]
    {
        if target_path.exists() {
            let _ = fs::remove_file(target_path);
        }
        fs::rename(tmp_path, target_path)?;
    }

    Ok(())
}
