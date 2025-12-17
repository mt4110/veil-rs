use super::CacheStore;
use crate::guardian_next::types::{CacheEntry, CacheKey};
use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct DiskCache {
    dir: PathBuf,
}

impl DiskCache {
    pub fn new(dir: PathBuf) -> std::io::Result<Self> {
        fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    pub fn default_dir(app_name: &str) -> PathBuf {
        // XDG_CACHE_HOME > ~/.cache
        let base = std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
            .unwrap_or_else(|| PathBuf::from(".cache"));

        base.join(app_name).join("guardian_next")
    }

    fn key_path(&self, key: &CacheKey) -> PathBuf {
        // 安全にファイル名化（超単純版）: ':' -> '_'
        let safe = key.0.replace(':', "_");
        self.dir.join(format!("{safe}.json"))
    }
}

impl<T> CacheStore<T> for DiskCache
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    fn get(&self, key: &CacheKey) -> std::io::Result<Option<CacheEntry<T>>> {
        let path = self.key_path(key);
        crate::util::file_lock::with_file_lock(&path, || {
            if !path.exists() {
                return Ok(None);
            }
            let bytes = fs::read(&path)?;
            let entry: CacheEntry<T> = serde_json::from_slice(&bytes)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            Ok(Some(entry))
        })
    }

    fn put(&self, entry: &CacheEntry<T>) -> std::io::Result<()> {
        let key = CacheKey(entry.meta.key.clone());
        let path = self.key_path(&key);
        let bytes = serde_json::to_vec_pretty(entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        crate::util::file_lock::with_file_lock(&path, || {
            crate::util::atomic_write::atomic_write_bytes(&path, &bytes)
        })?;
        Ok(())
    }

    fn touch(&self, key: &CacheKey, now_unix: u64) -> std::io::Result<()> {
        if let Some(mut entry) = <DiskCache as CacheStore<T>>::get(self, key)? {
            entry.meta.fetched_at_unix = now_unix;
            self.put(&entry)?;
        }
        Ok(())
    }
}
