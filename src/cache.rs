use crate::models::ProjectType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCache {
    pub project_type: ProjectType,
    pub entry_point: String,
    pub package_manager: Option<String>,
    pub last_updated: u64,
    pub metadata: HashMap<String, String>,
}

impl ProjectCache {
    pub fn new(project_type: ProjectType, entry_point: String) -> Self {
        Self {
            project_type,
            entry_point,
            package_manager: None,
            last_updated: Self::current_timestamp(),
            metadata: HashMap::new(),
        }
    }

    pub fn is_valid(&self, max_age_seconds: u64) -> bool {
        let now = Self::current_timestamp();
        now.saturating_sub(self.last_updated) < max_age_seconds
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

pub struct CacheManager {
    cache_dir: PathBuf,
    cache: HashMap<String, ProjectCache>,
    max_age_seconds: u64,
}

impl CacheManager {
    pub fn new() -> anyhow::Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            cache_dir,
            cache: HashMap::new(),
            max_age_seconds: 3600, // 1 hour default
        })
    }

    pub fn get(&mut self, path: &str) -> anyhow::Result<Option<ProjectCache>> {
        // Check memory cache first
        if let Some(cache) = self.cache.get(path) {
            if cache.is_valid(self.max_age_seconds) {
                return Ok(Some(cache.clone()));
            } else {
                // Remove expired cache
                self.cache.remove(path);
            }
        }

        // Check file cache
        let cache_file = self.get_cache_file_path(path);
        if cache_file.exists() {
            match self.load_cache_from_file(&cache_file) {
                Ok(cache) if cache.is_valid(self.max_age_seconds) => {
                    // Store in memory and return
                    let cache_clone = cache.clone();
                    self.cache.insert(path.to_string(), cache);
                    Ok(Some(cache_clone))
                }
                _ => {
                    // Remove invalid cache file
                    let _ = std::fs::remove_file(&cache_file);
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    pub fn set(&mut self, path: String, mut cache: ProjectCache) -> anyhow::Result<()> {
        cache.last_updated = ProjectCache::current_timestamp();

        // Store in memory
        self.cache.insert(path.clone(), cache.clone());

        // Store to file
        let cache_file = self.get_cache_file_path(&path);
        self.save_cache_to_file(&cache_file, &cache)?;

        Ok(())
    }

    pub fn invalidate(&mut self, path: &str) -> anyhow::Result<()> {
        // Remove from memory
        self.cache.remove(path);

        // Remove from disk
        let cache_file = self.get_cache_file_path(path);
        if cache_file.exists() {
            std::fs::remove_file(&cache_file)?;
        }

        Ok(())
    }

    pub fn clear_all(&mut self) -> anyhow::Result<()> {
        // Clear memory cache
        self.cache.clear();

        // Clear disk cache
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
            std::fs::create_dir_all(&self.cache_dir)?;
        }

        Ok(())
    }

    pub fn stats(&self) -> CacheStats {
        let memory_entries = self.cache.len();
        let mut file_entries = 0;
        let mut total_size = 0;

        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    file_entries += 1;
                    total_size += metadata.len();
                }
            }
        }

        CacheStats {
            memory_entries,
            file_entries,
            total_size,
            max_age_seconds: self.max_age_seconds,
        }
    }

    fn get_cache_dir() -> anyhow::Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home_dir.join(".app-hoist").join("cache"))
    }

    fn get_cache_file_path(&self, path: &str) -> PathBuf {
        // Create a safe filename from the path
        let safe_name = path
            .replace('/', "_")
            .replace('\\', "_")
            .replace(':', "_")
            .replace(' ', "_");

        self.cache_dir.join(format!("{}.json", safe_name))
    }

    fn load_cache_from_file(&self, path: &Path) -> anyhow::Result<ProjectCache> {
        let content = std::fs::read_to_string(path)?;
        let cache: ProjectCache = serde_json::from_str(&content)?;
        Ok(cache)
    }

    fn save_cache_to_file(&self, path: &Path, cache: &ProjectCache) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(cache)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub memory_entries: usize,
    pub file_entries: usize,
    pub total_size: u64,
    pub max_age_seconds: u64,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache Stats: {} in memory, {} on disk, {} bytes total, {}s max age",
            self.memory_entries, self.file_entries, self.total_size, self.max_age_seconds
        )
    }
}