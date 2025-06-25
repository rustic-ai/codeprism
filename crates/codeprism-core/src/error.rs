//! Error types for codeprism

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for codeprism operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error severity levels for classification and handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorSeverity {
    /// Low severity - informational, can continue normally
    Info,
    /// Medium severity - warning, might affect functionality but not critical
    Warning,
    /// High severity - error that affects functionality but system can continue
    Error,
    /// Critical severity - system stability at risk, immediate attention needed
    Critical,
}

/// Error recovery strategy for different error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Error is recoverable with retry
    Retry,
    /// Error requires fallback mechanism
    Fallback,
    /// Error requires graceful degradation
    Degrade,
    /// Error requires user intervention
    UserIntervention,
    /// Error is not recoverable
    Fatal,
}

/// Error context for better debugging and tracing
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Request ID for correlation
    pub request_id: Option<String>,
    /// Operation being performed when error occurred
    pub operation: Option<String>,
    /// Additional context data
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new() -> Self {
        Self {
            request_id: None,
            operation: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set request ID for correlation
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Set operation context
    pub fn with_operation(mut self, operation: String) -> Self {
        self.operation = Some(operation);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Main error type for codeprism
#[derive(Debug, Error)]
pub enum Error {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Parser error
    #[error("Parse error in {file}: {message}")]
    Parse {
        /// File that failed to parse
        file: PathBuf,
        /// Error message
        message: String,
        /// Error context
        context: Option<ErrorContext>,
    },

    /// Language not supported
    #[error("Language not supported: {0}")]
    UnsupportedLanguage(String),

    /// Tree-sitter error
    #[error("Tree-sitter error: {0}")]
    TreeSitter(String),

    /// Graph storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// File watcher error
    #[error("File watcher error: {0}")]
    Watcher(String),

    /// Indexing error
    #[error("Indexing error: {0}")]
    Indexing(String),

    /// Invalid node ID
    #[error("Invalid node ID: {0}")]
    InvalidNodeId(String),

    /// Node not found
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    /// Edge not found
    #[error("Edge not found: {0}")]
    EdgeNotFound(String),

    /// Timeout error
    #[error("Operation timed out: {operation}")]
    Timeout {
        /// Operation that timed out
        operation: String,
        /// Timeout duration
        duration: std::time::Duration,
    },

    /// Resource exhaustion error
    #[error("Resource exhausted: {resource} (limit: {limit})")]
    ResourceExhausted {
        /// Resource type
        resource: String,
        /// Resource limit
        limit: String,
    },

    /// Cancellation error
    #[error("Operation cancelled: {operation}")]
    Cancelled {
        /// Operation that was cancelled
        operation: String,
        /// Cancellation reason
        reason: Option<String>,
    },

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Permission error
    #[error("Permission denied: {operation}")]
    Permission {
        /// Operation that was denied
        operation: String,
    },

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Other error
    #[error("{0}")]
    Other(String),
}

impl Clone for Error {
    fn clone(&self) -> Self {
        match self {
            Self::Io(e) => Self::Io(std::io::Error::other(e.to_string())),
            Self::Parse {
                file,
                message,
                context,
            } => Self::Parse {
                file: file.clone(),
                message: message.clone(),
                context: context.clone(),
            },
            Self::UnsupportedLanguage(s) => Self::UnsupportedLanguage(s.clone()),
            Self::TreeSitter(s) => Self::TreeSitter(s.clone()),
            Self::Storage(s) => Self::Storage(s.clone()),
            Self::Serialization(e) => {
                Self::Serialization(serde_json::Error::io(std::io::Error::other(e.to_string())))
            }
            Self::Config(s) => Self::Config(s.clone()),
            Self::Watcher(s) => Self::Watcher(s.clone()),
            Self::Indexing(s) => Self::Indexing(s.clone()),
            Self::InvalidNodeId(s) => Self::InvalidNodeId(s.clone()),
            Self::NodeNotFound(s) => Self::NodeNotFound(s.clone()),
            Self::EdgeNotFound(s) => Self::EdgeNotFound(s.clone()),
            Self::Timeout {
                operation,
                duration,
            } => Self::Timeout {
                operation: operation.clone(),
                duration: *duration,
            },
            Self::ResourceExhausted { resource, limit } => Self::ResourceExhausted {
                resource: resource.clone(),
                limit: limit.clone(),
            },
            Self::Cancelled { operation, reason } => Self::Cancelled {
                operation: operation.clone(),
                reason: reason.clone(),
            },
            Self::Network(s) => Self::Network(s.clone()),
            Self::Permission { operation } => Self::Permission {
                operation: operation.clone(),
            },
            Self::Validation(s) => Self::Validation(s.clone()),
            Self::Other(s) => Self::Other(s.clone()),
        }
    }
}

impl Error {
    /// Get the severity level of this error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Io(_) => ErrorSeverity::Error,
            Self::Parse { .. } => ErrorSeverity::Warning,
            Self::UnsupportedLanguage(_) => ErrorSeverity::Warning,
            Self::TreeSitter(_) => ErrorSeverity::Warning,
            Self::Storage(_) => ErrorSeverity::Error,
            Self::Serialization(_) => ErrorSeverity::Error,
            Self::Config(_) => ErrorSeverity::Critical,
            Self::Watcher(_) => ErrorSeverity::Warning,
            Self::Indexing(_) => ErrorSeverity::Warning,
            Self::InvalidNodeId(_) => ErrorSeverity::Warning,
            Self::NodeNotFound(_) => ErrorSeverity::Info,
            Self::EdgeNotFound(_) => ErrorSeverity::Info,
            Self::Timeout { .. } => ErrorSeverity::Warning,
            Self::ResourceExhausted { .. } => ErrorSeverity::Error,
            Self::Cancelled { .. } => ErrorSeverity::Info,
            Self::Network(_) => ErrorSeverity::Error,
            Self::Permission { .. } => ErrorSeverity::Error,
            Self::Validation(_) => ErrorSeverity::Warning,
            Self::Other(_) => ErrorSeverity::Error,
        }
    }

    /// Get the recovery strategy for this error
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            Self::Io(_) => RecoveryStrategy::Retry,
            Self::Parse { .. } => RecoveryStrategy::Fallback,
            Self::UnsupportedLanguage(_) => RecoveryStrategy::Degrade,
            Self::TreeSitter(_) => RecoveryStrategy::Fallback,
            Self::Storage(_) => RecoveryStrategy::Retry,
            Self::Serialization(_) => RecoveryStrategy::Fatal,
            Self::Config(_) => RecoveryStrategy::UserIntervention,
            Self::Watcher(_) => RecoveryStrategy::Retry,
            Self::Indexing(_) => RecoveryStrategy::Fallback,
            Self::InvalidNodeId(_) => RecoveryStrategy::UserIntervention,
            Self::NodeNotFound(_) => RecoveryStrategy::Fallback,
            Self::EdgeNotFound(_) => RecoveryStrategy::Fallback,
            Self::Timeout { .. } => RecoveryStrategy::Retry,
            Self::ResourceExhausted { .. } => RecoveryStrategy::Degrade,
            Self::Cancelled { .. } => RecoveryStrategy::UserIntervention,
            Self::Network(_) => RecoveryStrategy::Retry,
            Self::Permission { .. } => RecoveryStrategy::UserIntervention,
            Self::Validation(_) => RecoveryStrategy::UserIntervention,
            Self::Other(_) => RecoveryStrategy::Fatal,
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self.recovery_strategy(),
            RecoveryStrategy::Retry | RecoveryStrategy::Fallback | RecoveryStrategy::Degrade
        )
    }

    /// Check if this error should trigger a retry
    pub fn should_retry(&self) -> bool {
        matches!(self.recovery_strategy(), RecoveryStrategy::Retry)
    }

    /// Get error code for JSON-RPC responses
    pub fn error_code(&self) -> i32 {
        match self {
            Self::Io(_) => -32000,
            Self::Parse { .. } => -32001,
            Self::UnsupportedLanguage(_) => -32002,
            Self::TreeSitter(_) => -32003,
            Self::Storage(_) => -32004,
            Self::Serialization(_) => -32005,
            Self::Config(_) => -32006,
            Self::Watcher(_) => -32007,
            Self::Indexing(_) => -32008,
            Self::InvalidNodeId(_) => -32009,
            Self::NodeNotFound(_) => -32010,
            Self::EdgeNotFound(_) => -32011,
            Self::Timeout { .. } => -32012,
            Self::ResourceExhausted { .. } => -32013,
            Self::Cancelled { .. } => -32014,
            Self::Network(_) => -32015,
            Self::Permission { .. } => -32016,
            Self::Validation(_) => -32017,
            Self::Other(_) => -32603, // Internal error
        }
    }

    /// Create a parse error with context
    pub fn parse_with_context(
        file: impl Into<PathBuf>,
        message: impl Into<String>,
        context: Option<ErrorContext>,
    ) -> Self {
        Self::Parse {
            file: file.into(),
            message: message.into(),
            context,
        }
    }

    /// Create a parse error
    pub fn parse(file: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::parse_with_context(file, message, None)
    }

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>, duration: std::time::Duration) -> Self {
        Self::Timeout {
            operation: operation.into(),
            duration,
        }
    }

    /// Create a resource exhaustion error
    pub fn resource_exhausted(resource: impl Into<String>, limit: impl Into<String>) -> Self {
        Self::ResourceExhausted {
            resource: resource.into(),
            limit: limit.into(),
        }
    }

    /// Create a cancellation error
    pub fn cancelled(operation: impl Into<String>, reason: Option<String>) -> Self {
        Self::Cancelled {
            operation: operation.into(),
            reason,
        }
    }

    /// Create a permission error
    pub fn permission(operation: impl Into<String>) -> Self {
        Self::Permission {
            operation: operation.into(),
        }
    }

    /// Create a storage error
    pub fn storage(message: impl Into<String>) -> Self {
        Self::Storage(message.into())
    }

    /// Create a tree-sitter error
    pub fn tree_sitter(message: impl Into<String>) -> Self {
        Self::TreeSitter(message.into())
    }

    /// Create a watcher error
    pub fn watcher(message: impl Into<String>) -> Self {
        Self::Watcher(message.into())
    }

    /// Create an indexing error
    pub fn indexing(message: impl Into<String>) -> Self {
        Self::Indexing(message.into())
    }

    /// Create an IO error from a string message
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io(std::io::Error::other(message.into()))
    }

    /// Create a network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network(message.into())
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    /// Create an other error
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity() {
        let error = Error::parse("test.rs", "test error");
        assert_eq!(error.severity(), ErrorSeverity::Warning);

        let error = Error::Config("test config error".to_string());
        assert_eq!(error.severity(), ErrorSeverity::Critical);
    }

    #[test]
    fn test_recovery_strategy() {
        let error = Error::storage("test storage error");
        assert_eq!(error.recovery_strategy(), RecoveryStrategy::Retry);
        assert!(error.should_retry());

        let error = Error::parse("test.rs", "test error");
        assert_eq!(error.recovery_strategy(), RecoveryStrategy::Fallback);
        assert!(!error.should_retry());
    }

    #[test]
    fn test_error_recoverability() {
        let error = Error::storage("test error");
        assert!(error.is_recoverable());

        let error = Error::Serialization(serde_json::Error::io(std::io::Error::other("test")));
        assert!(!error.is_recoverable());
    }

    #[test]
    fn test_error_codes() {
        let error = Error::parse("test.rs", "test error");
        assert_eq!(error.error_code(), -32001);

        let error = Error::timeout("test_operation", std::time::Duration::from_secs(30));
        assert_eq!(error.error_code(), -32012);
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new()
            .with_request_id("req-123".to_string())
            .with_operation("parse_file".to_string())
            .with_metadata(
                "file_size".to_string(),
                serde_json::Value::Number(1024.into()),
            );

        assert_eq!(context.request_id, Some("req-123".to_string()));
        assert_eq!(context.operation, Some("parse_file".to_string()));
        assert_eq!(context.metadata.len(), 1);
    }

    #[test]
    fn test_parse_error() {
        match Error::parse("test.rs", "test error") {
            Error::Parse { file, message, .. } => {
                assert_eq!(file.to_string_lossy(), "test.rs");
                assert_eq!(message, "test error");
            }
            _ => panic!("Expected Parse error"),
        }
    }
}
