//! Comprehensive error handling and logging system for Mandrel MCP Test Harness
//!
//! This module provides a robust error handling framework with structured error types,
//! error context, categorization, and integration with the logging system.

pub mod context;
pub mod metrics;
pub mod recovery;
pub mod types;

pub use context::*;
pub use metrics::*;
pub use recovery::*;
pub use types::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(test)]
use chrono::Utc;

/// Error severity levels for categorization and reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Critical errors that prevent test execution
    Critical,
    /// Major errors that cause test failures
    Major,
    /// Minor errors that may affect test results
    Minor,
    /// Warnings that don't prevent execution
    Warning,
    /// Informational messages
    Info,
}

/// Error categories for classification and recovery strategies
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// MCP client connection and communication errors
    Connection,
    /// MCP protocol violations and format errors
    Protocol,
    /// Test execution and assertion errors
    Execution,
    /// Configuration and setup errors
    Configuration,
    /// File system and I/O errors
    Io,
    /// Validation and schema errors
    Validation,
    /// Report generation and output errors
    Reporting,
    /// Network and transport errors
    Network,
    /// Timeout and performance errors
    Performance,
    /// Authentication and authorization errors
    Security,
    /// Unknown or unclassified errors
    Unknown,
}

impl ErrorCategory {
    /// Check if errors in this category are typically retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ErrorCategory::Connection
                | ErrorCategory::Network
                | ErrorCategory::Performance
                | ErrorCategory::Io
        )
    }

    /// Get the default retry configuration for this error category
    pub fn default_retry_config(&self) -> Option<RetryConfig> {
        match self {
            ErrorCategory::Connection => Some(RetryConfig {
                max_attempts: 3,
                initial_delay_ms: 100,
                max_delay_ms: 5000,
                backoff_multiplier: 2.0,
                jitter_factor: 0.1,
            }),
            ErrorCategory::Network => Some(RetryConfig {
                max_attempts: 5,
                initial_delay_ms: 50,
                max_delay_ms: 2000,
                backoff_multiplier: 1.5,
                jitter_factor: 0.2,
            }),
            ErrorCategory::Performance => Some(RetryConfig {
                max_attempts: 2,
                initial_delay_ms: 1000,
                max_delay_ms: 10000,
                backoff_multiplier: 3.0,
                jitter_factor: 0.05,
            }),
            ErrorCategory::Io => Some(RetryConfig {
                max_attempts: 3,
                initial_delay_ms: 200,
                max_delay_ms: 3000,
                backoff_multiplier: 2.0,
                jitter_factor: 0.15,
            }),
            _ => None,
        }
    }
}

/// Retry configuration for error recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub jitter_factor: f64,
}

/// Error classifier for determining retry and recovery strategies
#[derive(Debug, Clone, Default)]
pub struct ErrorClassifier {
    custom_rules: HashMap<String, bool>,
}

impl ErrorClassifier {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if an error is retryable based on its type and context
    pub fn is_retryable(&self, error: &TestHarnessError) -> bool {
        match error {
            TestHarnessError::Client(client_error) => match client_error {
                McpClientError::ConnectionFailed { .. } => true,
                McpClientError::RequestTimeout { .. } => true,
                McpClientError::ServerError { code, .. } => {
                    // Retry on server errors that indicate temporary issues
                    matches!(code, 500..=599)
                }
                McpClientError::ProtocolViolation { .. } => false,
                McpClientError::TransportError { recoverable, .. } => *recoverable,
                McpClientError::AuthenticationError { retry_allowed, .. } => *retry_allowed,
            },
            TestHarnessError::Network(_) => true,
            TestHarnessError::Io(_) => true,
            TestHarnessError::Performance(_) => true,
            TestHarnessError::Execution(_) => false,
            TestHarnessError::Configuration(_) => false,
            TestHarnessError::Validation(_) => false,
            TestHarnessError::Reporting(_) => false,
            TestHarnessError::Security(_) => false,
        }
    }

    /// Get the error category for classification
    pub fn categorize(&self, error: &TestHarnessError) -> ErrorCategory {
        match error {
            TestHarnessError::Client(_) => ErrorCategory::Connection,
            TestHarnessError::Network(_) => ErrorCategory::Network,
            TestHarnessError::Execution(_) => ErrorCategory::Execution,
            TestHarnessError::Configuration(_) => ErrorCategory::Configuration,
            TestHarnessError::Io(_) => ErrorCategory::Io,
            TestHarnessError::Validation(_) => ErrorCategory::Validation,
            TestHarnessError::Reporting(_) => ErrorCategory::Reporting,
            TestHarnessError::Performance(_) => ErrorCategory::Performance,
            TestHarnessError::Security(_) => ErrorCategory::Security,
        }
    }

    /// Add a custom rule for error classification
    pub fn add_custom_rule(&mut self, pattern: String, is_retryable: bool) {
        self.custom_rules.insert(pattern, is_retryable);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_category_retry_behavior() {
        assert!(ErrorCategory::Connection.is_retryable());
        assert!(ErrorCategory::Network.is_retryable());
        assert!(ErrorCategory::Performance.is_retryable());
        assert!(ErrorCategory::Io.is_retryable());

        assert!(!ErrorCategory::Protocol.is_retryable());
        assert!(!ErrorCategory::Execution.is_retryable());
        assert!(!ErrorCategory::Configuration.is_retryable());
        assert!(!ErrorCategory::Validation.is_retryable());
    }

    #[test]
    fn test_error_category_default_retry_config() {
        let connection_config = ErrorCategory::Connection.default_retry_config().unwrap();
        assert_eq!(connection_config.max_attempts, 3);
        assert_eq!(connection_config.initial_delay_ms, 100);

        let protocol_config = ErrorCategory::Protocol.default_retry_config();
        assert!(protocol_config.is_none());
    }

    #[test]
    fn test_error_classifier_retryable_errors() {
        let classifier = ErrorClassifier::new();

        let connection_error = TestHarnessError::Client(McpClientError::ConnectionFailed {
            server_name: "test-server".to_string(),
            message: "Connection refused".to_string(),
            retry_count: 0,
            last_attempt: Utc::now(),
            underlying_error: None,
        });

        assert!(classifier.is_retryable(&connection_error));

        let protocol_error = TestHarnessError::Client(McpClientError::ProtocolViolation {
            method: "tools/list".to_string(),
            message: "Invalid JSON-RPC format".to_string(),
            request_id: Some("123".to_string()),
            invalid_payload: None,
        });

        assert!(!classifier.is_retryable(&protocol_error));
    }

    #[test]
    fn test_error_classifier_categorization() {
        let classifier = ErrorClassifier::new();

        let client_error = TestHarnessError::Client(McpClientError::ConnectionFailed {
            server_name: "test".to_string(),
            message: "Failed".to_string(),
            retry_count: 0,
            last_attempt: Utc::now(),
            underlying_error: None,
        });

        assert_eq!(
            classifier.categorize(&client_error),
            ErrorCategory::Connection
        );
    }

    #[test]
    fn test_error_classifier_custom_rules() {
        let mut classifier = ErrorClassifier::new();
        classifier.add_custom_rule("timeout".to_string(), true);
        classifier.add_custom_rule("auth".to_string(), false);

        assert!(classifier.custom_rules.contains_key("timeout"));
        assert!(classifier.custom_rules.contains_key("auth"));
    }
}
