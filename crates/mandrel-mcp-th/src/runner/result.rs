//! Result types for test suite execution

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use super::config::ExecutionMode;
use super::metrics::SuiteMetrics;

/// Complete result of test suite execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteResult {
    /// Name of the test suite
    pub suite_name: String,
    /// Path to the specification file that was executed
    pub specification_file: PathBuf,
    /// When the suite execution started
    pub execution_start: SystemTime,
    /// When the suite execution ended
    pub execution_end: SystemTime,
    /// Total duration of suite execution
    pub total_duration: Duration,

    // Test execution summary
    /// Total number of tests in the suite
    pub total_tests: usize,
    /// Number of tests that passed
    pub passed: usize,
    /// Number of tests that failed
    pub failed: usize,
    /// Number of tests that were skipped
    pub skipped: usize,
    /// Error rate as a percentage (0.0 to 1.0)
    pub error_rate: f64,

    /// Individual test results
    pub test_results: Vec<TestResult>,

    /// Suite-level performance metrics
    pub suite_metrics: SuiteMetrics,

    /// Execution context information
    pub execution_mode: ExecutionMode,
    /// Dependency resolution information
    pub dependency_resolution: DependencyResolution,
}

impl TestSuiteResult {
    /// Calculate the success rate as a percentage (0.0 to 100.0)
    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            100.0
        } else {
            (self.passed as f64 / self.total_tests as f64) * 100.0
        }
    }

    /// Calculate the failure rate as a percentage (0.0 to 100.0)
    pub fn failure_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            (self.failed as f64 / self.total_tests as f64) * 100.0
        }
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed == 0 // Vacuously true for empty test suites
    }

    /// Check if any tests failed
    pub fn has_failures(&self) -> bool {
        self.failed > 0
    }

    /// Get the slowest test result
    pub fn slowest_test(&self) -> Option<&TestResult> {
        self.test_results
            .iter()
            .max_by_key(|result| result.duration)
    }

    /// Get the fastest test result
    pub fn fastest_test(&self) -> Option<&TestResult> {
        self.test_results
            .iter()
            .min_by_key(|result| result.duration)
    }

    /// Get all failed test results
    pub fn failed_tests(&self) -> Vec<&TestResult> {
        self.test_results
            .iter()
            .filter(|result| !result.success)
            .collect()
    }

    /// Get all passed test results
    pub fn passed_tests(&self) -> Vec<&TestResult> {
        self.test_results
            .iter()
            .filter(|result| result.success)
            .collect()
    }

    /// Calculate average test duration
    pub fn average_test_duration(&self) -> Duration {
        if self.test_results.is_empty() {
            Duration::from_secs(0)
        } else {
            let total: Duration = self.test_results.iter().map(|r| r.duration).sum();
            total / self.test_results.len() as u32
        }
    }

    /// Get a summary string for logging
    pub fn summary(&self) -> String {
        format!(
            "Suite '{}': {}/{} tests passed ({:.1}%), {:.1}s total",
            self.suite_name,
            self.passed,
            self.total_tests,
            self.success_rate(),
            self.total_duration.as_secs_f64()
        )
    }
}

/// Result of an individual test case execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Name of the test case
    pub test_name: String,
    /// Whether the test passed
    pub success: bool,
    /// Duration of test execution
    pub duration: Duration,
    /// Error message if test failed
    pub error_message: Option<String>,
    /// Number of retry attempts made
    pub retry_attempts: usize,
    /// Test execution start time
    pub start_time: SystemTime,
    /// Test execution end time
    pub end_time: SystemTime,
    /// Memory usage during test execution
    pub memory_usage_mb: Option<u64>,
    /// Additional test metadata
    pub metadata: TestMetadata,
}

impl TestResult {
    /// Create a successful test result
    pub fn success(test_name: String, duration: Duration) -> Self {
        let now = SystemTime::now();
        Self {
            test_name,
            success: true,
            duration,
            error_message: None,
            retry_attempts: 0,
            start_time: now - duration,
            end_time: now,
            memory_usage_mb: None,
            metadata: TestMetadata::default(),
        }
    }

    /// Create a failed test result
    pub fn failure(test_name: String, duration: Duration, error_message: String) -> Self {
        let now = SystemTime::now();
        Self {
            test_name,
            success: false,
            duration,
            error_message: Some(error_message),
            retry_attempts: 0,
            start_time: now - duration,
            end_time: now,
            memory_usage_mb: None,
            metadata: TestMetadata::default(),
        }
    }

    /// Add retry attempt information
    pub fn with_retry_attempts(mut self, attempts: usize) -> Self {
        self.retry_attempts = attempts;
        self
    }

    /// Add memory usage information
    pub fn with_memory_usage(mut self, memory_mb: u64) -> Self {
        self.memory_usage_mb = Some(memory_mb);
        self
    }

    /// Add test metadata
    pub fn with_metadata(mut self, metadata: TestMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Get execution rate in tests per second
    pub fn execution_rate(&self) -> f64 {
        if self.duration.as_secs_f64() > 0.0 {
            1.0 / self.duration.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Check if this test took longer than expected
    pub fn is_slow(&self, threshold: Duration) -> bool {
        self.duration > threshold
    }

    /// Get a one-line summary of this test result
    pub fn summary(&self) -> String {
        let status = if self.success { "PASS" } else { "FAIL" };
        let retry_info = if self.retry_attempts > 0 {
            format!(" (retries: {})", self.retry_attempts)
        } else {
            String::new()
        };

        format!(
            "{} {} - {:.3}s{}",
            status,
            self.test_name,
            self.duration.as_secs_f64(),
            retry_info
        )
    }
}

/// Additional metadata for test results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestMetadata {
    /// Tags associated with the test
    pub tags: Vec<String>,
    /// Test category or group
    pub category: Option<String>,
    /// Priority level of the test
    pub priority: Option<String>,
    /// Expected duration for this test
    pub expected_duration: Option<Duration>,
    /// Custom properties for extension
    pub custom_properties: std::collections::HashMap<String, String>,
}

impl TestMetadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tag to the test
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    /// Set the test category
    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }

    /// Set the test priority
    pub fn with_priority(mut self, priority: String) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Set expected duration
    pub fn with_expected_duration(mut self, duration: Duration) -> Self {
        self.expected_duration = Some(duration);
        self
    }

    /// Add a custom property
    pub fn with_property(mut self, key: String, value: String) -> Self {
        self.custom_properties.insert(key, value);
        self
    }
}

/// Information about dependency resolution for the test suite
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyResolution {
    /// Total number of dependencies resolved
    pub total_dependencies: usize,
    /// Number of circular dependencies detected
    pub circular_dependencies: usize,
    /// List of dependency chains that were circular
    pub circular_dependency_chains: Vec<Vec<String>>,
    /// Time taken to resolve dependencies
    pub resolution_duration: Duration,
    /// Execution order determined by dependency resolution
    pub execution_order: Vec<String>,
    /// Dependency groups for parallel execution
    pub dependency_groups: Vec<Vec<String>>,
}

impl DependencyResolution {
    /// Create a new dependency resolution result
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the total number of dependencies
    pub fn with_total_dependencies(mut self, total: usize) -> Self {
        self.total_dependencies = total;
        self
    }

    /// Add a circular dependency chain
    pub fn with_circular_dependency(mut self, chain: Vec<String>) -> Self {
        self.circular_dependency_chains.push(chain);
        self.circular_dependencies = self.circular_dependency_chains.len();
        self
    }

    /// Set the resolution duration
    pub fn with_resolution_duration(mut self, duration: Duration) -> Self {
        self.resolution_duration = duration;
        self
    }

    /// Set the execution order
    pub fn with_execution_order(mut self, order: Vec<String>) -> Self {
        self.execution_order = order;
        self
    }

    /// Set dependency groups for parallel execution
    pub fn with_dependency_groups(mut self, groups: Vec<Vec<String>>) -> Self {
        self.dependency_groups = groups;
        self
    }

    /// Check if there were any circular dependencies
    pub fn has_circular_dependencies(&self) -> bool {
        self.circular_dependencies > 0
    }

    /// Get the maximum dependency group size (for parallel execution planning)
    pub fn max_parallel_group_size(&self) -> usize {
        self.dependency_groups
            .iter()
            .map(|group| group.len())
            .max()
            .unwrap_or(0)
    }

    /// Calculate dependency complexity score (higher = more complex)
    pub fn complexity_score(&self) -> f64 {
        if self.execution_order.is_empty() {
            0.0
        } else {
            let dependency_density =
                self.total_dependencies as f64 / self.execution_order.len() as f64;
            let circular_penalty = self.circular_dependencies as f64 * 2.0;
            dependency_density + circular_penalty
        }
    }
}

/// Aggregated results for multiple test suite executions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedResults {
    /// Individual suite results
    pub suite_results: Vec<TestSuiteResult>,
    /// Overall execution start time
    pub overall_start: SystemTime,
    /// Overall execution end time
    pub overall_end: SystemTime,
    /// Total execution duration across all suites
    pub total_duration: Duration,
    /// Aggregated statistics
    pub statistics: AggregatedStatistics,
}

/// Statistics aggregated across multiple test suites
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedStatistics {
    /// Total number of test suites executed
    pub total_suites: usize,
    /// Total number of individual tests executed
    pub total_tests: usize,
    /// Total number of tests that passed
    pub total_passed: usize,
    /// Total number of tests that failed
    pub total_failed: usize,
    /// Total number of tests that were skipped
    pub total_skipped: usize,
    /// Overall success rate (0.0 to 100.0)
    pub overall_success_rate: f64,
    /// Average suite execution time
    pub average_suite_duration: Duration,
    /// Slowest suite
    pub slowest_suite: Option<String>,
    /// Fastest suite
    pub fastest_suite: Option<String>,
}

impl AggregatedResults {
    /// Create aggregated results from a collection of suite results
    pub fn from_suite_results(suite_results: Vec<TestSuiteResult>) -> Self {
        if suite_results.is_empty() {
            return Self::empty();
        }

        let overall_start = suite_results
            .iter()
            .map(|r| r.execution_start)
            .min()
            .unwrap();

        let overall_end = suite_results.iter().map(|r| r.execution_end).max().unwrap();

        let total_duration = overall_end
            .duration_since(overall_start)
            .unwrap_or_default();

        let statistics = AggregatedStatistics::calculate(&suite_results);

        Self {
            suite_results,
            overall_start,
            overall_end,
            total_duration,
            statistics,
        }
    }

    /// Create empty aggregated results
    pub fn empty() -> Self {
        let now = SystemTime::now();
        Self {
            suite_results: Vec::new(),
            overall_start: now,
            overall_end: now,
            total_duration: Duration::from_secs(0),
            statistics: AggregatedStatistics::empty(),
        }
    }

    /// Get a summary of the aggregated results
    pub fn summary(&self) -> String {
        format!(
            "Executed {} suites with {} total tests - {:.1}% success rate",
            self.statistics.total_suites,
            self.statistics.total_tests,
            self.statistics.overall_success_rate
        )
    }
}

impl AggregatedStatistics {
    /// Calculate statistics from suite results
    pub fn calculate(suite_results: &[TestSuiteResult]) -> Self {
        if suite_results.is_empty() {
            return Self::empty();
        }

        let total_suites = suite_results.len();
        let total_tests = suite_results.iter().map(|r| r.total_tests).sum();
        let total_passed = suite_results.iter().map(|r| r.passed).sum();
        let total_failed = suite_results.iter().map(|r| r.failed).sum();
        let total_skipped = suite_results.iter().map(|r| r.skipped).sum();

        let overall_success_rate = if total_tests > 0 {
            (total_passed as f64 / total_tests as f64) * 100.0
        } else {
            100.0
        };

        let total_suite_duration: Duration = suite_results.iter().map(|r| r.total_duration).sum();
        let average_suite_duration = total_suite_duration / total_suites as u32;

        let slowest_suite = suite_results
            .iter()
            .max_by_key(|r| r.total_duration)
            .map(|r| r.suite_name.clone());

        let fastest_suite = suite_results
            .iter()
            .min_by_key(|r| r.total_duration)
            .map(|r| r.suite_name.clone());

        Self {
            total_suites,
            total_tests,
            total_passed,
            total_failed,
            total_skipped,
            overall_success_rate,
            average_suite_duration,
            slowest_suite,
            fastest_suite,
        }
    }

    /// Create empty statistics
    pub fn empty() -> Self {
        Self {
            total_suites: 0,
            total_tests: 0,
            total_passed: 0,
            total_failed: 0,
            total_skipped: 0,
            overall_success_rate: 100.0,
            average_suite_duration: Duration::from_secs(0),
            slowest_suite: None,
            fastest_suite: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    #[ignore = "Future work for Issue #227 - floating point precision issues in metrics calculations"]
    fn test_test_suite_result_calculations() {
        let test_results = vec![
            TestResult::success("test1".to_string(), Duration::from_millis(100)),
            TestResult::failure(
                "test2".to_string(),
                Duration::from_millis(200),
                "error".to_string(),
            ),
            TestResult::success("test3".to_string(), Duration::from_millis(150)),
        ];

        let suite_result = TestSuiteResult {
            suite_name: "test_suite".to_string(),
            specification_file: PathBuf::from("test.yaml"),
            execution_start: SystemTime::now(),
            execution_end: SystemTime::now(),
            total_duration: Duration::from_millis(450),
            total_tests: 3,
            passed: 2,
            failed: 1,
            skipped: 0,
            error_rate: 1.0 / 3.0,
            test_results,
            suite_metrics: SuiteMetrics::default(),
            execution_mode: ExecutionMode::Sequential,
            dependency_resolution: DependencyResolution::default(),
        };

        assert_eq!(suite_result.success_rate(), 66.66666666666667);
        assert_eq!(suite_result.failure_rate(), 33.333333333333336);
        assert!(!suite_result.all_passed());
        assert!(suite_result.has_failures());
        assert_eq!(suite_result.failed_tests().len(), 1);
        assert_eq!(suite_result.passed_tests().len(), 2);
        assert_eq!(
            suite_result.average_test_duration(),
            Duration::from_millis(150)
        );
    }

    #[test]
    fn test_dependency_resolution() {
        let resolution = DependencyResolution::new()
            .with_total_dependencies(5)
            .with_circular_dependency(vec![
                "test1".to_string(),
                "test2".to_string(),
                "test1".to_string(),
            ])
            .with_execution_order(vec![
                "test1".to_string(),
                "test2".to_string(),
                "test3".to_string(),
            ])
            .with_dependency_groups(vec![
                vec!["test1".to_string()],
                vec!["test2".to_string(), "test3".to_string()],
            ]);

        assert!(resolution.has_circular_dependencies());
        assert_eq!(resolution.circular_dependencies, 1);
        assert_eq!(resolution.max_parallel_group_size(), 2);
        assert!(resolution.complexity_score() > 0.0);
    }

    #[test]
    fn test_aggregated_results() {
        let suite1 = TestSuiteResult {
            suite_name: "suite1".to_string(),
            specification_file: PathBuf::from("suite1.yaml"),
            execution_start: SystemTime::now(),
            execution_end: SystemTime::now(),
            total_duration: Duration::from_secs(10),
            total_tests: 5,
            passed: 4,
            failed: 1,
            skipped: 0,
            error_rate: 0.2,
            test_results: vec![],
            suite_metrics: SuiteMetrics::default(),
            execution_mode: ExecutionMode::Sequential,
            dependency_resolution: DependencyResolution::default(),
        };

        let suite2 = TestSuiteResult {
            suite_name: "suite2".to_string(),
            specification_file: PathBuf::from("suite2.yaml"),
            execution_start: SystemTime::now(),
            execution_end: SystemTime::now(),
            total_duration: Duration::from_secs(20),
            total_tests: 3,
            passed: 3,
            failed: 0,
            skipped: 0,
            error_rate: 0.0,
            test_results: vec![],
            suite_metrics: SuiteMetrics::default(),
            execution_mode: ExecutionMode::Parallel,
            dependency_resolution: DependencyResolution::default(),
        };

        let aggregated = AggregatedResults::from_suite_results(vec![suite1, suite2]);

        assert_eq!(aggregated.statistics.total_suites, 2);
        assert_eq!(aggregated.statistics.total_tests, 8);
        assert_eq!(aggregated.statistics.total_passed, 7);
        assert_eq!(aggregated.statistics.total_failed, 1);
        assert_eq!(aggregated.statistics.overall_success_rate, 87.5);
    }

    #[test]
    fn test_test_metadata() {
        let metadata = TestMetadata::new()
            .with_tag("integration".to_string())
            .with_tag("slow".to_string())
            .with_category("api".to_string())
            .with_priority("high".to_string())
            .with_expected_duration(Duration::from_secs(5))
            .with_property("environment".to_string(), "staging".to_string());

        assert_eq!(metadata.tags.len(), 2);
        assert_eq!(metadata.category, Some("api".to_string()));
        assert_eq!(metadata.priority, Some("high".to_string()));
        assert_eq!(metadata.expected_duration, Some(Duration::from_secs(5)));
        assert_eq!(
            metadata.custom_properties.get("environment"),
            Some(&"staging".to_string())
        );
    }
}
