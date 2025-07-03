//! Error types for the CodePrism MCP Server

use thiserror::Error;

/// Result type alias for the MCP server
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the CodePrism MCP Server
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration related errors
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    /// IO related errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML serialization/deserialization errors
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// TOML serialization errors
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    /// YAML serialization/deserialization errors
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// MCP protocol errors - will be defined when rust-sdk is added
    #[error("MCP protocol error: {0}")]
    Protocol(String),

    /// Server initialization errors
    #[error("Server initialization error: {0}")]
    ServerInit(String),

    /// Tool execution errors
    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    /// Generic errors
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl Error {
    /// Create a new protocol error
    pub fn protocol(msg: impl Into<String>) -> Self {
        Self::Protocol(msg.into())
    }

    /// Create a new server initialization error
    pub fn server_init(msg: impl Into<String>) -> Self {
        Self::ServerInit(msg.into())
    }

    /// Create a new tool execution error
    pub fn tool_execution(msg: impl Into<String>) -> Self {
        Self::ToolExecution(msg.into())
    }
}
