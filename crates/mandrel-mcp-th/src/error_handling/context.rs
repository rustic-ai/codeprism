//! Error context and debugging information for the Mandrel MCP Test Harness
//!
//! This module provides rich context information for errors, including test execution
//! context, server information, and debugging data for error analysis and resolution.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Comprehensive error context for debugging and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Unique identifier for this error occurrence
    pub error_id: Uuid,
    /// Timestamp when the error occurred
    pub timestamp: DateTime<Utc>,
    /// Test execution context if available
    pub test_context: Option<TestExecutionContext>,
    /// MCP server context if available
    pub server_context: Option<McpServerContext>,
    /// Operation being performed when error occurred
    pub operation: String,
    /// Trace and span information for distributed tracing
    pub trace_info: Option<TraceInfo>,
    /// User-defined context data
    pub user_data: HashMap<String, serde_json::Value>,
    /// Environment information
    pub environment: EnvironmentInfo,
    /// Performance metrics at time of error
    pub performance_snapshot: Option<PerformanceSnapshot>,
}

/// Test execution context for test-related errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecutionContext {
    /// Name of the test being executed
    pub test_name: String,
    /// Test suite or group name
    pub test_suite: Option<String>,
    /// Current test step or phase
    pub current_step: u32,
    /// Total number of steps in the test
    pub total_steps: Option<u32>,
    /// Test execution start time
    pub started_at: DateTime<Utc>,
    /// Test configuration used
    pub test_config: HashMap<String, serde_json::Value>,
    /// Previous test results or state
    pub previous_results: Option<serde_json::Value>,
    /// Test isolation mode
    pub isolation_mode: Option<String>,
    /// Test retry information
    pub retry_info: Option<RetryInfo>,
}

/// MCP server context for server-related errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerContext {
    /// Server name or identifier
    pub server_name: String,
    /// Server endpoint or connection details
    pub endpoint: String,
    /// Server version if known
    pub version: Option<String>,
    /// Server capabilities
    pub capabilities: Option<serde_json::Value>,
    /// Connection state at time of error
    pub connection_state: String,
    /// Last successful operation timestamp
    pub last_success: Option<DateTime<Utc>>,
    /// Number of active connections
    pub active_connections: Option<u32>,
    /// Transport type being used
    pub transport_type: String,
    /// Server process information if available
    pub process_info: Option<ProcessInfo>,
}

/// Distributed tracing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceInfo {
    /// Trace ID for distributed tracing
    pub trace_id: Option<String>,
    /// Span ID for the current operation
    pub span_id: Option<String>,
    /// Parent span ID if this is a child span
    pub parent_span_id: Option<String>,
    /// Baggage or additional trace context
    pub baggage: HashMap<String, String>,
    /// Sampling decision
    pub sampled: bool,
}

/// Environment information at time of error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    /// Operating system name and version
    pub os_info: String,
    /// Rust version
    pub rust_version: String,
    /// Test harness version
    pub harness_version: String,
    /// Environment variables (filtered for security)
    pub env_vars: HashMap<String, String>,
    /// Current working directory
    pub working_directory: String,
    /// System resource information
    pub system_resources: SystemResources,
}

/// System resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    /// Available memory in MB
    pub available_memory_mb: Option<u64>,
    /// Total memory in MB
    pub total_memory_mb: Option<u64>,
    /// CPU usage percentage
    pub cpu_usage_percent: Option<f64>,
    /// Disk space available in MB
    pub disk_space_mb: Option<u64>,
    /// Number of open file descriptors
    pub open_file_descriptors: Option<u32>,
    /// Network connectivity status
    pub network_status: Option<String>,
}

/// Performance snapshot at time of error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    /// Operation duration up to the error
    pub operation_duration_ms: u64,
    /// Memory usage at time of error
    pub memory_usage_mb: Option<u64>,
    /// CPU time consumed
    pub cpu_time_ms: Option<u64>,
    /// I/O operations performed
    pub io_operations: Option<u64>,
    /// Network bytes transferred
    pub network_bytes: Option<u64>,
    /// Garbage collection statistics
    pub gc_stats: Option<serde_json::Value>,
}

/// Retry attempt information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryInfo {
    /// Current retry attempt number
    pub attempt: u32,
    /// Maximum retry attempts allowed
    pub max_attempts: u32,
    /// Previous attempt timestamps
    pub previous_attempts: Vec<DateTime<Utc>>,
    /// Delay between retries in milliseconds
    pub retry_delay_ms: u64,
    /// Reason for retrying
    pub retry_reason: String,
}

/// Process information for server processes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: Option<u32>,
    /// Process start time
    pub started_at: Option<DateTime<Utc>>,
    /// Process command line
    pub command_line: Option<String>,
    /// Working directory of the process
    pub working_directory: Option<String>,
    /// Process status
    pub status: Option<String>,
    /// Memory usage of the process
    pub memory_usage_mb: Option<u64>,
    /// CPU usage of the process
    pub cpu_usage_percent: Option<f64>,
}

impl ErrorContext {
    /// Create a new error context with minimal information
    pub fn new(operation: String) -> Self {
        Self {
            error_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            test_context: None,
            server_context: None,
            operation,
            trace_info: None,
            user_data: HashMap::new(),
            environment: EnvironmentInfo::current(),
            performance_snapshot: None,
        }
    }

    /// Create error context for a test execution
    pub fn for_test(test_name: String, operation: String) -> Self {
        let mut context = Self::new(operation);
        context.test_context = Some(TestExecutionContext {
            test_name,
            test_suite: None,
            current_step: 0,
            total_steps: None,
            started_at: Utc::now(),
            test_config: HashMap::new(),
            previous_results: None,
            isolation_mode: None,
            retry_info: None,
        });
        context
    }

    /// Create error context for MCP server operations
    pub fn for_server(server_name: String, endpoint: String, operation: String) -> Self {
        let mut context = Self::new(operation);
        context.server_context = Some(McpServerContext {
            server_name,
            endpoint,
            version: None,
            capabilities: None,
            connection_state: "unknown".to_string(),
            last_success: None,
            active_connections: None,
            transport_type: "unknown".to_string(),
            process_info: None,
        });
        context
    }

    /// Add user-defined context data
    pub fn with_user_data(mut self, key: String, value: serde_json::Value) -> Self {
        self.user_data.insert(key, value);
        self
    }

    /// Add test configuration to context
    pub fn with_test_config(mut self, config: HashMap<String, serde_json::Value>) -> Self {
        if let Some(ref mut test_ctx) = self.test_context {
            test_ctx.test_config = config;
        }
        self
    }

    /// Add server capabilities to context
    pub fn with_server_capabilities(mut self, capabilities: serde_json::Value) -> Self {
        if let Some(ref mut server_ctx) = self.server_context {
            server_ctx.capabilities = Some(capabilities);
        }
        self
    }

    /// Add trace information to context
    pub fn with_trace_info(mut self, trace_id: String, span_id: String) -> Self {
        self.trace_info = Some(TraceInfo {
            trace_id: Some(trace_id),
            span_id: Some(span_id),
            parent_span_id: None,
            baggage: HashMap::new(),
            sampled: true,
        });
        self
    }

    /// Add performance snapshot to context
    pub fn with_performance_snapshot(mut self, snapshot: PerformanceSnapshot) -> Self {
        self.performance_snapshot = Some(snapshot);
        self
    }

    /// Generate a concise summary of the error context
    pub fn summary(&self) -> String {
        let mut parts = vec![
            format!("operation: {}", self.operation),
            format!("error_id: {}", self.error_id),
        ];

        if let Some(ref test_ctx) = self.test_context {
            parts.push(format!("test: {}", test_ctx.test_name));
            parts.push(format!("step: {}", test_ctx.current_step));
        }

        if let Some(ref server_ctx) = self.server_context {
            parts.push(format!("server: {}", server_ctx.server_name));
            parts.push(format!("endpoint: {}", server_ctx.endpoint));
        }

        if let Some(ref trace) = self.trace_info {
            if let Some(ref trace_id) = trace.trace_id {
                parts.push(format!("trace_id: {}", trace_id));
            }
        }

        parts.join(", ")
    }

    /// Check if this context indicates a retry scenario
    pub fn is_retry_context(&self) -> bool {
        self.test_context
            .as_ref()
            .and_then(|ctx| ctx.retry_info.as_ref())
            .map(|retry| retry.attempt > 1)
            .unwrap_or(false)
    }

    /// Get the current retry attempt number
    pub fn retry_attempt(&self) -> Option<u32> {
        self.test_context
            .as_ref()
            .and_then(|ctx| ctx.retry_info.as_ref())
            .map(|retry| retry.attempt)
    }
}

impl EnvironmentInfo {
    /// Capture current environment information
    pub fn current() -> Self {
        Self {
            os_info: std::env::consts::OS.to_string(),
            rust_version: env!("CARGO_PKG_RUST_VERSION").to_string(),
            harness_version: env!("CARGO_PKG_VERSION").to_string(),
            env_vars: Self::safe_env_vars(),
            working_directory: std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            system_resources: SystemResources::current(),
        }
    }

    /// Get safe environment variables (excluding sensitive ones)
    fn safe_env_vars() -> HashMap<String, String> {
        let safe_prefixes = ["PATH", "RUST_", "CARGO_", "PWD", "HOME", "USER"];
        let sensitive_patterns = ["PASSWORD", "SECRET", "KEY", "TOKEN", "CREDENTIAL"];

        std::env::vars()
            .filter(|(key, _)| {
                // Include if it starts with a safe prefix
                safe_prefixes.iter().any(|prefix| key.starts_with(prefix)) &&
                // But exclude if it contains sensitive patterns
                !sensitive_patterns.iter().any(|pattern| key.to_uppercase().contains(pattern))
            })
            .collect()
    }
}

impl SystemResources {
    /// Capture current system resource information
    pub fn current() -> Self {
        Self {
            available_memory_mb: None, // Would need system-specific implementation
            total_memory_mb: None,
            cpu_usage_percent: None,
            disk_space_mb: None,
            open_file_descriptors: None,
            network_status: Some("unknown".to_string()),
        }
    }
}

impl PerformanceSnapshot {
    /// Create a performance snapshot for the current moment
    pub fn capture(operation_start: DateTime<Utc>) -> Self {
        let duration = Utc::now().signed_duration_since(operation_start);

        Self {
            operation_duration_ms: duration.num_milliseconds().max(0) as u64,
            memory_usage_mb: None, // Would need memory tracking implementation
            cpu_time_ms: None,
            io_operations: None,
            network_bytes: None,
            gc_stats: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_creation() {
        let context = ErrorContext::new("test_operation".to_string());

        assert_eq!(context.operation, "test_operation");
        assert!(!context.error_id.is_nil());
        assert!(context.test_context.is_none());
        assert!(context.server_context.is_none());
    }

    #[test]
    fn test_error_context_for_test() {
        let context =
            ErrorContext::for_test("test_example".to_string(), "execute_test".to_string());

        assert_eq!(context.operation, "execute_test");
        assert!(context.test_context.is_some());

        let test_ctx = context.test_context.unwrap();
        assert_eq!(test_ctx.test_name, "test_example");
        assert_eq!(test_ctx.current_step, 0);
    }

    #[test]
    fn test_error_context_for_server() {
        let context = ErrorContext::for_server(
            "test-server".to_string(),
            "stdio://path/to/server".to_string(),
            "tools/list".to_string(),
        );

        assert_eq!(context.operation, "tools/list");
        assert!(context.server_context.is_some());

        let server_ctx = context.server_context.unwrap();
        assert_eq!(server_ctx.server_name, "test-server");
        assert_eq!(server_ctx.endpoint, "stdio://path/to/server");
    }

    #[test]
    fn test_error_context_with_user_data() {
        let context = ErrorContext::new("test_op".to_string()).with_user_data(
            "custom_field".to_string(),
            serde_json::json!("custom_value"),
        );

        assert_eq!(
            context.user_data.get("custom_field"),
            Some(&serde_json::json!("custom_value"))
        );
    }

    #[test]
    fn test_error_context_with_trace_info() {
        let context = ErrorContext::new("test_op".to_string())
            .with_trace_info("trace123".to_string(), "span456".to_string());

        assert!(context.trace_info.is_some());
        let trace = context.trace_info.unwrap();
        assert_eq!(trace.trace_id, Some("trace123".to_string()));
        assert_eq!(trace.span_id, Some("span456".to_string()));
    }

    #[test]
    fn test_error_context_summary() {
        let context =
            ErrorContext::for_test("test_example".to_string(), "execute_test".to_string())
                .with_trace_info("trace123".to_string(), "span456".to_string());

        let summary = context.summary();
        assert!(summary.contains("execute_test"));
        assert!(summary.contains("test_example"));
        assert!(summary.contains("trace123"));
    }

    #[test]
    fn test_retry_context_detection() {
        let mut context =
            ErrorContext::for_test("test_retry".to_string(), "retry_operation".to_string());

        // Initially not a retry context
        assert!(!context.is_retry_context());
        assert!(context.retry_attempt().is_none());

        // Add retry information
        if let Some(ref mut test_ctx) = context.test_context {
            test_ctx.retry_info = Some(RetryInfo {
                attempt: 3,
                max_attempts: 5,
                previous_attempts: vec![Utc::now()],
                retry_delay_ms: 1000,
                retry_reason: "connection_failed".to_string(),
            });
        }

        assert!(context.is_retry_context());
        assert_eq!(context.retry_attempt(), Some(3));
    }

    #[test]
    fn test_environment_info_creation() {
        let env_info = EnvironmentInfo::current();

        assert!(!env_info.os_info.is_empty());
        assert!(!env_info.working_directory.is_empty());
        // Test that no sensitive variables are included
        for key in env_info.env_vars.keys() {
            assert!(!key.contains("PASSWORD"));
            assert!(!key.contains("SECRET"));
        }
    }

    #[test]
    fn test_performance_snapshot_capture() {
        let start_time = Utc::now() - chrono::Duration::milliseconds(500);
        let snapshot = PerformanceSnapshot::capture(start_time);

        assert!(snapshot.operation_duration_ms >= 500);
        assert!(snapshot.operation_duration_ms < 1000); // Should be reasonable
    }
}
