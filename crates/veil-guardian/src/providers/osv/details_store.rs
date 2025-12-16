use super::details::CachedVuln;
use directories::ProjectDirs;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Clone)]
pub struct DetailsStore {
    dir: PathBuf,
}

impl DetailsStore {
    /// Production default: ~/.cache/veil/guardian/osv/vulns (OS dependent)
    /// Production default: ~/.cache/veil/guardian/osv/vulns (OS dependent)
    /// If custom_path is provided, uses that as the ROOT for details (not appending vulns?).
    /// Wait, Cache uses `guardian/osv`. Details uses `guardian/osv/vulns`?
    /// `Cache::new`: `proj_dirs.cache_dir().join("guardian").join("osv")`.
    /// `DetailsStore::new`: `proj_dirs...join("guardian").join("osv").join("vulns")`.
    ///
    /// If I pass `temp/details`, I expect store to use `temp/details` directly?
    /// Yes, `Cache::new` uses custom path directly.
    /// So `DetailsStore::new` should too.
    pub fn new(custom_path: Option<PathBuf>) -> Option<Self> {
        let dir = if let Some(p) = custom_path {
            p
        } else {
            let proj_dirs = ProjectDirs::from("com", "veil-rs", "veil")?;
            proj_dirs
                .cache_dir()
                .join("guardian")
                .join("osv")
                .join("vulns")
        };
        fs::create_dir_all(&dir).ok()?;
        Some(Self { dir })
    }

    /// For tests / custom cache roots.
    pub fn with_dir(dir: impl Into<PathBuf>) -> io::Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    pub fn load(&self, vuln_id: &str) -> Option<CachedVuln> {
        let path = self.path_for(vuln_id);
        let s = fs::read_to_string(path).ok()?;
        serde_json::from_str::<CachedVuln>(&s).ok()
    }

    pub fn save(&self, entry: &CachedVuln) -> io::Result<()> {
        let path = self.path_for(&entry.vuln_id);
        let tmp = path.with_extension("tmp");

        let body = serde_json::to_string(&entry)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        fs::write(&tmp, body)?;
        if path.exists() {
            fs::remove_file(&path)?;
        }
        fs::rename(tmp, path)?;
        Ok(())
    }

    fn path_for(&self, vuln_id: &str) -> PathBuf {
        self.dir.join(format!("{}.json", sanitize_id(vuln_id)))
    }
}

// OSV IDs are usually safe, but just in case (Windows / weird IDs).
fn sanitize_id(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => c,
            _ => '_',
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::osv::details::{CachePolicy, CacheStatus, CachedVuln};
    use serde_json::json;
    use std::time::{Duration, SystemTime};

    #[test]
    fn ttl_boundaries() {
        let policy = CachePolicy::default();
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);

        let fresh = now - policy.fresh_ttl;
        assert_eq!(policy.classify(fresh, now), CacheStatus::Fresh);

        let stale = now - (policy.fresh_ttl + Duration::from_secs(1));
        assert_eq!(policy.classify(stale, now), CacheStatus::Stale);

        let expired = now - (policy.stale_ttl + Duration::from_secs(1));
        assert_eq!(policy.classify(expired, now), CacheStatus::Expired);
    }

    #[test]
    fn cache_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let store = DetailsStore::with_dir(tmp.path()).unwrap();

        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
        let entry = CachedVuln::new(
            "GHSA-aaaa-bbbb-cccc",
            now,
            json!({"id":"GHSA-aaaa-bbbb-cccc"}),
        );

        store.save(&entry).unwrap();
        let loaded = store.load("GHSA-aaaa-bbbb-cccc").unwrap();

        assert_eq!(loaded.vuln_id, entry.vuln_id);
        assert_eq!(loaded.fetched_at_unix, entry.fetched_at_unix);
        assert_eq!(loaded.vuln["id"], "GHSA-aaaa-bbbb-cccc");
    }

    #[test]
    fn corrupt_json_returns_none() {
        let tmp = tempfile::tempdir().unwrap();
        let store = DetailsStore::with_dir(tmp.path()).unwrap();

        let path = store.path_for("GHSA-bad");
        fs::write(path, "{not json").unwrap();

        assert!(store.load("GHSA-bad").is_none());
    }

    #[test]
    fn test_load_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = DetailsStore::with_dir(dir.path()).unwrap();

        let path = dir.path().join("GHSA-empty.json");
        std::fs::write(&path, "").unwrap();

        // Should silently fail/return None
        assert!(store.load("GHSA-empty").is_none());
    }

    #[test]
    fn test_load_non_json() {
        let dir = tempfile::tempdir().unwrap();
        let store = DetailsStore::with_dir(dir.path()).unwrap();

        let path = dir.path().join("GHSA-corrupt.json");
        std::fs::write(&path, "This is not JSON").unwrap();

        assert!(store.load("GHSA-corrupt").is_none());
    }

    #[test]
    fn test_load_legacy_format() {
        let dir = tempfile::tempdir().unwrap();
        let store = DetailsStore::with_dir(dir.path()).unwrap();

        let path = dir.path().join("GHSA-legacy.json");
        // Missing fetched_at_unix, schema_version, vuln_id
        let legacy = r#"{ "vuln": { "id": "GHSA-legacy", "summary": "Legacy" } }"#;
        std::fs::write(&path, legacy).unwrap();

        let loaded = store
            .load("GHSA-legacy")
            .expect("Should load legacy format");
        assert_eq!(loaded.fetched_at_unix, 0); // Default
        assert_eq!(loaded.schema_version, 0); // Default
                                              // vuln_id default is ""
        assert_eq!(loaded.vuln_id, "");

        // fetched_at() should be UNIX_EPOCH
        assert_eq!(loaded.fetched_at(), std::time::UNIX_EPOCH);
    }
}
