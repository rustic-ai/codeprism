//! CodePrism Test Harness
//!
//! A generic MCP (Model Context Protocol) server test harness for validating
//! protocol compliance, testing capabilities, and ensuring quality for any
//! MCP server implementation.
//!
//! This library provides comprehensive testing infrastructure including
//! test execution, pattern validation, and performance monitoring.

pub mod config;
pub mod executor;
pub mod prompts;
pub mod protocol;
pub mod resources;
pub mod server;
pub mod tools;
pub mod transport;
pub mod types;

// Re-export main types for convenience
pub use config::TestConfig;
pub use executor::TestExecutor;
pub use prompts::{PromptTester, PromptValidator};
pub use protocol::{JsonRpcMessage, McpCapabilities, McpClient, ProtocolValidator};
pub use resources::{ResourceTester, ResourceValidator};
pub use tools::{ToolTester, ToolValidator};
pub use types::{TestCase, TestResult, TestSuite, TestSuiteResult};

use anyhow::Result;
use tracing_subscriber::{fmt, EnvFilter};

/// Test harness for MCP servers
pub struct TestHarness {
    executor: TestExecutor,
    #[allow(dead_code)] // Will be used in future implementations
    config: TestConfig,
}

impl TestHarness {
    /// Create a new test harness from configuration
    pub fn new(config: TestConfig) -> Self {
        let executor = TestExecutor::new(config.clone());
        Self { executor, config }
    }

    /// Load test harness from configuration file
    pub fn from_config_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let config = TestConfig::from_file(path)?;
        Ok(Self::new(config))
    }

    /// Execute all test suites
    pub async fn run_all_tests(&self) -> Result<Vec<TestSuiteResult>> {
        self.executor.execute_all_test_suites().await
    }

    /// Execute a specific test suite by name
    pub async fn run_test_suite(&self, suite_name: &str) -> Result<Option<TestSuiteResult>> {
        // Find the test suite by name
        let test_suites = self.executor.execute_all_test_suites().await?;
        Ok(test_suites
            .into_iter()
            .find(|result| result.test_suite.name == suite_name))
    }
}

/// Initialize the test harness library
pub fn init() -> Result<()> {
    // Initialize tracing subscriber for logging
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }

    #[test]
    fn test_harness_creation() {
        let config = TestConfig::default_for_tests();
        let _harness = TestHarness::new(config);
    }
}
