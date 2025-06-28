//! Storage layer for CodePrism code intelligence
//!
//! This module provides persistent storage for code graphs, analysis results,
//! and cached data to improve performance and enable incremental analysis.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

pub mod backends;
pub mod cache;
pub mod config;
pub mod graph;
pub mod serialization;

pub use backends::*;
pub use cache::*;
pub use config::*;
pub use graph::*;

/// Core storage trait for code graphs
#[async_trait]
pub trait GraphStorage: Send + Sync {
    /// Store a complete code graph
    async fn store_graph(&self, graph: &SerializableGraph) -> Result<()>;

    /// Load a code graph by repository ID
    async fn load_graph(&self, repo_id: &str) -> Result<Option<SerializableGraph>>;

    /// Update specific nodes in the graph
    async fn update_nodes(&self, repo_id: &str, nodes: &[SerializableNode]) -> Result<()>;

    /// Update specific edges in the graph
    async fn update_edges(&self, repo_id: &str, edges: &[SerializableEdge]) -> Result<()>;

    /// Delete nodes by IDs
    async fn delete_nodes(&self, repo_id: &str, node_ids: &[String]) -> Result<()>;

    /// Delete edges by source and target
    async fn delete_edges(&self, repo_id: &str, edge_refs: &[EdgeReference]) -> Result<()>;

    /// Get graph metadata
    async fn get_graph_metadata(&self, repo_id: &str) -> Result<Option<GraphMetadata>>;

    /// Update graph metadata
    async fn update_graph_metadata(&self, repo_id: &str, metadata: &GraphMetadata) -> Result<()>;

    /// List all stored repositories
    async fn list_repositories(&self) -> Result<Vec<String>>;

    /// Delete entire graph for a repository
    async fn delete_graph(&self, repo_id: &str) -> Result<()>;

    /// Check if a graph exists
    async fn graph_exists(&self, repo_id: &str) -> Result<bool>;
}

/// Cache storage trait for temporary data
#[async_trait]
pub trait CacheStorage: Send + Sync {
    /// Get a cached value by key
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send;

    /// Set a cached value with optional TTL
    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync;

    /// Delete a cached value
    async fn delete(&self, key: &str) -> Result<()>;

    /// Invalidate all keys matching a pattern
    async fn invalidate_pattern(&self, pattern: &str) -> Result<()>;

    /// Get cache statistics
    async fn get_stats(&self) -> Result<CacheStats>;

    /// Clear all cached data
    async fn clear(&self) -> Result<()>;
}

/// Analysis results storage trait
#[async_trait]
pub trait AnalysisStorage: Send + Sync {
    /// Store analysis results
    async fn store_analysis(&self, result: &AnalysisResult) -> Result<()>;

    /// Load analysis results by ID
    async fn load_analysis(&self, result_id: &str) -> Result<Option<AnalysisResult>>;

    /// Find analysis results by repository and type
    async fn find_analysis(
        &self,
        repo_id: &str,
        analysis_type: Option<&str>,
        since: Option<SystemTime>,
    ) -> Result<Vec<AnalysisResult>>;

    /// Delete analysis results
    async fn delete_analysis(&self, result_id: &str) -> Result<()>;

    /// Clean up old analysis results
    async fn cleanup_old_results(&self, older_than: SystemTime) -> Result<usize>;
}

/// Edge reference for deletion operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeReference {
    pub source: String,
    pub target: String,
    pub kind: String,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_keys: usize,
    pub memory_usage_bytes: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u64,
}

/// Analysis result storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub id: String,
    pub repo_id: String,
    pub analysis_type: String,
    pub timestamp: SystemTime,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

/// Storage backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageBackend {
    InMemory,
    File,
    Sqlite,
    Neo4j,
}

/// Main storage manager that coordinates different storage backends
pub struct StorageManager {
    graph_storage: Box<dyn GraphStorage>,
    cache_storage: cache::LruCacheStorage,
    analysis_storage: Box<dyn AnalysisStorage>,
    config: StorageConfig,
}

impl StorageManager {
    /// Create a new storage manager with the specified configuration
    pub async fn new(config: StorageConfig) -> Result<Self> {
        let graph_storage = create_graph_storage(&config).await?;
        let cache_storage = cache::LruCacheStorage::new(config.cache_size_mb * 1024 * 1024);
        let analysis_storage = create_analysis_storage(&config).await?;

        Ok(Self {
            graph_storage,
            cache_storage,
            analysis_storage,
            config,
        })
    }

    /// Get a reference to the graph storage
    pub fn graph(&self) -> &dyn GraphStorage {
        self.graph_storage.as_ref()
    }

    /// Get a reference to the cache storage
    pub fn cache(&self) -> &cache::LruCacheStorage {
        &self.cache_storage
    }

    /// Get a reference to the analysis storage
    pub fn analysis(&self) -> &dyn AnalysisStorage {
        self.analysis_storage.as_ref()
    }

    /// Get the storage configuration
    pub fn config(&self) -> &StorageConfig {
        &self.config
    }

    /// Perform maintenance operations (cleanup, optimization, etc.)
    pub async fn maintenance(&self) -> Result<()> {
        // Clean up old analysis results
        let cutoff = SystemTime::now() - self.config.retention_period;
        let deleted = self.analysis_storage.cleanup_old_results(cutoff).await?;
        tracing::info!("Cleaned up {} old analysis results", deleted);

        // Additional maintenance operations can be added here
        Ok(())
    }
}

/// Create appropriate graph storage backend
async fn create_graph_storage(config: &StorageConfig) -> Result<Box<dyn GraphStorage>> {
    match config.backend {
        StorageBackend::InMemory => Ok(Box::new(backends::InMemoryGraphStorage::new())),
        StorageBackend::File => Ok(Box::new(
            backends::FileGraphStorage::new(&config.data_path).await?,
        )),
        StorageBackend::Sqlite => Ok(Box::new(
            backends::SqliteGraphStorage::new(&config.data_path).await?,
        )),
        StorageBackend::Neo4j => Ok(Box::new(
            backends::Neo4jGraphStorage::new(&config.connection_string).await?,
        )),
    }
}

/// Create appropriate analysis storage backend
async fn create_analysis_storage(config: &StorageConfig) -> Result<Box<dyn AnalysisStorage>> {
    match config.backend {
        StorageBackend::InMemory => Ok(Box::new(backends::InMemoryAnalysisStorage::new())),
        _ => Ok(Box::new(
            backends::FileAnalysisStorage::new(&config.data_path).await?,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_storage_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig {
            backend: StorageBackend::InMemory,
            data_path: temp_dir.path().to_path_buf(),
            cache_size_mb: 64,
            persistence_interval: Duration::from_secs(60),
            compression_enabled: false,
            retention_period: Duration::from_secs(86400),
            connection_string: None,
        };

        let storage = StorageManager::new(config).await.unwrap();
        assert_eq!(storage.config().backend, StorageBackend::InMemory);
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let temp_dir = tempdir().unwrap();
        let config = StorageConfig::default_for_testing(temp_dir.path());
        let storage = StorageManager::new(config).await.unwrap();

        // Test cache set/get
        storage
            .cache()
            .set("test_key", &"test_value", None)
            .await
            .unwrap();
        let value: Option<String> = storage.cache().get("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // Test cache delete
        storage.cache().delete("test_key").await.unwrap();
        let value: Option<String> = storage.cache().get("test_key").await.unwrap();
        assert_eq!(value, None);
    }
}
