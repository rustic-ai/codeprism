//! Test execution engine for the standalone MCP test harness
//!
//! Orchestrates test execution with performance monitoring and result reporting.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, warn};

use crate::config::{ServerConfig, TestCase, TestConfig, TestSuite};
use crate::server::McpServer;
use crate::validation::TestValidator;

/// Test execution orchestrator
#[derive(Debug)]
pub struct TestRunner {
    config: TestConfig,
    server: Option<McpServer>,
    validator: TestValidator,
    #[allow(dead_code)]
    output_format: String,

    // Execution options
    validation_only: bool,
    #[allow(dead_code)]
    comprehensive: bool,
    parallel_execution: usize,
}

/// Results from test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    pub summary: TestSummary,
    pub suite_results: Vec<SuiteResult>,
    pub performance_data: Option<PerformanceData>,
    pub execution_metadata: ExecutionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub skipped_tests: usize,
    pub execution_time_seconds: f64,
    pub overall_success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteResult {
    pub suite_name: String,
    pub test_case_results: Vec<TestCaseResult>,
    pub suite_summary: SuiteSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteSummary {
    pub total_cases: usize,
    pub passed_cases: usize,
    pub failed_cases: usize,
    pub skipped_cases: usize,
    pub execution_time_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseResult {
    pub test_id: String,
    pub test_name: String,
    pub status: TestStatus,
    pub execution_time_ms: u64,
    pub memory_usage_mb: Option<f64>,
    pub error_message: Option<String>,
    pub response: Option<serde_json::Value>,
    pub validation_results: Vec<ValidationResult>,
    pub performance_metrics: Option<PerformanceMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub rule_name: String,
    pub passed: bool,
    pub message: String,
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub execution_time_ms: u64,
    pub memory_peak_mb: Option<f64>,
    pub memory_average_mb: Option<f64>,
    pub cpu_usage_percent: Option<f64>,
    pub network_requests: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceData {
    pub overall_metrics: PerformanceMetrics,
    pub baseline_comparison: Option<BaselineComparison>,
    pub regression_analysis: Option<RegressionAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    pub performance_change_percent: f64,
    pub memory_change_percent: f64,
    pub is_improvement: bool,
    pub is_regression: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAnalysis {
    pub detected_regressions: Vec<RegressionAlert>,
    pub detected_improvements: Vec<ImprovementAlert>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAlert {
    pub test_name: String,
    pub metric: String,
    pub change_percent: f64,
    pub severity: AlertSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementAlert {
    pub test_name: String,
    pub metric: String,
    pub improvement_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    pub start_time: String,
    pub end_time: String,
    pub execution_environment: String,
    pub test_harness_version: String,
    pub server_info: Option<ServerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub server_type: String,
    pub version: Option<String>,
    pub capabilities: Option<serde_json::Value>,
    pub transport: String,
}

impl TestRunner {
    /// Create new test runner
    pub fn new(config: TestConfig, output_format: String) -> Result<Self> {
        let validator = TestValidator::new();

        Ok(Self {
            config,
            server: None,
            validator,
            output_format,
            validation_only: false,
            comprehensive: false,
            parallel_execution: 1,
        })
    }

    /// Set server command for stdio servers
    pub fn set_server_command(&mut self, command: String, working_dir: Option<PathBuf>) {
        self.config.server.command = Some(command);
        if let Some(dir) = working_dir {
            self.config.server.working_dir = Some(dir.to_string_lossy().to_string());
        }
    }

    /// Set execution options
    pub fn set_validation_only(&mut self, validation_only: bool) {
        self.validation_only = validation_only;
    }

    pub fn set_comprehensive(&mut self, comprehensive: bool) {
        self.comprehensive = comprehensive;
    }

    pub fn set_parallel_execution(&mut self, parallel: usize) {
        self.parallel_execution = parallel;
    }

    /// Run all tests
    pub async fn run(&mut self) -> Result<TestResults> {
        let start_time = Instant::now();
        info!(
            "Starting test execution with {} test suites",
            self.config.test_suites.len()
        );

        // Initialize server if not validation-only
        if !self.validation_only {
            self.initialize_server().await?;
        }

        let mut suite_results = Vec::new();
        let mut total_tests = 0;
        let mut passed_tests = 0;
        let mut failed_tests = 0;
        let mut skipped_tests = 0;

        // Clone test suites to avoid borrow checker issues
        let test_suites = self.config.test_suites.clone();

        // Execute test suites
        for suite in &test_suites {
            info!("Executing test suite: {}", suite.name);

            let suite_result = if self.parallel_execution > 1 {
                self.run_suite_parallel(suite).await?
            } else {
                self.run_suite_sequential(suite).await?
            };

            total_tests += suite_result.suite_summary.total_cases;
            passed_tests += suite_result.suite_summary.passed_cases;
            failed_tests += suite_result.suite_summary.failed_cases;
            skipped_tests += suite_result.suite_summary.skipped_cases;

            suite_results.push(suite_result);
        }

        // Clean up server
        if let Some(ref mut server) = self.server {
            server.stop().await?;
        }

        let execution_time = start_time.elapsed();
        let overall_success_rate = if total_tests > 0 {
            passed_tests as f64 / total_tests as f64
        } else {
            0.0
        };

        let results = TestResults {
            summary: TestSummary {
                total_tests,
                passed_tests,
                failed_tests,
                skipped_tests,
                execution_time_seconds: execution_time.as_secs_f64(),
                overall_success_rate,
            },
            suite_results,
            performance_data: None, // Performance monitoring removed as out of scope
            execution_metadata: ExecutionMetadata {
                start_time: chrono::Utc::now().to_rfc3339(),
                end_time: chrono::Utc::now().to_rfc3339(),
                execution_environment: format!(
                    "{}_{}",
                    std::env::consts::OS,
                    std::env::consts::ARCH
                ),
                test_harness_version: env!("CARGO_PKG_VERSION").to_string(),
                server_info: None, // Server introspection not implemented for this scope
            },
        };

        info!(
            "Test execution completed: {}/{} passed",
            passed_tests, total_tests
        );
        Ok(results)
    }

    async fn initialize_server(&mut self) -> Result<()> {
        info!("Initializing MCP server");

        let mut server = McpServer::new(self.config.server.clone());
        server.start().await?;

        // Perform health check
        if !server.health_check().await? {
            return Err(anyhow!("Server failed health check"));
        }

        self.server = Some(server);
        Ok(())
    }

    async fn run_suite_sequential(&mut self, suite: &TestSuite) -> Result<SuiteResult> {
        let start_time = Instant::now();
        let mut test_case_results = Vec::new();
        let mut passed_cases = 0;
        let mut failed_cases = 0;
        let mut skipped_cases = 0;

        for test_case in &suite.test_cases {
            let result = self.run_test_case(test_case).await?;

            match result.status {
                TestStatus::Passed => passed_cases += 1,
                TestStatus::Failed | TestStatus::Error => failed_cases += 1,
                TestStatus::Skipped => skipped_cases += 1,
            }

            test_case_results.push(result);

            // Check fail_fast condition
            if self.config.global.fail_fast && failed_cases > 0 {
                warn!("Fail-fast enabled, stopping suite execution");
                break;
            }
        }

        Ok(SuiteResult {
            suite_name: suite.name.clone(),
            test_case_results,
            suite_summary: SuiteSummary {
                total_cases: suite.test_cases.len(),
                passed_cases,
                failed_cases,
                skipped_cases,
                execution_time_seconds: start_time.elapsed().as_secs_f64(),
            },
        })
    }

    async fn run_suite_parallel(&mut self, suite: &TestSuite) -> Result<SuiteResult> {
        let start_time = Instant::now();
        let mut test_case_results = Vec::new();
        let mut passed_cases = 0;
        let mut failed_cases = 0;
        let mut skipped_cases = 0;

        info!(
            "Running suite '{}' with parallel execution (max concurrent: {})",
            suite.name, self.parallel_execution
        );

        // Create semaphore to limit concurrent execution
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.parallel_execution));
        let mut handles = Vec::new();

        // Spawn tasks for each test case
        for test_case in &suite.test_cases {
            let test_case = test_case.clone();
            let semaphore = semaphore.clone();
            let _fail_fast = self.config.global.fail_fast;
            let validator = self.validator.clone();
            let validation_only = self.validation_only;
            let server_config = self.config.server.clone();

            let handle = tokio::spawn(async move {
                // Acquire semaphore permit
                let _permit = semaphore
                    .acquire()
                    .await
                    .expect("Semaphore should not be closed");

                // Execute test case
                Self::execute_test_case_standalone(
                    test_case,
                    validator,
                    validation_only,
                    server_config,
                )
                .await
            });

            handles.push(handle);
        }

        // Collect results as they complete
        for handle in handles {
            match handle.await {
                Ok(result) => {
                    match result.status {
                        TestStatus::Passed => passed_cases += 1,
                        TestStatus::Failed | TestStatus::Error => failed_cases += 1,
                        TestStatus::Skipped => skipped_cases += 1,
                    }

                    test_case_results.push(result);

                    // Check fail_fast condition
                    if self.config.global.fail_fast && failed_cases > 0 {
                        warn!("Fail-fast enabled, cancelling remaining tests");
                        break;
                    }
                }
                Err(e) => {
                    error!("Task execution error: {}", e);
                    failed_cases += 1;
                    test_case_results.push(TestCaseResult {
                        test_id: "unknown".to_string(),
                        test_name: "unknown".to_string(),
                        status: TestStatus::Error,
                        execution_time_ms: 0,
                        memory_usage_mb: None,
                        error_message: Some(format!("Task execution error: {}", e)),
                        response: None,
                        validation_results: vec![],
                        performance_metrics: None,
                    });
                }
            }
        }

        Ok(SuiteResult {
            suite_name: suite.name.clone(),
            test_case_results,
            suite_summary: SuiteSummary {
                total_cases: suite.test_cases.len(),
                passed_cases,
                failed_cases,
                skipped_cases,
                execution_time_seconds: start_time.elapsed().as_secs_f64(),
            },
        })
    }

    /// Execute a single test case in a standalone context (for parallel execution)
    async fn execute_test_case_standalone(
        test_case: TestCase,
        validator: TestValidator,
        validation_only: bool,
        server_config: ServerConfig,
    ) -> TestCaseResult {
        if !test_case.enabled {
            return TestCaseResult {
                test_id: test_case.id.clone(),
                test_name: test_case.tool_name.clone(),
                status: TestStatus::Skipped,
                execution_time_ms: 0,
                memory_usage_mb: None,
                error_message: Some("Test case disabled".to_string()),
                response: None,
                validation_results: vec![],
                performance_metrics: None,
            };
        }

        debug!(
            "Running test case: {} ({})",
            test_case.id, test_case.tool_name
        );
        let start_time = Instant::now();

        // Validation-only mode
        if validation_only {
            let validation_results = validator.validate_test_case(&test_case);
            return TestCaseResult {
                test_id: test_case.id.clone(),
                test_name: test_case.tool_name.clone(),
                status: if validation_results.iter().all(|r| r.passed) {
                    TestStatus::Passed
                } else {
                    TestStatus::Failed
                },
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                memory_usage_mb: None,
                error_message: None,
                response: None,
                validation_results,
                performance_metrics: None,
            };
        }

        // Execute actual test with real MCP client
        match Self::execute_mcp_test_case(&test_case, &server_config).await {
            Ok(response) => {
                let validation_results = validator.validate_response(&test_case, &response);
                let status = if validation_results.iter().all(|r| r.passed) {
                    TestStatus::Passed
                } else {
                    TestStatus::Failed
                };

                TestCaseResult {
                    test_id: test_case.id.clone(),
                    test_name: test_case.tool_name.clone(),
                    status,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    memory_usage_mb: None, // Memory tracking not required for MCP protocol compliance testing
                    error_message: None,
                    response: Some(response),
                    validation_results,
                    performance_metrics: None, // Performance metrics not required for protocol compliance testing
                }
            }
            Err(e) => TestCaseResult {
                test_id: test_case.id.clone(),
                test_name: test_case.tool_name.clone(),
                status: TestStatus::Error,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                memory_usage_mb: None,
                error_message: Some(e.to_string()),
                response: None,
                validation_results: vec![],
                performance_metrics: None,
            },
        }
    }

    /// Execute MCP test case with real server communication
    async fn execute_mcp_test_case(
        test_case: &TestCase,
        server_config: &ServerConfig,
    ) -> Result<serde_json::Value> {
        // Create and start MCP server
        let mut server = McpServer::new(server_config.clone());
        server
            .start()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start MCP server: {}", e))?;

        // Execute tool call via the server using the proper MCP method
        let result = server
            .call_tool(&test_case.tool_name, test_case.input_params.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Tool call failed: {}", e))?;

        // Stop server
        server
            .stop()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to stop MCP server: {}", e))?;

        Ok(result)
    }

    async fn run_test_case(&mut self, test_case: &TestCase) -> Result<TestCaseResult> {
        if !test_case.enabled {
            return Ok(TestCaseResult {
                test_id: test_case.id.clone(),
                test_name: test_case.tool_name.clone(),
                status: TestStatus::Skipped,
                execution_time_ms: 0,
                memory_usage_mb: None,
                error_message: Some("Test case disabled".to_string()),
                response: None,
                validation_results: vec![],
                performance_metrics: None,
            });
        }

        debug!(
            "Running test case: {} ({})",
            test_case.id, test_case.tool_name
        );
        let start_time = Instant::now();

        // Validation-only mode
        if self.validation_only {
            let validation_results = self.validator.validate_test_case(test_case);
            return Ok(TestCaseResult {
                test_id: test_case.id.clone(),
                test_name: test_case.tool_name.clone(),
                status: if validation_results.iter().all(|r| r.passed) {
                    TestStatus::Passed
                } else {
                    TestStatus::Failed
                },
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                memory_usage_mb: None,
                error_message: None,
                response: None,
                validation_results,
                performance_metrics: None,
            });
        }

        // Execute actual test
        let result = match self.execute_test_case(test_case).await {
            Ok(response) => {
                let validation_results = self.validator.validate_response(test_case, &response);
                let status = if validation_results.iter().all(|r| r.passed) {
                    TestStatus::Passed
                } else {
                    TestStatus::Failed
                };

                TestCaseResult {
                    test_id: test_case.id.clone(),
                    test_name: test_case.tool_name.clone(),
                    status,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    memory_usage_mb: None, // Memory monitoring not required for protocol compliance
                    error_message: None,
                    response: Some(response),
                    validation_results,
                    performance_metrics: None, // Performance metrics not used in protocol compliance testing
                }
            }
            Err(e) => TestCaseResult {
                test_id: test_case.id.clone(),
                test_name: test_case.tool_name.clone(),
                status: TestStatus::Error,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                memory_usage_mb: None,
                error_message: Some(e.to_string()),
                response: None,
                validation_results: vec![],
                performance_metrics: None,
            },
        };

        Ok(result)
    }

    async fn execute_test_case(&mut self, test_case: &TestCase) -> Result<serde_json::Value> {
        let server = self
            .server
            .as_mut()
            .ok_or_else(|| anyhow!("Server not initialized"))?;

        server
            .call_tool(&test_case.tool_name, test_case.input_params.clone())
            .await
    }
}

impl TestResults {
    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.summary.failed_tests == 0
    }

    /// Display results in table format
    pub fn display_table(&self) {
        println!("\n📊 Test Execution Summary");
        println!("═══════════════════════════════════════");
        println!("Total Tests:    {}", self.summary.total_tests);
        println!("Passed:         {} ✅", self.summary.passed_tests);
        println!("Failed:         {} ❌", self.summary.failed_tests);
        println!("Skipped:        {} ⏭️", self.summary.skipped_tests);
        println!(
            "Success Rate:   {:.1}%",
            self.summary.overall_success_rate * 100.0
        );
        println!(
            "Execution Time: {:.2}s",
            self.summary.execution_time_seconds
        );

        for suite_result in &self.suite_results {
            println!("\n📁 {}", suite_result.suite_name);
            println!("───────────────────────────────────────");

            for test_result in &suite_result.test_case_results {
                let status_icon = match test_result.status {
                    TestStatus::Passed => "✅",
                    TestStatus::Failed => "❌",
                    TestStatus::Skipped => "⏭️",
                    TestStatus::Error => "💥",
                };

                println!(
                    "  {} {} ({}ms)",
                    status_icon, test_result.test_id, test_result.execution_time_ms
                );

                if let Some(ref error) = test_result.error_message {
                    println!("    Error: {}", error);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TestConfig;

    #[tokio::test]
    async fn test_runner_creation() {
        let config = TestConfig::default_config();
        let runner = TestRunner::new(config, "table".to_string());
        assert!(runner.is_ok());
    }

    #[test]
    fn test_results_serialization() {
        let results = TestResults {
            summary: TestSummary {
                total_tests: 10,
                passed_tests: 8,
                failed_tests: 2,
                skipped_tests: 0,
                execution_time_seconds: 30.5,
                overall_success_rate: 0.8,
            },
            suite_results: vec![],
            performance_data: None,
            execution_metadata: ExecutionMetadata {
                start_time: "2024-01-01T00:00:00Z".to_string(),
                end_time: "2024-01-01T00:00:30Z".to_string(),
                execution_environment: "linux_x86_64".to_string(),
                test_harness_version: "0.1.0".to_string(),
                server_info: None,
            },
        };

        let json = serde_json::to_string(&results).unwrap();
        let _deserialized: TestResults = serde_json::from_str(&json).unwrap();
    }
}
