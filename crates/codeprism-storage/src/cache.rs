//! Cache storage implementations

use crate::{CacheStats, CacheStorage};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// LRU cache entry with TTL support
#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    last_accessed: SystemTime,
    expires_at: Option<SystemTime>,
}

/// In-memory LRU cache storage
pub struct LruCacheStorage {
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    max_size_bytes: usize,
    current_size_bytes: Arc<Mutex<usize>>,
    stats: Arc<Mutex<CacheStats>>,
}

impl LruCacheStorage {
    /// Create a new LRU cache with the specified maximum size in bytes
    pub fn new(max_size_bytes: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_size_bytes,
            current_size_bytes: Arc::new(Mutex::new(0)),
            stats: Arc::new(Mutex::new(CacheStats {
                total_keys: 0,
                memory_usage_bytes: 0,
                hit_count: 0,
                miss_count: 0,
                eviction_count: 0,
            })),
        }
    }

    /// Evict expired entries
    fn evict_expired(&self) -> Result<()> {
        let now = SystemTime::now();
        let mut cache = self.cache.lock().unwrap();
        let mut current_size = self.current_size_bytes.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        let keys_to_remove: Vec<String> = cache
            .iter()
            .filter_map(|(key, entry)| {
                if let Some(expires_at) = entry.expires_at {
                    if now > expires_at {
                        Some(key.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        for key in keys_to_remove {
            if let Some(entry) = cache.remove(&key) {
                *current_size -= entry.data.len();
                stats.eviction_count += 1;
            }
        }

        stats.total_keys = cache.len();
        stats.memory_usage_bytes = *current_size;

        Ok(())
    }

    /// Evict least recently used entries to make space
    fn evict_lru(&self, needed_space: usize) -> Result<()> {
        let mut cache = self.cache.lock().unwrap();
        let mut current_size = self.current_size_bytes.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        while *current_size + needed_space > self.max_size_bytes && !cache.is_empty() {
            // Find the least recently used entry
            let lru_key = cache
                .iter()
                .min_by_key(|(_, entry)| entry.last_accessed)
                .map(|(key, _)| key.clone());

            if let Some(key) = lru_key {
                if let Some(entry) = cache.remove(&key) {
                    *current_size -= entry.data.len();
                    stats.eviction_count += 1;
                }
            } else {
                break;
            }
        }

        stats.total_keys = cache.len();
        stats.memory_usage_bytes = *current_size;

        Ok(())
    }
}

#[async_trait]
impl CacheStorage for LruCacheStorage {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        // First evict expired entries
        self.evict_expired()?;

        let mut cache = self.cache.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        if let Some(entry) = cache.get_mut(key) {
            // Check if entry has expired
            if let Some(expires_at) = entry.expires_at {
                if SystemTime::now() > expires_at {
                    cache.remove(key);
                    stats.miss_count += 1;
                    stats.total_keys = cache.len();
                    return Ok(None);
                }
            }

            // Update last accessed time
            entry.last_accessed = SystemTime::now();
            stats.hit_count += 1;

            // Deserialize the data
            let value: T = bincode::deserialize(&entry.data)?;
            Ok(Some(value))
        } else {
            stats.miss_count += 1;
            Ok(None)
        }
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        // First evict expired entries
        self.evict_expired()?;

        let serialized = bincode::serialize(value)?;
        let needed_space = serialized.len();

        // Evict LRU entries if needed
        self.evict_lru(needed_space)?;

        let expires_at = ttl.map(|duration| SystemTime::now() + duration);

        let entry = CacheEntry {
            data: serialized,
            last_accessed: SystemTime::now(),
            expires_at,
        };

        let mut cache = self.cache.lock().unwrap();
        let mut current_size = self.current_size_bytes.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        // Remove old entry if it exists
        if let Some(old_entry) = cache.remove(key) {
            *current_size -= old_entry.data.len();
        }

        // Add new entry
        *current_size += entry.data.len();
        cache.insert(key.to_string(), entry);

        stats.total_keys = cache.len();
        stats.memory_usage_bytes = *current_size;

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut cache = self.cache.lock().unwrap();
        let mut current_size = self.current_size_bytes.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        if let Some(entry) = cache.remove(key) {
            *current_size -= entry.data.len();
        }

        stats.total_keys = cache.len();
        stats.memory_usage_bytes = *current_size;

        Ok(())
    }

    async fn invalidate_pattern(&self, pattern: &str) -> Result<()> {
        let mut cache = self.cache.lock().unwrap();
        let mut current_size = self.current_size_bytes.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        let keys_to_remove: Vec<String> = cache
            .keys()
            .filter(|key| key.contains(pattern))
            .cloned()
            .collect();

        for key in keys_to_remove {
            if let Some(entry) = cache.remove(&key) {
                *current_size -= entry.data.len();
            }
        }

        stats.total_keys = cache.len();
        stats.memory_usage_bytes = *current_size;

        Ok(())
    }

    async fn get_stats(&self) -> Result<CacheStats> {
        let stats = self.stats.lock().unwrap();
        Ok(stats.clone())
    }

    async fn clear(&self) -> Result<()> {
        let mut cache = self.cache.lock().unwrap();
        let mut current_size = self.current_size_bytes.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        cache.clear();
        *current_size = 0;

        stats.total_keys = 0;
        stats.memory_usage_bytes = 0;

        Ok(())
    }
}
