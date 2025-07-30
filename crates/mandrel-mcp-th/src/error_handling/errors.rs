//! Comprehensive error types for MOTH test harness
//!
//! This module implements the hierarchical error system with rich context and debugging
//! information as specified in the design document.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Result type alias for MOTH operations
pub type Result<T> = std::result::Result<T, TestHarnessError>;

/// Root error type for all test harness operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum TestHarnessError {
    #[error("MCP client error: {0}")]
    Client(#[from] McpClientError),

    #[error("Test execution error: {0}")]
    Execution(#[from] TestExecutionError),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigurationError),

    #[error("I/O error: {0}")]
    Io(#[from] IoError),

    #[error("Reporting error: {0}")]
    Reporting(#[from] ReportingError),

    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("Performance error: {0}")]
    Performance(#[from] PerformanceError),

    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
}

/// MCP client-specific errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum McpClientError {
    #[error("Connection failed: {message} (server: {server_name})")]
    ConnectionFailed {
        server_name: String,
        message: String,
        retry_count: u32,
        last_attempt: chrono::DateTime<chrono::Utc>,
        underlying_error: Option<String>,
    },

    #[error("Protocol violation: {message} (method: {method})")]
    ProtocolViolation {
        method: String,
        message: String,
        request_id: Option<String>,
        invalid_payload: Option<serde_json::Value>,
    },

    #[error("Request timeout: {method} took {duration_ms}ms (limit: {timeout_ms}ms)")]
    RequestTimeout {
        method: String,
        duration_ms: u64,
        timeout_ms: u64,
        request_id: Option<String>,
    },

    #[error("Server error: {error} (code: {code})")]
    ServerError {
        code: i32,
        error: String,
        data: Option<serde_json::Value>,
        method: Option<String>,
    },

    #[error("Transport error: {message}")]
    TransportError {
        message: String,
        recoverable: bool,
        transport_type: String,
    },

    #[error("Authentication error: {message}")]
    AuthenticationError {
        message: String,
        retry_allowed: bool,
        auth_type: String,
    },
}

/// Test execution errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum TestExecutionError {
    #[error("Test setup failed: {message} (test: {test_name})")]
    SetupFailed {
        test_name: String,
        message: String,
        phase: String,
    },

    #[error("Test assertion failed: {message} (test: {test_name}, step: {step})")]
    AssertionFailed {
        test_name: String,
        step: u32,
        message: String,
        expected: Option<serde_json::Value>,
        actual: Option<serde_json::Value>,
    },

    #[error("Test timeout: {test_name} exceeded {timeout_seconds}s")]
    TestTimeout {
        test_name: String,
        timeout_seconds: u64,
        elapsed_seconds: u64,
    },

    #[error("Test dependency failed: {test_name} depends on {dependency}")]
    DependencyFailed {
        test_name: String,
        dependency: String,
        dependency_error: String,
    },

    #[error("Script execution failed: {message} (test: {test_name})")]
    ScriptExecutionFailed {
        test_name: String,
        script_name: String,
        message: String,
        exit_code: Option<i32>,
    },
}

/// Validation errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    #[error("Schema validation failed: {message} (path: {path})")]
    SchemaValidation {
        path: String,
        message: String,
        expected_schema: Option<String>,
        actual_value: Option<serde_json::Value>,
    },

    #[error("JSONPath validation failed: {message} (path: {path})")]
    JsonPathValidation {
        path: String,
        message: String,
        expected: Option<serde_json::Value>,
        actual: Option<serde_json::Value>,
    },

    #[error("Response format invalid: {message}")]
    ResponseFormat {
        message: String,
        response_type: String,
        validation_rule: String,
    },
}

/// Configuration errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ConfigurationError {
    #[error("Invalid configuration: {message} (field: {field})")]
    InvalidConfig {
        field: String,
        message: String,
        provided_value: Option<String>,
    },

    #[error("Missing configuration: {field} is required")]
    MissingConfig {
        field: String,
        config_file: Option<String>,
    },

    #[error("Configuration parsing failed: {message}")]
    ParsingFailed {
        message: String,
        file_path: String,
        line_number: Option<u32>,
    },
}

/// I/O errors with additional context
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum IoError {
    #[error("File not found: {path}")]
    FileNotFound { path: String, operation: String },

    #[error("Permission denied: {path} (operation: {operation})")]
    PermissionDenied { path: String, operation: String },

    #[error("I/O operation failed: {message} (path: {path})")]
    OperationFailed {
        path: String,
        operation: String,
        message: String,
    },
}

/// Reporting errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ReportingError {
    #[error("Report generation failed: {message} (format: {format})")]
    GenerationFailed {
        format: String,
        message: String,
        output_path: Option<String>,
    },

    #[error("Template error: {message} (template: {template})")]
    TemplateError {
        template: String,
        message: String,
        line_number: Option<u32>,
    },
}

/// Network errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum NetworkError {
    #[error("Connection timeout: {endpoint} (timeout: {timeout_ms}ms)")]
    ConnectionTimeout { endpoint: String, timeout_ms: u64 },

    #[error("DNS resolution failed: {hostname}")]
    DnsResolutionFailed {
        hostname: String,
        error_message: String,
    },
}

/// Performance errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceError {
    #[error("Operation too slow: {operation} took {duration_ms}ms (limit: {limit_ms}ms)")]
    OperationTooSlow {
        operation: String,
        duration_ms: u64,
        limit_ms: u64,
    },

    #[error("Memory usage exceeded: {usage_mb}MB (limit: {limit_mb}MB)")]
    MemoryExceeded {
        usage_mb: u64,
        limit_mb: u64,
        operation: String,
    },
}

/// Security errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum SecurityError {
    #[error("Access denied: {message} (resource: {resource})")]
    AccessDenied {
        resource: String,
        message: String,
        required_permission: String,
    },

    #[error("Security policy violation: {message}")]
    PolicyViolation {
        policy: String,
        message: String,
        violation_type: String,
    },
}

/// Error context for additional debugging information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub test_name: Option<String>,
    pub server_name: Option<String>,
    pub operation: String,
    pub span_id: Option<String>,
    pub trace_id: Option<String>,
    pub user_data: HashMap<String, serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            test_name: None,
            server_name: None,
            operation: operation.into(),
            span_id: None,
            trace_id: None,
            user_data: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_test(mut self, test_name: impl Into<String>) -> Self {
        self.test_name = Some(test_name.into());
        self
    }

    pub fn with_server(mut self, server_name: impl Into<String>) -> Self {
        self.server_name = Some(server_name.into());
        self
    }

    pub fn with_span_id(mut self, span_id: impl Into<String>) -> Self {
        self.span_id = Some(span_id.into());
        self
    }

    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    pub fn add_data(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.user_data.insert(key.into(), value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_creation() {
        let context = ErrorContext::new("test_operation")
            .with_test("sample_test")
            .with_server("test_server");

        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.test_name, Some("sample_test".to_string()));
        assert_eq!(context.server_name, Some("test_server".to_string()));
    }

    #[test]
    fn test_error_context_with_data() {
        let mut context = ErrorContext::new("test_operation");
        context.add_data("key1", serde_json::json!("value1"));
        context.add_data("key2", serde_json::json!(42));

        assert_eq!(context.user_data.len(), 2, "Should have 2 items");
        assert_eq!(context.user_data["key1"], "value1");
        assert_eq!(context.user_data["key2"], 42);
    }

    #[test]
    fn test_mcp_client_error_serialization() {
        let error = McpClientError::ConnectionFailed {
            server_name: "test-server".to_string(),
            message: "Connection refused".to_string(),
            retry_count: 2,
            last_attempt: chrono::Utc::now(),
            underlying_error: Some("TCP connection failed".to_string()),
        };

        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: McpClientError = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            McpClientError::ConnectionFailed {
                server_name,
                retry_count,
                ..
            } => {
                assert_eq!(server_name, "test-server");
                assert_eq!(retry_count, 2);
            }
            _ => panic!("Unexpected error type after deserialization"),
        }
    }

    #[test]
    fn test_test_execution_error_display() {
        let error = TestExecutionError::AssertionFailed {
            test_name: "test_example".to_string(),
            step: 3,
            message: "Expected 'success' but got 'failure'".to_string(),
            expected: Some(serde_json::json!("success")),
            actual: Some(serde_json::json!("failure")),
        };

        let error_message = error.to_string();
        assert!(error_message.contains("test_example"));
        assert!(error_message.contains("step: 3"));
        assert!(error_message.contains("Expected 'success' but got 'failure'"));
    }

    #[test]
    fn test_validation_error_creation() {
        let error = ValidationError::SchemaValidation {
            path: "$.response.data".to_string(),
            message: "Expected string, got number".to_string(),
            expected_schema: Some("string".to_string()),
            actual_value: Some(serde_json::json!(42)),
        };

        assert!(error.to_string().contains("$.response.data"));
        assert!(error.to_string().contains("Expected string, got number"));
    }

    #[test]
    fn test_error_conversion() {
        let mcp_error = McpClientError::RequestTimeout {
            method: "tools/list".to_string(),
            duration_ms: 5000,
            timeout_ms: 3000,
            request_id: Some("req-123".to_string()),
        };

        let harness_error: TestHarnessError = mcp_error.into();
        match harness_error {
            TestHarnessError::Client(McpClientError::RequestTimeout { method, .. }) => {
                assert_eq!(method, "tools/list");
            }
            _ => panic!("Unexpected error type after conversion"),
        }
    }
}
