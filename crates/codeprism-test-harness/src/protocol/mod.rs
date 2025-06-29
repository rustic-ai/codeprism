//! Generic MCP Protocol Implementation
//!
//! This module provides a complete implementation of the Model Context Protocol (MCP)
//! specification including JSON-RPC 2.0 message handling, capability negotiation,
//! and transport abstraction for testing any MCP server implementation.

pub mod capabilities;
pub mod client;
pub mod jsonrpc;
pub mod messages;
pub mod validation;

// Re-export main types for convenience
pub use capabilities::{
    McpCapabilities, PromptsCapability, ResourcesCapability, SamplingCapability, ToolsCapability,
};
pub use client::McpClient;
pub use jsonrpc::{
    JsonRpcError, JsonRpcMessage, JsonRpcNotification, JsonRpcProcessingError, JsonRpcRequest,
    JsonRpcResponse,
};
pub use messages::{InitializeParams, InitializeResult, McpMethod};
pub use validation::{ProtocolValidator, ValidationResult};

use serde::{Deserialize, Serialize};

/// MCP Protocol version supported by this implementation
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// Connection state for MCP client
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    /// Initial state before connection
    Disconnected,
    /// Connecting to server
    Connecting,
    /// Connected but not initialized
    Connected,
    /// Initialize request sent, waiting for response
    Initializing,
    /// Fully initialized and ready for use
    Ready,
    /// Connection is being closed
    Closing,
    /// Connection closed
    Closed,
    /// Error state
    Error(String),
}

/// Errors that can occur during MCP protocol operations
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("JSON-RPC processing error: {0}")]
    JsonRpc(#[from] jsonrpc::JsonRpcProcessingError),
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("Invalid state for operation: expected {expected:?}, got {actual:?}")]
    InvalidState {
        expected: ConnectionState,
        actual: ConnectionState,
    },
    #[error("Capability not supported: {capability}")]
    UnsupportedCapability { capability: String },
    #[error("Timeout waiting for response")]
    Timeout,
    #[error("Server rejected request: {reason}")]
    ServerRejected { reason: String },
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for MCP operations
pub type McpResult<T> = Result<T, McpError>;

/// Transport abstraction for different MCP communication methods
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    /// Send a JSON-RPC message to the server
    async fn send(&mut self, message: JsonRpcMessage) -> Result<(), TransportError>;

    /// Receive a JSON-RPC message from the server
    async fn receive(&mut self) -> Result<JsonRpcMessage, TransportError>;

    /// Close the transport connection
    async fn close(&mut self) -> Result<(), TransportError>;

    /// Check if the transport is still connected
    fn is_connected(&self) -> bool;
}

/// Errors that can occur during transport operations
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Receive failed: {0}")]
    ReceiveFailed(String),
    #[error("Transport closed")]
    Closed,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Configuration for MCP protocol behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Protocol version to use
    pub protocol_version: String,
    /// Client information
    pub client_info: ClientInfo,
    /// Requested capabilities
    pub capabilities: McpCapabilities,
    /// Timeout settings
    pub timeouts: TimeoutConfig,
    /// Validation settings
    pub validation: ValidationConfig,
}

/// Client information for MCP initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client name
    pub name: String,
    /// Client version
    pub version: String,
}

/// Timeout configuration for various operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Connection timeout in seconds
    pub connection: u64,
    /// Request timeout in seconds
    pub request: u64,
    /// Initialization timeout in seconds
    pub initialization: u64,
}

/// Validation configuration for protocol compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Enforce strict JSON-RPC 2.0 compliance
    pub strict_json_rpc: bool,
    /// Validate server capabilities
    pub validate_capabilities: bool,
    /// Check protocol version compatibility
    pub check_protocol_version: bool,
    /// Validate message schemas
    pub validate_schemas: bool,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            client_info: ClientInfo {
                name: "MCP Test Harness".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            capabilities: McpCapabilities::default(),
            timeouts: TimeoutConfig {
                connection: 10,
                request: 30,
                initialization: 15,
            },
            validation: ValidationConfig {
                strict_json_rpc: true,
                validate_capabilities: true,
                check_protocol_version: true,
                validate_schemas: true,
            },
        }
    }
}
