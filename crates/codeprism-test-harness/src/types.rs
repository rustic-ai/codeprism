//! Core types for the CodePrism Test Harness
//!
//! This module defines the fundamental data structures used throughout
//! the test harness for organizing tests, validating results, and
//! tracking execution metrics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// A single test case that validates a specific MCP tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Unique identifier for the test case
    pub id: String,
    /// Human-readable description of what this test validates
    pub description: String,
    /// The MCP tool being tested
    pub tool_name: String,
    /// Input parameters to pass to the tool
    pub input_params: serde_json::Value,
    /// Expected validation patterns for the response
    pub expected: ValidationPattern,
    /// Performance constraints for this test
    pub performance: PerformanceConstraints,
    /// Project or context to run the test against
    pub project_path: Option<String>,
    /// Whether this test is currently enabled
    pub enabled: bool,
}

/// A collection of related test cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    /// Name of the test suite
    pub name: String,
    /// Description of what this suite validates
    pub description: String,
    /// List of test cases in this suite
    pub test_cases: Vec<TestCase>,
    /// Whether to run tests in parallel
    pub parallel_execution: bool,
    /// Maximum concurrent tests for this suite
    pub max_concurrency: Option<usize>,
    /// Setup steps required before running this suite
    pub setup: Option<SetupConfig>,
    /// Cleanup steps after running this suite
    pub cleanup: Option<CleanupConfig>,
}

/// Validation patterns for test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationPattern {
    /// JSON path patterns to validate in the response
    pub patterns: Vec<JsonPathPattern>,
    /// Custom validation scripts to run
    pub custom_scripts: Vec<CustomScript>,
    /// Response schema validation
    pub schema: Option<serde_json::Value>,
    /// Whether to allow extra fields not specified in patterns
    pub allow_extra_fields: bool,
}

/// JSON path pattern for validating specific fields in responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonPathPattern {
    /// JSON path to the field (e.g., "result.total_files")
    pub key: String,
    /// The type of validation to perform
    pub validation: PatternValidation,
    /// Whether this pattern is required (fails test if not met)
    pub required: bool,
}

/// Types of validation that can be performed on JSON values
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PatternValidation {
    /// Exact value match
    Equals { value: serde_json::Value },
    /// Value must be within a numeric range
    Range { min: f64, max: f64 },
    /// Array must contain specific values
    Contains { values: Vec<serde_json::Value> },
    /// String must match regex pattern
    Regex { pattern: String },
    /// Value must be one of the specified options
    OneOf { options: Vec<serde_json::Value> },
    /// Array length constraints
    ArrayLength {
        min: Option<usize>,
        max: Option<usize>,
    },
    /// Object must have specified keys
    HasKeys { keys: Vec<String> },
    /// Custom validation expression
    Expression { expr: String },
}

/// Custom validation script configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomScript {
    /// Name of the script
    pub name: String,
    /// Script language (python, bash, etc.)
    pub language: String,
    /// Script content or path to script file
    pub content: String,
    /// Environment variables to set for the script
    pub env: HashMap<String, String>,
    /// Timeout for script execution
    pub timeout_seconds: u64,
}

/// Performance constraints for test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConstraints {
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
    /// Maximum memory usage in MB
    pub max_memory_mb: Option<f64>,
    /// Expected response time percentiles
    pub response_time_percentiles: Option<ResponseTimePercentiles>,
}

/// Response time percentile expectations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimePercentiles {
    pub p50_ms: Option<u64>,
    pub p90_ms: Option<u64>,
    pub p95_ms: Option<u64>,
    pub p99_ms: Option<u64>,
}

/// Test execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// The test case that was executed
    pub test_case: TestCase,
    /// Whether the test passed
    pub success: bool,
    /// Start time of test execution
    pub start_time: DateTime<Utc>,
    /// End time of test execution
    pub end_time: DateTime<Utc>,
    /// Total execution duration
    pub duration: Duration,
    /// Memory usage during test execution
    pub memory_usage_mb: Option<f64>,
    /// The actual response received from the tool
    pub actual_response: Option<serde_json::Value>,
    /// Validation results for each pattern
    pub validation_results: Vec<ValidationResult>,
    /// Error message if the test failed
    pub error_message: Option<String>,
    /// Additional context or debugging information
    pub debug_info: HashMap<String, serde_json::Value>,
}

/// Result of a single validation pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// The pattern that was validated
    pub pattern: JsonPathPattern,
    /// Whether this validation passed
    pub passed: bool,
    /// Actual value found at the JSON path
    pub actual_value: Option<serde_json::Value>,
    /// Error message if validation failed
    pub error_message: Option<String>,
}

/// Setup configuration for test suites
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupConfig {
    /// Commands to run before the test suite
    pub commands: Vec<String>,
    /// Environment variables to set
    pub env: HashMap<String, String>,
    /// Working directory for setup commands
    pub working_dir: Option<String>,
    /// Timeout for setup operations
    pub timeout_seconds: u64,
}

/// Cleanup configuration for test suites
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupConfig {
    /// Commands to run after the test suite
    pub commands: Vec<String>,
    /// Whether to run cleanup even if tests fail
    pub always_run: bool,
    /// Timeout for cleanup operations
    pub timeout_seconds: u64,
}

/// Test execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecutionStats {
    /// Total number of tests executed
    pub total_tests: usize,
    /// Number of tests that passed
    pub passed_tests: usize,
    /// Number of tests that failed
    pub failed_tests: usize,
    /// Number of tests that were skipped
    pub skipped_tests: usize,
    /// Total execution time for all tests
    pub total_duration: Duration,
    /// Average execution time per test
    pub average_duration: Duration,
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
    /// Performance percentiles across all tests
    pub performance_percentiles: ResponseTimePercentiles,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Average memory usage across all tests
    pub average_mb: f64,
    /// Peak memory usage
    pub peak_mb: f64,
    /// Minimum memory usage
    pub min_mb: f64,
}

/// Test suite execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteResult {
    /// The test suite that was executed
    pub test_suite: TestSuite,
    /// Results for each test case
    pub test_results: Vec<TestResult>,
    /// Overall execution statistics
    pub stats: TestExecutionStats,
    /// Start time of suite execution
    pub start_time: DateTime<Utc>,
    /// End time of suite execution
    pub end_time: DateTime<Utc>,
    /// Whether the entire suite passed
    pub suite_passed: bool,
}

impl TestCase {
    /// Create a new test case with minimal required fields
    pub fn new(id: String, tool_name: String, input_params: serde_json::Value) -> Self {
        Self {
            id,
            description: String::new(),
            tool_name,
            input_params,
            expected: ValidationPattern::default(),
            performance: PerformanceConstraints::default(),
            project_path: None,
            enabled: true,
        }
    }
}

impl Default for ValidationPattern {
    fn default() -> Self {
        Self {
            patterns: Vec::new(),
            custom_scripts: Vec::new(),
            schema: None,
            allow_extra_fields: true,
        }
    }
}

impl Default for PerformanceConstraints {
    fn default() -> Self {
        Self {
            max_execution_time_ms: 30000, // 30 seconds default
            max_memory_mb: None,
            response_time_percentiles: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_creation() {
        let test_case = TestCase::new(
            "test_1".to_string(),
            "repository_stats".to_string(),
            serde_json::json!({}),
        );

        assert_eq!(test_case.id, "test_1");
        assert_eq!(test_case.tool_name, "repository_stats");
        assert!(test_case.enabled);
    }

    #[test]
    fn test_validation_pattern_serialization() {
        let pattern = ValidationPattern {
            patterns: vec![JsonPathPattern {
                key: "result.total_files".to_string(),
                validation: PatternValidation::Range {
                    min: 1.0,
                    max: 100.0,
                },
                required: true,
            }],
            custom_scripts: vec![],
            schema: None,
            allow_extra_fields: true,
        };

        let serialized = serde_yaml::to_string(&pattern).unwrap();
        let deserialized: ValidationPattern = serde_yaml::from_str(&serialized).unwrap();

        assert_eq!(pattern.patterns.len(), deserialized.patterns.len());
    }
}
