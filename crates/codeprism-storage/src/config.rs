//! Storage configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

use crate::StorageBackend;

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage backend type
    pub backend: StorageBackend,
    /// Data storage path (for file-based backends)
    pub data_path: PathBuf,
    /// Cache size in megabytes
    pub cache_size_mb: usize,
    /// Persistence interval for periodic saves
    pub persistence_interval: Duration,
    /// Enable compression for stored data
    pub compression_enabled: bool,
    /// Retention period for analysis results
    pub retention_period: Duration,
    /// Connection string for database backends
    pub connection_string: Option<String>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::InMemory,
            data_path: PathBuf::from("./codeprism_data"),
            cache_size_mb: 256,
            persistence_interval: Duration::from_secs(300), // 5 minutes
            compression_enabled: true,
            retention_period: Duration::from_secs(86400 * 7), // 1 week
            connection_string: None,
        }
    }
}

impl StorageConfig {
    /// Create a new storage configuration
    pub fn new(backend: StorageBackend, data_path: PathBuf) -> Self {
        Self {
            backend,
            data_path,
            ..Default::default()
        }
    }

    /// Create configuration for in-memory storage
    pub fn in_memory() -> Self {
        Self {
            backend: StorageBackend::InMemory,
            ..Default::default()
        }
    }

    /// Create configuration for file-based storage
    pub fn file_based(data_path: PathBuf) -> Self {
        Self {
            backend: StorageBackend::File,
            data_path,
            ..Default::default()
        }
    }

    /// Create configuration for SQLite storage
    pub fn sqlite(data_path: PathBuf) -> Self {
        Self {
            backend: StorageBackend::Sqlite,
            data_path,
            ..Default::default()
        }
    }

    /// Set cache size in megabytes
    pub fn with_cache_size(mut self, size_mb: usize) -> Self {
        self.cache_size_mb = size_mb;
        self
    }

    /// Set persistence interval
    pub fn with_persistence_interval(mut self, interval: Duration) -> Self {
        self.persistence_interval = interval;
        self
    }

    /// Enable or disable compression
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.compression_enabled = enabled;
        self
    }

    /// Set retention period for analysis results
    pub fn with_retention_period(mut self, period: Duration) -> Self {
        self.retention_period = period;
        self
    }

    #[cfg(test)]
    pub fn default_for_testing(data_path: &std::path::Path) -> Self {
        Self {
            backend: StorageBackend::InMemory,
            data_path: data_path.to_path_buf(),
            cache_size_mb: 64,
            persistence_interval: Duration::from_secs(60),
            compression_enabled: false,
            retention_period: Duration::from_secs(86400),
            connection_string: None,
        }
    }
}
