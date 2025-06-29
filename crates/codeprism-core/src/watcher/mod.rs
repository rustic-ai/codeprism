//! File system watcher for detecting changes

use crate::error::{Error, Result};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;

/// Type of file change
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeKind {
    /// File created
    Created,
    /// File modified
    Modified,
    /// File deleted
    Deleted,
    /// File renamed
    Renamed {
        /// Old file path
        old: PathBuf,
        /// New file path
        new: PathBuf,
    },
}

/// File change event
#[derive(Debug, Clone)]
pub struct ChangeEvent {
    /// Repository root
    pub repo_root: PathBuf,
    /// Changed file path
    pub path: PathBuf,
    /// Type of change
    pub kind: ChangeKind,
    /// Timestamp of the event
    pub timestamp: Instant,
}

impl ChangeEvent {
    /// Create a new change event
    pub fn new(repo_root: PathBuf, path: PathBuf, kind: ChangeKind) -> Self {
        Self {
            repo_root,
            path,
            kind,
            timestamp: Instant::now(),
        }
    }
}

/// Debouncer for file events
struct Debouncer {
    pending: Arc<Mutex<HashMap<PathBuf, (ChangeKind, Instant)>>>,
    tx: mpsc::UnboundedSender<ChangeEvent>,
    debounce_duration: Duration,
}

impl Debouncer {
    /// Create a new debouncer
    fn new(tx: mpsc::UnboundedSender<ChangeEvent>, debounce_duration: Duration) -> Self {
        Self {
            pending: Arc::new(Mutex::new(HashMap::new())),
            tx,
            debounce_duration,
        }
    }

    /// Add an event to be debounced
    fn add_event(&self, event: ChangeEvent) {
        let mut pending = self.pending.lock().unwrap();
        pending.insert(event.path.clone(), (event.kind.clone(), event.timestamp));

        // Schedule flush
        let pending_clone = Arc::clone(&self.pending);
        let tx = self.tx.clone();
        let path = event.path.clone();
        let repo_root = event.repo_root.clone();
        let duration = self.debounce_duration;

        tokio::spawn(async move {
            sleep(duration).await;

            let mut pending = pending_clone.lock().unwrap();
            if let Some((kind, timestamp)) = pending.remove(&path) {
                // Check if enough time has passed
                if timestamp.elapsed() >= duration {
                    let event = ChangeEvent {
                        repo_root,
                        path,
                        kind,
                        timestamp,
                    };
                    let _ = tx.send(event);
                }
            }
        });
    }
}

/// File system watcher
pub struct FileWatcher {
    watcher: RecommendedWatcher,
    debouncer: Arc<Debouncer>,
    change_rx: mpsc::UnboundedReceiver<ChangeEvent>,
    watched_paths: Arc<Mutex<Vec<PathBuf>>>,
}

impl FileWatcher {
    /// Create a new file watcher with default 50ms debounce
    pub fn new() -> Result<Self> {
        Self::with_debounce(Duration::from_millis(50))
    }

    /// Create a new file watcher with custom debounce duration
    pub fn with_debounce(debounce_duration: Duration) -> Result<Self> {
        let (change_tx, change_rx) = mpsc::unbounded_channel();
        let debouncer = Arc::new(Debouncer::new(change_tx.clone(), debounce_duration));

        // Create a channel for notify events
        let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

        let watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    let _ = notify_tx.send(event);
                }
            },
            Config::default(),
        )
        .map_err(|e| Error::watcher(format!("Failed to create watcher: {}", e)))?;

        // Start event processor
        let debouncer_clone = Arc::clone(&debouncer);
        tokio::spawn(async move {
            while let Some(event) = notify_rx.recv().await {
                // For now, we'll use a placeholder repo_root
                // In real usage, this would be tracked per watched path
                if let Some(path) = event.paths.first() {
                    let repo_root = path.clone();
                    if let Some(change_event) = Self::convert_event(event, repo_root) {
                        debouncer_clone.add_event(change_event);
                    }
                }
            }
        });

        Ok(Self {
            watcher,
            debouncer,
            change_rx,
            watched_paths: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Watch a directory recursively
    pub fn watch_dir(&mut self, path: &Path, _repo_root: PathBuf) -> Result<()> {
        self.watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(|e| Error::watcher(format!("Failed to watch {}: {}", path.display(), e)))?;

        let mut paths = self.watched_paths.lock().unwrap();
        paths.push(path.to_path_buf());

        Ok(())
    }

    /// Stop watching a directory
    pub fn unwatch(&mut self, path: &Path) -> Result<()> {
        self.watcher
            .unwatch(path)
            .map_err(|e| Error::watcher(format!("Failed to unwatch {}: {}", path.display(), e)))?;

        let mut paths = self.watched_paths.lock().unwrap();
        paths.retain(|p| p != path);

        Ok(())
    }

    /// Get the next change event
    pub async fn next_change(&mut self) -> Option<ChangeEvent> {
        self.change_rx.recv().await
    }

    /// Convert notify event to our ChangeEvent
    fn convert_event(event: Event, repo_root: PathBuf) -> Option<ChangeEvent> {
        let path = event.paths.first()?.clone();

        let kind = match event.kind {
            EventKind::Create(_) => ChangeKind::Created,
            EventKind::Modify(_) => ChangeKind::Modified,
            EventKind::Remove(_) => ChangeKind::Deleted,
            EventKind::Any => ChangeKind::Modified,
            _ => return None,
        };

        Some(ChangeEvent::new(repo_root, path, kind))
    }
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new().expect("Failed to create file watcher")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_watcher_creation() {
        let watcher = FileWatcher::new();
        assert!(watcher.is_ok());
    }

    #[tokio::test]
    async fn test_debouncer() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let debouncer = Debouncer::new(tx, Duration::from_millis(50));

        let event = ChangeEvent::new(
            PathBuf::from("/repo"),
            PathBuf::from("/repo/file.txt"),
            ChangeKind::Modified,
        );

        debouncer.add_event(event);

        // Wait for debounce
        sleep(Duration::from_millis(100)).await;

        let received = rx.recv().await;
        assert!(received.is_some());
    }

    #[tokio::test]
    async fn test_watch_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mut watcher = FileWatcher::new().unwrap();

        let result = watcher.watch_dir(temp_dir.path(), temp_dir.path().to_path_buf());
        assert!(result.is_ok());

        // Create a file
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        // Wait for event
        sleep(Duration::from_millis(100)).await;

        // Note: In a real test, we'd need to properly handle async event reception
    }
}
