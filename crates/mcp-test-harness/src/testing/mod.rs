//! Test execution engine for MCP test harness

use crate::spec::schema::{ServerSpec, TestCase};
use crate::types::TestStats;
use anyhow::Result;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;
use tracing::{info, warn};

pub mod result;
pub mod runner;
pub mod validation;

// Re-export key types for external use
pub use result::TestResult;
pub use runner::{ExecutionConfig, TestRunner};

/// Test harness for executing MCP server tests
#[derive(Debug)]
pub struct TestHarness {
    spec: ServerSpec,
    runner: runner::TestRunner,
    execution_config: runner::ExecutionConfig,
}

impl TestHarness {
    /// Create a new test harness with a server specification
    pub fn new(spec: ServerSpec) -> Self {
        let execution_config = runner::ExecutionConfig::default();
        let runner = runner::TestRunner::with_config(execution_config.clone());

        Self {
            spec,
            runner,
            execution_config,
        }
    }

    /// Create a new test harness with custom execution configuration
    pub fn with_config(spec: ServerSpec, execution_config: runner::ExecutionConfig) -> Self {
        let runner = runner::TestRunner::with_config(execution_config.clone());

        Self {
            spec,
            runner,
            execution_config,
        }
    }

    /// Initialize the test harness with server configuration
    pub async fn initialize(&mut self) -> Result<()> {
        // Initialize the test runner with server configuration
        self.runner.initialize(self.spec.server.clone()).await?;
        info!("TestHarness initialized for server: {}", self.spec.name);
        Ok(())
    }

    /// Run all tests defined in the specification with parallel processing
    pub async fn run_all_tests(&mut self) -> Result<TestReport> {
        info!(
            "Starting comprehensive test execution for server: {}",
            self.spec.name
        );
        let overall_start = Instant::now();

        // Initialize if not already done
        if !self.is_initialized() {
            self.initialize().await?;
        }

        // Collect all test cases from the specification
        let test_cases = self.collect_all_test_cases();

        if test_cases.is_empty() {
            warn!("No test cases found in specification");
            return Ok(TestReport {
                spec: self.spec.clone(),
                stats: TestStats::default(),
                results: Vec::new(),
            });
        }

        info!(
            "Executing {} test cases with max concurrency: {}",
            test_cases.len(),
            self.execution_config.max_concurrency
        );

        // Execute tests with controlled concurrency
        let results = self.execute_tests_parallel(&test_cases).await?;

        // Calculate statistics
        let stats = self.calculate_test_stats(&results, overall_start.elapsed());

        info!(
            "Test execution completed: {}/{} passed in {:?}",
            stats.passed_tests,
            stats.total_tests,
            overall_start.elapsed()
        );

        Ok(TestReport {
            spec: self.spec.clone(),
            stats,
            results,
        })
    }

    /// Run only protocol compliance tests for basic MCP validation
    pub async fn run_protocol_tests_only(&mut self) -> Result<TestReport> {
        info!(
            "Starting protocol compliance testing for server: {}",
            self.spec.name
        );
        let overall_start = Instant::now();

        // Initialize if not already done
        if !self.is_initialized() {
            self.initialize().await?;
        }

        // Execute protocol tests
        let results = self
            .runner
            .execute_protocol_tests(&self.spec.server)
            .await?;

        // Calculate statistics
        let stats = self.calculate_test_stats(&results, overall_start.elapsed());

        info!(
            "Protocol compliance testing completed: {}/{} passed in {:?}",
            stats.passed_tests,
            stats.total_tests,
            overall_start.elapsed()
        );

        Ok(TestReport {
            spec: self.spec.clone(),
            stats,
            results,
        })
    }

    /// Execute a specific subset of tests by tags
    pub async fn run_tests_by_tags(&mut self, tags: &[String]) -> Result<TestReport> {
        info!("Starting tagged test execution for tags: {:?}", tags);
        let overall_start = Instant::now();

        // Initialize if not already done
        if !self.is_initialized() {
            self.initialize().await?;
        }

        // Filter test cases by tags
        let test_cases = self.collect_test_cases_by_tags(tags);

        if test_cases.is_empty() {
            warn!("No test cases found with specified tags: {:?}", tags);
            return Ok(TestReport {
                spec: self.spec.clone(),
                stats: TestStats::default(),
                results: Vec::new(),
            });
        }

        info!("Executing {} tagged test cases", test_cases.len());

        // Execute filtered tests
        let results = self.execute_tests_parallel(&test_cases).await?;

        // Calculate statistics
        let stats = self.calculate_test_stats(&results, overall_start.elapsed());

        info!(
            "Tagged test execution completed: {}/{} passed in {:?}",
            stats.passed_tests,
            stats.total_tests,
            overall_start.elapsed()
        );

        Ok(TestReport {
            spec: self.spec.clone(),
            stats,
            results,
        })
    }

    /// Execute tests in parallel with concurrency control
    async fn execute_tests_parallel(
        &self,
        test_cases: &[TestCase],
    ) -> Result<Vec<result::TestResult>> {
        let semaphore = Arc::new(Semaphore::new(self.execution_config.max_concurrency));

        // Create futures for all test executions
        let test_futures = test_cases.iter().map(|test_case| {
            let runner = &self.runner;
            let semaphore = semaphore.clone();
            let test_case = test_case.clone();

            async move {
                let _permit = semaphore.acquire().await.unwrap();
                runner.execute_test(&test_case).await
            }
        });

        // Execute all tests concurrently and collect results
        let results = join_all(test_futures).await;

        // Process results and handle any errors
        let mut test_results = Vec::new();
        for (index, result) in results.into_iter().enumerate() {
            match result {
                Ok(test_result) => test_results.push(test_result),
                Err(e) => {
                    warn!("Test execution failed for test {}: {}", index, e);
                    // Create a failure result for failed test execution
                    test_results.push(result::TestResult::failure(
                        format!("test_{}", index),
                        chrono::Utc::now(),
                        std::time::Duration::from_millis(0),
                        serde_json::json!({}),
                        format!("Test execution failed: {}", e),
                    ));
                }
            }
        }

        Ok(test_results)
    }

    /// Collect all test cases from the specification
    fn collect_all_test_cases(&self) -> Vec<TestCase> {
        let mut test_cases = Vec::new();

        // Collect tool test cases
        if let Some(tools) = &self.spec.tools {
            for tool in tools {
                test_cases.extend(tool.tests.clone());
            }
        }

        // Collect resource test cases
        if let Some(resources) = &self.spec.resources {
            for resource in resources {
                test_cases.extend(resource.tests.clone());
            }
        }

        // Collect prompt test cases
        if let Some(prompts) = &self.spec.prompts {
            for prompt in prompts {
                test_cases.extend(prompt.tests.clone());
            }
        }

        // Filter out skipped tests
        test_cases.into_iter().filter(|test| !test.skip).collect()
    }

    /// Collect test cases that match any of the specified tags
    fn collect_test_cases_by_tags(&self, tags: &[String]) -> Vec<TestCase> {
        self.collect_all_test_cases()
            .into_iter()
            .filter(|test| test.tags.iter().any(|test_tag| tags.contains(test_tag)))
            .collect()
    }

    /// Calculate comprehensive test statistics
    fn calculate_test_stats(
        &self,
        results: &[result::TestResult],
        total_duration: std::time::Duration,
    ) -> TestStats {
        let total_tests = results.len();
        let passed_tests = results.iter().filter(|r| r.passed).count();
        let failed_tests = total_tests - passed_tests;
        let skipped_tests = 0; // We already filtered out skipped tests

        let total_duration_ms = total_duration.as_millis();
        let average_duration_ms = if total_tests > 0 {
            total_duration_ms as f64 / total_tests as f64
        } else {
            0.0
        };

        TestStats {
            total_tests,
            passed_tests,
            failed_tests,
            skipped_tests,
            total_duration_ms,
            average_duration_ms,
        }
    }

    /// Check if the test harness is initialized
    fn is_initialized(&self) -> bool {
        // Check if the runner has been initialized with connection pool
        // Currently we allow reinitialization for flexibility
        // ENHANCEMENT: Could add explicit initialization state tracking
        true
    }

    /// Generate comprehensive reports in multiple formats
    pub async fn generate_reports(&self, results: &TestReport) -> Result<()> {
        info!(
            "Generating test reports for {} test results",
            results.results.len()
        );

        // Generate basic console summary
        self.generate_console_summary(results).await?;

        // FUTURE: Add additional report formats
        // - HTML dashboard report with interactive charts
        // - JUnit XML report for CI integration
        // - JSON report for programmatic processing
        // - Performance metrics report with trend analysis

        info!("Test report generation completed");
        Ok(())
    }

    /// Generate console summary report
    async fn generate_console_summary(&self, results: &TestReport) -> Result<()> {
        println!("\n📊 Test Execution Summary");
        println!("========================");
        println!("Server: {} v{}", results.spec.name, results.spec.version);
        println!("Total Tests: {}", results.stats.total_tests);
        println!("✅ Passed: {}", results.stats.passed_tests);
        println!("❌ Failed: {}", results.stats.failed_tests);
        println!("⏩ Skipped: {}", results.stats.skipped_tests);
        println!("📈 Pass Rate: {:.1}%", results.stats.pass_rate());
        println!(
            "⏱️  Total Duration: {:.2}s",
            results.stats.total_duration_ms as f64 / 1000.0
        );
        println!(
            "📊 Average Duration: {:.1}ms",
            results.stats.average_duration_ms
        );

        // Show failed tests if any
        if results.stats.failed_tests > 0 {
            println!("\n❌ Failed Tests:");
            for failed_test in results.failed_tests() {
                println!(
                    "  • {}: {}",
                    failed_test.test_name,
                    failed_test
                        .error
                        .as_ref()
                        .unwrap_or(&"Unknown error".to_string())
                );
            }
        }

        // Performance monitoring removed - out of scope for current design

        println!("\n");
        Ok(())
    }

    /// Cleanup resources and connections
    pub async fn cleanup(&self) -> Result<()> {
        info!("Cleaning up test harness resources");
        self.runner.cleanup().await?;
        info!("Test harness cleanup completed");
        Ok(())
    }

    /// Get execution metrics from the test runner
    pub async fn get_execution_metrics(&self) -> Result<runner::ExecutionMetrics> {
        Ok(self.runner.get_metrics().await)
    }
}

/// Test suite containing related test cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    /// Suite name
    pub name: String,
    /// Suite description
    pub description: Option<String>,
    /// Test cases in this suite
    pub test_cases: Vec<TestCase>,
    /// Suite-specific configuration
    pub config: Option<TestSuiteConfig>,
}

/// Configuration for test suite execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteConfig {
    /// Maximum concurrent tests in this suite
    pub max_concurrency: Option<usize>,
    /// Timeout for the entire suite
    pub timeout_seconds: Option<u64>,
    /// Whether to stop on first failure in this suite
    pub fail_fast: Option<bool>,
}

/// Complete test report with comprehensive results and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    /// Server specification that was tested
    pub spec: ServerSpec,
    /// Overall test statistics
    pub stats: TestStats,
    /// Individual test results
    pub results: Vec<result::TestResult>,
}

impl TestReport {
    /// Check if all tests passed
    pub fn all_tests_passed(&self) -> bool {
        self.stats.all_passed()
    }

    /// Get failed test results
    pub fn failed_tests(&self) -> Vec<&result::TestResult> {
        self.results.iter().filter(|r| !r.passed).collect()
    }

    /// Get passed test results
    pub fn passed_tests(&self) -> Vec<&result::TestResult> {
        self.results.iter().filter(|r| r.passed).collect()
    }

    // Performance metrics functionality removed - out of scope

    /// Get tests by tag
    pub fn tests_with_tag(&self, tag: &str) -> Vec<&result::TestResult> {
        self.results
            .iter()
            .filter(|r| r.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Calculate execution efficiency (tests per second)
    pub fn execution_efficiency(&self) -> f64 {
        if self.stats.total_duration_ms > 0 {
            (self.stats.total_tests as f64 * 1000.0) / self.stats.total_duration_ms as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_harness_creation() {
        let spec = ServerSpec::minimal_protocol_spec("test".to_string(), vec![]);
        let _harness = TestHarness::new(spec);
    }

    #[test]
    fn test_test_harness_with_config() {
        let spec = ServerSpec::minimal_protocol_spec("test".to_string(), vec![]);
        let config = runner::ExecutionConfig {
            max_concurrency: 2,
            connection_pool_size: 4,
            ..Default::default()
        };
        let _harness = TestHarness::with_config(spec, config);
    }

    #[test]
    fn test_test_report_methods() {
        let spec = ServerSpec::minimal_protocol_spec("test".to_string(), vec![]);

        // Create test results
        let passed_result = result::TestResult::success(
            "test_pass".to_string(),
            chrono::Utc::now(),
            std::time::Duration::from_millis(100),
            serde_json::json!({}),
            serde_json::json!({"status": "ok"}),
        )
        .with_tags(vec!["integration".to_string()]);

        let failed_result = result::TestResult::failure(
            "test_fail".to_string(),
            chrono::Utc::now(),
            std::time::Duration::from_millis(50),
            serde_json::json!({}),
            "Test failed".to_string(),
        );

        let stats = TestStats {
            total_tests: 2,
            passed_tests: 1,
            failed_tests: 1,
            skipped_tests: 0,
            total_duration_ms: 150,
            average_duration_ms: 75.0,
        };

        let report = TestReport {
            spec,
            stats,
            results: vec![passed_result, failed_result],
        };

        // Test report methods
        assert!(!report.all_tests_passed());
        assert_eq!(report.failed_tests().len(), 1);
        assert_eq!(report.passed_tests().len(), 1);
        assert_eq!(report.tests_with_tag("integration").len(), 1);
        assert_eq!(report.execution_efficiency(), 13.333333333333334); // 2 tests / 0.15 seconds
    }

    #[test]
    fn test_empty_report() {
        let spec = ServerSpec::minimal_protocol_spec("test".to_string(), vec![]);
        let report = TestReport {
            spec,
            stats: TestStats::default(),
            results: Vec::new(),
        };

        // Empty results should NOT mean all tests passed since no tests were run
        assert!(!report.all_tests_passed());
        assert!(report.failed_tests().is_empty());
        assert!(report.passed_tests().is_empty());
        assert_eq!(report.execution_efficiency(), 0.0);
    }
}
