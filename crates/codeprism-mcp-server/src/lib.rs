//! CodePrism MCP Server
//!
//! This crate provides an MCP (Model Context Protocol) server implementation
//! built on the official Rust SDK. It exposes CodePrism's code analysis
//! capabilities through the standardized MCP protocol.
//!
//! # Architecture
//!
//! The server is organized into several modules:
//! - `server`: Core MCP server implementation
//! - `tools`: MCP tool implementations (core, search, analysis, workflow)
//! - `config`: Configuration management
//! - `error`: Error types and handling
//!
//! # Usage
//!
//! The server can be run as a standalone binary or embedded in other applications.
//! It supports stdio transport for communication with MCP clients.

pub mod config;
pub mod error;
pub mod response;
pub mod server;
pub mod tools;

#[cfg(test)]
mod integration_test;

pub use config::Config;
pub use error::{Error, Result};
pub use server::CodePrismMcpServer;

/// The current version of the CodePrism MCP Server
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The name of the MCP server for identification
pub const SERVER_NAME: &str = "codeprism-mcp-server";

/// The MCP protocol version this server implements
pub const MCP_VERSION: &str = "2025-06-18";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_import_rmcp_crate() {
        // Test that we can import the rmcp crate - specific types will be explored in task #159
        extern crate rmcp;

        // Basic verification that the constants are defined correctly
        assert_eq!(SERVER_NAME, "codeprism-mcp-server");
        assert_eq!(MCP_VERSION, "2025-06-18");
    }

    #[tokio::test]
    async fn test_server_creation() {
        // Test that we can create a server instance with default configuration
        let config = Config::default();
        let server = CodePrismMcpServer::new(config).await;
        assert!(server.is_ok(), "Server creation should succeed");

        let server = server.unwrap();
        assert_eq!(server.config().server().name, "codeprism-mcp-server");
    }

    #[tokio::test]
    async fn test_server_info() {
        // Test that server info is correctly configured
        use rmcp::{model::ProtocolVersion, ServerHandler};

        let config = Config::default();
        let server = CodePrismMcpServer::new(config).await.unwrap();

        let info = server.get_info();
        assert_eq!(info.protocol_version, ProtocolVersion::V_2024_11_05);
        assert_eq!(info.server_info.name, "codeprism-mcp-server");
        assert!(info.instructions.is_some());
        assert!(info.capabilities.tools.is_some());
    }

    // Additional tests moved to integration_test.rs module
}
