//! Core types for the MCP test harness
//!
//! This module defines the fundamental data structures used throughout the
//! test harness for representing MCP servers, capabilities, messages, and
//! test execution results.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Information about an MCP server being tested
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
    /// Server description
    pub description: Option<String>,
    /// Command used to start the server
    pub command: String,
    /// Arguments passed to the server
    pub args: Vec<String>,
    /// Environment variables for the server
    pub environment: HashMap<String, String>,
}

/// Core MCP server capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct McpCapabilities {
    /// Whether the server supports tools
    pub tools: bool,
    /// Whether the server supports resources
    pub resources: bool,
    /// Whether the server supports prompts
    pub prompts: bool,
    /// Whether the server supports sampling
    pub sampling: bool,
    /// Whether the server supports logging
    pub logging: bool,
    /// Experimental features
    pub experimental: Option<Vec<String>>,
}

/// Generic MCP message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    /// JSON-RPC version (should be "2.0")
    pub jsonrpc: String,
    /// Message ID (for requests and responses)
    pub id: Option<serde_json::Value>,
    /// Method name (for requests and notifications)
    pub method: Option<String>,
    /// Parameters (for requests and notifications)  
    pub params: Option<serde_json::Value>,
    /// Result (for successful responses)
    pub result: Option<serde_json::Value>,
    /// Error (for error responses)
    pub error: Option<McpError>,
}

/// MCP error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Additional error data
    pub data: Option<serde_json::Value>,
}

/// Test execution statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TestStats {
    /// Total number of tests executed
    pub total_tests: usize,
    /// Number of tests that passed
    pub passed_tests: usize,
    /// Number of tests that failed
    pub failed_tests: usize,
    /// Number of tests that were skipped
    pub skipped_tests: usize,
    /// Total execution time in milliseconds
    pub total_duration_ms: u128,
    /// Average execution time per test in milliseconds
    pub average_duration_ms: f64,
}

impl TestStats {
    /// Calculate the pass rate as a percentage
    pub fn pass_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            (self.passed_tests as f64 / self.total_tests as f64) * 100.0
        }
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed_tests == 0 && self.total_tests > 0
    }

    /// Update average duration
    pub fn update_average(&mut self) {
        if self.total_tests > 0 {
            self.average_duration_ms = self.total_duration_ms as f64 / self.total_tests as f64;
        }
    }
}

/// Performance metrics for a test execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PerformanceMetrics {
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Network latency in milliseconds
    pub network_latency_ms: Option<u64>,
    /// Throughput in operations per second
    pub throughput_ops_per_sec: Option<f64>,
}

/// Test execution context
#[derive(Debug, Clone)]
pub struct TestContext {
    /// Unique test execution ID
    pub execution_id: Uuid,
    /// Test start time
    pub start_time: DateTime<Utc>,
    /// Server being tested
    pub server_info: ServerInfo,
    /// Test configuration
    pub config: TestConfig,
}

/// Configuration for test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Maximum test execution timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum number of concurrent tests
    pub max_concurrency: usize,
    /// Whether to stop on first failure
    pub fail_fast: bool,
    /// Retry configuration
    pub retry: RetryConfig,
    /// Performance thresholds
    pub performance: PerformanceConfig,
}

/// Retry configuration for failed tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: usize,
    /// Delay between retries in milliseconds
    pub retry_delay_ms: u64,
    /// Whether to use exponential backoff
    pub exponential_backoff: bool,
    /// Error patterns that should trigger retries
    pub retry_on_patterns: Vec<String>,
}

/// Performance testing configuration  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum acceptable response time in milliseconds
    pub max_response_time_ms: u128,
    /// Maximum acceptable memory usage in megabytes
    pub max_memory_usage_mb: f64,
    /// Whether to collect detailed performance metrics
    pub collect_metrics: bool,
    /// Performance test sample size
    pub sample_size: usize,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_concurrency: 4,
            fail_fast: false,
            retry: RetryConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 2,
            retry_delay_ms: 1000,
            exponential_backoff: true,
            retry_on_patterns: vec![
                "connection refused".to_string(),
                "timeout".to_string(),
                "temporary failure".to_string(),
            ],
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_response_time_ms: 5000, // 5 seconds
            max_memory_usage_mb: 100.0, // 100 MB
            collect_metrics: true,
            sample_size: 10,
        }
    }
}

/// Validation result for test responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the response passed validation
    pub valid: bool,
    /// Validation error messages
    pub errors: Vec<String>,
    /// Validation warnings (non-fatal issues)
    pub warnings: Vec<String>,
    /// Detailed validation information
    pub details: HashMap<String, serde_json::Value>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn success() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            details: HashMap::new(),
        }
    }

    /// Create a failed validation result with error message
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            valid: false,
            errors: vec![message.into()],
            warnings: Vec::new(),
            details: HashMap::new(),
        }
    }

    /// Add a warning to the validation result
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Add detail information to the validation result
    pub fn with_detail(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.details.insert(key.into(), value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_stats_pass_rate() {
        let stats = TestStats {
            total_tests: 10,
            passed_tests: 8,
            failed_tests: 2,
            ..Default::default()
        };

        assert_eq!(stats.pass_rate(), 80.0);
        assert!(!stats.all_passed());
    }

    #[test]
    fn test_test_stats_all_passed() {
        let stats = TestStats {
            total_tests: 5,
            passed_tests: 5,
            failed_tests: 0,
            ..Default::default()
        };

        assert_eq!(stats.pass_rate(), 100.0);
        assert!(stats.all_passed());
    }

    #[test]
    fn test_validation_result() {
        let success = ValidationResult::success();
        assert!(success.valid);
        assert!(success.errors.is_empty());

        let error = ValidationResult::error("Test error");
        assert!(!error.valid);
        assert_eq!(error.errors.len(), 1);
        assert_eq!(error.errors[0], "Test error");
    }

    #[test]
    fn test_default_configurations() {
        let config = TestConfig::default();
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_concurrency, 4);
        assert!(!config.fail_fast);

        let perf_config = PerformanceConfig::default();
        assert_eq!(perf_config.max_response_time_ms, 5000);
        assert_eq!(perf_config.max_memory_usage_mb, 100.0);
        assert!(perf_config.collect_metrics);
    }

    #[test]
    fn test_all_passed_with_failures() {
        let stats = TestStats {
            total_tests: 10,
            passed_tests: 8,
            failed_tests: 2,
            ..Default::default()
        };
        assert!(!stats.all_passed());
    }

    #[test]
    fn test_all_passed_success() {
        let stats = TestStats {
            total_tests: 5,
            passed_tests: 5,
            failed_tests: 0,
            ..Default::default()
        };
        assert!(stats.all_passed());
    }
}
