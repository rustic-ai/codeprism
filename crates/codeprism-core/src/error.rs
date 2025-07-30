//! Error handling for the codeprism library

use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

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

/// The main error type for the codeprism library
#[derive(Debug)]
pub enum Error {
    /// Input/output related errors
    Io(std::io::Error),

    /// JSON serialization/deserialization errors
    Json(serde_json::Error),

    /// Configuration related errors
    Config {
        /// Configuration key or path that caused the error
        key: String,
        /// Error message
        message: String,
        /// Error context for correlation and debugging
        context: Option<Box<ErrorContext>>,
    },

    /// Parsing related errors
    Parse {
        /// File that failed to parse
        file: PathBuf,
        /// Error message
        message: String,
        /// Line number where the error occurred (if available)
        line: Option<usize>,
        /// Error context for correlation and debugging
        context: Option<Box<ErrorContext>>,
    },

    /// Network related errors
    Network {
        /// Error message
        message: String,
        /// Error context for correlation and debugging
        context: Option<Box<ErrorContext>>,
    },

    /// Database related errors
    Database {
        /// Error message
        message: String,
        /// Error context for correlation and debugging
        context: Option<Box<ErrorContext>>,
    },

    /// Resource exhaustion errors
    ResourceExhausted {
        /// Resource type that was exhausted
        resource: String,
        /// Error message
        message: String,
        /// Error context for correlation and debugging
        context: Option<Box<ErrorContext>>,
    },

    /// Timeout errors
    Timeout {
        /// Operation that timed out
        operation: String,
        /// Timeout duration
        timeout: Duration,
        /// Error context for correlation and debugging
        context: Option<Box<ErrorContext>>,
    },

    /// Operation was cancelled
    Cancelled {
        /// Operation that was cancelled
        operation: String,
        /// Error context for correlation and debugging
        context: Option<Box<ErrorContext>>,
    },

    /// Permission/authorization errors
    Permission {
        /// Resource or operation that was denied
        resource: String,
        /// Error message
        message: String,
        /// Error context for correlation and debugging
        context: Option<Box<ErrorContext>>,
    },

    /// Input validation errors
    Validation {
        /// Field that failed validation
        field: String,
        /// Error message
        message: String,
        /// Error context for correlation and debugging
        context: Option<Box<ErrorContext>>,
    },

    /// Generic errors for cases not covered by specific types
    Generic {
        /// Error message
        message: String,
        /// Error severity
        severity: ErrorSeverity,
        /// Recovery strategy suggestion
        recovery_strategy: RecoveryStrategy,
        /// Error context for correlation and debugging
        context: Option<Box<ErrorContext>>,
    },
}

impl Clone for Error {
    fn clone(&self) -> Self {
        match self {
            Self::Io(e) => {
                // std::io::Error doesn't implement Clone, so we create a new one
                Self::Io(std::io::Error::other(e.to_string()))
            }
            Self::Json(e) => {
                // serde_json::Error doesn't implement Clone, so we create a new one
                Self::Json(serde_json::Error::io(std::io::Error::other(e.to_string())))
            }
            Self::Config {
                key,
                message,
                context,
            } => Self::Config {
                key: key.clone(),
                message: message.clone(),
                context: context.clone(),
            },
            Self::Parse {
                file,
                message,
                line,
                context,
            } => Self::Parse {
                file: file.clone(),
                message: message.clone(),
                line: *line,
                context: context.clone(),
            },
            Self::Network { message, context } => Self::Network {
                message: message.clone(),
                context: context.clone(),
            },
            Self::Database { message, context } => Self::Database {
                message: message.clone(),
                context: context.clone(),
            },
            Self::ResourceExhausted {
                resource,
                message,
                context,
            } => Self::ResourceExhausted {
                resource: resource.clone(),
                message: message.clone(),
                context: context.clone(),
            },
            Self::Timeout {
                operation,
                timeout,
                context,
            } => Self::Timeout {
                operation: operation.clone(),
                timeout: *timeout,
                context: context.clone(),
            },
            Self::Cancelled { operation, context } => Self::Cancelled {
                operation: operation.clone(),
                context: context.clone(),
            },
            Self::Permission {
                resource,
                message,
                context,
            } => Self::Permission {
                resource: resource.clone(),
                message: message.clone(),
                context: context.clone(),
            },
            Self::Validation {
                field,
                message,
                context,
            } => Self::Validation {
                field: field.clone(),
                message: message.clone(),
                context: context.clone(),
            },
            Self::Generic {
                message,
                severity,
                recovery_strategy,
                context,
            } => Self::Generic {
                message: message.clone(),
                severity: *severity,
                recovery_strategy: *recovery_strategy,
                context: context.clone(),
            },
        }
    }
}

impl Error {
    /// Get error code for logging and monitoring
    pub fn get_error_code(&self) -> &'static str {
        match self {
            Self::Io(_) => "ERR_IO",
            Self::Json(_) => "ERR_JSON",
            Self::Config { .. } => "ERR_CONFIG",
            Self::Parse { .. } => "ERR_PARSE",
            Self::Network { .. } => "ERR_NETWORK",
            Self::Database { .. } => "ERR_DATABASE",
            Self::ResourceExhausted { .. } => "ERR_RESOURCE_EXHAUSTED",
            Self::Timeout { .. } => "ERR_TIMEOUT",
            Self::Cancelled { .. } => "ERR_CANCELLED",
            Self::Permission { .. } => "ERR_PERMISSION",
            Self::Validation { .. } => "ERR_VALIDATION",
            Self::Generic { .. } => "ERR_GENERIC",
        }
    }

    /// Get error severity
    pub fn get_severity(&self) -> ErrorSeverity {
        match self {
            Self::Io(_) => ErrorSeverity::Error,
            Self::Json(_) => ErrorSeverity::Error,
            Self::Config { .. } => ErrorSeverity::Error,
            Self::Parse { .. } => ErrorSeverity::Error,
            Self::Network { .. } => ErrorSeverity::Warning,
            Self::Database { .. } => ErrorSeverity::Critical,
            Self::ResourceExhausted { .. } => ErrorSeverity::Critical,
            Self::Timeout { .. } => ErrorSeverity::Warning,
            Self::Cancelled { .. } => ErrorSeverity::Info,
            Self::Permission { .. } => ErrorSeverity::Error,
            Self::Validation { .. } => ErrorSeverity::Warning,
            Self::Generic { severity, .. } => *severity,
        }
    }

    /// Get recovery strategy
    pub fn get_recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            Self::Io(_) => RecoveryStrategy::Retry,
            Self::Json(_) => RecoveryStrategy::UserIntervention,
            Self::Config { .. } => RecoveryStrategy::UserIntervention,
            Self::Parse { .. } => RecoveryStrategy::UserIntervention,
            Self::Network { .. } => RecoveryStrategy::Retry,
            Self::Database { .. } => RecoveryStrategy::Retry,
            Self::ResourceExhausted { .. } => RecoveryStrategy::Degrade,
            Self::Timeout { .. } => RecoveryStrategy::Retry,
            Self::Cancelled { .. } => RecoveryStrategy::UserIntervention,
            Self::Permission { .. } => RecoveryStrategy::UserIntervention,
            Self::Validation { .. } => RecoveryStrategy::UserIntervention,
            Self::Generic {
                recovery_strategy, ..
            } => *recovery_strategy,
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        !matches!(self.get_recovery_strategy(), RecoveryStrategy::Fatal)
    }

    /// Get error context if available
    pub fn get_context(&self) -> Option<&ErrorContext> {
        match self {
            Self::Config { context, .. } => context.as_deref(),
            Self::Parse { context, .. } => context.as_deref(),
            Self::Network { context, .. } => context.as_deref(),
            Self::Database { context, .. } => context.as_deref(),
            Self::ResourceExhausted { context, .. } => context.as_deref(),
            Self::Timeout { context, .. } => context.as_deref(),
            Self::Cancelled { context, .. } => context.as_deref(),
            Self::Permission { context, .. } => context.as_deref(),
            Self::Validation { context, .. } => context.as_deref(),
            Self::Generic { context, .. } => context.as_deref(),
            _ => None,
        }
    }

    /// Create a parse error with context
    pub fn parse_with_context(
        file: impl Into<PathBuf>,
        message: impl Into<String>,
        context: ErrorContext,
    ) -> Self {
        Self::Parse {
            file: file.into(),
            message: message.into(),
            line: None,
            context: Some(Box::new(context)),
        }
    }

    /// Create a simple parse error
    pub fn parse(file: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::Parse {
            file: file.into(),
            message: message.into(),
            line: None,
            context: None,
        }
    }

    /// Create a configuration error
    pub fn config(key: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Config {
            key: key.into(),
            message: message.into(),
            context: None,
        }
    }

    /// Create a permission error
    pub fn permission(resource: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Permission {
            resource: resource.into(),
            message: message.into(),
            context: None,
        }
    }

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>, timeout: Duration) -> Self {
        Self::Timeout {
            operation: operation.into(),
            timeout,
            context: None,
        }
    }

    /// Create a network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
            context: None,
        }
    }

    /// Create a validation error
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
            context: None,
        }
    }

    /// Create a generic error
    pub fn generic(
        message: impl Into<String>,
        severity: ErrorSeverity,
        recovery_strategy: RecoveryStrategy,
    ) -> Self {
        Self::Generic {
            message: message.into(),
            severity,
            recovery_strategy,
            context: None,
        }
    }

    /// Create a resource exhausted error
    pub fn resource_exhausted(resource: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ResourceExhausted {
            resource: resource.into(),
            message: message.into(),
            context: None,
        }
    }

    /// Create a cancelled error
    pub fn cancelled(operation: impl Into<String>) -> Self {
        Self::Cancelled {
            operation: operation.into(),
            context: None,
        }
    }

    /// Create a database error
    pub fn database(message: impl Into<String>) -> Self {
        Self::Database {
            message: message.into(),
            context: None,
        }
    }

    /// Create an IO error from a string message
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io(std::io::Error::other(message.into()))
    }

    /// Create a storage error (mapped to database error)
    pub fn storage(message: impl Into<String>) -> Self {
        Self::Database {
            message: format!("Storage: {}", message.into()),
            context: None,
        }
    }

    /// Create a watcher error (mapped to generic error)
    pub fn watcher(message: impl Into<String>) -> Self {
        Self::Generic {
            message: format!("File watcher: {}", message.into()),
            severity: ErrorSeverity::Warning,
            recovery_strategy: RecoveryStrategy::Retry,
            context: None,
        }
    }

    /// Create an indexing error (mapped to generic error)
    pub fn indexing(message: impl Into<String>) -> Self {
        Self::Generic {
            message: format!("Indexing: {}", message.into()),
            severity: ErrorSeverity::Warning,
            recovery_strategy: RecoveryStrategy::Fallback,
            context: None,
        }
    }

    /// Create an unsupported language error (mapped to validation error)
    pub fn unsupported_language(language: impl Into<String>) -> Self {
        Self::Validation {
            field: "language".to_string(),
            message: format!("Unsupported language: {}", language.into()),
            context: None,
        }
    }

    /// Create a node not found error (mapped to validation error)
    pub fn node_not_found(node_id: impl Into<String>) -> Self {
        Self::Validation {
            field: "node_id".to_string(),
            message: format!("Node not found: {}", node_id.into()),
            context: None,
        }
    }

    /// Create a generic other error
    pub fn other(message: impl Into<String>) -> Self {
        Self::Generic {
            message: message.into(),
            severity: ErrorSeverity::Error,
            recovery_strategy: RecoveryStrategy::Fatal,
            context: None,
        }
    }

    /// Check if this error should trigger a retry
    pub fn should_retry(&self) -> bool {
        matches!(self.get_recovery_strategy(), RecoveryStrategy::Retry)
    }

    /// Get the severity level of this error (for backward compatibility)
    pub fn severity(&self) -> ErrorSeverity {
        self.get_severity()
    }

    /// Get the recovery strategy for this error (for backward compatibility)
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        self.get_recovery_strategy()
    }

    /// Get error code for JSON-RPC responses (for backward compatibility)
    pub fn error_code(&self) -> i32 {
        match self {
            Self::Io(_) => -32000,
            Self::Json(_) => -32005,
            Self::Config { .. } => -32006,
            Self::Parse { .. } => -32001,
            Self::Network { .. } => -32015,
            Self::Database { .. } => -32004,
            Self::ResourceExhausted { .. } => -32013,
            Self::Timeout { .. } => -32012,
            Self::Cancelled { .. } => -32014,
            Self::Permission { .. } => -32016,
            Self::Validation { .. } => -32017,
            Self::Generic { .. } => -32603, // Internal error
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
            Self::Config { key, message, .. } => {
                write!(f, "Config error in '{key}': {message}")
            }
            Self::Parse {
                file,
                message,
                line,
                ..
            } => {
                if let Some(line) = line {
                    write!(f, "Parse error in {}:{}: {}", file.display(), line, message)
                } else {
                    write!(f, "Parse error in {}: {}", file.display(), message)
                }
            }
            Self::Network { message, .. } => write!(f, "Network error: {message}"),
            Self::Database { message, .. } => write!(f, "Database error: {message}"),
            Self::ResourceExhausted {
                resource, message, ..
            } => {
                write!(f, "Resource exhausted ({resource}): {message}")
            }
            Self::Timeout {
                operation, timeout, ..
            } => {
                write!(f, "Timeout in '{operation}' after {timeout:?}")
            }
            Self::Cancelled { operation, .. } => {
                write!(f, "Operation '{operation}' was cancelled")
            }
            Self::Permission {
                resource, message, ..
            } => {
                write!(f, "Permission denied for '{resource}': {message}")
            }
            Self::Validation { field, message, .. } => {
                write!(f, "Validation error in '{field}': {message}")
            }
            Self::Generic { message, .. } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity() {
        let error = Error::parse("test.rs", "test error");
        assert_eq!(error.get_severity(), ErrorSeverity::Error);

        let error = Error::config("test_key", "test config error");
        assert_eq!(error.get_severity(), ErrorSeverity::Error);
    }

    #[test]
    fn test_recovery_strategy() {
        let error = Error::database("test database error");
        assert_eq!(error.get_recovery_strategy(), RecoveryStrategy::Retry);
        assert!(error.is_recoverable());

        let error = Error::parse("test.rs", "test error");
        assert_eq!(
            error.get_recovery_strategy(),
            RecoveryStrategy::UserIntervention
        );
        assert!(error.is_recoverable());
    }

    #[test]
    fn test_parse_error() {
        let error = Error::parse("test.rs", "syntax error");
        assert_eq!(error.get_error_code(), "ERR_PARSE");
        assert!(error.is_recoverable());
    }

    #[test]
    fn test_error_codes() {
        let error = Error::parse("test.rs", "test error");
        assert_eq!(error.get_error_code(), "ERR_PARSE");

        let error = Error::timeout("test_operation", std::time::Duration::from_secs(30));
        assert_eq!(error.get_error_code(), "ERR_TIMEOUT");
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new()
            .with_request_id("req-123".to_string())
            .with_operation("test-op".to_string())
            .with_metadata(
                "file_size".to_string(),
                serde_json::Value::Number(1024.into()),
            );

        let error = Error::parse_with_context("test.rs", "test error", context);
        assert!(
            error.get_context().is_some(),
            "Error should have context when created with context"
        );

        let error_context = error.get_context().unwrap();
        assert_eq!(
            error_context.request_id,
            Some("req-123".to_string()),
            "Context should preserve request_id"
        );
        assert_eq!(
            error_context.operation,
            Some("test-op".to_string()),
            "Context should preserve operation"
        );
    }

    #[test]
    fn test_error_recoverability() {
        let recoverable_error = Error::network("connection failed");
        assert!(recoverable_error.is_recoverable());
        assert_eq!(
            recoverable_error.get_recovery_strategy(),
            RecoveryStrategy::Retry
        );

        let validation_error = Error::validation("field", "invalid value");
        assert!(validation_error.is_recoverable());
        assert_eq!(
            validation_error.get_recovery_strategy(),
            RecoveryStrategy::UserIntervention
        );
    }
}
