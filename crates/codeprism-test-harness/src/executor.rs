//! Test execution engine for the CodePrism Test Harness

use crate::config::TestConfig;
use crate::performance::{PerformanceConfig, PerformanceMonitor};
use crate::reporting::{ReportGenerator, ReportFormat};
use crate::script::{SandboxConfig, ScriptExecutor};
use crate::types::{
    JsonPathPattern, MemoryStats, PatternValidation, ResponseTimePercentiles, TestCase,
    TestExecutionStats, TestResult, TestSuite, TestSuiteResult, ValidationResult,
};
use anyhow::Result;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

/// Main test executor that manages test execution lifecycle
pub struct TestExecutor {
    config: TestConfig,
    concurrency_limiter: Arc<Semaphore>,
    script_executor: ScriptExecutor,
    performance_monitor: PerformanceMonitor,
}

impl TestExecutor {
    /// Create a new test executor with the given configuration
    pub fn new(config: TestConfig) -> Self {
        let concurrency_limiter = Arc::new(Semaphore::new(config.global.max_global_concurrency));

        // Create script executor with default sandbox configuration
        let sandbox_config = SandboxConfig::default();
        let script_executor = ScriptExecutor::new(None, sandbox_config);

        // Create performance monitor
        let performance_config = PerformanceConfig::default();
        let performance_monitor = PerformanceMonitor::new(performance_config);

        Self {
            config,
            concurrency_limiter,
            script_executor,
            performance_monitor,
        }
    }

    /// Initialize the test executor
    pub async fn initialize(&mut self) -> Result<()> {
        self.performance_monitor.initialize().await?;
        info!("Test executor initialized with performance monitoring");
        Ok(())
    }

    /// Execute all test suites in the configuration
    pub async fn execute_all_test_suites(&mut self) -> Result<Vec<TestSuiteResult>> {
        info!(
            "Starting test execution for {} test suites",
            self.config.test_suites.len()
        );

        let mut results = Vec::new();

        // Clone the test suites to avoid borrowing conflicts
        let test_suites = self.config.test_suites.clone();
        let fail_fast = self.config.global.fail_fast;

        for test_suite in &test_suites {
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

                    if fail_fast {
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
    pub async fn execute_test_suite(&mut self, test_suite: &TestSuite) -> Result<TestSuiteResult> {
        let start_time = Utc::now();

        let mut test_results = Vec::new();

        let fail_fast = self.config.global.fail_fast;

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

                    if !success && fail_fast {
                        break;
                    }
                }
                Err(e) => {
                    error!("Test '{}' failed: {}", test_case.id, e);
                    test_results
                        .push(self.create_failed_test_result(test_case.clone(), e.to_string()));

                    if fail_fast {
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

    /// Execute a single test case with comprehensive validation
    async fn execute_single_test(
        &mut self,
        test_case: TestCase,
        project_path: String,
    ) -> Result<TestResult> {
        let _permit = self.concurrency_limiter.acquire().await.unwrap();

        let start_time = Utc::now();
        let execution_start = Instant::now();

        debug!(
            "Executing test '{}' for tool '{}'",
            test_case.id, test_case.tool_name
        );

        // Start performance monitoring
        self.performance_monitor
            .start_test_monitoring(test_case.id.clone(), test_case.tool_name.clone());

        // Execute the MCP tool
        let actual_response = self.execute_mcp_tool(&test_case, &project_path).await?;

        // Update performance metrics with response size
        let response_size = serde_json::to_string(&actual_response)
            .map(|s| s.len() as u64)
            .unwrap_or(0);

        self.performance_monitor.update_metrics(|metrics| {
            metrics.response_size_bytes = response_size;
            // Simulate some memory and CPU metrics (in real implementation, these would be collected)
            metrics.peak_memory_mb = 64.0 + (response_size as f64 / 10000.0);
            metrics.cpu_usage_percent = 30.0 + (response_size as f64 / 100000.0);
        });

        // Validate the response against expected patterns
        let validation_results = self.validate_response(&test_case, &actual_response).await?;

        // Determine overall success
        let success = validation_results.iter().all(|v| v.passed);

        // Finish performance monitoring
        let performance_result = self
            .performance_monitor
            .finish_test_monitoring("v1.0.0".to_string())
            .await?;

        let end_time = Utc::now();
        let duration = execution_start.elapsed();

        let error_message = if !success {
            Some(format!(
                "Validation failed: {}",
                validation_results
                    .iter()
                    .filter(|v| !v.passed)
                    .map(|v| v.error_message.as_deref().unwrap_or("Unknown error"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        } else {
            None
        };

        // Add performance data to debug info
        let mut debug_info = HashMap::new();
        if let Some(perf_result) = &performance_result {
            debug_info.insert(
                "performance".to_string(),
                serde_json::to_value(perf_result).unwrap_or(serde_json::Value::Null),
            );
        }

        // Extract memory usage from performance metrics
        let memory_usage_mb = performance_result
            .as_ref()
            .map(|p| p.metrics.peak_memory_mb);

        Ok(TestResult {
            test_case,
            success,
            start_time,
            end_time,
            duration,
            memory_usage_mb,
            actual_response: Some(actual_response),
            validation_results,
            error_message,
            debug_info,
        })
    }

    /// Execute an MCP tool (mock implementation for now)
    async fn execute_mcp_tool(&self, test_case: &TestCase, _project_path: &str) -> Result<Value> {
        // TODO: Replace with actual MCP client execution
        // For now, generate mock responses based on tool name
        tokio::time::sleep(Duration::from_millis(50)).await;

        let mock_response = match test_case.tool_name.as_str() {
            "repository_stats" => serde_json::json!({
                "result": {
                    "total_files": 25,
                    "total_lines": 5000,
                    "languages_detected": ["Python", "JavaScript"],
                    "file_types": {
                        "py": 15,
                        "js": 10
                    }
                }
            }),
            "search_symbols" => serde_json::json!({
                "result": {
                    "total_matches": 12,
                    "symbols": [
                        {"name": "UserService", "type": "class", "file": "services/user.py"},
                        {"name": "validate_user", "type": "function", "file": "utils/validation.py"}
                    ]
                }
            }),
            "analyze_complexity" => serde_json::json!({
                "result": {
                    "cyclomatic_complexity": 8,
                    "cognitive_complexity": 10,
                    "max_nesting_depth": 4,
                    "total_functions": 15
                }
            }),
            _ => serde_json::json!({
                "result": {
                    "status": "completed",
                    "message": format!("Mock execution of {}", test_case.tool_name)
                }
            }),
        };

        Ok(mock_response)
    }

    /// Validate response against test case expectations
    async fn validate_response(
        &self,
        test_case: &TestCase,
        response: &Value,
    ) -> Result<Vec<ValidationResult>> {
        let mut validation_results = Vec::new();

        // Validate JSON path patterns
        for pattern in &test_case.expected.patterns {
            let result = self.validate_json_path_pattern(pattern, response).await?;
            validation_results.push(result);
        }

        // Execute custom validation scripts
        for script in &test_case.expected.custom_scripts {
            match self.script_executor.execute_script(script, response).await {
                Ok(script_result) => {
                    // Convert script result to validation result
                    validation_results.push(ValidationResult {
                        pattern: JsonPathPattern {
                            key: format!("custom_script_{}", script.name),
                            validation: PatternValidation::Expression {
                                expr: "custom_script".to_string(),
                            },
                            required: true,
                        },
                        passed: script_result.passed,
                        actual_value: Some(serde_json::json!({
                            "score": script_result.score,
                            "message": script_result.message,
                            "data": script_result.data
                        })),
                        error_message: if script_result.passed {
                            None
                        } else {
                            Some(script_result.message)
                        },
                    });
                }
                Err(e) => {
                    warn!("Custom script '{}' failed: {}", script.name, e);
                    validation_results.push(ValidationResult {
                        pattern: JsonPathPattern {
                            key: format!("custom_script_{}", script.name),
                            validation: PatternValidation::Expression {
                                expr: "custom_script".to_string(),
                            },
                            required: true,
                        },
                        passed: false,
                        actual_value: None,
                        error_message: Some(format!("Script execution failed: {}", e)),
                    });
                }
            }
        }

        Ok(validation_results)
    }

    /// Validate a JSON path pattern against the response
    async fn validate_json_path_pattern(
        &self,
        pattern: &JsonPathPattern,
        response: &Value,
    ) -> Result<ValidationResult> {
        // Extract value from JSON path
        let actual_value = self.extract_json_path_value(&pattern.key, response);

        let (passed, error_message) = match &pattern.validation {
            PatternValidation::Equals { value } => {
                let passed = actual_value.as_ref() == Some(value);
                let error = if !passed {
                    Some(format!("Expected {:?}, got {:?}", value, actual_value))
                } else {
                    None
                };
                (passed, error)
            }
            PatternValidation::Range { min, max } => {
                if let Some(actual) = actual_value.as_ref().and_then(|v| v.as_f64()) {
                    let passed = actual >= *min && actual <= *max;
                    let error = if !passed {
                        Some(format!("Value {} not in range [{}, {}]", actual, min, max))
                    } else {
                        None
                    };
                    (passed, error)
                } else {
                    (false, Some("Value is not a number".to_string()))
                }
            }
            PatternValidation::Contains { values } => {
                if let Some(actual_array) = actual_value.as_ref().and_then(|v| v.as_array()) {
                    let passed = values
                        .iter()
                        .all(|expected| actual_array.contains(expected));
                    let error = if !passed {
                        Some(format!(
                            "Array does not contain all expected values: {:?}",
                            values
                        ))
                    } else {
                        None
                    };
                    (passed, error)
                } else {
                    (false, Some("Value is not an array".to_string()))
                }
            }
            _ => {
                // For other validation types, implement as needed
                (true, None)
            }
        };

        Ok(ValidationResult {
            pattern: pattern.clone(),
            passed,
            actual_value,
            error_message,
        })
    }

    /// Extract value from JSON using a simple path notation
    fn extract_json_path_value(&self, path: &str, json: &Value) -> Option<Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = json;

        for part in parts {
            match current {
                Value::Object(map) => {
                    current = map.get(part)?;
                }
                Value::Array(arr) => {
                    if let Ok(index) = part.parse::<usize>() {
                        current = arr.get(index)?;
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        Some(current.clone())
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

    /// Generate comprehensive test report
    pub async fn generate_report(
        &self,
        test_results: &[TestSuiteResult],
        format: ReportFormat,
        output_path: Option<PathBuf>,
    ) -> Result<crate::reporting::Report> {
        let report_generator = ReportGenerator::new();
        report_generator.generate_report(test_results, format, output_path).await
    }

    /// Generate multiple report formats at once
    pub async fn generate_reports(
        &self,
        test_results: &[TestSuiteResult],
        output_dir: &PathBuf,
    ) -> Result<Vec<crate::reporting::Report>> {
        let report_generator = ReportGenerator::new();
        let mut reports = Vec::new();

        // Generate HTML report
        if let Ok(report) = report_generator
            .generate_report(
                test_results,
                ReportFormat::Html,
                Some(output_dir.join("test-report.html")),
            )
            .await
        {
            reports.push(report);
        }

        // Generate JSON report
        if let Ok(report) = report_generator
            .generate_report(
                test_results,
                ReportFormat::Json,
                Some(output_dir.join("test-report.json")),
            )
            .await
        {
            reports.push(report);
        }

        // Generate JUnit XML report
        if let Ok(report) = report_generator
            .generate_report(
                test_results,
                ReportFormat::JunitXml,
                Some(output_dir.join("test-report.xml")),
            )
            .await
        {
            reports.push(report);
        }

        Ok(reports)
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
