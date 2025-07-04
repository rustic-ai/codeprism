//! Transport layer for MCP communication

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod http;
pub mod stdio;

/// Transport error types
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Transport not supported: {0}")]
    NotSupported(String),
}

/// Transport type enumeration
///
/// MCP specification only supports stdio and HTTP transports.
/// WebSocket is not part of the official MCP protocol.
#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum TransportType {
    /// Standard input/output transport
    Stdio,
    /// HTTP with Server-Sent Events
    #[clap(skip)]
    Http { host: String, port: u16 },
}

impl fmt::Display for TransportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportType::Stdio => write!(f, "stdio"),
            TransportType::Http { host, port } => write!(f, "http://{}:{}", host, port),
        }
    }
}

/// Generic transport trait for MCP communication
#[async_trait]
pub trait Transport: Send + Sync {
    /// Connect to the MCP server
    async fn connect(&mut self) -> Result<(), TransportError>;

    /// Send a message to the server
    async fn send(&mut self, message: serde_json::Value) -> Result<(), TransportError>;

    /// Receive a message from the server
    async fn receive(&mut self) -> Result<serde_json::Value, TransportError>;

    /// Disconnect from the server
    async fn disconnect(&mut self) -> Result<(), TransportError>;

    /// Check if the transport is connected
    fn is_connected(&self) -> bool;
}

/// Create a transport instance based on type
pub fn create_transport(
    transport_type: TransportType,
) -> Result<Box<dyn Transport>, TransportError> {
    match transport_type {
        TransportType::Stdio => Ok(Box::new(stdio::StdioTransport::new())),
        TransportType::Http { host, port } => {
            let config = http::HttpTransportConfig {
                base_url: format!("http://{}:{}", host, port),
                ..Default::default()
            };
            let transport = http::HttpTransport::new(config)?;
            Ok(Box::new(transport))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_type_display() {
        assert_eq!(TransportType::Stdio.to_string(), "stdio");

        let http = TransportType::Http {
            host: "localhost".to_string(),
            port: 8080,
        };
        assert_eq!(http.to_string(), "http://localhost:8080");
    }

    #[test]
    fn test_create_stdio_transport() {
        let result = create_transport(TransportType::Stdio);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_http_transport() {
        let http = TransportType::Http {
            host: "localhost".to_string(),
            port: 8080,
        };
        let result = create_transport(http);
        assert!(result.is_ok());
    }
}
