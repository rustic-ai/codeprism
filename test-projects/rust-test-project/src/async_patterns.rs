//! Async patterns and utilities for concurrent programming.

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::info;

/// Async processor demonstrating various async patterns
#[derive(Debug)]
pub struct AsyncProcessor {
    name: String,
}

impl AsyncProcessor {
    pub fn new() -> Self {
        Self {
            name: "AsyncProcessor".to_string(),
        }
    }

    /// Process data asynchronously with simulated work
    pub async fn process_data(&self, data: &str) -> Result<String> {
        info!("Processing data: {}", data);
        
        // Simulate async work
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(format!("Processed: {}", data.to_uppercase()))
    }

    /// Create a data stream for async iteration
    pub async fn create_data_stream(&self) -> mpsc::Receiver<String> {
        let (tx, rx) = mpsc::channel(32);
        
        tokio::spawn(async move {
            for i in 0..10 {
                let data = format!("Stream item {}", i);
                if tx.send(data).await.is_err() {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
        });
        
        rx
    }
} 