use super::details::CachedVuln;
use directories::ProjectDirs;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Clone)]
pub struct DetailsStore {
    dir: PathBuf,
}

#[derive(Debug, Default)]
pub struct QuarantineFlags {
    pub corrupt: bool,
    pub unsupported: bool,
    pub conflict: bool,
}

#[derive(Debug)]
pub enum StoreLoad {
    Hit {
        entry: CachedVuln,
        source: StoreSource,
        migrated: bool,
        quarantined: QuarantineFlags,
    },
    Miss {
        quarantined: QuarantineFlags,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum StoreSource {
    V1,
    Legacy,
}

impl DetailsStore {
    /// Recommended: <cache_dir>/osv (Unified Root)
    pub fn new(custom_path: Option<PathBuf>) -> Option<Self> {
        let dir = if let Some(p) = custom_path {
            p
        } else {
            let proj_dirs = ProjectDirs::from("com", "veil-rs", "veil")?;
            proj_dirs.cache_dir().join("guardian").join("osv")
        };
        fs::create_dir_all(&dir).ok()?;
        // Attempt to create v1 dir: <root>/vulns/v1
        let _ = fs::create_dir_all(dir.join("vulns").join("v1"));
        Some(Self { dir })
    }

    /// For tests / custom cache roots. Root is base dir (e.g. .../osv)
    pub fn with_dir(dir: impl Into<PathBuf>) -> io::Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(&dir)?;
        let _ = fs::create_dir_all(dir.join("vulns").join("v1"));
        Ok(Self { dir })
    }

    pub fn load(&self, vuln_id: &str) -> StoreLoad {
        let mut flags = QuarantineFlags::default();

        // Enforce Layout Conflict (Constitution Section 4.4) on v1 dir
        if let Err(_e) = self.ensure_v1_dir(&mut flags) {
            flags.conflict = true;
            return StoreLoad::Miss { quarantined: flags };
        }

        let normalized_key = crate::util::key::normalize_key(vuln_id);

        // 1. Try v1 (The Law: Step 1)
        let v1_path = self.path_for_v1(vuln_id);

        let v1_result = crate::util::file_lock::with_file_lock(&v1_path, || {
            match fs::read(&v1_path) {
                Ok(bytes) => {
                    // Parse Envelope
                    match serde_json::from_slice::<Envelope>(&bytes) {
                        Ok(env) => {
                            if env.schema_version != 1 {
                                let reason = format!("unsupported_v{}", env.schema_version);
                                return Err(io::Error::new(io::ErrorKind::InvalidData, reason));
                            }
                            if env.key != vuln_id && env.key != normalized_key {
                                // Strict mismatch -> Corruption
                                if env.key != vuln_id {
                                    return Err(io::Error::new(
                                        io::ErrorKind::InvalidData,
                                        "key_mismatch",
                                    ));
                                }
                            }
                            Ok(Some(env))
                        }
                        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(e),
            }
        });

        match v1_result {
            Ok(Some(env)) => {
                // Hit (v1)
                let entry = CachedVuln {
                    schema_version: 1,
                    vuln_id: vuln_id.to_string(), // Keep logical ID stable
                    fetched_at_unix: env.fetched_at_unix,
                    etag: env.etag, // Load ETag
                    vuln: env.payload,
                };
                return StoreLoad::Hit {
                    entry,
                    source: StoreSource::V1,
                    migrated: false,
                    quarantined: flags,
                };
            }
            Ok(None) => {}
            Err(e) if e.kind() == io::ErrorKind::InvalidData => {
                let reason = e.to_string();
                let q_reason = if reason.starts_with("unsupported_v") || reason == "key_mismatch" {
                    if reason.starts_with("unsupported_v") {
                        flags.unsupported = true;
                    } else {
                        flags.corrupt = true; // mismatch is corrupt
                    }
                    reason
                } else {
                    flags.corrupt = true;
                    "corrupt".to_string()
                };
                let _ = self.quarantine_file(&v1_path, &q_reason);
            }
            Err(_) => {}
        }

        // 2. Try Legacy (The Law: Step 2)
        // Legacy Source: <root>/GHSA-xxx.json (or <root>/osv/GHSA... if root is guardian)
        // Note: New Unified Strategy -> `load` looks in `dir` (root) for legacy files.
        let leg_path = self.path_for_legacy(vuln_id);
        let leg_result = crate::util::file_lock::with_file_lock(&leg_path, || {
            match fs::read_to_string(&leg_path) {
                Ok(s) => Ok(serde_json::from_str::<CachedVuln>(&s).ok()),
                Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(e),
            }
        });

        match leg_result {
            Ok(Some(legacy)) => {
                // Legacy Hit -> Migrate-on-Read
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                // Point B: Timestamp Mapping
                // If legacy has valid time (>0), keep it.
                // If 0, use conservative "Stale" mapping (NOT Expired).
                // Stale = now - (fresh_ttl + 1s).
                let mut fetched_at = legacy.fetched_at_unix;
                if fetched_at == 0 {
                    let policy = super::details::CachePolicy::default();
                    let fresh_secs = policy.fresh_ttl.as_secs();
                    // Make it stale by 1 second (safe conservative)
                    fetched_at = now.saturating_sub(fresh_secs + 1);
                }

                let env = Envelope {
                    schema_version: 1,
                    key: vuln_id.to_string(),
                    created_at_unix: now,
                    fetched_at_unix: fetched_at,
                    expires_at_unix: None,
                    etag: legacy.etag.clone(), // Preserve if present (rare)
                    source: "legacy_migration".to_string(),
                    payload: legacy.vuln.clone(),
                };

                let _ = self.save_envelope(&env);

                let mut entry = legacy;
                entry.fetched_at_unix = fetched_at;

                StoreLoad::Hit {
                    entry,
                    source: StoreSource::Legacy,
                    migrated: true,
                    quarantined: flags,
                }
            }
            Ok(None) => StoreLoad::Miss { quarantined: flags },
            Err(_) => StoreLoad::Miss { quarantined: flags },
        }
    }

    pub fn save(&self, entry: &CachedVuln) -> io::Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let env = Envelope {
            schema_version: 1,
            key: entry.vuln_id.clone(),
            created_at_unix: now,
            fetched_at_unix: entry.fetched_at_unix,
            expires_at_unix: None,
            etag: entry.etag.clone(), // Persist ETag
            source: "fetch".to_string(),
            payload: entry.vuln.clone(),
        };

        self.save_envelope(&env)
    }

    fn ensure_v1_dir(&self, flags: &mut QuarantineFlags) -> io::Result<()> {
        // v1 path: <root>/vulns/v1
        let v1_dir = self.dir.join("vulns").join("v1");
        if v1_dir.exists() {
            if v1_dir.is_file() {
                // Conflict: v1 exists as file.
                flags.conflict = true;
                // Action: Quarantine it.
                let _ = self.quarantine_file(&v1_dir, "corrupt_dirs_conflict");
                // Create dir
                fs::create_dir_all(&v1_dir)?;
            }
        } else {
            // Create if missing. Note: `new` tries, but `load` should ensure.
            // Parent `vulns` might need creation too.
            fs::create_dir_all(&v1_dir)?;
        }
        Ok(())
    }

    fn save_envelope(&self, env: &Envelope) -> io::Result<()> {
        let path = self.path_for_v1(&env.key);
        // Ensure parent dir exists (atomic write usually handles file logic but not dir creation)
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let body = serde_json::to_vec_pretty(env)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        crate::util::file_lock::with_file_lock(&path, || {
            crate::util::atomic_write::atomic_write_bytes(&path, &body)
        })
    }

    fn quarantine_file(&self, path: &std::path::Path, reason: &str) -> io::Result<()> {
        if !path.exists() {
            return Ok(());
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros();
        let pid = std::process::id();

        // Pattern: <filename>.corrupt.<micros>.<pid>
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let new_name = format!("{}.{}.{}.{}", file_name, reason, now, pid);
        let new_path = path.with_file_name(new_name);

        fs::rename(path, new_path)
    }

    fn path_for_v1(&self, vuln_id: &str) -> PathBuf {
        let name = crate::util::key::normalize_key(vuln_id);
        // Unified Path: <root>/vulns/v1/<norm>.json
        self.dir
            .join("vulns")
            .join("v1")
            .join(format!("{}.json", name))
    }

    fn path_for_legacy(&self, vuln_id: &str) -> PathBuf {
        // Legacy: <root>/<clean_id>.json (historical location in root)
        self.dir.join(format!("{}.json", sanitize_id(vuln_id)))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Envelope {
    schema_version: u32,
    key: String,
    created_at_unix: u64,
    fetched_at_unix: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_at_unix: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    etag: Option<String>,
    source: String,
    payload: serde_json::Value,
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
    use crate::providers::osv::details::CachedVuln;
    use serde_json::json;
    use std::time::{Duration, SystemTime};

    #[test]
    fn cache_roundtrip_with_etag() {
        let tmp = tempfile::tempdir().unwrap();
        let store = DetailsStore::with_dir(tmp.path()).unwrap();

        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
        let etag = Some("W/\"123\"".to_string());
        let entry = CachedVuln::new("GHSA-etag", now, json!({"id":"GHSA-etag"}), etag.clone());

        store.save(&entry).unwrap();
        match store.load("GHSA-etag") {
            StoreLoad::Hit { entry: loaded, .. } => {
                assert_eq!(loaded.vuln_id, entry.vuln_id);
                assert_eq!(loaded.etag, etag);
            }
            _ => panic!("Expected Hit"),
        }
    }

    #[test]
    fn test_load_legacy_format_path_check() {
        let dir = tempfile::tempdir().unwrap();
        let store = DetailsStore::with_dir(dir.path()).unwrap();

        // Legacy format in ROOT (where legacy lives)
        let path = dir.path().join("GHSA-legacy.json");
        let legacy = r#"{ "vuln": { "id": "GHSA-legacy", "summary": "Legacy" } }"#;
        std::fs::write(&path, legacy).unwrap();

        match store.load("GHSA-legacy") {
            StoreLoad::Hit {
                source,
                migrated,
                quarantined: _,
                ..
            } => {
                assert_eq!(source, StoreSource::Legacy);
                assert!(migrated);
            }
            _ => panic!("Expected Legacy Hit"),
        }

        // Verify MIGRATED to <root>/vulns/v1
        let v1_path = dir.path().join("vulns").join("v1").join("GHSA-legacy.json");
        assert!(v1_path.exists(), "Should have migrated to vulns/v1");
    }
}
