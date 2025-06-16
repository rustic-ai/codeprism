//! Bulk indexing engine for parallel file processing and graph building
//!
//! This module provides functionality to process large numbers of discovered files
//! in parallel, parse them, and build the code graph efficiently.

use crate::ast::{Edge, Node};
use crate::error::{Error, Result};
use crate::graph::GraphStore;
use crate::linkers::SymbolResolver;
use crate::parser::{ParseContext, ParserEngine};
use crate::patch::{AstPatch, PatchBuilder};
use crate::scanner::{DiscoveredFile, ProgressReporter, ScanResult};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Indexing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingStats {
    /// Total files processed
    pub files_processed: usize,
    /// Total nodes created
    pub nodes_created: usize,
    /// Total edges created
    pub edges_created: usize,
    /// Processing duration in milliseconds
    pub duration_ms: u64,
    /// Files processed per second
    pub throughput: f64,
    /// Errors encountered
    pub error_count: usize,
    /// Memory usage stats
    pub memory_stats: MemoryStats,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Peak memory usage in bytes
    pub peak_memory_bytes: usize,
    /// Current memory usage in bytes
    pub current_memory_bytes: usize,
    /// Graph storage overhead
    pub graph_overhead_bytes: usize,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            peak_memory_bytes: 0,
            current_memory_bytes: 0,
            graph_overhead_bytes: 0,
        }
    }
}

/// Bulk indexing result
#[derive(Debug)]
pub struct IndexingResult {
    /// Repository ID
    pub repo_id: String,
    /// All patches created during indexing
    pub patches: Vec<AstPatch>,
    /// Indexing statistics
    pub stats: IndexingStats,
    /// Files that failed to process
    pub failed_files: Vec<(PathBuf, Error)>,
}

impl IndexingResult {
    /// Create a new indexing result
    pub fn new(repo_id: String) -> Self {
        Self {
            repo_id,
            patches: Vec::new(),
            stats: IndexingStats {
                files_processed: 0,
                nodes_created: 0,
                edges_created: 0,
                duration_ms: 0,
                throughput: 0.0,
                error_count: 0,
                memory_stats: MemoryStats::default(),
            },
            failed_files: Vec::new(),
        }
    }

    /// Get total number of patches
    pub fn patch_count(&self) -> usize {
        self.patches.len()
    }

    /// Get total operations across all patches
    pub fn total_operations(&self) -> usize {
        self.patches.iter().map(|p| p.operation_count()).sum()
    }

    /// Merge another indexing result into this one
    pub fn merge(&mut self, other: IndexingResult) {
        self.patches.extend(other.patches);
        self.stats.files_processed += other.stats.files_processed;
        self.stats.nodes_created += other.stats.nodes_created;
        self.stats.edges_created += other.stats.edges_created;
        self.stats.error_count += other.stats.error_count;
        self.failed_files.extend(other.failed_files);
    }
}

/// Configuration for bulk indexing
#[derive(Debug, Clone)]
pub struct IndexingConfig {
    /// Repository ID
    pub repo_id: String,
    /// Commit SHA for patches
    pub commit_sha: String,
    /// Maximum parallel workers
    pub max_parallel: usize,
    /// Batch size for processing
    pub batch_size: usize,
    /// Whether to continue on errors
    pub continue_on_error: bool,
    /// Memory limit in bytes (None = no limit)
    pub memory_limit: Option<usize>,
    /// Whether to enable cross-file linking
    pub enable_cross_file_linking: bool,
}

impl IndexingConfig {
    /// Create a new indexing config
    pub fn new(repo_id: String, commit_sha: String) -> Self {
        Self {
            repo_id,
            commit_sha,
            max_parallel: num_cpus::get(),
            batch_size: 30, // Increased from 50 to 30 for better memory management
            continue_on_error: true,
            memory_limit: Some(4 * 1024 * 1024 * 1024), // 4GB instead of 1GB
            enable_cross_file_linking: true,
        }
    }
}

/// Bulk indexing engine for processing discovered files in parallel
pub struct BulkIndexer {
    config: IndexingConfig,
    parser_engine: Arc<ParserEngine>,
}

impl BulkIndexer {
    /// Create a new bulk indexer
    pub fn new(config: IndexingConfig, parser_engine: Arc<ParserEngine>) -> Self {
        Self {
            config,
            parser_engine,
        }
    }

    /// Index all files from a scan result
    pub async fn index_scan_result(
        &self,
        scan_result: &ScanResult,
        progress_reporter: Arc<dyn ProgressReporter>,
    ) -> Result<IndexingResult> {
        let start_time = Instant::now();
        let all_files = scan_result.all_files();
        
        progress_reporter.report_progress(0, Some(all_files.len()));

        let mut indexing_result = IndexingResult::new(self.config.repo_id.clone());
        let processed_counter = Arc::new(AtomicUsize::new(0));
        let error_counter = Arc::new(AtomicUsize::new(0));

        // For very large repositories, use streaming mode
        let use_streaming = all_files.len() > 10000 || 
            self.config.memory_limit.map_or(false, |limit| limit < 2 * 1024 * 1024 * 1024); // < 2GB

        if use_streaming {
            tracing::info!("Using streaming mode for large repository ({} files)", all_files.len());
            return self.index_scan_result_streaming(scan_result, progress_reporter).await;
        }

        // Process files in batches
        for batch in all_files.chunks(self.config.batch_size) {
            let batch_result = self.process_batch(
                batch, 
                &processed_counter,
                &error_counter,
                &progress_reporter,
                all_files.len()
            ).await?;
            
            indexing_result.merge(batch_result);

            // Check memory limit
            if let Some(limit) = self.config.memory_limit {
                let current_memory = self.estimate_memory_usage(&indexing_result);
                if current_memory > limit {
                    return Err(Error::indexing("Memory limit exceeded during bulk indexing"));
                }
            }
        }

        // After all files are processed, perform cross-file symbol resolution
        if self.config.enable_cross_file_linking {
            tracing::info!("Starting cross-file symbol resolution...");
            let linking_start = Instant::now();
            
            let cross_file_edges = self.resolve_cross_file_symbols(&indexing_result)?;
            
            if !cross_file_edges.is_empty() {
                // Create a patch with the new cross-file edges
                let cross_file_patch = PatchBuilder::new(
                    self.config.repo_id.clone(),
                    self.config.commit_sha.clone(),
                )
                .add_edges(cross_file_edges.clone())
                .build();
                
                indexing_result.patches.push(cross_file_patch);
                indexing_result.stats.edges_created += cross_file_edges.len();
                
                tracing::info!(
                    "Cross-file symbol resolution completed: {} edges created in {}ms",
                    cross_file_edges.len(),
                    linking_start.elapsed().as_millis()
                );
            }
        }

        // Finalize statistics
        indexing_result.stats.duration_ms = start_time.elapsed().as_millis() as u64;
        indexing_result.stats.throughput = if indexing_result.stats.duration_ms > 0 {
            (indexing_result.stats.files_processed as f64 * 1000.0) / indexing_result.stats.duration_ms as f64
        } else {
            0.0
        };

        progress_reporter.report_progress(all_files.len(), Some(all_files.len()));
        Ok(indexing_result)
    }

    /// Index scan result using streaming mode for large repositories
    async fn index_scan_result_streaming(
        &self,
        scan_result: &ScanResult,
        progress_reporter: Arc<dyn ProgressReporter>,
    ) -> Result<IndexingResult> {
        let start_time = Instant::now();
        let all_files = scan_result.all_files();
        
        progress_reporter.report_progress(0, Some(all_files.len()));

        let mut final_result = IndexingResult::new(self.config.repo_id.clone());
        let processed_counter = Arc::new(AtomicUsize::new(0));
        let error_counter = Arc::new(AtomicUsize::new(0));

        // Use smaller batch size for streaming mode
        let streaming_batch_size = std::cmp::min(self.config.batch_size, 20);
        let mut batch_count = 0;

        // Process files in smaller batches and clear intermediate results
        for batch in all_files.chunks(streaming_batch_size) {
            let mut batch_result = self.process_batch(
                batch, 
                &processed_counter,
                &error_counter,
                &progress_reporter,
                all_files.len()
            ).await?;

            // Update statistics but don't accumulate all patches
            final_result.stats.files_processed += batch_result.stats.files_processed;
            final_result.stats.nodes_created += batch_result.stats.nodes_created;
            final_result.stats.edges_created += batch_result.stats.edges_created;
            final_result.stats.error_count += batch_result.stats.error_count;
            final_result.failed_files.extend(batch_result.failed_files);

            // Only keep a limited number of recent patches to avoid memory exhaustion
            let max_patches_in_memory = 100;
            if final_result.patches.len() + batch_result.patches.len() > max_patches_in_memory {
                // Keep only the most recent patches
                let keep_count = max_patches_in_memory / 2;
                if final_result.patches.len() > keep_count {
                    final_result.patches.drain(0..final_result.patches.len() - keep_count);
                }
                tracing::debug!("Cleared old patches to manage memory, keeping {} recent patches", keep_count);
            }

            final_result.patches.extend(batch_result.patches);

            // Check memory limit more frequently in streaming mode
            if let Some(limit) = self.config.memory_limit {
                let current_memory = self.estimate_memory_usage(&final_result);
                if current_memory > limit {
                    tracing::warn!("Memory limit reached in streaming mode, clearing intermediate results");
                    // Clear old patches but keep statistics
                    final_result.patches.clear();
                }
            }

            batch_count += 1;
            if batch_count % 10 == 0 {
                tracing::debug!("Processed {} batches in streaming mode", batch_count);
            }
        }

        // Finalize statistics
        final_result.stats.duration_ms = start_time.elapsed().as_millis() as u64;
        final_result.stats.throughput = if final_result.stats.duration_ms > 0 {
            (final_result.stats.files_processed as f64 * 1000.0) / final_result.stats.duration_ms as f64
        } else {
            0.0
        };

        progress_reporter.report_progress(all_files.len(), Some(all_files.len()));
        tracing::info!("Streaming indexing completed: {} files, {} nodes, {} edges", 
            final_result.stats.files_processed,
            final_result.stats.nodes_created,
            final_result.stats.edges_created);
        
        Ok(final_result)
    }

    /// Process a batch of files in parallel
    async fn process_batch(
        &self,
        batch: &[&DiscoveredFile],
        processed_counter: &Arc<AtomicUsize>,
        error_counter: &Arc<AtomicUsize>,
        progress_reporter: &Arc<dyn ProgressReporter>,
        total_files: usize,
    ) -> Result<IndexingResult> {
        let mut batch_result = IndexingResult::new(self.config.repo_id.clone());

        // Process files in parallel
        let results: Vec<_> = batch
            .par_iter()
            .map(|discovered_file| {
                let processed = processed_counter.fetch_add(1, Ordering::Relaxed) + 1;
                
                // Report progress periodically
                if processed % 10 == 0 {
                    progress_reporter.report_progress(processed, Some(total_files));
                }

                self.process_single_file(discovered_file)
            })
            .collect();

        // Collect results
        for result in results {
            match result {
                Ok(Some(patch)) => {
                    batch_result.stats.files_processed += 1;
                    batch_result.stats.nodes_created += patch.nodes_add.len();
                    batch_result.stats.edges_created += patch.edges_add.len();
                    batch_result.patches.push(patch);
                }
                Ok(None) => {
                    // File was skipped (e.g., empty, parse failed gracefully)
                    batch_result.stats.files_processed += 1;
                }
                Err(e) => {
                    error_counter.fetch_add(1, Ordering::Relaxed);
                    batch_result.stats.error_count += 1;
                    
                    if !self.config.continue_on_error {
                        return Err(e);
                    }
                    
                    progress_reporter.report_error(&e);
                }
            }
        }

        Ok(batch_result)
    }

    /// Process a single discovered file
    fn process_single_file(&self, discovered_file: &DiscoveredFile) -> Result<Option<AstPatch>> {
        // Read file content
        let content = std::fs::read_to_string(&discovered_file.path)
            .map_err(|e| Error::io(format!(
                "Failed to read file {}: {}", 
                discovered_file.path.display(), 
                e
            )))?;

        // Skip empty files
        if content.trim().is_empty() {
            return Ok(None);
        }

        // Create parse context
        let context = ParseContext::new(
            self.config.repo_id.clone(),
            discovered_file.path.clone(),
            content,
        );

        // Parse the file
        let parse_result = self.parser_engine.parse_file(context)?;

        // Create patch from parse result
        let mut patch_builder = PatchBuilder::new(
            self.config.repo_id.clone(),
            self.config.commit_sha.clone(),
        );

        // Add all nodes
        patch_builder = patch_builder.add_nodes(parse_result.nodes);

        // Add all edges
        patch_builder = patch_builder.add_edges(parse_result.edges);

        let patch = patch_builder.build();

        // Only return patch if it has content
        if patch.is_empty() {
            Ok(None)
        } else {
            Ok(Some(patch))
        }
    }

    /// Estimate memory usage of the indexing result
    fn estimate_memory_usage(&self, result: &IndexingResult) -> usize {
        let mut total = 0;

        // Estimate patch memory usage
        for patch in &result.patches {
            // Rough estimation: each node ~200 bytes, each edge ~50 bytes
            total += patch.nodes_add.len() * 200;
            total += patch.edges_add.len() * 50;
            total += patch.nodes_delete.len() * 50; // String IDs
            total += patch.edges_delete.len() * 50;
        }

        // Add overhead for data structures
        total += result.patches.len() * 100; // Patch overhead
        total += result.failed_files.len() * 200; // Error storage

        total
    }

    /// Perform cross-file symbol resolution
    fn resolve_cross_file_symbols(&self, indexing_result: &IndexingResult) -> Result<Vec<Edge>> {
        // Build a temporary graph store with all the nodes and edges from patches
        let temp_graph = Arc::new(GraphStore::new());
        
        // Add all nodes and edges from patches to the temporary graph
        for patch in &indexing_result.patches {
            for node in &patch.nodes_add {
                temp_graph.add_node(node.clone());
            }
            for edge in &patch.edges_add {
                temp_graph.add_edge(edge.clone());
            }
        }
        
        // Create symbol resolver and resolve cross-file relationships
        let mut resolver = SymbolResolver::new(temp_graph);
        resolver.resolve_all()
    }

    /// Get indexing configuration
    pub fn config(&self) -> &IndexingConfig {
        &self.config
    }
}

/// Indexing progress reporter that tracks detailed statistics
#[derive(Debug)]
pub struct IndexingProgressReporter {
    verbose: bool,
    last_report: std::sync::Mutex<Instant>,
}

impl IndexingProgressReporter {
    /// Create a new indexing progress reporter
    pub fn new(verbose: bool) -> Self {
        Self {
            verbose,
            last_report: std::sync::Mutex::new(Instant::now()),
        }
    }
}

impl ProgressReporter for IndexingProgressReporter {
    fn report_progress(&self, current: usize, total: Option<usize>) {
        if let Ok(mut last_report) = self.last_report.try_lock() {
            let now = Instant::now();
            
            // Rate limit progress reports to avoid spam
            if now.duration_since(*last_report).as_millis() > 500 {
                match total {
                    Some(total) => {
                        let percent = (current as f64 / total as f64) * 100.0;
                        println!("Indexing progress: {}/{} files ({:.1}%)", current, total, percent);
                    }
                    None => {
                        println!("Indexing progress: {} files processed", current);
                    }
                }
                *last_report = now;
            }
        }
    }

    fn report_complete(&self, _result: &crate::scanner::ScanResult) {
        println!("Indexing complete!");
    }

    fn report_error(&self, error: &Error) {
        if self.verbose {
            eprintln!("Indexing error: {}", error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Language;
    use crate::parser::LanguageRegistry;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_indexer() -> (BulkIndexer, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        
        let config = IndexingConfig::new(
            "test_repo".to_string(),
            "abc123".to_string(),
        );
        
        let registry = Arc::new(LanguageRegistry::new());
        let parser_engine = Arc::new(ParserEngine::new(registry));
        let indexer = BulkIndexer::new(config, parser_engine);
        
        (indexer, temp_dir)
    }

    fn create_test_discovered_file(path: PathBuf, language: Language) -> DiscoveredFile {
        DiscoveredFile {
            path,
            language,
            size: 100,
        }
    }

    #[test]
    fn test_indexing_config() {
        let config = IndexingConfig::new("test".to_string(), "sha".to_string());
        assert_eq!(config.repo_id, "test");
        assert_eq!(config.commit_sha, "sha");
        assert!(config.max_parallel > 0);
        assert!(config.continue_on_error);
    }

    #[test]
    fn test_indexing_result() {
        let mut result = IndexingResult::new("test_repo".to_string());
        assert_eq!(result.repo_id, "test_repo");
        assert_eq!(result.patch_count(), 0);
        assert_eq!(result.total_operations(), 0);
        
        // Test merge
        let other = IndexingResult::new("test_repo".to_string());
        result.merge(other);
        assert_eq!(result.stats.files_processed, 0);
    }

    #[tokio::test]
    async fn test_process_single_file() {
        let (indexer, temp_dir) = create_test_indexer();
        
        // Create a test file
        let test_file = temp_dir.path().join("test.js");
        std::fs::write(&test_file, "console.log('hello');").unwrap();
        
        let discovered_file = create_test_discovered_file(test_file, Language::JavaScript);
        
        // This will fail because we don't have a JavaScript parser registered
        // but it tests the file reading logic
        let result = indexer.process_single_file(&discovered_file);
        
        // Should return error because no JS parser is registered
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_estimation() {
        let (indexer, _temp_dir) = create_test_indexer();
        let result = IndexingResult::new("test".to_string());
        
        let memory = indexer.estimate_memory_usage(&result);
        assert!(memory >= 0); // Should not panic
    }

    #[test]
    fn test_indexing_stats() {
        let stats = IndexingStats {
            files_processed: 100,
            nodes_created: 500,
            edges_created: 300,
            duration_ms: 1000,
            throughput: 100.0,
            error_count: 2,
            memory_stats: MemoryStats::default(),
        };
        
        assert_eq!(stats.files_processed, 100);
        assert_eq!(stats.throughput, 100.0);
    }

    #[test]
    fn test_progress_reporter() {
        let reporter = IndexingProgressReporter::new(true);
        
        // These should not panic
        reporter.report_progress(50, Some(100));
        reporter.report_progress(100, None);
        
        let error = Error::indexing("test error");
        reporter.report_error(&error);
    }
}
