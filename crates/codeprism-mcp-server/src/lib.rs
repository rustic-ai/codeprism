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
pub mod server;
pub mod tools;

pub use config::Config;
pub use error::{Error, Result};
pub use server::CodePrismMcpServer;

/// The current version of the CodePrism MCP Server
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The name of the MCP server for identification
pub const SERVER_NAME: &str = "codeprism-mcp-server";

/// The MCP protocol version this server implements
pub const MCP_VERSION: &str = "2025-06-18";
