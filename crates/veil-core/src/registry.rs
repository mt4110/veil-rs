use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::FindingId;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("registry not found at {0}")]
    NotFound(PathBuf),
    #[error("failed to parse registry at {0}: {1}")]
    ParseError(PathBuf, String),
    #[error("failed to serialize registry: {0}")]
    SerializationError(String),
    #[error("version mismatch: expected {expected}, found {found}")]
    VersionMismatch { found: u32, expected: u32 },
    #[error("permission denied at {0}")]
    PermissionDenied(PathBuf),
    #[error("registry is locked by another process")]
    LockBusy,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExceptionEntry {
    pub id: FindingId,
    pub reason: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Registry {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub exceptions: Vec<ExceptionEntry>,
}

fn default_version() -> u32 {
    1
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            version: 1,
            exceptions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExceptionStatus {
    Active,
    Expired(DateTime<Utc>),
    NotExcepted,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(path: &Path) -> Result<Self, RegistryError> {
        if !path.exists() {
            return Err(RegistryError::NotFound(path.to_path_buf()));
        }

        let content = std::fs::read_to_string(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                RegistryError::PermissionDenied(path.to_path_buf())
            } else {
                RegistryError::Io(e)
            }
        })?;

        let registry: Registry = toml::from_str(&content).map_err(|e| {
            RegistryError::ParseError(path.to_path_buf(), e.to_string())
        })?;

        if registry.version != 1 {
            return Err(RegistryError::VersionMismatch {
                found: registry.version,
                expected: 1,
            });
        }

        Ok(registry)
    }

    pub fn save(&mut self, path: &Path) -> Result<(), RegistryError> {
        // 1. Sort entries by ID for canonical output
        self.exceptions.sort_by(|a, b| a.id.cmp(&b.id));

        // 2. Lock
        let lock_path = path.with_extension("lock");
        let lock_file = File::create(&lock_path).map_err(RegistryError::Io)?;
        
        lock_file.lock_exclusive().map_err(RegistryError::Io)?;

        // 3. Serialize (canonical format)
        let content = toml::to_string_pretty(self).map_err(|e| {
            RegistryError::SerializationError(e.to_string())
        })?;

        // 4. Atomic Write
        let dir = path.parent().unwrap_or_else(|| Path::new("."));
        
        let mut tmp = tempfile::Builder::new()
            .prefix(".exception_registry.tmp.")
            .suffix(".toml")
            .tempfile_in(dir)
            .map_err(RegistryError::Io)?;
            
        tmp.write_all(content.as_bytes()).map_err(RegistryError::Io)?;
        tmp.flush().map_err(RegistryError::Io)?;
        tmp.as_file().sync_all().map_err(RegistryError::Io)?;
        
        tmp.persist(path).map_err(|e| RegistryError::Io(e.error))?;
        
        // Unlock happens automatically when lock_file is dropped
        Ok(())
    }

    pub fn check(&self, id: &FindingId, now: DateTime<Utc>) -> ExceptionStatus {
        if let Some(entry) = self.exceptions.iter().find(|e| &e.id == id) {
            if let Some(expires_at) = entry.expires_at {
                if expires_at < now {
                    return ExceptionStatus::Expired(expires_at);
                }
            }
            ExceptionStatus::Active
        } else {
            ExceptionStatus::NotExcepted
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding_id::SpanData;

    fn make_id(s: &str) -> FindingId {
        FindingId::new("rule", Path::new("file"), &SpanData{start_line:0,start_col:0,end_line:0,end_col:0}, s)
    }

    #[test]
    fn test_serde_roundtrip() {
        let entry = ExceptionEntry {
            id: make_id("a"),
            reason: "reason".to_string(),
            created_at: Some(Utc::now()),
            created_by: Some("user".to_string()),
            expires_at: None,
        };
        let registry = Registry {
            version: 1,
            exceptions: vec![entry.clone()],
        };

        let toml_str = toml::to_string(&registry).unwrap();
        let loaded: Registry = toml::from_str(&toml_str).unwrap();

        assert_eq!(registry, loaded);
        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.exceptions.len(), 1);
    }
    
    #[test]
    fn test_version_mismatch() {
        let bad_toml = r#"
            version = 2
            [[exceptions]]
            id = "fx_..."
            reason = "test"
        "#;
        
        // Use underscore to suppress unused variable warning if only writing to file
        let _ = bad_toml;
        
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("registry.toml");
        
        // Construct a valid TOML but with version 2
        let id_str = make_id("test").to_string();
        let valid_but_v2 = format!(r#"
            version = 2
            [[exceptions]]
            id = "{}"
            reason = "test"
        "#, id_str);
        
        std::fs::write(&path, valid_but_v2).unwrap();
        
        let err = Registry::load(&path).unwrap_err();
        match err {
            RegistryError::VersionMismatch { found, expected } => {
                assert_eq!(found, 2);
                assert_eq!(expected, 1);
            }
            _ => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    fn test_save_canonical_sort() {
        let mut registry = Registry::new();
        let id1 = make_id("a");
        let id2 = make_id("b");
        
        let entry1 = ExceptionEntry { id: id1.clone(), reason: "1".into(), created_at: None, created_by: None, expires_at: None };
        let entry2 = ExceptionEntry { id: id2.clone(), reason: "2".into(), created_at: None, created_by: None, expires_at: None };
        
        if id1 < id2 {
            registry.exceptions.push(entry2.clone());
            registry.exceptions.push(entry1.clone());
        } else {
            registry.exceptions.push(entry1.clone());
            registry.exceptions.push(entry2.clone());
        }
        
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("registry.toml");
        
        registry.save(&path).unwrap();
        
        let loaded = Registry::load(&path).unwrap();
        assert!(loaded.exceptions[0].id <= loaded.exceptions[1].id);
        
        let content = std::fs::read_to_string(&path).unwrap();
        let pos1 = content.find(&loaded.exceptions[0].id.to_string()).unwrap();
        let pos2 = content.find(&loaded.exceptions[1].id.to_string()).unwrap();
        assert!(pos1 < pos2);
    }

    #[test]
    fn test_save_atomic() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("registry.toml");
        
        let mut registry = Registry::new();
        registry.exceptions.push(ExceptionEntry {
            id: make_id("test"),
            reason: "test".into(),
            created_at: None,
            created_by: None,
            expires_at: None,
        });
        
        registry.save(&path).unwrap();
        assert!(path.exists());
        assert!(path.with_extension("lock").exists());
        
        let loaded = Registry::load(&path).unwrap();
        assert_eq!(loaded.exceptions.len(), 1);
    }

    #[test]
    fn test_registry_check() {
        let mut registry = Registry::new();
        let id_active = make_id("active");
        let id_expired = make_id("expired");
        let id_future = make_id("future");
        let id_missing = make_id("missing");

        let now = Utc::now();
        let one_hour = chrono::Duration::hours(1);

        registry.exceptions.push(ExceptionEntry {
            id: id_active.clone(),
            reason: "active".into(),
            created_at: None,
            created_by: None,
            expires_at: None,
        });

        registry.exceptions.push(ExceptionEntry {
            id: id_expired.clone(),
            reason: "expired".into(),
            created_at: None,
            created_by: None,
            expires_at: Some(now - one_hour),
        });

        registry.exceptions.push(ExceptionEntry {
            id: id_future.clone(),
            reason: "future".into(),
            created_at: None,
            created_by: None,
            expires_at: Some(now + one_hour),
        });

        assert_eq!(registry.check(&id_active, now), ExceptionStatus::Active);
        
        match registry.check(&id_expired, now) {
            ExceptionStatus::Expired(_) => {},
            _ => panic!("expected expired"),
        }

        assert_eq!(registry.check(&id_future, now), ExceptionStatus::Active);
        assert_eq!(registry.check(&id_missing, now), ExceptionStatus::NotExcepted);
    }
}
