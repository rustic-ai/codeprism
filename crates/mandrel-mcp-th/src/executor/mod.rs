//! Test execution module
//!
//! This module provides the core test execution framework for running
//! MCP test suites and collecting results.

use crate::client::{McpClient, ServerConfig};
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Main test execution framework
pub struct TestRunner {
    /// Configuration for test execution
    config: TestConfig,
    /// Progress tracking and reporting
    progress_tracker: ProgressTracker,
}

/// Configuration for test execution behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Maximum number of concurrent tests
    pub max_concurrency: usize,
    /// Stop execution on first failure
    pub fail_fast: bool,
    /// Test filter pattern
    pub filter: Option<String>,
    /// Timeout for individual tests
    pub test_timeout: Duration,
    /// Timeout for server startup
    pub server_timeout: Duration,
    /// Number of retry attempts for failed tests
    pub retry_attempts: u32,
}

/// Represents a test suite loaded from specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    /// Test suite metadata
    pub name: String,
    pub version: String,
    pub description: Option<String>,

    /// Server configuration
    pub server: ServerConfig,

    /// Test cases in this suite
    pub tests: Vec<TestCase>,

    /// Suite-level configuration
    pub config: TestConfig,
}

/// Individual test case definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Test identification
    pub name: String,
    pub description: Option<String>,

    /// Test type and parameters
    pub test_type: TestType,
    pub parameters: serde_json::Value,

    /// Expected results
    pub expected: ExpectedResult,

    /// Test-specific configuration
    pub timeout: Option<Duration>,
    pub retry_attempts: Option<u32>,
}

/// Different types of tests that can be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestType {
    /// Test tool functionality
    ToolCall { tool_name: String },
    /// Test resource access
    ResourceRead { resource_uri: String },
    /// Test server capabilities
    CapabilityCheck,
    /// Test connection health
    HealthCheck,
}

/// Expected test results for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedResult {
    /// Whether the test should succeed or fail
    pub should_succeed: bool,
    /// Expected response content patterns
    pub content_patterns: Vec<String>,
    /// Performance expectations
    pub performance: Option<PerformanceExpectation>,
}

/// Performance expectations for tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceExpectation {
    /// Maximum allowed response time
    pub max_response_time: Duration,
    /// Expected memory usage limits
    pub max_memory_usage: Option<u64>,
}

/// Result of test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test identification
    pub test_name: String,
    pub suite_name: String,

    /// Execution outcome
    pub status: TestStatus,
    pub error_message: Option<String>,

    /// Timing information
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub duration: Duration,

    /// Response data (if successful)
    pub response_data: Option<serde_json::Value>,

    /// Performance metrics
    pub performance: PerformanceMetrics,
}

/// Test execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestStatus {
    /// Test passed all validations
    Passed,
    /// Test failed validation or execution
    Failed,
    /// Test was skipped due to filter or dependency
    Skipped,
    /// Test execution timed out
    Timeout,
    /// Test encountered an error during execution
    Error,
}

/// Performance metrics collected during test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Memory usage during test (if available)
    pub memory_usage_bytes: Option<u64>,
    /// Number of retry attempts made
    pub retry_attempts: u32,
}

/// Progress tracking for test execution
pub struct ProgressTracker {
    /// Total number of tests to execute
    pub total_tests: usize,
    /// Number of completed tests
    pub completed_tests: usize,
    /// Number of passed tests
    pub passed_tests: usize,
    /// Number of failed tests
    pub failed_tests: usize,
    /// Number of skipped tests
    pub skipped_tests: usize,
    /// Execution start time
    pub start_time: Instant,
}

/// Complete test suite execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteResult {
    pub suite_name: String,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration: Duration,
    pub test_results: Vec<TestResult>,
}

/// Validation result for test expectations
#[derive(Debug)]
struct ValidationResult {
    passed: bool,
    error_message: Option<String>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 4,
            fail_fast: false,
            filter: None,
            test_timeout: Duration::from_secs(30),
            server_timeout: Duration::from_secs(10),
            retry_attempts: 2,
        }
    }
}

impl TestRunner {
    /// Create a new test runner with configuration
    pub fn new(config: TestConfig) -> Self {
        Self {
            config,
            progress_tracker: ProgressTracker::new(),
        }
    }

    /// Execute a complete test suite
    pub async fn execute_suite(&mut self, suite: TestSuite) -> Result<SuiteResult> {
        info!("Starting test suite execution: {}", suite.name);

        // Initialize progress tracking
        self.progress_tracker.initialize(suite.tests.len());

        // Create MCP client
        let mut client = McpClient::new(suite.server.clone()).await?;

        // Connect to server with timeout
        let connection_result = timeout(self.config.server_timeout, client.connect()).await;

        match connection_result {
            Ok(Ok(())) => info!("Successfully connected to MCP server"),
            Ok(Err(e)) => return Err(Error::connection(format!("Failed to connect: {}", e))),
            Err(_) => return Err(Error::connection("Server connection timeout")),
        }

        // Execute tests
        let results = self
            .execute_tests(&mut client, &suite.tests, &suite.name)
            .await?;

        // Disconnect from server
        client.disconnect().await?;

        // Generate suite result
        Ok(SuiteResult::from_test_results(suite.name, results))
    }

    /// Execute individual test cases
    async fn execute_tests(
        &mut self,
        client: &mut McpClient,
        tests: &[TestCase],
        suite_name: &str,
    ) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();

        for test in tests {
            // Check for shutdown signal before each test
            if crate::is_shutdown_requested() {
                warn!("Shutdown signal received, stopping test execution gracefully");

                // Mark remaining tests as skipped
                for remaining_test in tests.iter().skip(results.len()) {
                    let skipped_result = TestResult::skipped(
                        remaining_test.name.clone(),
                        suite_name.to_string(),
                        "Interrupted by shutdown signal".to_string(),
                    );
                    self.progress_tracker.record_result(&skipped_result);
                    results.push(skipped_result);
                }
                break;
            }

            // Check if test should be executed (filtering)
            if !self.should_execute_test(test) {
                let skipped_result = TestResult::skipped(
                    test.name.clone(),
                    suite_name.to_string(),
                    "Filtered".to_string(),
                );
                self.progress_tracker.record_result(&skipped_result);
                results.push(skipped_result);
                continue;
            }

            // Execute test with retries
            let mut result = self.execute_test_with_retries(client, test).await;
            result.suite_name = suite_name.to_string();

            // Update progress
            self.progress_tracker.record_result(&result);

            // Check fail-fast condition
            if self.config.fail_fast && result.status == TestStatus::Failed {
                warn!("Stopping execution due to fail-fast mode");
                results.push(result);
                break;
            }

            results.push(result);
        }

        Ok(results)
    }

    /// Check if a test should be executed based on filters
    fn should_execute_test(&self, test: &TestCase) -> bool {
        if let Some(filter) = &self.config.filter {
            test.name.contains(filter)
        } else {
            true
        }
    }

    /// Execute a single test with retry logic
    async fn execute_test_with_retries(
        &self,
        client: &mut McpClient,
        test: &TestCase,
    ) -> TestResult {
        let max_attempts = test.retry_attempts.unwrap_or(self.config.retry_attempts);
        let test_timeout = test.timeout.unwrap_or(self.config.test_timeout);

        for attempt in 0..=max_attempts {
            debug!(
                "Executing test '{}' (attempt {} of {})",
                test.name,
                attempt + 1,
                max_attempts + 1
            );

            let result = timeout(
                test_timeout,
                self.execute_single_test(client, test, attempt),
            )
            .await;

            match result {
                Ok(Ok(test_result)) => {
                    if test_result.status == TestStatus::Passed || attempt == max_attempts {
                        return test_result;
                    }
                }
                Ok(Err(e)) => {
                    if attempt == max_attempts {
                        return TestResult::error(
                            test.name.clone(),
                            format!("Execution error: {}", e),
                        );
                    }
                }
                Err(_) => {
                    if attempt == max_attempts {
                        return TestResult::timeout(test.name.clone());
                    }
                }
            }

            // Brief delay before retry
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        TestResult::error(
            test.name.clone(),
            "Maximum retry attempts exceeded".to_string(),
        )
    }

    /// Execute a single test case
    async fn execute_single_test(
        &self,
        client: &mut McpClient,
        test: &TestCase,
        retry_attempt: u32,
    ) -> Result<TestResult> {
        let start_time = chrono::Utc::now();
        let start_instant = Instant::now();

        // Execute based on test type
        let response_data = match &test.test_type {
            TestType::ToolCall { tool_name } => {
                let result = client
                    .call_tool(tool_name, Some(test.parameters.clone()))
                    .await?;
                serde_json::to_value(result)?
            }
            TestType::ResourceRead { resource_uri } => {
                let result = client.read_resource(resource_uri).await?;
                serde_json::to_value(result)?
            }
            TestType::CapabilityCheck => {
                let tools = client.list_tools().await?;
                serde_json::to_value(tools)?
            }
            TestType::HealthCheck => {
                let is_healthy = client.health_check().await?;
                serde_json::to_value(is_healthy)?
            }
        };

        let duration = start_instant.elapsed();

        // Validate results against expectations
        let validation_result =
            self.validate_test_result(&response_data, &test.expected, duration)?;

        Ok(TestResult {
            test_name: test.name.clone(),
            suite_name: "default".to_string(), // Will be set by caller
            status: if validation_result.passed {
                TestStatus::Passed
            } else {
                TestStatus::Failed
            },
            error_message: validation_result.error_message,
            start_time,
            duration,
            response_data: Some(response_data),
            performance: PerformanceMetrics {
                response_time_ms: duration.as_millis() as u64,
                memory_usage_bytes: None,
                retry_attempts: retry_attempt,
            },
        })
    }

    /// Validate test results against expectations
    fn validate_test_result(
        &self,
        response_data: &serde_json::Value,
        expected: &ExpectedResult,
        duration: Duration,
    ) -> Result<ValidationResult> {
        let mut errors = Vec::new();

        // Check performance expectations
        if let Some(perf) = &expected.performance {
            if duration > perf.max_response_time {
                errors.push(format!(
                    "Response time {}ms exceeded maximum {}ms",
                    duration.as_millis(),
                    perf.max_response_time.as_millis()
                ));
            }
        }

        // Check content patterns
        let response_text = response_data.to_string();
        for pattern in &expected.content_patterns {
            if !response_text.contains(pattern) {
                errors.push(format!(
                    "Response does not contain expected pattern: {}",
                    pattern
                ));
            }
        }

        Ok(ValidationResult {
            passed: errors.is_empty(),
            error_message: if errors.is_empty() {
                None
            } else {
                Some(errors.join("; "))
            },
        })
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new() -> Self {
        Self {
            total_tests: 0,
            completed_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            skipped_tests: 0,
            start_time: Instant::now(),
        }
    }

    /// Initialize progress tracking with total test count
    pub fn initialize(&mut self, total_tests: usize) {
        self.total_tests = total_tests;
        self.start_time = Instant::now();
    }

    /// Record a test result and update progress
    pub fn record_result(&mut self, result: &TestResult) {
        self.completed_tests += 1;

        match result.status {
            TestStatus::Passed => self.passed_tests += 1,
            TestStatus::Failed | TestStatus::Error | TestStatus::Timeout => self.failed_tests += 1,
            TestStatus::Skipped => self.skipped_tests += 1,
        }
    }

    /// Get current progress percentage
    pub fn progress_percentage(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            (self.completed_tests as f64 / self.total_tests as f64) * 100.0
        }
    }
}

impl TestResult {
    /// Create a skipped test result
    pub fn skipped(test_name: String, suite_name: String, reason: String) -> Self {
        Self {
            test_name,
            suite_name,
            status: TestStatus::Skipped,
            error_message: Some(reason),
            start_time: chrono::Utc::now(),
            duration: Duration::from_millis(0),
            response_data: None,
            performance: PerformanceMetrics {
                response_time_ms: 0,
                memory_usage_bytes: None,
                retry_attempts: 0,
            },
        }
    }

    /// Create an error test result
    pub fn error(test_name: String, error_message: String) -> Self {
        Self {
            test_name,
            suite_name: "default".to_string(),
            status: TestStatus::Error,
            error_message: Some(error_message),
            start_time: chrono::Utc::now(),
            duration: Duration::from_millis(0),
            response_data: None,
            performance: PerformanceMetrics {
                response_time_ms: 0,
                memory_usage_bytes: None,
                retry_attempts: 0,
            },
        }
    }

    /// Create a timeout test result
    pub fn timeout(test_name: String) -> Self {
        Self {
            test_name,
            suite_name: "default".to_string(),
            status: TestStatus::Timeout,
            error_message: Some("Test execution timed out".to_string()),
            start_time: chrono::Utc::now(),
            duration: Duration::from_millis(0),
            response_data: None,
            performance: PerformanceMetrics {
                response_time_ms: 0,
                memory_usage_bytes: None,
                retry_attempts: 0,
            },
        }
    }
}

impl SuiteResult {
    /// Create suite result from individual test results
    pub fn from_test_results(suite_name: String, test_results: Vec<TestResult>) -> Self {
        let total_tests = test_results.len();
        let passed = test_results
            .iter()
            .filter(|r| r.status == TestStatus::Passed)
            .count();
        let failed = test_results
            .iter()
            .filter(|r| {
                matches!(
                    r.status,
                    TestStatus::Failed | TestStatus::Error | TestStatus::Timeout
                )
            })
            .count();
        let skipped = test_results
            .iter()
            .filter(|r| r.status == TestStatus::Skipped)
            .count();

        let duration = if test_results.is_empty() {
            Duration::from_millis(0)
        } else {
            test_results.iter().map(|r| r.duration).sum()
        };

        Self {
            suite_name,
            total_tests,
            passed,
            failed,
            skipped,
            duration,
            test_results,
        }
    }

    /// Check if the suite execution was successful
    pub fn is_successful(&self) -> bool {
        self.failed == 0
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            100.0
        } else {
            (self.passed as f64 / self.total_tests as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_config() -> TestConfig {
        TestConfig {
            max_concurrency: 4,
            fail_fast: false,
            filter: None,
            test_timeout: Duration::from_secs(30),
            server_timeout: Duration::from_secs(10),
            retry_attempts: 2,
        }
    }

    fn create_sample_test_case() -> TestCase {
        TestCase {
            name: "test_tool_call".to_string(),
            description: Some("Test basic tool calling".to_string()),
            test_type: TestType::ToolCall {
                tool_name: "echo".to_string(),
            },
            parameters: json!({"message": "hello"}),
            expected: ExpectedResult {
                should_succeed: true,
                content_patterns: vec!["hello".to_string()],
                performance: Some(PerformanceExpectation {
                    max_response_time: Duration::from_millis(1000),
                    max_memory_usage: None,
                }),
            },
            timeout: None,
            retry_attempts: None,
        }
    }

    #[test]
    fn test_runner_creation() {
        let config = create_test_config();
        let runner = TestRunner::new(config);
        assert_eq!(runner.progress_tracker.total_tests, 0);
    }

    #[test]
    fn test_config_default() {
        let config = TestConfig::default();
        assert_eq!(config.max_concurrency, 4);
        assert!(!config.fail_fast);
        assert!(config.filter.is_none());
        assert_eq!(config.test_timeout, Duration::from_secs(30));
        assert_eq!(config.server_timeout, Duration::from_secs(10));
        assert_eq!(config.retry_attempts, 2);
    }

    #[test]
    fn test_result_validation_success() {
        let config = create_test_config();
        let runner = TestRunner::new(config);

        let response = json!({"content": "hello world"});
        let expected = ExpectedResult {
            should_succeed: true,
            content_patterns: vec!["hello".to_string()],
            performance: None,
        };

        let result = runner.validate_test_result(&response, &expected, Duration::from_millis(100));
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.passed);
        assert!(validation.error_message.is_none());
    }

    #[test]
    fn test_result_validation_content_failure() {
        let config = create_test_config();
        let runner = TestRunner::new(config);

        let response = json!({"content": "goodbye world"});
        let expected = ExpectedResult {
            should_succeed: true,
            content_patterns: vec!["hello".to_string()],
            performance: None,
        };

        let result = runner.validate_test_result(&response, &expected, Duration::from_millis(100));
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.passed);
        assert!(validation.error_message.is_some());
        assert!(validation.error_message.unwrap().contains("hello"));
    }

    #[test]
    fn test_result_validation_performance_failure() {
        let config = create_test_config();
        let runner = TestRunner::new(config);

        let response = json!({"content": "hello"});
        let expected = ExpectedResult {
            should_succeed: true,
            content_patterns: vec![],
            performance: Some(PerformanceExpectation {
                max_response_time: Duration::from_millis(100),
                max_memory_usage: None,
            }),
        };

        // Test with duration exceeding expectation
        let result = runner.validate_test_result(&response, &expected, Duration::from_millis(200));
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.passed);
        assert!(validation.error_message.is_some());
        assert!(validation.error_message.unwrap().contains("Response time"));
    }

    #[test]
    fn test_should_execute_test_with_filter() {
        let mut config = create_test_config();
        config.filter = Some("tool".to_string());
        let runner = TestRunner::new(config);

        let test_case = create_sample_test_case();
        assert!(runner.should_execute_test(&test_case)); // "test_tool_call" contains "tool"

        let mut other_test = test_case.clone();
        other_test.name = "health_check".to_string();
        assert!(!runner.should_execute_test(&other_test)); // "health_check" doesn't contain "tool"
    }

    #[test]
    fn test_should_execute_test_without_filter() {
        let config = create_test_config(); // No filter
        let runner = TestRunner::new(config);

        let test_case = create_sample_test_case();
        assert!(runner.should_execute_test(&test_case));
    }

    #[test]
    fn test_progress_tracker() {
        let mut tracker = ProgressTracker::new();

        // Initialize
        tracker.initialize(3);
        assert_eq!(tracker.total_tests, 3);
        assert_eq!(tracker.progress_percentage(), 0.0);

        // Record results
        let passed_result = TestResult {
            test_name: "test1".to_string(),
            suite_name: "suite".to_string(),
            status: TestStatus::Passed,
            error_message: None,
            start_time: chrono::Utc::now(),
            duration: Duration::from_millis(100),
            response_data: None,
            performance: PerformanceMetrics {
                response_time_ms: 100,
                memory_usage_bytes: None,
                retry_attempts: 0,
            },
        };

        tracker.record_result(&passed_result);
        assert_eq!(tracker.passed_tests, 1);
        assert_eq!(tracker.completed_tests, 1);
        assert!((tracker.progress_percentage() - 33.333333333333336).abs() < 0.0000001);

        // Record failed result
        let mut failed_result = passed_result.clone();
        failed_result.status = TestStatus::Failed;
        tracker.record_result(&failed_result);

        assert_eq!(tracker.failed_tests, 1);
        assert_eq!(tracker.completed_tests, 2);

        // Record skipped result
        let skipped_result = TestResult::skipped(
            "test3".to_string(),
            "suite".to_string(),
            "Filtered".to_string(),
        );
        tracker.record_result(&skipped_result);

        assert_eq!(tracker.skipped_tests, 1);
        assert_eq!(tracker.completed_tests, 3);
        assert_eq!(tracker.progress_percentage(), 100.0);
    }

    #[test]
    fn test_test_result_constructors() {
        let skipped = TestResult::skipped(
            "test1".to_string(),
            "suite".to_string(),
            "Filtered".to_string(),
        );
        assert_eq!(skipped.status, TestStatus::Skipped);
        assert_eq!(skipped.error_message, Some("Filtered".to_string()));

        let error = TestResult::error("test2".to_string(), "Connection failed".to_string());
        assert_eq!(error.status, TestStatus::Error);
        assert_eq!(error.error_message, Some("Connection failed".to_string()));

        let timeout = TestResult::timeout("test3".to_string());
        assert_eq!(timeout.status, TestStatus::Timeout);
        assert_eq!(
            timeout.error_message,
            Some("Test execution timed out".to_string())
        );
    }

    #[test]
    fn test_suite_result_creation() {
        let test_results = vec![
            TestResult::skipped(
                "test1".to_string(),
                "suite".to_string(),
                "Filtered".to_string(),
            ),
            TestResult::error("test2".to_string(), "Failed".to_string()),
            TestResult::timeout("test3".to_string()),
        ];

        // Set status to passed for one result
        let mut passed_result = test_results[0].clone();
        passed_result.status = TestStatus::Passed;

        let results = vec![
            passed_result,
            test_results[1].clone(),
            test_results[2].clone(),
        ];
        let suite_result = SuiteResult::from_test_results("test_suite".to_string(), results);

        assert_eq!(suite_result.suite_name, "test_suite");
        assert_eq!(suite_result.total_tests, 3);
        assert_eq!(suite_result.passed, 1);
        assert_eq!(suite_result.failed, 2); // error + timeout
        assert_eq!(suite_result.skipped, 0);
        assert!(!suite_result.is_successful());
        assert!((suite_result.success_rate() - 33.333333333333336).abs() < 0.0000001);
    }

    #[test]
    fn test_serialization() {
        let test_case = create_sample_test_case();
        let json = serde_json::to_string(&test_case).unwrap();
        let deserialized: TestCase = serde_json::from_str(&json).unwrap();
        assert_eq!(test_case.name, deserialized.name);
    }

    #[test]
    fn test_test_type_serialization() {
        let test_types = vec![
            TestType::ToolCall {
                tool_name: "echo".to_string(),
            },
            TestType::ResourceRead {
                resource_uri: "file://test.txt".to_string(),
            },
            TestType::CapabilityCheck,
            TestType::HealthCheck,
        ];

        for test_type in test_types {
            let json = serde_json::to_string(&test_type).unwrap();
            let deserialized: TestType = serde_json::from_str(&json).unwrap();

            match (&test_type, &deserialized) {
                (TestType::ToolCall { tool_name: t1 }, TestType::ToolCall { tool_name: t2 }) => {
                    assert_eq!(t1, t2);
                }
                (
                    TestType::ResourceRead { resource_uri: u1 },
                    TestType::ResourceRead { resource_uri: u2 },
                ) => {
                    assert_eq!(u1, u2);
                }
                (TestType::CapabilityCheck, TestType::CapabilityCheck) => {}
                (TestType::HealthCheck, TestType::HealthCheck) => {}
                _ => panic!("TestType serialization mismatch"),
            }
        }
    }

    #[test]
    fn test_test_status_equality() {
        assert_eq!(TestStatus::Passed, TestStatus::Passed);
        assert_eq!(TestStatus::Failed, TestStatus::Failed);
        assert_eq!(TestStatus::Skipped, TestStatus::Skipped);
        assert_eq!(TestStatus::Timeout, TestStatus::Timeout);
        assert_eq!(TestStatus::Error, TestStatus::Error);

        assert_ne!(TestStatus::Passed, TestStatus::Failed);
        assert_ne!(TestStatus::Error, TestStatus::Timeout);
    }
}
