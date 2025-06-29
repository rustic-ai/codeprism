//! Test execution engine for the CodePrism Test Harness

use crate::config::TestConfig;
use crate::types::{
    MemoryStats, ResponseTimePercentiles, TestCase, TestExecutionStats, TestResult, TestSuite,
    TestSuiteResult,
};
use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{debug, error, info};

/// Main test executor that manages test execution lifecycle
pub struct TestExecutor {
    config: TestConfig,
    concurrency_limiter: Arc<Semaphore>,
}

impl TestExecutor {
    /// Create a new test executor with the given configuration
    pub fn new(config: TestConfig) -> Self {
        let concurrency_limiter = Arc::new(Semaphore::new(config.global.max_global_concurrency));

        Self {
            config,
            concurrency_limiter,
        }
    }

    /// Execute all test suites in the configuration
    pub async fn execute_all_test_suites(&self) -> Result<Vec<TestSuiteResult>> {
        info!(
            "Starting test execution for {} test suites",
            self.config.test_suites.len()
        );

        let mut results = Vec::new();

        for test_suite in &self.config.test_suites {
            info!("Executing test suite: {}", test_suite.name);

            match self.execute_test_suite(test_suite).await {
                Ok(result) => {
                    info!(
                        "Test suite '{}' completed: {}/{} tests passed",
                        test_suite.name, result.stats.passed_tests, result.stats.total_tests
                    );
                    results.push(result);
                }
                Err(e) => {
                    error!("Test suite '{}' failed: {}", test_suite.name, e);

                    if self.config.global.fail_fast {
                        break;
                    }
                }
            }
        }

        info!(
            "Test execution completed. {} suites executed",
            results.len()
        );
        Ok(results)
    }

    /// Execute a single test suite
    pub async fn execute_test_suite(&self, test_suite: &TestSuite) -> Result<TestSuiteResult> {
        let start_time = Utc::now();

        let mut test_results = Vec::new();

        for test_case in &test_suite.test_cases {
            if !test_case.enabled {
                continue;
            }

            let project_path = self.resolve_project_path(test_case);

            match self
                .execute_single_test(test_case.clone(), project_path)
                .await
            {
                Ok(result) => {
                    debug!(
                        "Test '{}' completed: {}",
                        test_case.id,
                        if result.success { "PASS" } else { "FAIL" }
                    );
                    let success = result.success;
                    test_results.push(result);

                    if !success && self.config.global.fail_fast {
                        break;
                    }
                }
                Err(e) => {
                    error!("Test '{}' failed: {}", test_case.id, e);
                    test_results
                        .push(self.create_failed_test_result(test_case.clone(), e.to_string()));

                    if self.config.global.fail_fast {
                        break;
                    }
                }
            }
        }

        let end_time = Utc::now();
        let stats = self.calculate_execution_stats(&test_results);
        let suite_passed = test_results.iter().all(|r| r.success);

        Ok(TestSuiteResult {
            test_suite: test_suite.clone(),
            test_results,
            stats,
            start_time,
            end_time,
            suite_passed,
        })
    }

    /// Execute a single test case (simplified for initial implementation)
    async fn execute_single_test(
        &self,
        test_case: TestCase,
        _project_path: String,
    ) -> Result<TestResult> {
        let _permit = self.concurrency_limiter.acquire().await.unwrap();

        let start_time = Utc::now();
        let execution_start = Instant::now();

        debug!(
            "Executing test '{}' for tool '{}'",
            test_case.id, test_case.tool_name
        );

        // Simulate test execution
        tokio::time::sleep(Duration::from_millis(100)).await;

        let success = true; // Mock success for initial implementation
        let actual_response = Some(serde_json::json!({
            "result": {
                "status": "completed",
                "message": format!("Mock execution of {}", test_case.tool_name)
            }
        }));

        let end_time = Utc::now();
        let duration = execution_start.elapsed();

        Ok(TestResult {
            test_case,
            success,
            start_time,
            end_time,
            duration,
            memory_usage_mb: None,
            actual_response,
            validation_results: vec![],
            error_message: None,
            debug_info: HashMap::new(),
        })
    }

    fn resolve_project_path(&self, test_case: &TestCase) -> String {
        test_case
            .project_path
            .clone()
            .or_else(|| self.config.global.default_project_path.clone())
            .unwrap_or_else(|| "test-projects/python-sample".to_string())
    }

    fn create_failed_test_result(&self, test_case: TestCase, error_message: String) -> TestResult {
        let now = Utc::now();
        TestResult {
            test_case,
            success: false,
            start_time: now,
            end_time: now,
            duration: Duration::from_secs(0),
            memory_usage_mb: None,
            actual_response: None,
            validation_results: vec![],
            error_message: Some(error_message),
            debug_info: HashMap::new(),
        }
    }

    fn calculate_execution_stats(&self, test_results: &[TestResult]) -> TestExecutionStats {
        let total_tests = test_results.len();
        let passed_tests = test_results.iter().filter(|r| r.success).count();
        let failed_tests = total_tests - passed_tests;

        let total_duration = test_results.iter().map(|r| r.duration).sum::<Duration>();

        let average_duration = if total_tests > 0 {
            total_duration / total_tests as u32
        } else {
            Duration::from_secs(0)
        };

        TestExecutionStats {
            total_tests,
            passed_tests,
            failed_tests,
            skipped_tests: 0,
            total_duration,
            average_duration,
            memory_stats: MemoryStats {
                average_mb: 0.0,
                peak_mb: 0.0,
                min_mb: 0.0,
            },
            performance_percentiles: ResponseTimePercentiles {
                p50_ms: Some(0),
                p90_ms: Some(0),
                p95_ms: Some(0),
                p99_ms: Some(0),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_executor_creation() {
        let config = TestConfig::default_for_tests();
        let executor = TestExecutor::new(config);

        assert_eq!(executor.config.global.max_global_concurrency, 4);
    }
}
