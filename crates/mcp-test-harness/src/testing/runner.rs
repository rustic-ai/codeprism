//! Test execution runner for MCP test harness

use crate::spec::schema::{ServerConfig, TestCase};
use crate::testing::result::TestResult;
use crate::transport::{create_transport, Transport, TransportType};
use crate::types::{McpCapabilities, PerformanceMetrics, RetryConfig, ValidationResult};
use anyhow::Result;
use chrono::Utc;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, instrument, warn};

/// Advanced execution configuration for test runner
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Maximum number of concurrent test executions
    pub max_concurrency: usize,
    /// Connection pool size for reusing server connections
    pub connection_pool_size: usize,
    /// Default timeout for test operations
    pub default_timeout: Duration,
    /// Timeout for connection operations
    pub connection_timeout: Duration,
    /// Timeout for individual message operations
    pub message_timeout: Duration,
    /// Retry configuration for failed operations
    pub retry_config: RetryConfig,
    /// Whether to isolate tests (fresh connection per test)
    pub isolation_mode: TestIsolationMode,
}

/// Test isolation strategies
#[derive(Debug, Clone, PartialEq)]
pub enum TestIsolationMode {
    /// Each test gets a fresh server connection (maximum isolation)
    PerTest,
    /// Tests share connections from a pool (better performance)
    Shared,
    /// All tests use a single connection (fastest, least isolation)
    SingleConnection,
}

/// Connection pool for managing MCP server connections
pub struct ConnectionPool {
    pool: Arc<Mutex<Vec<Box<dyn Transport>>>>,
    server_config: ServerConfig,
    max_size: usize,
    transport_type: TransportType,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub async fn new(server_config: ServerConfig, max_size: usize) -> Result<Self> {
        let transport_type = match server_config.transport.as_str() {
            "stdio" => TransportType::Stdio,
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported transport type: {}",
                    server_config.transport
                ))
            }
        };

        let pool = Arc::new(Mutex::new(Vec::new()));

        Ok(Self {
            pool,
            server_config,
            max_size,
            transport_type,
        })
    }

    /// Get a connection from the pool or create a new one
    pub async fn get_connection(&self) -> Result<Box<dyn Transport>> {
        // Try to get existing connection from pool
        if let Some(connection) = self.pool.lock().await.pop() {
            if connection.is_connected() {
                debug!("Reusing existing connection from pool");
                return Ok(connection);
            }
        }

        // Create new connection
        debug!("Creating new connection for pool");
        let mut transport = create_transport(self.transport_type.clone())?;

        // Configure stdio transport with server details
        if let TransportType::Stdio = self.transport_type {
            // For stdio transport, we need to configure it with server command
            // This will be done through the connect method
        }

        transport.connect().await?;
        Ok(transport)
    }

    /// Return a connection to the pool
    pub async fn return_connection(&self, connection: Box<dyn Transport>) {
        let mut pool = self.pool.lock().await;
        if pool.len() < self.max_size && connection.is_connected() {
            pool.push(connection);
            debug!("Connection returned to pool");
        } else {
            debug!("Connection not returned to pool (full or disconnected)");
        }
    }

    /// Close all connections in the pool
    pub async fn close_all(&self) -> Result<()> {
        let mut pool = self.pool.lock().await;
        for mut connection in pool.drain(..) {
            if let Err(e) = connection.disconnect().await {
                warn!("Error closing pooled connection: {}", e);
            }
        }
        info!("All pooled connections closed");
        Ok(())
    }
}

impl std::fmt::Debug for ConnectionPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionPool")
            .field("max_size", &self.max_size)
            .field("transport_type", &self.transport_type)
            .field("server_config", &self.server_config)
            .field("pool_size", &format!("<{} connections>", self.max_size))
            .finish()
    }
}

/// Enhanced test execution engine with comprehensive MCP server communication
#[derive(Debug)]
pub struct TestRunner {
    config: ExecutionConfig,
    connection_pool: Option<Arc<ConnectionPool>>,
    concurrency_limiter: Arc<Semaphore>,
    metrics: Arc<Mutex<ExecutionMetrics>>,
}

/// Execution metrics tracking
#[derive(Debug, Default)]
pub struct ExecutionMetrics {
    pub total_tests_executed: usize,
    pub total_execution_time: Duration,
    pub connection_pool_hits: usize,
    pub connection_pool_misses: usize,
    pub retry_attempts: usize,
    pub protocol_errors: usize,
}

impl TestRunner {
    /// Create a new test runner with default configuration
    pub fn new() -> Self {
        Self::with_config(ExecutionConfig::default())
    }

    /// Create a new test runner with custom configuration
    pub fn with_config(config: ExecutionConfig) -> Self {
        let concurrency_limiter = Arc::new(Semaphore::new(config.max_concurrency));

        Self {
            config,
            connection_pool: None,
            concurrency_limiter,
            metrics: Arc::new(Mutex::new(ExecutionMetrics::default())),
        }
    }

    /// Initialize the test runner with server configuration
    pub async fn initialize(&mut self, server_config: ServerConfig) -> Result<()> {
        let pool = ConnectionPool::new(server_config, self.config.connection_pool_size).await?;

        self.connection_pool = Some(Arc::new(pool));
        info!("TestRunner initialized with connection pool");
        Ok(())
    }

    /// Execute a single test case with comprehensive error handling and validation
    #[instrument(skip(self, test_case), fields(test_name = %test_case.name))]
    pub async fn execute_test(&self, test_case: &TestCase) -> Result<TestResult> {
        let _permit = self.concurrency_limiter.acquire().await?;
        let start_time = Utc::now();
        let execution_start = Instant::now();

        info!("Executing test case: {}", test_case.name);

        match self.execute_test_with_retries(test_case).await {
            Ok(mut result) => {
                let duration = execution_start.elapsed();
                result.duration = duration;
                result.start_time = start_time;

                // Update metrics
                self.update_metrics(duration, true).await;

                info!(
                    "Test case '{}' completed successfully in {:?}",
                    test_case.name, duration
                );
                Ok(result)
            }
            Err(e) => {
                let duration = execution_start.elapsed();
                self.update_metrics(duration, false).await;

                error!(
                    "Test case '{}' failed after {:?}: {}",
                    test_case.name, duration, e
                );
                Ok(TestResult::failure(
                    test_case.name.clone(),
                    start_time,
                    duration,
                    test_case.input.clone(),
                    e.to_string(),
                ))
            }
        }
    }

    /// Execute test with retry logic
    async fn execute_test_with_retries(&self, test_case: &TestCase) -> Result<TestResult> {
        let mut attempts = 0;
        let max_attempts = self.config.retry_config.max_retries + 1;

        loop {
            attempts += 1;

            match self.execute_test_once(test_case).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempts >= max_attempts {
                        return Err(e);
                    }

                    // Check if error is retryable
                    if !self.is_retryable_error(&e) {
                        return Err(e);
                    }

                    // Calculate delay with exponential backoff
                    let delay = if self.config.retry_config.exponential_backoff {
                        Duration::from_millis(
                            self.config.retry_config.retry_delay_ms
                                * (2_u64.pow((attempts - 1) as u32)),
                        )
                    } else {
                        Duration::from_millis(self.config.retry_config.retry_delay_ms)
                    };

                    warn!(
                        "Test attempt {} failed, retrying in {:?}: {}",
                        attempts, delay, e
                    );

                    // Update retry metrics
                    self.update_retry_metrics().await;

                    sleep(delay).await;
                }
            }
        }
    }

    /// Execute a single test attempt
    async fn execute_test_once(&self, test_case: &TestCase) -> Result<TestResult> {
        let start_time = Utc::now();
        let execution_start = Instant::now();

        // Get connection based on isolation mode
        let mut connection = self.get_connection().await?;

        // Perform MCP handshake if needed
        self.ensure_mcp_handshake(&mut connection).await?;

        // Execute the actual test
        let response = self.execute_mcp_request(&mut connection, test_case).await?;

        // Validate the response
        let validation = self.validate_response(test_case, &response).await?;

        // Return connection to pool if using shared connections
        if matches!(self.config.isolation_mode, TestIsolationMode::Shared) {
            if let Some(pool) = &self.connection_pool {
                pool.return_connection(connection).await;
            }
        }

        // Build performance metrics
        let performance = self.collect_performance_metrics(&execution_start).await;

        let result = TestResult::success(
            test_case.name.clone(),
            start_time,
            execution_start.elapsed(),
            test_case.input.clone(),
            response,
        )
        .with_performance(performance)
        .with_tags(test_case.tags.clone());

        // Update validation result
        Ok(TestResult {
            validation,
            ..result
        })
    }

    /// Get a connection based on the isolation mode
    async fn get_connection(&self) -> Result<Box<dyn Transport>> {
        match self.config.isolation_mode {
            TestIsolationMode::PerTest => {
                // Create fresh connection for each test
                let transport_type = TransportType::Stdio; // Default for now
                let mut connection = create_transport(transport_type)?;
                connection.connect().await?;
                Ok(connection)
            }
            TestIsolationMode::Shared | TestIsolationMode::SingleConnection => {
                // Use connection pool
                if let Some(pool) = &self.connection_pool {
                    pool.get_connection().await
                } else {
                    Err(anyhow::anyhow!("Connection pool not initialized"))
                }
            }
        }
    }

    /// Ensure MCP handshake has been performed
    async fn ensure_mcp_handshake(
        &self,
        connection: &mut Box<dyn Transport>,
    ) -> Result<McpCapabilities> {
        debug!("Performing MCP handshake");

        // Send initialize request
        let initialize_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "roots": {
                        "listChanged": true
                    },
                    "sampling": {}
                },
                "clientInfo": {
                    "name": "mcp-test-harness",
                    "version": "0.1.0"
                }
            }
        });

        // Send with timeout
        let response = timeout(
            self.config.message_timeout,
            self.send_and_receive(connection, initialize_request),
        )
        .await??;

        // Validate initialize response
        let capabilities = self.parse_initialize_response(&response)?;

        // Send initialized notification
        let initialized_notification = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        });

        connection.send(initialized_notification).await?;

        debug!("MCP handshake completed successfully");
        Ok(capabilities)
    }

    /// Execute an MCP request and return the response
    async fn execute_mcp_request(
        &self,
        connection: &mut Box<dyn Transport>,
        test_case: &TestCase,
    ) -> Result<Value> {
        debug!("Executing MCP request for test: {}", test_case.name);

        // Create JSON-RPC request based on test case input
        let request = self.build_mcp_request(test_case)?;

        // Send request and receive response with timeout
        let response = timeout(
            self.config.default_timeout,
            self.send_and_receive(connection, request),
        )
        .await??;

        debug!("Received MCP response for test: {}", test_case.name);
        Ok(response)
    }

    /// Send a request and receive a response
    async fn send_and_receive(
        &self,
        connection: &mut Box<dyn Transport>,
        request: Value,
    ) -> Result<Value> {
        connection.send(request).await?;
        let response = connection.receive().await?;
        Ok(response)
    }

    /// Build MCP request from test case
    fn build_mcp_request(&self, test_case: &TestCase) -> Result<Value> {
        // Extract method and parameters from test case input
        let method = test_case
            .input
            .get("method")
            .and_then(|m| m.as_str())
            .ok_or_else(|| anyhow::anyhow!("Test case missing 'method' field"))?;

        let params = test_case.input.get("params").cloned().unwrap_or(json!({}));

        Ok(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        }))
    }

    /// Parse initialize response and extract capabilities
    fn parse_initialize_response(&self, response: &Value) -> Result<McpCapabilities> {
        let result = response
            .get("result")
            .ok_or_else(|| anyhow::anyhow!("Initialize response missing 'result' field"))?;

        let capabilities = result
            .get("capabilities")
            .ok_or_else(|| anyhow::anyhow!("Initialize result missing 'capabilities'"))?;

        Ok(McpCapabilities {
            tools: capabilities.get("tools").is_some(),
            resources: capabilities.get("resources").is_some(),
            prompts: capabilities.get("prompts").is_some(),
            sampling: capabilities.get("sampling").is_some(),
            logging: capabilities.get("logging").is_some(),
            experimental: None,
        })
    }

    /// Validate test response against expected output
    async fn validate_response(
        &self,
        test_case: &TestCase,
        response: &Value,
    ) -> Result<ValidationResult> {
        let mut validation = ValidationResult::success();

        // Check if response is error when error is expected
        if test_case.expected.error {
            if response.get("error").is_none() {
                validation = ValidationResult::error("Expected error response but got success");
            }
        } else if response.get("error").is_some() {
            validation = ValidationResult::error("Unexpected error response");
        }

        // Validate specific fields if configured
        for field_validation in &test_case.expected.fields {
            if let Err(e) = self.validate_field(response, field_validation) {
                validation = ValidationResult::error(format!("Field validation failed: {}", e));
                break;
            }
        }

        // Additional JSON-RPC validation
        if response
            .get("jsonrpc")
            .is_none_or(|v| v.as_str() != Some("2.0"))
        {
            validation = validation.with_warning("Response missing or invalid JSON-RPC version");
        }

        Ok(validation)
    }

    /// Validate a specific field in the response
    fn validate_field(
        &self,
        response: &Value,
        _field_validation: &crate::spec::schema::FieldValidation,
    ) -> Result<()> {
        // FUTURE: Implement JSONPath-based field validation using jsonpath_lib crate
        // Basic validation: ensure response is valid non-null JSON
        if response.is_null() {
            return Err(anyhow::anyhow!("Response is null"));
        }

        // Validate response structure based on field validation requirements
        if let Some(expected_value) = &_field_validation.value {
            if response != expected_value {
                return Err(anyhow::anyhow!("Response value mismatch"));
            }
        }

        Ok(())
    }

    /// Collect performance metrics for the test execution
    async fn collect_performance_metrics(&self, start_time: &Instant) -> PerformanceMetrics {
        let execution_time = start_time.elapsed();

        PerformanceMetrics {
            response_time_ms: execution_time.as_millis() as u64,
            memory_usage_bytes: 0, // FUTURE: Implement actual memory monitoring using sysinfo crate
            cpu_usage_percent: 0.0, // FUTURE: Implement CPU monitoring using sysinfo crate
            network_latency_ms: None,
            throughput_ops_per_sec: None,
        }
    }

    /// Check if an error is retryable based on configuration
    fn is_retryable_error(&self, error: &anyhow::Error) -> bool {
        let error_msg = error.to_string().to_lowercase();

        self.config
            .retry_config
            .retry_on_patterns
            .iter()
            .any(|pattern| error_msg.contains(&pattern.to_lowercase()))
    }

    /// Update execution metrics
    async fn update_metrics(&self, duration: Duration, success: bool) {
        let mut metrics = self.metrics.lock().await;
        metrics.total_tests_executed += 1;
        metrics.total_execution_time += duration;

        if !success {
            metrics.protocol_errors += 1;
        }
    }

    /// Update retry metrics
    async fn update_retry_metrics(&self) {
        let mut metrics = self.metrics.lock().await;
        metrics.retry_attempts += 1;
    }

    /// Execute protocol-only tests for basic MCP compliance
    pub async fn execute_protocol_tests(
        &self,
        server_config: &ServerConfig,
    ) -> Result<Vec<TestResult>> {
        info!("Executing protocol compliance tests");

        let mut results = Vec::new();

        // Test 1: Basic connection
        results.push(self.test_basic_connection(server_config).await?);

        // Test 2: Initialize handshake
        results.push(self.test_initialize_handshake(server_config).await?);

        // Test 3: Invalid request handling
        results.push(self.test_invalid_request_handling(server_config).await?);

        info!(
            "Protocol compliance tests completed: {}/{} passed",
            results.iter().filter(|r| r.passed).count(),
            results.len()
        );

        Ok(results)
    }

    /// Test basic connection to MCP server
    async fn test_basic_connection(&self, _server_config: &ServerConfig) -> Result<TestResult> {
        let start_time = Utc::now();
        let execution_start = Instant::now();

        match self.get_connection().await {
            Ok(_connection) => Ok(TestResult::success(
                "protocol_basic_connection".to_string(),
                start_time,
                execution_start.elapsed(),
                json!({}),
                json!({"status": "connected"}),
            )),
            Err(e) => Ok(TestResult::failure(
                "protocol_basic_connection".to_string(),
                start_time,
                execution_start.elapsed(),
                json!({}),
                format!("Connection failed: {}", e),
            )),
        }
    }

    /// Test MCP initialize handshake
    async fn test_initialize_handshake(&self, _server_config: &ServerConfig) -> Result<TestResult> {
        let start_time = Utc::now();
        let execution_start = Instant::now();

        match self.get_connection().await {
            Ok(mut connection) => match self.ensure_mcp_handshake(&mut connection).await {
                Ok(capabilities) => Ok(TestResult::success(
                    "protocol_initialize_handshake".to_string(),
                    start_time,
                    execution_start.elapsed(),
                    json!({}),
                    json!({"capabilities": capabilities}),
                )),
                Err(e) => Ok(TestResult::failure(
                    "protocol_initialize_handshake".to_string(),
                    start_time,
                    execution_start.elapsed(),
                    json!({}),
                    format!("Handshake failed: {}", e),
                )),
            },
            Err(e) => Ok(TestResult::failure(
                "protocol_initialize_handshake".to_string(),
                start_time,
                execution_start.elapsed(),
                json!({}),
                format!("Connection failed: {}", e),
            )),
        }
    }

    /// Test invalid request handling
    async fn test_invalid_request_handling(
        &self,
        _server_config: &ServerConfig,
    ) -> Result<TestResult> {
        let start_time = Utc::now();
        let execution_start = Instant::now();

        match self.get_connection().await {
            Ok(mut connection) => {
                // Send invalid JSON-RPC request
                let invalid_request = json!({
                    "jsonrpc": "1.0", // Invalid version
                    "method": "nonexistent_method",
                    "id": 1
                });

                match self
                    .send_and_receive(&mut connection, invalid_request)
                    .await
                {
                    Ok(response) => {
                        // Should receive an error response
                        if response.get("error").is_some() {
                            Ok(TestResult::success(
                                "protocol_invalid_request_handling".to_string(),
                                start_time,
                                execution_start.elapsed(),
                                json!({}),
                                response,
                            ))
                        } else {
                            Ok(TestResult::failure(
                                "protocol_invalid_request_handling".to_string(),
                                start_time,
                                execution_start.elapsed(),
                                json!({}),
                                "Server should return error for invalid request".to_string(),
                            ))
                        }
                    }
                    Err(e) => Ok(TestResult::failure(
                        "protocol_invalid_request_handling".to_string(),
                        start_time,
                        execution_start.elapsed(),
                        json!({}),
                        format!("Request failed: {}", e),
                    )),
                }
            }
            Err(e) => Ok(TestResult::failure(
                "protocol_invalid_request_handling".to_string(),
                start_time,
                execution_start.elapsed(),
                json!({}),
                format!("Connection failed: {}", e),
            )),
        }
    }

    /// Get execution metrics
    pub async fn get_metrics(&self) -> ExecutionMetrics {
        self.metrics.lock().await.clone()
    }

    /// Cleanup resources
    pub async fn cleanup(&self) -> Result<()> {
        if let Some(pool) = &self.connection_pool {
            pool.close_all().await?;
        }
        info!("TestRunner cleanup completed");
        Ok(())
    }
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 4,
            connection_pool_size: 8,
            default_timeout: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            message_timeout: Duration::from_secs(5),
            retry_config: RetryConfig::default(),
            isolation_mode: TestIsolationMode::Shared,
        }
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ExecutionMetrics {
    fn clone(&self) -> Self {
        Self {
            total_tests_executed: self.total_tests_executed,
            total_execution_time: self.total_execution_time,
            connection_pool_hits: self.connection_pool_hits,
            connection_pool_misses: self.connection_pool_misses,
            retry_attempts: self.retry_attempts,
            protocol_errors: self.protocol_errors,
        }
    }
}
