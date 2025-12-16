use directories::ProjectDirs;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    pub fn new() -> Option<Self> {
        // Use standard XDG dirs: ~/.cache/veil-rs/guardian/osv or equivalent
        let proj_dirs = ProjectDirs::from("com", "veil-rs", "veil")?;
        let cache_dir = proj_dirs.cache_dir().join("guardian").join("osv");

        fs::create_dir_all(&cache_dir).ok()?;

        Some(Self { cache_dir })
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let path = self.path_for(key);
        fs::read_to_string(path).ok()
    }

    pub fn put(&self, key: &str, content: &str) -> io::Result<()> {
        let path = self.path_for(key);
        // Atomic write
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, content)?;
        fs::rename(temp_path, path)?;
        Ok(())
    }

    fn path_for(&self, key: &str) -> PathBuf {
        let hash = blake3::hash(key.as_bytes()).to_hex();
        self.cache_dir.join(format!("{}.json", hash))
    }
}
