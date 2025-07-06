//! Error type definitions for the Mandrel MCP Test Harness
//!
//! This module defines a comprehensive hierarchy of error types using thiserror
//! for all components of the test harness, providing rich context and debugging information.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Root error type for all test harness operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum TestHarnessError {
    #[error("MCP client error: {0}")]
    Client(#[from] McpClientError),

    #[error("Test execution error: {0}")]
    Execution(#[from] TestExecutionError),

    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigurationError),

    #[error("I/O error: {0}")]
    Io(#[from] IoError),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Reporting error: {0}")]
    Reporting(#[from] ReportingError),

    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("Performance error: {0}")]
    Performance(#[from] PerformanceError),

    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
}

/// MCP client-specific errors with detailed context
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum McpClientError {
    #[error("Connection failed: {message} (server: {server_name}, attempts: {retry_count})")]
    ConnectionFailed {
        server_name: String,
        message: String,
        retry_count: u32,
        last_attempt: DateTime<Utc>,
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
        partial_response: Option<serde_json::Value>,
    },

    #[error("Server error: {error} (code: {code})")]
    ServerError {
        code: i32,
        error: String,
        data: Option<serde_json::Value>,
        method: Option<String>,
        request_id: Option<String>,
    },

    #[error("Transport error: {message} (transport: {transport_type})")]
    TransportError {
        transport_type: String,
        message: String,
        recoverable: bool,
        details: Option<serde_json::Value>,
    },

    #[error("Authentication failed: {message}")]
    AuthenticationError {
        message: String,
        auth_type: Option<String>,
        retry_allowed: bool,
    },
}

/// Test execution errors with rich context
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum TestExecutionError {
    #[error("Test setup failed: {message} (test: {test_name}, phase: {phase})")]
    SetupFailed {
        test_name: String,
        message: String,
        phase: String,
        component: Option<String>,
        suggestions: Vec<String>,
    },

    #[error("Test assertion failed: {message} (test: {test_name}, step: {step})")]
    AssertionFailed {
        test_name: String,
        step: u32,
        message: String,
        expected: Option<serde_json::Value>,
        actual: Option<serde_json::Value>,
        assertion_type: String,
        context: Option<serde_json::Value>,
    },

    #[error("Test timeout: {test_name} exceeded {timeout_seconds}s (elapsed: {elapsed_seconds}s)")]
    TestTimeout {
        test_name: String,
        timeout_seconds: u64,
        elapsed_seconds: u64,
        phase: String,
        partial_results: Option<serde_json::Value>,
    },

    #[error("Test prerequisite failed: {message} (test: {test_name})")]
    PrerequisiteFailed {
        test_name: String,
        prerequisite: String,
        message: String,
        dependency_chain: Vec<String>,
    },

    #[error("Test data corruption: {message} (test: {test_name})")]
    DataCorruption {
        test_name: String,
        message: String,
        corrupted_field: Option<String>,
        expected_checksum: Option<String>,
        actual_checksum: Option<String>,
    },
}

/// Configuration and setup errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ConfigurationError {
    #[error("Configuration file not found: {path}")]
    FileNotFound {
        path: PathBuf,
        searched_paths: Vec<PathBuf>,
        suggestions: Vec<String>,
    },

    #[error("Invalid configuration: {message} (field: {field})")]
    InvalidValue {
        field: String,
        message: String,
        provided_value: serde_json::Value,
        expected_format: Option<String>,
        examples: Vec<String>,
    },

    #[error("Missing required configuration: {field}")]
    MissingRequired {
        field: String,
        section: Option<String>,
        default_available: bool,
        documentation_url: Option<String>,
    },

    #[error("Configuration parse error: {message} (line: {line:?}, column: {column:?})")]
    ParseError {
        message: String,
        line: Option<u32>,
        column: Option<u32>,
        file_path: Option<PathBuf>,
        context: Option<String>,
    },

    #[error("Schema validation failed: {message}")]
    SchemaValidation {
        message: String,
        violated_constraints: Vec<String>,
        schema_version: Option<String>,
    },
}

/// I/O and file system errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum IoError {
    #[error("File access error: {message} (path: {path})")]
    FileAccess {
        path: PathBuf,
        message: String,
        operation: String,
        permissions_issue: bool,
    },

    #[error("Directory operation failed: {message} (path: {path})")]
    DirectoryOperation {
        path: PathBuf,
        message: String,
        operation: String,
        space_available: Option<u64>,
    },

    #[error("Temporary file creation failed: {message}")]
    TempFileCreation {
        message: String,
        attempted_path: Option<PathBuf>,
        disk_space_mb: Option<u64>,
    },

    #[error("File format error: {message} (file: {file_path})")]
    FileFormat {
        file_path: PathBuf,
        message: String,
        expected_format: String,
        detected_format: Option<String>,
    },
}

/// Validation and schema errors  
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    #[error("Schema validation failed: {message} (field: {field})")]
    SchemaViolation {
        field: String,
        message: String,
        schema_rule: String,
        provided_value: serde_json::Value,
        suggestions: Vec<String>,
    },

    #[error("JSONPath validation failed: {message} (path: {path})")]
    JsonPathViolation {
        path: String,
        message: String,
        constraint_type: String,
        expected: Option<serde_json::Value>,
        actual: Option<serde_json::Value>,
    },

    #[error("Protocol compliance error: {message} (protocol: {protocol})")]
    ProtocolCompliance {
        protocol: String,
        message: String,
        violation_type: String,
        specification_section: Option<String>,
    },

    #[error("Data integrity check failed: {message}")]
    DataIntegrity {
        message: String,
        checksum_mismatch: bool,
        corruption_type: Option<String>,
        affected_fields: Vec<String>,
    },
}

/// Reporting and output errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ReportingError {
    #[error("Report generation failed: {message} (format: {format})")]
    GenerationFailed {
        format: String,
        message: String,
        output_path: Option<PathBuf>,
        template_issue: bool,
    },

    #[error("Template processing error: {message} (template: {template_name})")]
    TemplateError {
        template_name: String,
        message: String,
        line_number: Option<u32>,
        variable_name: Option<String>,
    },

    #[error("Output serialization failed: {message} (format: {format})")]
    SerializationError {
        format: String,
        message: String,
        data_type: Option<String>,
        size_limit_exceeded: bool,
    },

    #[error("Asset processing failed: {message} (asset: {asset_path})")]
    AssetProcessing {
        asset_path: PathBuf,
        message: String,
        processing_stage: String,
        file_size_mb: Option<f64>,
    },
}

/// Network and transport errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum NetworkError {
    #[error("Connection refused: {endpoint} (reason: {reason})")]
    ConnectionRefused {
        endpoint: String,
        reason: String,
        retry_after_seconds: Option<u64>,
    },

    #[error("DNS resolution failed: {hostname} (error: {error})")]
    DnsResolution {
        hostname: String,
        error: String,
        nameservers: Vec<String>,
    },

    #[error("SSL/TLS error: {message} (endpoint: {endpoint})")]
    TlsError {
        endpoint: String,
        message: String,
        certificate_issue: bool,
        protocol_version: Option<String>,
    },

    #[error("Network timeout: {operation} to {endpoint} (duration: {duration_ms}ms)")]
    NetworkTimeout {
        endpoint: String,
        operation: String,
        duration_ms: u64,
        bytes_transferred: Option<u64>,
    },
}

/// Performance and timing errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceError {
    #[error("Operation timeout: {operation} exceeded {limit_ms}ms (actual: {actual_ms}ms)")]
    OperationTimeout {
        operation: String,
        limit_ms: u64,
        actual_ms: u64,
        resource_contention: bool,
    },

    #[error("Memory limit exceeded: {current_mb}MB > {limit_mb}MB (operation: {operation})")]
    MemoryLimitExceeded {
        operation: String,
        current_mb: u64,
        limit_mb: u64,
        peak_usage_mb: Option<u64>,
    },

    #[error("Rate limit exceeded: {requests} requests in {window_seconds}s (limit: {limit})")]
    RateLimitExceeded {
        requests: u64,
        window_seconds: u64,
        limit: u64,
        reset_time: Option<DateTime<Utc>>,
    },

    #[error("Resource exhaustion: {resource} (available: {available}, required: {required})")]
    ResourceExhaustion {
        resource: String,
        available: u64,
        required: u64,
        unit: String,
    },
}

/// Security and authentication errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum SecurityError {
    #[error("Authentication failed: {message} (method: {auth_method})")]
    AuthenticationFailed {
        auth_method: String,
        message: String,
        retry_allowed: bool,
        lockout_until: Option<DateTime<Utc>>,
    },

    #[error("Authorization denied: {operation} (user: {user}, resource: {resource})")]
    AuthorizationDenied {
        user: String,
        operation: String,
        resource: String,
        required_permissions: Vec<String>,
    },

    #[error("Credential error: {message}")]
    CredentialError {
        message: String,
        credential_type: String,
        expired: bool,
        expires_at: Option<DateTime<Utc>>,
    },

    #[error("Security policy violation: {message} (policy: {policy_name})")]
    PolicyViolation {
        policy_name: String,
        message: String,
        severity: String,
        remediation_steps: Vec<String>,
    },
}

impl TestHarnessError {
    /// Get the error category for this error
    pub fn category(&self) -> crate::error_handling::ErrorCategory {
        match self {
            TestHarnessError::Client(_) => crate::error_handling::ErrorCategory::Connection,
            TestHarnessError::Execution(_) => crate::error_handling::ErrorCategory::Execution,
            TestHarnessError::Configuration(_) => {
                crate::error_handling::ErrorCategory::Configuration
            }
            TestHarnessError::Io(_) => crate::error_handling::ErrorCategory::Io,
            TestHarnessError::Validation(_) => crate::error_handling::ErrorCategory::Validation,
            TestHarnessError::Reporting(_) => crate::error_handling::ErrorCategory::Reporting,
            TestHarnessError::Network(_) => crate::error_handling::ErrorCategory::Network,
            TestHarnessError::Performance(_) => crate::error_handling::ErrorCategory::Performance,
            TestHarnessError::Security(_) => crate::error_handling::ErrorCategory::Security,
        }
    }

    /// Get the severity level for this error
    pub fn severity(&self) -> crate::error_handling::ErrorSeverity {
        use crate::error_handling::ErrorSeverity;

        match self {
            TestHarnessError::Security(_) => ErrorSeverity::Critical,
            TestHarnessError::Configuration(cfg_err) => match cfg_err {
                ConfigurationError::MissingRequired { .. } => ErrorSeverity::Critical,
                ConfigurationError::FileNotFound { .. } => ErrorSeverity::Major,
                _ => ErrorSeverity::Minor,
            },
            TestHarnessError::Client(client_err) => match client_err {
                McpClientError::ConnectionFailed { .. } => ErrorSeverity::Major,
                McpClientError::AuthenticationError { .. } => ErrorSeverity::Critical,
                _ => ErrorSeverity::Minor,
            },
            TestHarnessError::Execution(exec_err) => match exec_err {
                TestExecutionError::AssertionFailed { .. } => ErrorSeverity::Major,
                TestExecutionError::TestTimeout { .. } => ErrorSeverity::Major,
                _ => ErrorSeverity::Minor,
            },
            TestHarnessError::Performance(perf_err) => match perf_err {
                PerformanceError::MemoryLimitExceeded { .. } => ErrorSeverity::Critical,
                _ => ErrorSeverity::Major,
            },
            _ => ErrorSeverity::Minor,
        }
    }

    /// Check if this error type is typically retryable
    pub fn is_retryable(&self) -> bool {
        self.category().is_retryable()
    }

    /// Get actionable suggestions for resolving this error
    pub fn suggestions(&self) -> Vec<String> {
        match self {
            TestHarnessError::Client(McpClientError::ConnectionFailed { .. }) => vec![
                "Check if the MCP server is running".to_string(),
                "Verify the server configuration and connection parameters".to_string(),
                "Check network connectivity and firewall settings".to_string(),
            ],
            TestHarnessError::Configuration(ConfigurationError::FileNotFound {
                suggestions,
                ..
            }) => suggestions.clone(),
            TestHarnessError::Configuration(ConfigurationError::InvalidValue {
                examples, ..
            }) => examples.clone(),
            _ => vec!["Check the logs for more detailed error information".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_client_error_creation() {
        let error = McpClientError::ConnectionFailed {
            server_name: "test-server".to_string(),
            message: "Connection refused".to_string(),
            retry_count: 3,
            last_attempt: Utc::now(),
            underlying_error: Some("TCP connection failed".to_string()),
        };

        assert!(error.to_string().contains("test-server"));
        assert!(error.to_string().contains("Connection refused"));
    }

    #[test]
    fn test_test_execution_error_assertion_failed() {
        let error = TestExecutionError::AssertionFailed {
            test_name: "test_tool_response".to_string(),
            step: 3,
            message: "Expected status 'success', got 'error'".to_string(),
            expected: Some(serde_json::json!("success")),
            actual: Some(serde_json::json!("error")),
            assertion_type: "status_check".to_string(),
            context: Some(serde_json::json!({"tool": "list_files"})),
        };

        assert!(error.to_string().contains("test_tool_response"));
        assert!(error.to_string().contains("step: 3"));
    }

    #[test]
    fn test_configuration_error_missing_required() {
        let error = ConfigurationError::MissingRequired {
            field: "server.endpoint".to_string(),
            section: Some("mcp_servers".to_string()),
            default_available: false,
            documentation_url: Some("https://docs.example.com/config".to_string()),
        };

        assert!(error.to_string().contains("server.endpoint"));
        assert!(error.to_string().contains("Missing required configuration"));
    }

    #[test]
    fn test_error_categorization() {
        let client_error = TestHarnessError::Client(McpClientError::ConnectionFailed {
            server_name: "test".to_string(),
            message: "Failed".to_string(),
            retry_count: 0,
            last_attempt: Utc::now(),
            underlying_error: None,
        });

        assert_eq!(
            client_error.category(),
            crate::error_handling::ErrorCategory::Connection
        );
        assert!(client_error.is_retryable());
    }

    #[test]
    fn test_error_severity_levels() {
        let security_error = TestHarnessError::Security(SecurityError::AuthenticationFailed {
            auth_method: "bearer_token".to_string(),
            message: "Invalid token".to_string(),
            retry_allowed: false,
            lockout_until: None,
        });

        assert_eq!(
            security_error.severity(),
            crate::error_handling::ErrorSeverity::Critical
        );

        let io_error = TestHarnessError::Io(IoError::FileAccess {
            path: PathBuf::from("/tmp/test.txt"),
            message: "Permission denied".to_string(),
            operation: "read".to_string(),
            permissions_issue: true,
        });

        assert_eq!(
            io_error.severity(),
            crate::error_handling::ErrorSeverity::Minor
        );
    }

    #[test]
    fn test_error_suggestions() {
        let connection_error = TestHarnessError::Client(McpClientError::ConnectionFailed {
            server_name: "test-server".to_string(),
            message: "Connection refused".to_string(),
            retry_count: 0,
            last_attempt: Utc::now(),
            underlying_error: None,
        });

        let suggestions = connection_error.suggestions();
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("MCP server")));
    }
}
