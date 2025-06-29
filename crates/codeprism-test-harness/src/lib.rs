//! CodePrism Test Harness
//!
//! A comprehensive test harness for validating MCP tools with automated
//! test execution, pattern validation, and performance monitoring.

pub mod config;
pub mod executor;
pub mod types;

// Re-export main types for convenience
pub use config::TestConfig;
pub use executor::TestExecutor;
pub use types::{TestCase, TestResult, TestSuite, TestSuiteResult};

use anyhow::Result;
use tracing_subscriber::{fmt, EnvFilter};

/// Initialize the test harness with logging
pub fn init() -> Result<()> {
    // Initialize tracing subscriber for logging
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    Ok(())
}

/// Main test harness struct that orchestrates test execution
pub struct TestHarness {
    executor: TestExecutor,
}

impl TestHarness {
    /// Create a new test harness from configuration
    pub fn new(config: TestConfig) -> Self {
        let executor = TestExecutor::new(config);
        Self { executor }
    }

    /// Load test harness from configuration file
    pub fn from_config_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let config = TestConfig::from_file(path)?;
        Ok(Self::new(config))
    }

    /// Run all configured test suites
    pub async fn run_all_tests(&self) -> Result<Vec<TestSuiteResult>> {
        self.executor.execute_all_test_suites().await
    }

    /// Run a specific test suite by name
    pub async fn run_test_suite(&self, suite_name: &str) -> Result<Option<TestSuiteResult>> {
        // Find the test suite by name
        let test_suites = self.executor.execute_all_test_suites().await?;
        Ok(test_suites
            .into_iter()
            .find(|result| result.test_suite.name == suite_name))
    }
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
