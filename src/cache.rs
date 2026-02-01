use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

const CACHE_FILE_NAME: &str = "cache.json";
const DEFAULT_TTL_SECONDS: i64 = 300; // 5 minutes

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    value: String,
    expires_at: DateTime<Utc>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Cache {
    entries: HashMap<String, CacheEntry>,
}

impl Cache {
    pub fn load() -> Result<Self> {
        let path = Self::cache_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read cache file: {:?}", path))?;
            let cache: Cache = serde_json::from_str(&content).unwrap_or_default();
            Ok(cache)
        } else {
            Ok(Cache::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::cache_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string(&self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn cache_path() -> Result<PathBuf> {
        if let Ok(path) = std::env::var("KSECRET_CACHE_FILE") {
            return Ok(PathBuf::from(path));
        }
        let home = dirs::home_dir().context("Could not determine home directory")?;
        let config_dir = home.join(".config").join("ksecret");
        Ok(config_dir.join(CACHE_FILE_NAME))
    }

    pub fn get(&self, env: &str, name: &str) -> Option<String> {
        let key = format!("{}:{}", env, name);
        if let Some(entry) = self.entries.get(&key) {
            if entry.expires_at > Utc::now() {
                return Some(entry.value.clone());
            }
        }
        None
    }

    pub fn set(&mut self, env: &str, name: &str, value: String) {
        let key = format!("{}:{}", env, name);
        self.entries.insert(
            key,
            CacheEntry {
                value,
                expires_at: Utc::now() + Duration::seconds(DEFAULT_TTL_SECONDS),
            },
        );
    }
    
    pub fn delete(&mut self, env: &str, name: &str) {
        let key = format!("{}:{}", env, name);
        self.entries.remove(&key);
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
