//! Analysis result caching system
//! 
//! Provides intelligent caching of expensive analysis operations to reduce
//! redundant computations and improve performance.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Unique identifier for cached analysis results
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    /// Tool name that generated the result
    pub tool_name: String,
    /// Parameters used (hashed for consistency)
    pub parameters_hash: u64,
    /// Target identifier (e.g., symbol_id, file_path)
    pub target: Option<String>,
}

impl CacheKey {
    /// Create a new cache key
    pub fn new(tool_name: String, parameters: &serde_json::Value, target: Option<String>) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        parameters.to_string().hash(&mut hasher);
        let parameters_hash = hasher.finish();

        Self {
            tool_name,
            parameters_hash,
            target,
        }
    }
}

/// Cached analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// The cached result
    pub result: serde_json::Value,
    /// When the result was cached
    pub cached_at: u64,
    /// How long the result is valid (in seconds)
    pub ttl_seconds: u64,
    /// Access count for LRU eviction
    pub access_count: u64,
    /// Last access time
    pub last_accessed: u64,
    /// Size estimate in bytes
    pub size_bytes: usize,
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(result: serde_json::Value, ttl_seconds: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        let size_bytes = result.to_string().len();

        Self {
            result,
            cached_at: now,
            ttl_seconds,
            access_count: 0,
            last_accessed: now,
            size_bytes,
        }
    }

    /// Check if the cache entry is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        now - self.cached_at > self.ttl_seconds
    }

    /// Record an access to this entry
    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
    }

    /// Get LRU score (lower is more likely to be evicted)
    pub fn lru_score(&self) -> u64 {
        // Combine access count and recency
        self.access_count + (self.last_accessed / 3600) // Favor recent access
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default, Serialize)]
pub struct CacheStats {
    /// Total number of cache hits
    pub hits: u64,
    /// Total number of cache misses
    pub misses: u64,
    /// Number of cached entries
    pub entry_count: usize,
    /// Total memory usage in bytes
    pub memory_usage_bytes: usize,
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f64,
}

impl CacheStats {
    /// Calculate hit rate
    pub fn calculate_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        self.hit_rate = if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        };
    }
}

/// Configuration for cache behavior
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries
    pub max_entries: usize,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Default TTL for cached results
    pub default_ttl_seconds: u64,
    /// TTL settings for specific tools
    pub tool_ttl_overrides: HashMap<String, u64>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        let mut tool_ttl_overrides = HashMap::new();
        
        // Long TTL for expensive operations
        tool_ttl_overrides.insert("trace_inheritance".to_string(), 3600); // 1 hour
        tool_ttl_overrides.insert("analyze_decorators".to_string(), 3600); // 1 hour
        tool_ttl_overrides.insert("analyze_complexity".to_string(), 1800); // 30 minutes
        tool_ttl_overrides.insert("analyze_security".to_string(), 1800); // 30 minutes
        
        // Medium TTL for moderately expensive operations
        tool_ttl_overrides.insert("find_dependencies".to_string(), 900); // 15 minutes
        tool_ttl_overrides.insert("find_references".to_string(), 900); // 15 minutes
        tool_ttl_overrides.insert("detect_patterns".to_string(), 600); // 10 minutes
        
        // Short TTL for fast changing results
        tool_ttl_overrides.insert("search_symbols".to_string(), 300); // 5 minutes
        tool_ttl_overrides.insert("search_content".to_string(), 300); // 5 minutes

        Self {
            max_entries: 1000,
            max_memory_bytes: 50 * 1024 * 1024, // 50MB
            default_ttl_seconds: 600, // 10 minutes
            tool_ttl_overrides,
        }
    }
}

impl CacheConfig {
    /// Get TTL for a specific tool
    pub fn get_ttl_for_tool(&self, tool_name: &str) -> u64 {
        self.tool_ttl_overrides
            .get(tool_name)
            .copied()
            .unwrap_or(self.default_ttl_seconds)
    }
}

/// Analysis result cache with LRU eviction and TTL expiration
#[derive(Debug)]
pub struct AnalysisCache {
    /// Cached entries
    cache: Arc<RwLock<HashMap<CacheKey, CacheEntry>>>,
    /// Cache configuration
    config: CacheConfig,
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
}

impl AnalysisCache {
    /// Create a new analysis cache
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new analysis cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Get a cached result
    pub fn get(
        &self,
        tool_name: &str,
        parameters: &serde_json::Value,
        target: Option<&str>,
    ) -> Result<Option<serde_json::Value>> {
        let key = CacheKey::new(tool_name.to_string(), parameters, target.map(|s| s.to_string()));
        
        let mut cache = self.cache.write().map_err(|_| {
            anyhow::anyhow!("Failed to acquire write lock on cache")
        })?;
        
        if let Some(entry) = cache.get_mut(&key) {
            if entry.is_expired() {
                // Remove expired entry
                cache.remove(&key);
                self.record_miss()?;
                return Ok(None);
            }
            
            // Record access and return result
            entry.record_access();
            self.record_hit()?;
            return Ok(Some(entry.result.clone()));
        }
        
        self.record_miss()?;
        Ok(None)
    }

    /// Store a result in the cache
    pub fn put(
        &self,
        tool_name: &str,
        parameters: &serde_json::Value,
        target: Option<&str>,
        result: serde_json::Value,
    ) -> Result<()> {
        let key = CacheKey::new(tool_name.to_string(), parameters, target.map(|s| s.to_string()));
        let ttl = self.config.get_ttl_for_tool(tool_name);
        let entry = CacheEntry::new(result, ttl);
        
        let mut cache = self.cache.write().map_err(|_| {
            anyhow::anyhow!("Failed to acquire write lock on cache")
        })?;
        
        // Check if we need to evict entries
        self.maybe_evict_entries(&mut cache)?;
        
        cache.insert(key, entry);
        self.update_memory_stats(&cache)?;
        
        Ok(())
    }

    /// Clear expired entries
    pub fn cleanup_expired(&self) -> Result<usize> {
        let mut cache = self.cache.write().map_err(|_| {
            anyhow::anyhow!("Failed to acquire write lock on cache")
        })?;
        
        let initial_count = cache.len();
        cache.retain(|_, entry| !entry.is_expired());
        
        let removed_count = initial_count - cache.len();
        self.update_memory_stats(&cache)?;
        
        Ok(removed_count)
    }

    /// Clear all cached entries
    pub fn clear(&self) -> Result<()> {
        let mut cache = self.cache.write().map_err(|_| {
            anyhow::anyhow!("Failed to acquire write lock on cache")
        })?;
        
        cache.clear();
        
        let mut stats = self.stats.write().map_err(|_| {
            anyhow::anyhow!("Failed to acquire write lock on stats")
        })?;
        
        stats.entry_count = 0;
        stats.memory_usage_bytes = 0;
        
        Ok(())
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> Result<CacheStats> {
        let stats = self.stats.read().map_err(|_| {
            anyhow::anyhow!("Failed to acquire read lock on stats")
        })?;
        
        Ok(stats.clone())
    }

    /// Check if caching is beneficial for a tool
    pub fn should_cache(&self, tool_name: &str) -> bool {
        // Don't cache fast operations
        matches!(tool_name, 
            "repository_stats" | "content_stats" | "find_files" | 
            "trace_path" | "explain_symbol" | "trace_inheritance" | 
            "analyze_decorators" | "analyze_complexity" | "find_duplicates" |
            "analyze_security" | "analyze_performance" | "detect_patterns" |
            "find_dependencies" | "find_references" | "analyze_transitive_dependencies"
        )
    }

    /// Record a cache hit
    fn record_hit(&self) -> Result<()> {
        let mut stats = self.stats.write().map_err(|_| {
            anyhow::anyhow!("Failed to acquire write lock on stats")
        })?;
        
        stats.hits += 1;
        stats.calculate_hit_rate();
        
        Ok(())
    }

    /// Record a cache miss
    fn record_miss(&self) -> Result<()> {
        let mut stats = self.stats.write().map_err(|_| {
            anyhow::anyhow!("Failed to acquire write lock on stats")
        })?;
        
        stats.misses += 1;
        stats.calculate_hit_rate();
        
        Ok(())
    }

    /// Update memory usage statistics
    fn update_memory_stats(&self, cache: &HashMap<CacheKey, CacheEntry>) -> Result<()> {
        let mut stats = self.stats.write().map_err(|_| {
            anyhow::anyhow!("Failed to acquire write lock on stats")
        })?;
        
        stats.entry_count = cache.len();
        stats.memory_usage_bytes = cache.values().map(|entry| entry.size_bytes).sum();
        
        Ok(())
    }

    /// Evict entries if necessary
    fn maybe_evict_entries(&self, cache: &mut HashMap<CacheKey, CacheEntry>) -> Result<()> {
        // Check entry count limit
        if cache.len() >= self.config.max_entries {
            self.evict_lru_entries(cache, cache.len() - self.config.max_entries + 1)?;
        }
        
        // Check memory limit
        let memory_usage: usize = cache.values().map(|entry| entry.size_bytes).sum();
        if memory_usage > self.config.max_memory_bytes {
            // Evict until we're under the limit
            let target_reduction = memory_usage - self.config.max_memory_bytes;
            self.evict_by_memory(cache, target_reduction)?;
        }
        
        Ok(())
    }

    /// Evict LRU entries
    fn evict_lru_entries(&self, cache: &mut HashMap<CacheKey, CacheEntry>, count: usize) -> Result<()> {
        // Collect entries with LRU scores
        let mut entries: Vec<_> = cache.iter().map(|(key, entry)| (key.clone(), entry.lru_score())).collect();
        
        // Sort by LRU score (ascending - lowest scores first)
        entries.sort_by_key(|(_, score)| *score);
        
        // Remove the least recently used entries
        for (key, _) in entries.into_iter().take(count) {
            cache.remove(&key);
        }
        
        Ok(())
    }

    /// Evict entries to reduce memory usage
    fn evict_by_memory(&self, cache: &mut HashMap<CacheKey, CacheEntry>, target_reduction: usize) -> Result<()> {
        // Collect entries sorted by LRU score
        let mut entries: Vec<_> = cache.iter()
            .map(|(key, entry)| (key.clone(), entry.lru_score(), entry.size_bytes))
            .collect();
        
        // Sort by LRU score (ascending)
        entries.sort_by_key(|(_, score, _)| *score);
        
        // Remove entries until we've freed enough memory
        let mut freed_bytes = 0;
        for (key, _, size) in entries {
            if freed_bytes >= target_reduction {
                break;
            }
            
            cache.remove(&key);
            freed_bytes += size;
        }
        
        Ok(())
    }
}

impl Default for AnalysisCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_creation() {
        let params = serde_json::json!({"symbol_id": "test123"});
        let key1 = CacheKey::new("explain_symbol".to_string(), &params, Some("target".to_string()));
        let key2 = CacheKey::new("explain_symbol".to_string(), &params, Some("target".to_string()));
        
        assert_eq!(key1, key2);
        assert_eq!(key1.tool_name, "explain_symbol");
        assert_eq!(key1.target, Some("target".to_string()));
    }

    #[test]
    fn test_cache_entry() {
        let result = serde_json::json!({"data": "test"});
        let mut entry = CacheEntry::new(result.clone(), 600);
        
        assert!(!entry.is_expired());
        assert_eq!(entry.access_count, 0);
        
        entry.record_access();
        assert_eq!(entry.access_count, 1);
    }

    #[test]
    fn test_cache_operations() {
        let cache = AnalysisCache::new();
        let params = serde_json::json!({"test": "value"});
        let result = serde_json::json!({"result": "data"});
        
        // Test miss
        let cached = cache.get("test_tool", &params, None).unwrap();
        assert!(cached.is_none());
        
        // Test put and hit
        cache.put("test_tool", &params, None, result.clone()).unwrap();
        let cached = cache.get("test_tool", &params, None).unwrap();
        assert_eq!(cached, Some(result));
        
        // Check stats
        let stats = cache.get_stats().unwrap();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }

    #[test]
    fn test_cache_config() {
        let config = CacheConfig::default();
        
        // Test tool-specific TTL
        assert_eq!(config.get_ttl_for_tool("trace_inheritance"), 3600);
        assert_eq!(config.get_ttl_for_tool("unknown_tool"), config.default_ttl_seconds);
    }

    #[test]
    fn test_cache_cleanup() {
        let cache = AnalysisCache::new();
        let params = serde_json::json!({"test": "value"});
        let result = serde_json::json!({"result": "data"});
        
        // Add an entry
        cache.put("test_tool", &params, None, result).unwrap();
        
        // Clear cache
        cache.clear().unwrap();
        
        // Verify it's empty
        let cached = cache.get("test_tool", &params, None).unwrap();
        assert!(cached.is_none());
    }
} 