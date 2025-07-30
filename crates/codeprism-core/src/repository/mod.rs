//! Repository manager for orchestrating scanning and indexing operations
//!
//! This module provides high-level repository management functionality,
//! coordinating the scanner, indexer, and file monitoring components.

use crate::error::{Error, Result};
use crate::indexer::{BulkIndexer, IndexingConfig, IndexingResult, IndexingStats};
use crate::parser::{LanguageRegistry, ParserEngine};
use crate::scanner::{NoOpProgressReporter, ProgressReporter, RepositoryScanner};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    /// Repository ID (usually path or name)
    pub repo_id: String,
    /// Repository root path
    pub root_path: PathBuf,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Languages to include (None = all supported)
    pub include_languages: Option<Vec<String>>,
    /// Maximum file size to process (bytes)
    pub max_file_size: Option<usize>,
    /// Whether to follow symlinks
    pub follow_symlinks: bool,
    /// Custom exclude patterns
    pub exclude_patterns: Vec<String>,
    /// Repository metadata
    pub metadata: HashMap<String, String>,
}

impl RepositoryConfig {
    /// Create a new repository config
    pub fn new<P: AsRef<Path>>(repo_id: String, root_path: P) -> Self {
        let root_path = root_path.as_ref().to_path_buf();
        let name = root_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&repo_id)
            .to_string();

        Self {
            repo_id,
            root_path,
            name,
            description: None,
            include_languages: None,
            max_file_size: Some(10 * 1024 * 1024), // 10MB
            follow_symlinks: false,
            exclude_patterns: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set the display name
    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    /// Set the description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Add a metadata entry
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Repository health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Repository is healthy and up to date
    Healthy,
    /// Repository needs reindexing
    Stale,
    /// Repository has indexing errors
    Degraded {
        /// Number of files that failed to index
        error_count: usize,
    },
    /// Repository is corrupted or inaccessible
    Unhealthy {
        /// Description of why the repository is unhealthy
        reason: String,
    },
}

/// Repository statistics and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    /// Repository configuration
    pub config: RepositoryConfig,
    /// Health status
    pub health: HealthStatus,
    /// Last scan timestamp
    pub last_scan: Option<u64>,
    /// Last successful index timestamp
    pub last_index: Option<u64>,
    /// Indexing statistics from last run
    pub last_stats: Option<IndexingStats>,
    /// Total files indexed
    pub total_files: usize,
    /// Total nodes in graph
    pub total_nodes: usize,
    /// Total edges in graph
    pub total_edges: usize,
    /// Repository size in bytes
    pub repo_size_bytes: usize,
}

impl RepositoryInfo {
    /// Create new repository info
    pub fn new(config: RepositoryConfig) -> Self {
        Self {
            config,
            health: HealthStatus::Stale, // Needs initial indexing
            last_scan: None,
            last_index: None,
            last_stats: None,
            total_files: 0,
            total_nodes: 0,
            total_edges: 0,
            repo_size_bytes: 0,
        }
    }

    /// Check if repository needs reindexing
    pub fn needs_reindexing(&self) -> bool {
        matches!(
            self.health,
            HealthStatus::Stale | HealthStatus::Unhealthy { .. }
        )
    }

    /// Get time since last index in seconds
    pub fn time_since_last_index(&self) -> Option<u64> {
        self.last_index.map(|last| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                - last
        })
    }
}

/// Repository manager for coordinating scanning and indexing
pub struct RepositoryManager {
    scanner: RepositoryScanner,
    parser_engine: Arc<ParserEngine>,
    repositories: HashMap<String, RepositoryInfo>,
}

impl RepositoryManager {
    /// Create a new repository manager
    pub fn new(language_registry: Arc<LanguageRegistry>) -> Self {
        let parser_engine = Arc::new(ParserEngine::new(language_registry));
        let scanner = RepositoryScanner::new();

        Self {
            scanner,
            parser_engine,
            repositories: HashMap::new(),
        }
    }

    /// Create a new repository manager with custom configuration
    pub fn new_with_config(
        language_registry: Arc<LanguageRegistry>,
        exclude_dirs: Option<Vec<String>>,
        include_extensions: Option<Vec<String>>,
        dependency_mode: Option<crate::scanner::DependencyMode>,
    ) -> Self {
        let parser_engine = Arc::new(ParserEngine::new(language_registry));

        let mut scanner = if let Some(exclude_dirs) = exclude_dirs {
            RepositoryScanner::with_exclude_dirs(exclude_dirs)
        } else {
            RepositoryScanner::new()
        };

        if let Some(extensions) = include_extensions {
            scanner = scanner.with_extensions(extensions);
        }

        // Apply dependency mode if provided
        if let Some(dep_mode) = dependency_mode {
            scanner = scanner.with_dependency_mode(dep_mode);
        }

        Self {
            scanner,
            parser_engine,
            repositories: HashMap::new(),
        }
    }

    /// Register a repository
    pub fn register_repository(&mut self, config: RepositoryConfig) -> Result<()> {
        // Validate repository path exists
        if !config.root_path.exists() {
            return Err(Error::io(format!(
                "Repository path does not exist: {}",
                config.root_path.display()
            )));
        }

        if !config.root_path.is_dir() {
            return Err(Error::io(format!(
                "Repository path is not a directory: {}",
                config.root_path.display()
            )));
        }

        let repo_info = RepositoryInfo::new(config.clone());
        self.repositories.insert(config.repo_id.clone(), repo_info);

        Ok(())
    }

    /// Unregister a repository
    pub fn unregister_repository(&mut self, repo_id: &str) {
        self.repositories.remove(repo_id);
    }

    /// Get repository info
    pub fn get_repository(&self, repo_id: &str) -> Option<&RepositoryInfo> {
        self.repositories.get(repo_id)
    }

    /// Get all registered repositories
    pub fn list_repositories(&self) -> Vec<&RepositoryInfo> {
        self.repositories.values().collect()
    }

    /// Perform full repository scan and indexing
    pub async fn index_repository(
        &mut self,
        repo_id: &str,
        progress_reporter: Option<Arc<dyn ProgressReporter>>,
    ) -> Result<IndexingResult> {
        let repo_info = self
            .repositories
            .get_mut(repo_id)
            .ok_or_else(|| Error::other(format!("Repository not found: {repo_id}")))?;

        let progress = progress_reporter.unwrap_or_else(|| Arc::new(NoOpProgressReporter));

        // Step 1: Scan repository
        let scan_result = self
            .scanner
            .scan_repository(&repo_info.config.root_path, Arc::clone(&progress))
            .await?;

        // Update repository info with scan results
        repo_info.last_scan = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        repo_info.total_files = scan_result.total_files;

        // Step 2: Index discovered files
        let indexing_config = IndexingConfig::new(
            repo_id.to_string(),
            format!("scan-{}", chrono::Utc::now().timestamp()),
        );

        let indexer = BulkIndexer::new(indexing_config, Arc::clone(&self.parser_engine));
        let indexing_result = indexer.index_scan_result(&scan_result, progress).await?;

        // Update repository info with indexing results
        repo_info.last_index = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        repo_info.last_stats = Some(indexing_result.stats.clone());
        repo_info.total_nodes = indexing_result.stats.nodes_created;
        repo_info.total_edges = indexing_result.stats.edges_created;

        // Update health status
        repo_info.health = if indexing_result.stats.error_count == 0 {
            HealthStatus::Healthy
        } else if indexing_result.stats.error_count < indexing_result.stats.files_processed / 10 {
            HealthStatus::Degraded {
                error_count: indexing_result.stats.error_count,
            }
        } else {
            HealthStatus::Unhealthy {
                reason: format!(
                    "High error rate: {}/{} files failed",
                    indexing_result.stats.error_count, indexing_result.stats.files_processed
                ),
            }
        };

        Ok(indexing_result)
    }

    /// Quick repository health check
    pub async fn health_check(&mut self, repo_id: &str) -> Result<HealthStatus> {
        let repo_info = self
            .repositories
            .get_mut(repo_id)
            .ok_or_else(|| Error::other(format!("Repository not found: {repo_id}")))?;

        // Check if repository path still exists
        if !repo_info.config.root_path.exists() {
            repo_info.health = HealthStatus::Unhealthy {
                reason: "Repository path no longer exists".to_string(),
            };
            return Ok(repo_info.health.clone());
        }

        // Check if indexing is stale (older than 24 hours)
        if let Some(time_since) = repo_info.time_since_last_index() {
            if time_since > 24 * 60 * 60 {
                // 24 hours
                repo_info.health = HealthStatus::Stale;
            }
        }

        Ok(repo_info.health.clone())
    }

    /// Get repository statistics
    pub fn get_stats(&self, repo_id: &str) -> Option<&IndexingStats> {
        self.repositories
            .get(repo_id)
            .and_then(|info| info.last_stats.as_ref())
    }

    /// Get total statistics across all repositories
    pub fn get_total_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();

        let total_repos = self.repositories.len();
        let total_files: usize = self
            .repositories
            .values()
            .map(|info| info.total_files)
            .sum();
        let total_nodes: usize = self
            .repositories
            .values()
            .map(|info| info.total_nodes)
            .sum();
        let total_edges: usize = self
            .repositories
            .values()
            .map(|info| info.total_edges)
            .sum();

        stats.insert("repositories".to_string(), total_repos);
        stats.insert("files".to_string(), total_files);
        stats.insert("nodes".to_string(), total_nodes);
        stats.insert("edges".to_string(), total_edges);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::LanguageRegistry;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_manager() -> (RepositoryManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let registry = Arc::new(LanguageRegistry::new());
        let manager = RepositoryManager::new(registry);
        (manager, temp_dir)
    }

    #[test]
    fn test_repository_config() {
        let config = RepositoryConfig::new("test_repo".to_string(), "/tmp/test");

        assert_eq!(config.repo_id, "test_repo");
        assert_eq!(config.root_path, PathBuf::from("/tmp/test"));
        assert_eq!(config.name, "test");
    }

    #[test]
    fn test_repository_config_builder() {
        let config = RepositoryConfig::new("test".to_string(), "/tmp/test")
            .with_name("My Test Repo".to_string())
            .with_description("A test repository".to_string())
            .with_metadata("version".to_string(), "1.0".to_string());

        assert_eq!(config.name, "My Test Repo");
        assert_eq!(config.description, Some("A test repository".to_string()));
        assert_eq!(config.metadata.get("version"), Some(&"1.0".to_string()));
    }

    #[test]
    fn test_repository_info() {
        let config = RepositoryConfig::new("test".to_string(), "/tmp/test");
        let info = RepositoryInfo::new(config);

        assert!(info.needs_reindexing());
        assert!(matches!(info.health, HealthStatus::Stale));
        assert_eq!(info.total_files, 0);
    }

    #[test]
    fn test_repository_manager_creation() {
        let registry = Arc::new(LanguageRegistry::new());
        let manager = RepositoryManager::new(registry);

        assert_eq!(
            manager.list_repositories().len(),
            0,
            "New manager should start with no repositories"
        );
        let repos = manager.list_repositories();
        assert!(
            repos.is_empty(),
            "Repository list should be empty initially"
        );
    }

    #[test]
    fn test_register_repository() {
        let (mut manager, temp_dir) = create_test_manager();

        let config = RepositoryConfig::new("test_repo".to_string(), temp_dir.path());

        let result = manager.register_repository(config);
        assert!(result.is_ok(), "Repository operation should succeed");
        assert_eq!(
            manager.list_repositories().len(),
            1,
            "Should have 1 repository after registration"
        );

        // Verify repository content and properties
        let repos = manager.list_repositories();
        let repo = &repos[0];
        assert_eq!(
            repo.config.name, "test_repo",
            "Repository should have correct name"
        );
        assert_eq!(
            repo.config.path,
            temp_dir.path(),
            "Repository should have correct path"
        );
    }

    #[test]
    fn test_register_nonexistent_repository() {
        let (mut manager, _temp_dir) = create_test_manager();

        let config = RepositoryConfig::new("test_repo".to_string(), "/nonexistent/path");

        let result = manager.register_repository(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_unregister_repository() {
        let (mut manager, temp_dir) = create_test_manager();

        let config = RepositoryConfig::new("test_repo".to_string(), temp_dir.path());

        manager.register_repository(config).unwrap();
        assert_eq!(
            manager.list_repositories().len(),
            1,
            "Should have 1 repository after registration"
        );

        // Verify repository exists with correct name
        let repos_before = manager.list_repositories();
        assert_eq!(
            repos_before[0].config.name, "test_repo",
            "Repository should have correct name"
        );

        manager.unregister_repository("test_repo");
        assert_eq!(
            manager.list_repositories().len(),
            0,
            "Should have 0 repositories after unregistration"
        );

        // Verify repository is actually removed
        let repos_after = manager.list_repositories();
        assert!(
            repos_after.is_empty(),
            "Repository list should be empty after unregistration"
        );
        assert!(
            !repos_after.iter().any(|r| r.config.name == "test_repo"),
            "test_repo should be completely removed"
        );
    }

    #[tokio::test]
    async fn test_index_nonexistent_repository() {
        let (mut manager, _temp_dir) = create_test_manager();

        let result = manager.index_repository("nonexistent", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_health_check() {
        let (mut manager, temp_dir) = create_test_manager();

        // Create test file
        fs::write(temp_dir.path().join("test.js"), "console.log('hello');").unwrap();

        let config = RepositoryConfig::new("test_repo".to_string(), temp_dir.path());

        manager.register_repository(config).unwrap();

        let health = manager.health_check("test_repo").await.unwrap();
        assert!(matches!(health, HealthStatus::Stale));
    }

    #[test]
    fn test_total_stats() {
        let (mut manager, temp_dir) = create_test_manager();

        let config = RepositoryConfig::new("test_repo".to_string(), temp_dir.path());

        manager.register_repository(config).unwrap();

        let stats = manager.get_total_stats();
        assert_eq!(stats.get("repositories"), Some(&1));
        assert_eq!(stats.get("files"), Some(&0));
    }
}
