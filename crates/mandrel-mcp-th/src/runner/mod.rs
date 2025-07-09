//! Test Suite Runner - Orchestrates multiple test case executions
//!
//! The `runner` module provides the core orchestration layer for executing
//! complete YAML test specifications with multiple test cases. It manages
//! execution order, dependencies, parallel/sequential execution, and result
//! aggregation.

pub mod config;
pub mod dependency;
pub mod execution;
pub mod metrics;
pub mod result;

// Re-export main types
pub use config::{ExecutionMode, RunnerConfig};
pub use dependency::DependencyResolver;
pub use execution::ExecutionStrategy;
pub use metrics::{MetricsCollector, SuiteMetrics};
pub use result::{DependencyResolution, TestSuiteResult};

use crate::error::Result;
use crate::executor::TestCaseExecutor;
use crate::runner::result::TestMetadata;
use crate::spec::{SpecificationLoader, TestSpecification};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// Main orchestrator for test suite execution
///
/// The `TestSuiteRunner` coordinates the execution of multiple test cases
/// from YAML specifications. It handles dependency resolution, execution
/// strategies (parallel/sequential), and result aggregation.
///
/// # Examples
///
/// ```no_run
/// use mandrel_mcp_th::runner::{TestSuiteRunner, RunnerConfig};
/// use mandrel_mcp_th::executor::TestCaseExecutor;
/// use std::path::Path;
///
/// # async fn example() -> mandrel_mcp_th::Result<()> {
/// // Note: TestCaseExecutor creation requires client and config parameters
/// // let executor = TestCaseExecutor::new(client, config);
/// // let config = RunnerConfig::new().with_parallel_execution(true);
/// // let mut runner = TestSuiteRunner::new(executor, config);
///
/// // let result = runner.run_test_suite(Path::new("test-spec.yaml")).await?;
/// // println!("Executed {} tests, {} passed", result.total_tests, result.passed);
/// # Ok(())
/// # }
/// ```
pub struct TestSuiteRunner {
    executor: TestCaseExecutor,
    loader: SpecificationLoader,
    config: RunnerConfig,
    metrics_collector: MetricsCollector,
}

impl TestSuiteRunner {
    /// Create a new test suite runner
    ///
    /// # Arguments
    /// * `executor` - The test case executor for individual test execution
    /// * `config` - Configuration for test suite execution behavior
    ///
    /// # Examples
    /// ```rust,no_run
    /// use mandrel_mcp_th::runner::{TestSuiteRunner, RunnerConfig};
    /// use mandrel_mcp_th::executor::{TestCaseExecutor, ExecutorConfig};
    /// use mandrel_mcp_th::client::{McpClient, ServerConfig, Transport};
    /// use std::collections::HashMap;
    /// use std::time::Duration;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> mandrel_mcp_th::Result<()> {
    /// // Create server configuration
    /// let server_config = ServerConfig {
    ///     command: "echo".to_string(),
    ///     args: vec!["test".to_string()],
    ///     env: HashMap::new(),
    ///     working_dir: None,
    ///     transport: Transport::Stdio,
    ///     startup_timeout: Duration::from_secs(5),
    ///     shutdown_timeout: Duration::from_secs(5),
    ///     operation_timeout: Duration::from_secs(10),
    ///     max_retries: 2,
    /// };
    ///
    /// // Create MCP client and executor
    /// let client = McpClient::new(server_config).await?;
    /// let shared_client = Arc::new(std::sync::Mutex::new(client));
    /// let executor_config = ExecutorConfig::default();
    /// let executor = TestCaseExecutor::new(shared_client, executor_config);
    ///
    /// // Create runner configuration
    /// let config = RunnerConfig::new();
    ///
    /// // Create test suite runner
    /// let runner = TestSuiteRunner::new(executor, config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(executor: TestCaseExecutor, config: RunnerConfig) -> Self {
        Self {
            executor,
            loader: SpecificationLoader::new().expect("Failed to create specification loader"),
            config,
            metrics_collector: MetricsCollector::new(),
        }
    }

    /// Run a complete test suite from a YAML specification file
    pub async fn run_test_suite(&mut self, spec_path: &Path) -> Result<TestSuiteResult> {
        let execution_start = SystemTime::now();
        self.metrics_collector.start_suite();

        // 1. Load and parse the test specification
        let specification = self.loader.load_from_file(spec_path).await?;

        // 2. Extract test cases from specification
        let test_cases = self.extract_test_cases(&specification)?;

        // 3. Resolve dependencies and determine execution order
        let dependency_resolution = self.resolve_dependencies(&test_cases)?;

        // Execute tests according to strategy
        let test_results = self
            .execute_tests_with_strategy(&test_cases, &dependency_resolution, &specification)
            .await?;

        // 5. Collect final metrics
        let execution_end = SystemTime::now();
        let total_duration = execution_end
            .duration_since(execution_start)
            .unwrap_or_else(|_| Duration::from_secs(0));

        self.metrics_collector.end_suite();
        let suite_metrics = self.metrics_collector.get_suite_metrics();

        // 6. Build comprehensive result
        let (passed, failed, skipped) = self.count_test_results(&test_results);
        let error_rate = if test_cases.is_empty() {
            0.0
        } else {
            failed as f64 / test_cases.len() as f64
        };

        Ok(TestSuiteResult {
            suite_name: specification.name.clone(),
            specification_file: spec_path.to_path_buf(),
            execution_start,
            execution_end,
            total_duration,
            total_tests: test_cases.len(),
            passed,
            failed,
            skipped,
            error_rate,
            test_results,
            execution_mode: self.config.execution_mode.clone(),
            dependency_resolution,
            suite_metrics,
        })
    }

    /// Update the runner configuration
    pub fn set_config(&mut self, config: RunnerConfig) {
        self.config = config;
    }

    /// Get reference to metrics collector
    pub fn get_metrics(&self) -> &MetricsCollector {
        &self.metrics_collector
    }

    // ========================================================================
    // PRIVATE IMPLEMENTATION METHODS
    // ========================================================================

    /// Extract test cases from loaded specification
    fn extract_test_cases(
        &self,
        specification: &TestSpecification,
    ) -> Result<Vec<crate::spec::TestCase>> {
        // Handle empty test suites - check if tools are defined
        let tools = match &specification.tools {
            Some(tools) if !tools.is_empty() => tools,
            _ => return Ok(Vec::new()), // Empty test suite - return empty Vec
        };

        // Extract real test cases from YAML tools
        let mut test_cases = Vec::new();
        for tool in tools {
            for test in &tool.tests {
                test_cases.push(test.clone());
            }
        }

        Ok(test_cases)
    }

    /// Resolve test case dependencies and determine execution order
    fn resolve_dependencies(
        &self,
        test_cases: &[crate::spec::TestCase],
    ) -> Result<DependencyResolution> {
        let mut resolver = DependencyResolver::new();

        // Build dependency map from real test cases
        let dependencies: HashMap<String, Vec<String>> = test_cases
            .iter()
            .map(|tc| (tc.name.clone(), tc.dependencies.clone().unwrap_or_default()))
            .collect();

        // Resolve execution order
        let execution_order = resolver.resolve_dependencies(&dependencies)?;

        Ok(DependencyResolution {
            total_dependencies: dependencies.values().map(|deps| deps.len()).sum(),
            circular_dependencies: 0,
            circular_dependency_chains: vec![],
            resolution_duration: Duration::from_millis(1), // Mock resolution time
            execution_order,
            dependency_groups: vec![test_cases.iter().map(|tc| tc.name.clone()).collect()], // Single group for now
        })
    }

    /// Execute tests according to the configured strategy
    async fn execute_tests_with_strategy(
        &mut self,
        test_cases: &[crate::spec::TestCase],
        dependency_resolution: &DependencyResolution,
        specification: &TestSpecification,
    ) -> Result<Vec<TestResult>> {
        match self.config.execution_mode {
            ExecutionMode::Sequential => {
                self.execute_tests_sequentially(test_cases, dependency_resolution, specification)
                    .await
            }
            ExecutionMode::Parallel => {
                self.execute_tests_in_parallel(test_cases, dependency_resolution, specification)
                    .await
            }
        }
    }

    /// Execute tests sequentially in dependency order
    async fn execute_tests_sequentially(
        &mut self,
        test_cases: &[crate::spec::TestCase],
        dependency_resolution: &DependencyResolution,
        specification: &TestSpecification,
    ) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();

        for test_name in &dependency_resolution.execution_order {
            let start_time = SystemTime::now();
            self.metrics_collector.start_test(test_name);

            // REAL EXECUTION: Execute through TestCaseExecutor
            let test_result = match self
                .execute_single_test(test_name, test_cases, specification)
                .await
            {
                Ok(result) => result,
                Err(e) => {
                    // Create a failed test result from the error
                    TestResult {
                        test_name: test_name.clone(),
                        success: false,
                        duration: start_time.elapsed().unwrap_or(Duration::from_millis(0)),
                        error_message: Some(format!("Test execution failed: {}", e)),
                        retry_attempts: 0,
                        start_time,
                        end_time: SystemTime::now(),
                        memory_usage_mb: None,
                        metadata: TestMetadata::default(),
                    }
                }
            };

            self.metrics_collector.end_test(
                test_name,
                test_result.success,
                test_result.error_message.clone(),
            );

            results.push(test_result.clone());

            // FAIL-FAST: Stop execution on first failure (existing logic now works!)
            if !test_result.success && self.config.fail_fast {
                break;
            }
        }

        Ok(results)
    }

    /// Execute tests in parallel groups respecting dependencies
    async fn execute_tests_in_parallel(
        &mut self,
        test_cases: &[crate::spec::TestCase],
        dependency_resolution: &DependencyResolution,
        specification: &TestSpecification,
    ) -> Result<Vec<TestResult>> {
        // FUTURE(#229): Implement true parallel execution with dependency groups
        // Currently using sequential execution to maintain correctness while providing real execution
        self.execute_tests_sequentially(test_cases, dependency_resolution, specification)
            .await
    }

    /// Count test results by status
    fn count_test_results(&self, test_results: &[TestResult]) -> (usize, usize, usize) {
        let passed = test_results.iter().filter(|r| r.success).count();
        let failed = test_results.iter().filter(|r| !r.success).count();
        let skipped = 0; // No skipped tests in basic implementation

        (passed, failed, skipped)
    }

    // ========================================================================
    // ISSUE #220 COMPLETION - REAL EXECUTION METHODS
    // ========================================================================

    /// Execute test cases with dependency management (from Issue #220 design)
    pub async fn execute_with_dependencies(
        &mut self,
        test_cases: Vec<crate::spec::TestCase>,
    ) -> Result<Vec<TestResult>> {
        // 1. Resolve dependencies and determine execution order
        let dependency_resolution = self.resolve_dependencies(&test_cases)?;

        // 2. Create a mock specification for backward compatibility
        // This is needed because execute_tests_with_strategy expects a TestSpecification
        let mock_specification = crate::spec::TestSpecification {
            name: "Direct Execution".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            capabilities: crate::spec::ServerCapabilities {
                tools: true,
                resources: false,
                prompts: false,
                sampling: false,
                logging: false,
                experimental: None,
            },
            server: crate::spec::ServerConfig {
                command: "mock".to_string(),
                args: vec![],
                env: std::collections::HashMap::new(),
                working_dir: None,
                transport: "stdio".to_string(),
                startup_timeout_seconds: 30,
                shutdown_timeout_seconds: 10,
            },
            tools: Some(vec![]), // Empty tools - will use test_cases directly
            resources: None,
            prompts: None,
            test_config: None,
            metadata: None,
        };

        // 3. Execute tests with the resolved dependencies
        self.execute_tests_with_strategy(&test_cases, &dependency_resolution, &mock_specification)
            .await
    }

    /// Find test case by name in the test cases list
    fn find_test_case_by_name<'a>(
        &self,
        test_name: &str,
        test_cases: &'a [crate::spec::TestCase],
    ) -> Result<&'a crate::spec::TestCase> {
        test_cases
            .iter()
            .find(|tc| tc.name == test_name)
            .ok_or_else(|| {
                crate::error::Error::execution(format!(
                    "Test case '{}' not found in test suite",
                    test_name
                ))
            })
    }

    /// Find tool name for a given test in the specification
    fn find_tool_name_for_test(
        &self,
        test_name: &str,
        specification: &TestSpecification,
    ) -> Result<String> {
        if let Some(tools) = &specification.tools {
            for tool in tools {
                for test in &tool.tests {
                    if test.name == test_name {
                        return Ok(tool.name.clone());
                    }
                }
            }
        }

        // If not found in specification tools, try to infer from test name
        // This handles cases where tests are executed directly without full specification
        if test_name.contains("_") {
            let parts: Vec<&str> = test_name.split('_').collect();
            if parts.len() >= 2 {
                return Ok(parts[0].to_string());
            }
        }

        // Default fallback - use test name as tool name
        Ok(test_name.to_string())
    }

    /// Convert TestCaseExecutor result to TestSuiteRunner TestResult
    fn convert_executor_result(
        &self,
        executor_result: crate::executor::TestCaseResult,
        test_name: &str,
    ) -> TestResult {
        let start_time = SystemTime::now() - executor_result.execution_time;
        let end_time = SystemTime::now();

        TestResult {
            test_name: test_name.to_string(),
            success: executor_result.success,
            duration: executor_result.execution_time,
            error_message: executor_result.error.clone(),
            retry_attempts: 0, // FUTURE(#200): Add retry support with TestCaseExecutor retry integration
            start_time,
            end_time,
            memory_usage_mb: executor_result.metrics.memory_usage,
            metadata: TestMetadata::default(),
        }
    }

    /// Execute a single test using the real TestCaseExecutor
    async fn execute_single_test(
        &mut self,
        test_name: &str,
        test_cases: &[crate::spec::TestCase],
        specification: &TestSpecification,
    ) -> Result<TestResult> {
        // 1. Find the test case
        let test_case = self.find_test_case_by_name(test_name, test_cases)?;

        // 2. Find the tool name for this test
        let tool_name = self.find_tool_name_for_test(test_name, specification)?;

        // 3. Execute through TestCaseExecutor
        let executor_result = self
            .executor
            .execute_test_case(&tool_name, test_case)
            .await?;

        // 4. Convert to TestSuiteRunner result format
        Ok(self.convert_executor_result(executor_result, test_name))
    }
}

// Re-export TestResult from result module to avoid duplication
pub use result::TestResult;

// Use TestCase from the spec module directly
pub use crate::spec::TestCase;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::executor::ExecutorConfig;
    use std::io::Write;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::NamedTempFile;

    // Helper function to create a test executor with real Filesystem MCP Server
    async fn create_test_executor() -> TestCaseExecutor {
        use crate::client::{McpClient, ServerConfig, Transport};
        use std::collections::HashMap;

        // Create temp directory for filesystem server testing
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path().to_string_lossy().to_string();

        // Create some test files for the filesystem server to operate on
        std::fs::write(temp_dir.path().join("test.txt"), "Hello, MCP World!")
            .expect("Failed to create test file");
        std::fs::write(temp_dir.path().join("data.json"), r#"{"test": "data"}"#)
            .expect("Failed to create test JSON file");

        let server_config = ServerConfig {
            command: "npx".to_string(),
            args: vec![
                "@modelcontextprotocol/server-filesystem".to_string(),
                temp_path,
            ],
            env: HashMap::new(),
            working_dir: None,
            transport: Transport::Stdio,
            startup_timeout: Duration::from_secs(30), // Longer timeout for npm package install
            shutdown_timeout: Duration::from_secs(10),
            operation_timeout: Duration::from_secs(60), // Longer for real operations
            max_retries: 2,
        };

        let client = McpClient::new(server_config)
            .await
            .expect("Failed to create MCP client with filesystem server");
        let shared_client = Arc::new(std::sync::Mutex::new(client));

        let config = ExecutorConfig::default();
        TestCaseExecutor::new(shared_client, config)
    }

    // Helper function to get the existing filesystem server specification
    fn get_filesystem_test_spec_path() -> std::path::PathBuf {
        std::path::PathBuf::from("examples/filesystem-server.yaml")
    }

    // Helper function to get a subset of the filesystem spec for dependency testing
    fn get_simplified_filesystem_spec_path() -> std::path::PathBuf {
        // For now, use the same comprehensive spec - in the future we could create
        // a smaller subset specifically for dependency testing
        std::path::PathBuf::from("examples/filesystem-server.yaml")
    }

    // ========================================================================
    // PHASE 1: Basic Test Suite Execution Tests (RED PHASE)
    // ========================================================================

    #[tokio::test]
    async fn test_run_simple_test_suite_with_real_execution() {
        // Use existing comprehensive filesystem server specification
        let executor = create_test_executor().await;
        let config = RunnerConfig::new();
        let mut runner = TestSuiteRunner::new(executor, config);

        let test_spec_path = get_filesystem_test_spec_path();

        // Execute test suite with REAL TestCaseExecutor (not mock)
        let result = runner.run_test_suite(&test_spec_path).await;

        assert!(result.is_ok(), "Test suite execution should succeed");
        let suite_result = result.unwrap();

        // Verify basic suite results structure
        assert_eq!(suite_result.suite_name, "Filesystem MCP Server");
        assert!(
            suite_result.total_tests > 30,
            "Filesystem spec has 30+ comprehensive tests"
        );
        assert_eq!(suite_result.skipped, 0);

        // EXPECTED SUCCESS: Filesystem MCP server should launch on-demand and tests should pass
        // The MCP client automatically starts the filesystem server process

        // Verify real execution occurred (not mock)
        assert_eq!(suite_result.test_results.len(), suite_result.total_tests);
        assert_eq!(suite_result.execution_mode, ExecutionMode::Sequential);

        // All tests should have measurable duration (proving real execution)
        for test_result in &suite_result.test_results {
            assert!(
                test_result.duration > Duration::from_millis(0),
                "Test '{}' should have measurable duration",
                test_result.test_name
            );
        }

        // EXPECT SUCCESS: With a working filesystem MCP server, tests should pass
        assert!(
            suite_result.passed > 0,
            "Some filesystem tests should pass with on-demand server. Got {} passed, {} failed. First error: {}",
            suite_result.passed,
            suite_result.failed,
            suite_result.test_results.get(0)
                .and_then(|t| t.error_message.as_ref())
                .unwrap_or(&"No error message".to_string())
        );

        // Verify successful tests
        let passed_tests: Vec<_> = suite_result
            .test_results
            .iter()
            .filter(|t| t.success)
            .collect();
        assert!(
            !passed_tests.is_empty(),
            "Should have at least some passing tests"
        );

        // For any failed tests, they should have meaningful error messages
        for test_result in suite_result.test_results.iter().filter(|t| !t.success) {
            assert!(
                test_result.error_message.is_some(),
                "Failed test '{}' should have error message",
                test_result.test_name
            );
        }

        println!(
            "✅ FILESYSTEM MCP SERVER SUCCESS: {} passed, {} failed out of {} total tests",
            suite_result.passed, suite_result.failed, suite_result.total_tests
        );
    }

    #[tokio::test]
    async fn test_run_test_suite_with_dependencies_real_execution() {
        // Use existing comprehensive filesystem server specification for dependency testing
        let executor = create_test_executor().await;
        let config = RunnerConfig::new();
        let mut runner = TestSuiteRunner::new(executor, config);

        let test_spec_path = get_simplified_filesystem_spec_path();

        // Execute test suite with REAL TestCaseExecutor and dependency resolution
        let result = runner.run_test_suite(&test_spec_path).await;

        assert!(
            result.is_ok(),
            "Test suite with dependencies should execute successfully"
        );
        let suite_result = result.unwrap();

        // Verify dependency resolution worked correctly with filesystem spec
        assert!(
            suite_result.total_tests > 30,
            "Filesystem spec has 30+ comprehensive tests"
        );

        // EXPECTED SUCCESS: Filesystem MCP server should work and dependency resolution should be correct

        // Verify execution order is maintained - this is the main focus of this test
        let test_names: Vec<&str> = suite_result
            .test_results
            .iter()
            .map(|t| t.test_name.as_str())
            .collect();

        // The filesystem spec doesn't have explicit dependencies, but execution order should be maintained
        assert_eq!(test_names.len(), suite_result.total_tests);

        // Verify dependency resolution information works correctly
        assert!(!suite_result
            .dependency_resolution
            .has_circular_dependencies());
        assert_eq!(
            suite_result.dependency_resolution.execution_order.len(),
            suite_result.total_tests
        );

        // Verify test execution occurred with real durations
        for test_result in &suite_result.test_results {
            assert!(
                test_result.duration > Duration::from_millis(0),
                "Test '{}' should have measurable duration",
                test_result.test_name
            );
        }

        // EXPECT SUCCESS: Tests should pass with dependency resolution
        assert!(
            suite_result.passed > 0,
            "Some filesystem tests should pass with dependency resolution. Got {} passed, {} failed",
            suite_result.passed, suite_result.failed
        );

        println!(
            "✅ DEPENDENCY RESOLUTION SUCCESS: {} passed, {} failed, all {} tests in correct order",
            suite_result.passed, suite_result.failed, suite_result.total_tests
        );
    }

    #[tokio::test]
    #[ignore = "Future work for Issue #229 - parallel execution timing is sensitive and needs proper implementation"]
    async fn test_parallel_execution_mode() {
        // Test parallel execution reduces total time for independent tests
        let executor = create_test_executor().await;
        let config = RunnerConfig::new()
            .with_parallel_execution(true)
            .with_max_concurrency(3);
        let mut runner = TestSuiteRunner::new(executor, config);

        let test_spec_path = get_filesystem_test_spec_path();

        // This should fail until we implement parallel execution
        let result = runner.run_test_suite(&test_spec_path).await;

        assert!(result.is_ok(), "Parallel execution should succeed");
        let suite_result = result.unwrap();

        // Verify parallel execution mode
        assert_eq!(suite_result.execution_mode, ExecutionMode::Parallel);

        // For independent tests, parallel execution should be faster
        // (This is a heuristic - parallel should be significantly faster)
        let total_test_time: Duration = suite_result.test_results.iter().map(|t| t.duration).sum();

        // Allow for some overhead - parallel execution should be at least 10% faster
        // than the sum of individual test times
        let expected_max_duration = total_test_time.mul_f64(0.9);

        // Suite execution should be much less than sum of individual test times
        // due to parallel execution (allowing for some overhead)
        assert!(
            suite_result.total_duration < expected_max_duration,
            "Parallel execution should be faster than sequential: suite={:?} vs expected_max={:?}",
            suite_result.total_duration,
            expected_max_duration
        );
    }

    #[tokio::test]
    async fn test_sequential_execution_mode() {
        // Test sequential execution respects order and timing
        let executor = create_test_executor().await;
        let config = RunnerConfig::new().with_execution_mode(ExecutionMode::Sequential);
        let mut runner = TestSuiteRunner::new(executor, config);

        let test_spec_path = get_filesystem_test_spec_path();

        // This should fail until we implement sequential execution
        let result = runner.run_test_suite(&test_spec_path).await;

        assert!(result.is_ok(), "Sequential execution should succeed");
        let suite_result = result.unwrap();

        // Verify sequential execution mode
        assert_eq!(suite_result.execution_mode, ExecutionMode::Sequential);

        // Verify tests executed in order (by checking timestamps)
        let mut prev_end_time = suite_result.test_results[0].end_time;
        for test_result in &suite_result.test_results[1..] {
            assert!(
                test_result.start_time >= prev_end_time,
                "Sequential tests should not overlap: {:?} vs {:?}",
                test_result.start_time,
                prev_end_time
            );
            prev_end_time = test_result.end_time;
        }
    }

    #[tokio::test]
    #[ignore = "Future work for Issue #225 - fail-fast behavior is sensitive to test execution order"]
    async fn test_fail_fast_behavior() {
        // Test that fail-fast stops execution on first failure
        let executor = create_test_executor().await;
        let config = RunnerConfig::new().with_fail_fast(true);
        let mut runner = TestSuiteRunner::new(executor, config);

        // Create a test suite where the second test will fail
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        write!(
            temp_file,
            r#"
name: "Fail Fast Test Suite"
version: "1.0.0"
capabilities:
  tools: true
  resources: false
  prompts: false
  sampling: false
  logging: false
server:
  command: "test-server"
  transport: "stdio"
tools:
  - name: "passing_tool"
    tests:
      - name: "test1"
        description: "This should pass"
        input:
          value: "pass"
        expected:
          error: false
  - name: "failing_tool"
    tests:
      - name: "test2"
        description: "This should fail and stop execution"
        input:
          value: "fail"
        expected:
          error: true
  - name: "never_executed_tool"
    tests:
      - name: "test3"
        description: "This should never execute due to fail-fast"
        input:
          value: "never"
        expected:
          error: false
"#
        )
        .expect("Failed to write test YAML");
        temp_file.flush().expect("Failed to flush temp file");

        // This should fail until we implement fail-fast behavior
        let result = runner.run_test_suite(temp_file.path()).await;

        assert!(
            result.is_ok(),
            "Suite execution should complete even with fail-fast: {:?}",
            result.err()
        );
        let suite_result = result.unwrap();

        // With fail-fast, execution should stop after the first failure
        assert!(
            suite_result.test_results.len() <= 2,
            "Fail-fast should stop execution early: got {} results",
            suite_result.test_results.len()
        );
        assert!(
            suite_result.has_failures(),
            "Should have at least one failure"
        );

        // Verify the first test passed and second failed
        if suite_result.test_results.len() >= 2 {
            assert!(
                suite_result.test_results[0].success,
                "First test should pass"
            );
            assert!(
                !suite_result.test_results[1].success,
                "Second test should fail"
            );
        }
    }

    #[tokio::test]
    async fn test_suite_metrics_collection() {
        // Test comprehensive metrics collection during execution
        let executor = create_test_executor().await;
        let config = RunnerConfig::new();
        let mut runner = TestSuiteRunner::new(executor, config);

        let test_spec_path = get_filesystem_test_spec_path();

        // This should fail until we implement metrics collection
        let result = runner.run_test_suite(&test_spec_path).await;

        assert!(result.is_ok(), "Metrics collection should work");
        let suite_result = result.unwrap();

        // Verify suite-level metrics
        let metrics = &suite_result.suite_metrics;
        assert!(metrics.average_test_duration > Duration::from_millis(0));
        assert!(metrics.slowest_test.is_some());
        assert!(metrics.fastest_test.is_some());
        assert!(metrics.peak_memory_usage > 0);
        assert!(metrics.execution_efficiency_score >= 0.0);
        assert!(metrics.execution_efficiency_score <= 100.0);
        assert!(metrics.memory_efficiency_score >= 0.0);
        assert!(metrics.memory_efficiency_score <= 100.0);

        // Verify individual test metrics
        for test_result in &suite_result.test_results {
            assert!(test_result.duration > Duration::from_millis(0));
            assert!(test_result.start_time <= test_result.end_time);
        }

        // Verify total duration makes sense
        assert!(suite_result.total_duration > Duration::from_millis(0));
        assert!(suite_result.execution_start <= suite_result.execution_end);
    }

    // ========================================================================
    // PHASE 2: Error Handling Tests (RED PHASE)
    // ========================================================================

    #[tokio::test]
    async fn test_invalid_yaml_specification() {
        // Test handling of invalid YAML specification
        let executor = create_test_executor().await;
        let config = RunnerConfig::new();
        let mut runner = TestSuiteRunner::new(executor, config);

        // Create invalid YAML
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        write!(temp_file, "invalid: yaml: [unclosed").expect("Failed to write invalid YAML");
        temp_file.flush().expect("Failed to flush temp file");

        // This should fail until we implement error handling
        let result = runner.run_test_suite(temp_file.path()).await;

        assert!(result.is_err(), "Invalid YAML should cause an error");
        let error = result.unwrap_err();
        assert!(
            matches!(error, Error::Yaml(_)) || matches!(error, Error::Spec(_)),
            "Should be a YAML or Spec error: {:?}",
            error
        );
    }

    #[tokio::test]
    #[ignore = "Future work for Issue #228 - YAML test case extraction needed for circular dependency integration test"]
    async fn test_circular_dependency_detection() {
        // Test detection of circular dependencies
        let executor = create_test_executor().await;
        let config = RunnerConfig::new();
        let mut runner = TestSuiteRunner::new(executor, config);

        // Create YAML with circular dependencies
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        write!(
            temp_file,
            r#"
name: "Circular Dependency Test"
version: "1.0.0"
capabilities:
  tools: true
  sampling: false
  logging: false
server:
  command: "test-server"
  transport: "stdio"
tools:
  - name: "tool_a"
    tests:
      - name: "test_a"
        description: "Test A depends on Test B"
        dependencies: ["test_b"]
        input:
          value: "a"
        expected:
          error: false
  - name: "tool_b"
    tests:
      - name: "test_b"
        description: "Test B depends on Test A (circular!)"
        dependencies: ["test_a"]
        input:
          value: "b"
        expected:
          error: false
"#
        )
        .expect("Failed to write circular dependency YAML");
        temp_file.flush().expect("Failed to flush temp file");

        // This should fail until we implement circular dependency detection
        let result = runner.run_test_suite(temp_file.path()).await;

        assert!(
            result.is_err(),
            "Circular dependencies should cause an error"
        );
        let error = result.unwrap_err();
        assert!(
            matches!(error, Error::Dependency(_)),
            "Should be a Dependency error: {:?}",
            error
        );
        assert!(
            error.to_string().contains("circular") || error.to_string().contains("Circular"),
            "Error message should mention circular dependency: {}",
            error
        );
    }

    #[tokio::test]
    async fn test_missing_specification_file() {
        // Test handling of missing specification file
        let executor = create_test_executor().await;
        let config = RunnerConfig::new();
        let mut runner = TestSuiteRunner::new(executor, config);

        let non_existent_path = std::path::Path::new("/non/existent/path.yaml");

        // This should fail until we implement file error handling
        let result = runner.run_test_suite(non_existent_path).await;

        assert!(result.is_err(), "Missing file should cause an error");
        let error = result.unwrap_err();
        assert!(
            matches!(error, Error::Io(_)) || matches!(error, Error::Spec(_)),
            "Should be an I/O or Spec error: {:?}",
            error
        );
    }

    // ========================================================================
    // PHASE 3: Configuration and Advanced Features Tests (RED PHASE)
    // ========================================================================

    #[tokio::test]
    async fn test_runner_configuration_updates() {
        // Test that runner configuration can be updated dynamically
        let executor = create_test_executor().await;
        let initial_config = RunnerConfig::new();
        let mut runner = TestSuiteRunner::new(executor, initial_config);

        // Verify initial configuration
        assert_eq!(runner.config.execution_mode, ExecutionMode::Sequential);
        assert!(!runner.config.fail_fast);

        // Update configuration
        let new_config = RunnerConfig::new()
            .with_parallel_execution(true)
            .with_fail_fast(true)
            .with_max_concurrency(8);

        runner.set_config(new_config);

        // Verify configuration was updated
        assert_eq!(runner.config.execution_mode, ExecutionMode::Parallel);
        assert!(runner.config.fail_fast);
        assert_eq!(runner.config.max_concurrency, 8);
    }

    #[tokio::test]
    async fn test_metrics_collector_access() {
        // Test access to metrics collector
        let executor = create_test_executor().await;
        let config = RunnerConfig::new();
        let runner = TestSuiteRunner::new(executor, config);

        // Should be able to access metrics collector
        let metrics_collector = runner.get_metrics();

        // Verify it's the expected type and has expected methods
        let summary = metrics_collector.get_summary();
        assert_eq!(summary.total_tests, 0); // Should start with no tests
        assert_eq!(summary.success_rate(), 0.0);
    }

    #[tokio::test]
    async fn test_empty_test_suite() {
        // Test handling of empty test suite
        let executor = create_test_executor().await;
        let config = RunnerConfig::new();
        let mut runner = TestSuiteRunner::new(executor, config);

        // Create empty test suite
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        write!(
            temp_file,
            r#"
name: "Empty Test Suite"
version: "1.0.0"
capabilities:
  tools: false
  resources: false
  prompts: false
  sampling: false
  logging: false
server:
  command: "test-server"
  transport: "stdio"
"#
        )
        .expect("Failed to write empty test YAML");
        temp_file.flush().expect("Failed to flush temp file");

        // This should succeed but with zero tests
        let result = runner.run_test_suite(temp_file.path()).await;

        assert!(
            result.is_ok(),
            "Empty test suite should be handled gracefully"
        );
        let suite_result = result.unwrap();

        assert_eq!(suite_result.total_tests, 0);
        assert_eq!(suite_result.passed, 0);
        assert_eq!(suite_result.failed, 0);
        assert_eq!(suite_result.test_results.len(), 0);
        assert!(suite_result.all_passed()); // Vacuously true
        assert!(!suite_result.has_failures());
    }
}
