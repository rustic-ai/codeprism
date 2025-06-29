//! MCP Server management for the CodePrism Test Harness
//!
//! This module provides comprehensive MCP server process management with:
//! - Robust process lifecycle management (start/stop/cleanup)
//! - JSON-RPC communication over stdio pipes
//! - Process isolation and resource management
//! - Timeout handling and graceful shutdown

// Module files will be added as needed for server management
// pub mod manager;
// pub mod instance;
// pub mod rpc;

// pub use manager::ServerManager;
// pub use instance::ServerInstance;
// pub use rpc::{JsonRpcClient, JsonRpcRequest, JsonRpcResponse, JsonRpcError};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for MCP server management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Command to start the MCP server
    pub start_command: String,
    /// Command line arguments
    pub args: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Working directory for the server process
    pub working_dir: Option<PathBuf>,
    /// Timeout for server startup
    pub startup_timeout_seconds: u64,
    /// Timeout for server shutdown
    pub shutdown_timeout_seconds: u64,
    /// Timeout for individual RPC requests
    pub request_timeout_seconds: u64,
    /// Maximum number of concurrent server instances
    pub max_instances: usize,
    /// Whether to reuse server instances across tests
    pub reuse_instances: bool,
    /// Additional server-specific configuration
    pub server_specific: HashMap<String, serde_json::Value>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            start_command: "node".to_string(),
            args: vec!["dist/index.js".to_string()],
            env: HashMap::new(),
            working_dir: None,
            startup_timeout_seconds: 10,
            shutdown_timeout_seconds: 5,
            request_timeout_seconds: 30,
            max_instances: 4,
            reuse_instances: true,
            server_specific: HashMap::new(),
        }
    }
}

/// Error types for server management operations
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Failed to start server process: {0}")]
    StartupFailed(String),
    #[error("Server process died unexpectedly: {0}")]
    ProcessDied(String),
    #[error("Communication timeout after {timeout:?}")]
    CommunicationTimeout { timeout: Duration },
    #[error("JSON-RPC error: {0}")]
    JsonRpc(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Server shutdown timeout")]
    ShutdownTimeout,
    #[error("No available server instances")]
    NoAvailableInstances,
}
