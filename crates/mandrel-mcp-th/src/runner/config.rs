//! Configuration types for test suite runner

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Execution mode for test suites
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ExecutionMode {
    /// Execute tests sequentially in dependency order
    #[default]
    Sequential,
    /// Execute tests in parallel groups based on dependencies
    Parallel,
}

/// Configuration for test suite runner behavior
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    /// Test execution mode (sequential or parallel)
    pub execution_mode: ExecutionMode,
    /// Maximum number of concurrent test executions in parallel mode
    pub max_concurrency: usize,
    /// Stop execution on first test failure
    pub fail_fast: bool,
    /// Timeout for setup operations
    pub setup_timeout: Duration,
    /// Timeout for teardown operations
    pub teardown_timeout: Duration,
    /// Timeout for dependency resolution
    pub dependency_timeout: Duration,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            execution_mode: ExecutionMode::Sequential,
            max_concurrency: 4,
            fail_fast: false,
            setup_timeout: Duration::from_secs(30),
            teardown_timeout: Duration::from_secs(10),
            dependency_timeout: Duration::from_secs(5),
        }
    }
}

impl RunnerConfig {
    /// Create a new runner configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set execution mode
    pub fn with_execution_mode(mut self, mode: ExecutionMode) -> Self {
        self.execution_mode = mode;
        self
    }

    /// Enable or disable parallel execution
    pub fn with_parallel_execution(mut self, enabled: bool) -> Self {
        self.execution_mode = if enabled {
            ExecutionMode::Parallel
        } else {
            ExecutionMode::Sequential
        };
        self
    }

    /// Set maximum concurrency for parallel execution
    pub fn with_max_concurrency(mut self, max: usize) -> Self {
        self.max_concurrency = max.clamp(1, 16); // Clamp between 1 and 16
        self
    }

    /// Enable or disable fail-fast behavior
    pub fn with_fail_fast(mut self, enabled: bool) -> Self {
        self.fail_fast = enabled;
        self
    }

    /// Set timeout values for setup and teardown operations
    pub fn with_timeouts(mut self, setup: Duration, teardown: Duration) -> Self {
        self.setup_timeout = setup;
        self.teardown_timeout = teardown;
        self
    }

    /// Set dependency resolution timeout
    pub fn with_dependency_timeout(mut self, timeout: Duration) -> Self {
        self.dependency_timeout = timeout;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RunnerConfig::default();
        assert_eq!(config.execution_mode, ExecutionMode::Sequential);
        assert_eq!(config.max_concurrency, 4);
        assert!(!config.fail_fast);
        assert_eq!(config.setup_timeout, Duration::from_secs(30));
        assert_eq!(config.teardown_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_config_builder_pattern() {
        let config = RunnerConfig::new()
            .with_parallel_execution(true)
            .with_max_concurrency(8)
            .with_fail_fast(true)
            .with_timeouts(Duration::from_secs(60), Duration::from_secs(20));

        assert_eq!(config.execution_mode, ExecutionMode::Parallel);
        assert_eq!(config.max_concurrency, 8);
        assert!(config.fail_fast);
        assert_eq!(config.setup_timeout, Duration::from_secs(60));
        assert_eq!(config.teardown_timeout, Duration::from_secs(20));
    }

    #[test]
    fn test_concurrency_clamping() {
        let config = RunnerConfig::new().with_max_concurrency(0);
        assert_eq!(config.max_concurrency, 1); // Should be clamped to minimum

        let config = RunnerConfig::new().with_max_concurrency(20);
        assert_eq!(config.max_concurrency, 16); // Should be clamped to maximum
    }
}
