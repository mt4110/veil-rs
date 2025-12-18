use super::CacheStore;
use crate::guardian_next::types::{CacheEntry, CacheKey};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct MemoryCache<T> {
    inner: Arc<RwLock<HashMap<CacheKey, CacheEntry<T>>>>,
}

impl<T> MemoryCache<T> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<T> Default for MemoryCache<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + Sync> CacheStore<T> for MemoryCache<T> {
    fn get(&self, key: &CacheKey) -> std::io::Result<Option<CacheEntry<T>>> {
        Ok(self.inner.read().unwrap().get(key).cloned())
    }

    fn put(&self, entry: &CacheEntry<T>) -> std::io::Result<()> {
        self.inner
            .write()
            .unwrap()
            .insert(CacheKey(entry.meta.key.clone()), entry.clone());
        Ok(())
    }

    fn touch(&self, key: &CacheKey, now_unix: u64) -> std::io::Result<()> {
        if let Some(e) = self.inner.write().unwrap().get_mut(key) {
            e.meta.fetched_at_unix = now_unix;
        }
        Ok(())
    }
}
