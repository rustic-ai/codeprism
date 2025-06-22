//! File monitoring pipeline for real-time graph updates
//!
//! This module provides functionality to monitor file changes and automatically
//! update the code graph through incremental parsing and patch generation.

use crate::error::{Error, Result};
use crate::parser::{ParseContext, ParserEngine};
use crate::patch::{AstPatch, PatchBuilder};
use crate::watcher::{ChangeEvent, ChangeKind, FileWatcher};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;
use tokio::time::sleep;

/// Pipeline event representing a processed file change
#[derive(Debug, Clone)]
pub struct PipelineEvent {
    /// Repository ID
    pub repo_id: String,
    /// Original change event
    pub change_event: ChangeEvent,
    /// Generated patch (if any)
    pub patch: Option<AstPatch>,
    /// Processing timestamp
    pub processed_at: Instant,
    /// Processing duration in milliseconds
    pub processing_duration_ms: u64,
}

/// Pipeline statistics
#[derive(Debug, Clone, Default)]
pub struct PipelineStats {
    /// Total events processed
    pub events_processed: usize,
    /// Events processed successfully
    pub events_success: usize,
    /// Events that failed processing
    pub events_failed: usize,
    /// Events that were filtered out
    pub events_filtered: usize,
    /// Average processing time in milliseconds
    pub avg_processing_ms: f64,
    /// Total patches generated
    pub patches_generated: usize,
    /// Total nodes added
    pub nodes_added: usize,
    /// Total edges added
    pub edges_added: usize,
    /// Total nodes removed
    pub nodes_removed: usize,
    /// Total edges removed
    pub edges_removed: usize,
}

impl PipelineStats {
    /// Update statistics with a new event
    pub fn update(&mut self, event: &PipelineEvent, success: bool) {
        self.events_processed += 1;

        if success {
            self.events_success += 1;
            if let Some(ref patch) = event.patch {
                self.patches_generated += 1;
                self.nodes_added += patch.nodes_add.len();
                self.edges_added += patch.edges_add.len();
                self.nodes_removed += patch.nodes_delete.len();
                self.edges_removed += patch.edges_delete.len();
            }
        } else {
            self.events_failed += 1;
        }

        // Update average processing time
        let total_time = self.avg_processing_ms * (self.events_processed - 1) as f64
            + event.processing_duration_ms as f64;
        self.avg_processing_ms = total_time / self.events_processed as f64;
    }

    /// Calculate success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.events_processed == 0 {
            0.0
        } else {
            (self.events_success as f64 / self.events_processed as f64) * 100.0
        }
    }

    /// Calculate events per second
    pub fn events_per_second(&self, duration_secs: f64) -> f64 {
        if duration_secs <= 0.0 {
            0.0
        } else {
            self.events_processed as f64 / duration_secs
        }
    }
}

/// Event handler trait for processing pipeline events
pub trait PipelineEventHandler: Send + Sync {
    /// Handle a processed pipeline event
    fn handle_event(&self, event: &PipelineEvent) -> Result<()>;

    /// Handle pipeline errors
    fn handle_error(&self, error: &Error, change_event: &ChangeEvent);
}

/// No-op event handler
#[derive(Debug, Default)]
pub struct NoOpEventHandler;

impl PipelineEventHandler for NoOpEventHandler {
    fn handle_event(&self, _event: &PipelineEvent) -> Result<()> {
        Ok(())
    }

    fn handle_error(&self, _error: &Error, _change_event: &ChangeEvent) {}
}

/// Logging event handler
#[derive(Debug)]
pub struct LoggingEventHandler {
    verbose: bool,
}

impl LoggingEventHandler {
    /// Create a new logging event handler
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl PipelineEventHandler for LoggingEventHandler {
    fn handle_event(&self, event: &PipelineEvent) -> Result<()> {
        if self.verbose {
            println!(
                "Pipeline event: {:?} processed in {}ms",
                event.change_event.kind, event.processing_duration_ms
            );

            if let Some(ref patch) = event.patch {
                println!(
                    "  Generated patch: +{} nodes, +{} edges, -{} nodes, -{} edges",
                    patch.nodes_add.len(),
                    patch.edges_add.len(),
                    patch.nodes_delete.len(),
                    patch.edges_delete.len()
                );
            }
        }
        Ok(())
    }

    fn handle_error(&self, error: &Error, change_event: &ChangeEvent) {
        eprintln!(
            "Pipeline error processing {:?}: {}",
            change_event.path, error
        );
    }
}

/// Configuration for the monitoring pipeline
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Repository ID
    pub repo_id: String,
    /// Commit SHA for generated patches
    pub commit_sha: String,
    /// Debounce duration for file changes
    pub debounce_duration: Duration,
    /// Maximum queue size for pending events
    pub max_queue_size: usize,
    /// Batch size for processing multiple events
    pub batch_size: usize,
    /// Whether to process events in batches
    pub enable_batching: bool,
    /// Timeout for processing individual events
    pub processing_timeout: Duration,
}

impl PipelineConfig {
    /// Create a new pipeline config
    pub fn new(repo_id: String, commit_sha: String) -> Self {
        Self {
            repo_id,
            commit_sha,
            debounce_duration: Duration::from_millis(100),
            max_queue_size: 1000,
            batch_size: 10,
            enable_batching: true,
            processing_timeout: Duration::from_secs(30),
        }
    }
}

/// File monitoring pipeline that connects FileWatcher to ParserEngine
pub struct MonitoringPipeline {
    config: PipelineConfig,
    parser_engine: Arc<ParserEngine>,
    file_watcher: FileWatcher,
    event_handler: Arc<dyn PipelineEventHandler>,
    stats: PipelineStats,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl MonitoringPipeline {
    /// Create a new monitoring pipeline
    pub fn new(
        config: PipelineConfig,
        parser_engine: Arc<ParserEngine>,
        event_handler: Arc<dyn PipelineEventHandler>,
    ) -> Result<Self> {
        let file_watcher = FileWatcher::with_debounce(config.debounce_duration)?;

        Ok(Self {
            config,
            parser_engine,
            file_watcher,
            event_handler,
            stats: PipelineStats::default(),
            shutdown_tx: None,
        })
    }

    /// Start monitoring a repository path
    pub async fn start_monitoring<P: AsRef<Path>>(&mut self, repo_path: P) -> Result<()> {
        let repo_path = repo_path.as_ref();

        // Start watching the repository
        self.file_watcher
            .watch_dir(repo_path, repo_path.to_path_buf())?;

        // Start the processing loop
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        let mut event_queue = Vec::new();
        let mut last_batch_time = Instant::now();

        tokio::select! {
            _ = self.process_events(&mut event_queue, &mut last_batch_time) => {
                // Processing loop ended
            }
            _ = shutdown_rx => {
                // Shutdown requested
                tracing::info!("Pipeline shutdown requested");
            }
        }

        Ok(())
    }

    /// Stop monitoring and shutdown the pipeline
    pub fn stop_monitoring(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
    }

    /// Process file change events
    async fn process_events(
        &mut self,
        event_queue: &mut Vec<ChangeEvent>,
        last_batch_time: &mut Instant,
    ) -> Result<()> {
        loop {
            // Try to get the next change event
            if let Some(change_event) = self.file_watcher.next_change().await {
                event_queue.push(change_event);

                // Process batch if conditions are met
                let should_process_batch = event_queue.len() >= self.config.batch_size
                    || (!self.config.enable_batching && !event_queue.is_empty())
                    || (last_batch_time.elapsed() > self.config.debounce_duration
                        && !event_queue.is_empty());

                if should_process_batch {
                    self.process_event_batch(event_queue).await?;
                    *last_batch_time = Instant::now();
                }
            } else {
                // No more events, process any remaining in queue
                if !event_queue.is_empty() {
                    self.process_event_batch(event_queue).await?;
                    *last_batch_time = Instant::now();
                }

                // Brief pause to avoid busy waiting
                sleep(Duration::from_millis(10)).await;
            }
        }
    }

    /// Process a batch of change events
    async fn process_event_batch(&mut self, event_queue: &mut Vec<ChangeEvent>) -> Result<()> {
        let events_to_process = std::mem::take(event_queue);

        for change_event in events_to_process {
            match self.process_single_event(change_event.clone()).await {
                Ok(pipeline_event) => {
                    self.stats.update(&pipeline_event, true);
                    if let Err(e) = self.event_handler.handle_event(&pipeline_event) {
                        self.event_handler.handle_error(&e, &change_event);
                    }
                }
                Err(e) => {
                    self.stats.events_failed += 1;
                    self.event_handler.handle_error(&e, &change_event);
                }
            }
        }

        Ok(())
    }

    /// Process a single change event
    async fn process_single_event(&self, change_event: ChangeEvent) -> Result<PipelineEvent> {
        let start_time = Instant::now();

        let patch = match change_event.kind {
            ChangeKind::Created | ChangeKind::Modified => {
                self.process_file_change(&change_event.path).await?
            }
            ChangeKind::Deleted => self.process_file_deletion(&change_event.path).await?,
            ChangeKind::Renamed { ref old, ref new } => self.process_file_rename(old, new).await?,
        };

        let processing_duration = start_time.elapsed();

        Ok(PipelineEvent {
            repo_id: self.config.repo_id.clone(),
            change_event,
            patch,
            processed_at: Instant::now(),
            processing_duration_ms: processing_duration.as_millis() as u64,
        })
    }

    /// Process a file creation or modification
    async fn process_file_change(&self, file_path: &Path) -> Result<Option<AstPatch>> {
        // Check if file still exists and is readable
        if !file_path.exists() {
            return Ok(None);
        }

        // Read file content
        let content = tokio::fs::read_to_string(file_path).await.map_err(|e| {
            Error::io(format!(
                "Failed to read file {}: {}",
                file_path.display(),
                e
            ))
        })?;

        // Skip empty files
        if content.trim().is_empty() {
            return Ok(None);
        }

        // Create parse context
        let context = ParseContext::new(
            self.config.repo_id.clone(),
            file_path.to_path_buf(),
            content,
        );

        // Parse the file
        let parse_result = self.parser_engine.parse_incremental(context)?;

        // Create patch with new nodes and edges
        let patch = PatchBuilder::new(self.config.repo_id.clone(), self.config.commit_sha.clone())
            .add_nodes(parse_result.nodes)
            .add_edges(parse_result.edges)
            .build();

        Ok(Some(patch))
    }

    /// Process a file deletion
    async fn process_file_deletion(&self, _file_path: &Path) -> Result<Option<AstPatch>> {
        // For deletion, we would need to track which nodes belong to which files
        // and generate deletion patches. For now, we'll create an empty patch
        // that represents the deletion event.

        let patch =
            PatchBuilder::new(self.config.repo_id.clone(), self.config.commit_sha.clone()).build();

        Ok(Some(patch))
    }

    /// Process a file rename
    async fn process_file_rename(
        &self,
        _old_path: &Path,
        new_path: &Path,
    ) -> Result<Option<AstPatch>> {
        // For rename, we could:
        // 1. Delete nodes from old file
        // 2. Parse and add nodes from new file
        // For now, we'll just process it as a new file
        self.process_file_change(new_path).await
    }

    /// Get pipeline statistics
    pub fn get_stats(&self) -> &PipelineStats {
        &self.stats
    }

    /// Reset pipeline statistics
    pub fn reset_stats(&mut self) {
        self.stats = PipelineStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::LanguageRegistry;
    use tempfile::TempDir;
    use tokio::fs;

    struct TestEventHandler {
        event_count: Arc<AtomicUsize>,
        error_count: Arc<AtomicUsize>,
    }

    impl TestEventHandler {
        fn new() -> Self {
            Self {
                event_count: Arc::new(AtomicUsize::new(0)),
                error_count: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    impl PipelineEventHandler for TestEventHandler {
        fn handle_event(&self, _event: &PipelineEvent) -> Result<()> {
            self.event_count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        fn handle_error(&self, _error: &Error, _change_event: &ChangeEvent) {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn create_test_pipeline() -> (MonitoringPipeline, TempDir, Arc<TestEventHandler>) {
        let temp_dir = TempDir::new().unwrap();
        let config = PipelineConfig::new("test_repo".to_string(), "abc123".to_string());
        let registry = Arc::new(LanguageRegistry::new());
        let parser_engine = Arc::new(ParserEngine::new(registry));
        let handler = Arc::new(TestEventHandler::new());

        let pipeline = MonitoringPipeline::new(
            config,
            parser_engine,
            handler.clone() as Arc<dyn PipelineEventHandler>,
        )
        .unwrap();

        (pipeline, temp_dir, handler)
    }

    #[test]
    fn test_pipeline_config() {
        let config = PipelineConfig::new("test".to_string(), "sha".to_string());
        assert_eq!(config.repo_id, "test");
        assert_eq!(config.commit_sha, "sha");
        assert!(config.enable_batching);
    }

    #[test]
    fn test_pipeline_stats() {
        let mut stats = PipelineStats::default();

        let event = PipelineEvent {
            repo_id: "test".to_string(),
            change_event: ChangeEvent {
                repo_root: PathBuf::from("/repo"),
                path: PathBuf::from("/repo/file.js"),
                kind: ChangeKind::Modified,
                timestamp: Instant::now(),
            },
            patch: None,
            processed_at: Instant::now(),
            processing_duration_ms: 100,
        };

        stats.update(&event, true);
        assert_eq!(stats.events_processed, 1);
        assert_eq!(stats.events_success, 1);
        assert_eq!(stats.avg_processing_ms, 100.0);
        assert_eq!(stats.success_rate(), 100.0);
    }

    #[tokio::test]
    async fn test_pipeline_creation() {
        let (pipeline, _temp_dir, _handler) = create_test_pipeline();
        assert_eq!(pipeline.config.repo_id, "test_repo");
        assert_eq!(pipeline.stats.events_processed, 0);
    }

    #[tokio::test]
    async fn test_process_file_change() {
        let (pipeline, temp_dir, _handler) = create_test_pipeline();

        // Create a test file
        let test_file = temp_dir.path().join("test.js");
        fs::write(&test_file, "console.log('hello');")
            .await
            .unwrap();

        // This will fail because no JS parser is registered, but tests the logic
        let result = pipeline.process_file_change(&test_file).await;
        assert!(result.is_err()); // Expected because no parser registered
    }

    #[tokio::test]
    async fn test_process_empty_file() {
        let (pipeline, temp_dir, _handler) = create_test_pipeline();

        // Create an empty test file
        let test_file = temp_dir.path().join("empty.js");
        fs::write(&test_file, "").await.unwrap();

        let result = pipeline.process_file_change(&test_file).await.unwrap();
        assert!(result.is_none()); // Should be None for empty files
    }

    #[tokio::test]
    async fn test_process_nonexistent_file() {
        let (pipeline, temp_dir, _handler) = create_test_pipeline();

        let test_file = temp_dir.path().join("nonexistent.js");

        let result = pipeline.process_file_change(&test_file).await.unwrap();
        assert!(result.is_none()); // Should be None for nonexistent files
    }

    #[test]
    fn test_event_handlers() {
        let handler = LoggingEventHandler::new(true);

        let event = PipelineEvent {
            repo_id: "test".to_string(),
            change_event: ChangeEvent {
                repo_root: PathBuf::from("/repo"),
                path: PathBuf::from("/repo/file.js"),
                kind: ChangeKind::Modified,
                timestamp: Instant::now(),
            },
            patch: None,
            processed_at: Instant::now(),
            processing_duration_ms: 100,
        };

        // Should not panic
        let _ = handler.handle_event(&event);

        let error = Error::other("test error");
        handler.handle_error(&error, &event.change_event);
    }
}
