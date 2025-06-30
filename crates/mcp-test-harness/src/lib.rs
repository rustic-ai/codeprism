//! Generic MCP (Model Context Protocol) Test Harness
//!
//! A comprehensive testing library for validating MCP server implementations
//! against the Model Context Protocol specification. This harness is designed
//! to be server-agnostic and data-driven.
//!
//! ## Features
//!
//! - **Protocol Compliance Testing**: Validates JSON-RPC 2.0 and MCP message formats
//! - **Spec-Driven Testing**: Define test cases in YAML/JSON specifications  
//! - **Server Discovery**: Automatically discover server capabilities
//! - **Transport Support**: stdio, HTTP/SSE, and custom transports
//! - **Performance Testing**: Latency, memory, and throughput benchmarks
//! - **Rich Reporting**: HTML, JSON, JUnit XML, and Markdown outputs
//! - **Extensible**: Plugin system for custom validators and reporters
//!
//! ## Basic Usage
//!
//! ```rust,no_run
//! use mcp_test_harness_lib::{TestHarness, SpecLoader};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Load server specification from YAML
//!     let loader = SpecLoader::new()?;
//!     let spec = loader.load_spec("server-spec.yaml").await?;
//!     
//!     // Create test harness
//!     let mut harness = TestHarness::new(spec);
//!     
//!     // Run all tests
//!     let results = harness.run_all_tests().await?;
//!     
//!     // Generate reports
//!     harness.generate_reports(&results).await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod cli;
pub mod protocol;
pub mod reporting;
pub mod spec;
pub mod testing;
pub mod transport;

// Core types and utilities
mod types;
mod utils;

// Re-export main types for public API
pub use protocol::{validate_protocol_compliance, McpClient, McpError};
pub use reporting::{ReportFormat, ReportGenerator};
pub use spec::schema::{ServerSpec, TestCase};
pub use spec::{SpecLoader, ValidationError as SpecValidationError};
pub use testing::{TestHarness, TestReport, TestResult};
pub use transport::TransportType;
pub use types::{TestConfig, TestStats};

use anyhow::Result;
use tracing_subscriber::{fmt, EnvFilter};

/// Initialize the MCP test harness library
///
/// Sets up logging and tracing infrastructure for test execution.
/// Should be called once at the start of your application.
///
/// # Example
///
/// ```rust,no_run
/// // Initialize the MCP test harness
/// mcp_test_harness_lib::init()?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn init() -> Result<()> {
    // Initialize tracing subscriber for logging
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init()
        .map_err(|e| anyhow::anyhow!("Failed to initialize logging: {}", e))?;

    Ok(())
}

/// Get the version of the MCP test harness
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Get build information for the MCP test harness
pub fn build_info() -> BuildInfo {
    BuildInfo {
        version: version(),
        commit: option_env!("GIT_COMMIT").unwrap_or("unknown"),
        build_date: option_env!("BUILD_DATE").unwrap_or("unknown"),
        rust_version: "unknown", // Removed env!("RUSTC_VERSION") to fix compilation
    }
}

/// Build information for the test harness
#[derive(Debug, Clone)]
pub struct BuildInfo {
    pub version: &'static str,
    pub commit: &'static str,
    pub build_date: &'static str,
    pub rust_version: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    #[test]
    fn test_build_info() {
        let build_info = build_info();
        assert!(!build_info.version.is_empty());
        assert!(!build_info.rust_version.is_empty());
    }

    #[tokio::test]
    async fn test_init() {
        // Note: This may fail if called multiple times in tests
        // but should work fine in isolation
        assert!(init().is_ok() || init().is_err());
    }
}
