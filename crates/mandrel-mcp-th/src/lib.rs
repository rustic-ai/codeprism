//! # MOTH - MOdel context protocol Test Harness
//!
//! A modern, comprehensive testing framework for MCP (Model Context Protocol) servers
//! built on the official Rust SDK. MOTH provides validation, compliance testing,
//! and detailed reporting for MCP server implementations.
//!
//! ## Features
//!
//! - **SDK-First**: Built on the official MCP Rust SDK for guaranteed protocol compliance
//! - **Transport Agnostic**: Supports stdio, HTTP, and SSE transports
//! - **Comprehensive Testing**: Protocol compliance, capability validation, and stress testing
//! - **Rich Reporting**: HTML, JSON, and JUnit XML report formats
//! - **Developer Friendly**: Clear error messages, detailed logs, and interactive CLI
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
//! │   Config    │───▶│   Client    │───▶│  Executor   │
//! │  (YAML)     │    │ (MCP/rmcp)  │    │ (Test Run)  │
//! └─────────────┘    └─────────────┘    └─────────────┘
//!        │                   │                   │
//!        ▼                   ▼                   ▼
//! ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
//! │ Validation  │    │   Server    │    │  Reporting  │
//! │  (Schema)   │    │ (Process)   │    │ (JSON/HTML) │
//! └─────────────┘    └─────────────┘    └─────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! # async fn example() -> mandrel_mcp_th::error::Result<()> {
//! use mandrel_mcp_th::cli::{Cli, Commands};
//! use clap::Parser;
//!
//! let cli = Cli::parse();
//! match cli.command {
//!     Commands::Test { spec, .. } => {
//!         // Test execution logic
//!         println!("Running tests from: {}", spec.display());
//!     }
//!     _ => {}
//! }
//! # Ok(())
//! # }
//! ```

pub mod cli;
pub mod client;
pub mod error;
pub mod executor;
pub mod reporting;
pub mod spec;
pub mod validation;

// Re-export commonly used types
pub use error::{Error, Result};

/// The version of MOTH
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The name of the test harness
pub const NAME: &str = "MOTH";

/// The full name
pub const FULL_NAME: &str = "MOdel context protocol Test Harness";

/// The MCP protocol version this harness supports
pub const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

/// Default timeout for MCP operations
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Maximum concurrent test executions by default
pub const DEFAULT_MAX_CONCURRENCY: usize = 4;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Test that all modules are accessible
        let _error = Error::config("test");
        // Module compilation test - if this compiles, modules are accessible
    }

    #[test]
    fn test_constants() {
        assert_eq!(NAME, "MOTH");
        assert_eq!(FULL_NAME, "MOdel context protocol Test Harness");
        assert_eq!(MCP_PROTOCOL_VERSION, "2025-06-18");
        assert_eq!(DEFAULT_TIMEOUT_SECS, 30);
        assert_eq!(DEFAULT_MAX_CONCURRENCY, 4);
    }
}
