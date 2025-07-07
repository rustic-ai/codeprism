//! Error types for MOTH test harness

use thiserror::Error;

/// Result type alias for MOTH operations
pub type Result<T> = std::result::Result<T, Error>;

/// Comprehensive error types for MOTH test harness
#[derive(Error, Debug)]
pub enum Error {
    /// MCP protocol errors from the SDK
    #[error("MCP protocol error: {0}")]
    Mcp(#[from] rmcp::Error),

    /// Configuration related errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Test specification validation errors
    #[error("Test specification error: {0}")]
    Spec(String),

    /// Dependency resolution errors
    #[error("Dependency resolution error: {0}")]
    Dependency(String),

    /// Server connection errors
    #[error("Server connection error: {0}")]
    Connection(String),

    /// Test execution errors
    #[error("Test execution error: {0}")]
    Execution(String),

    /// Test validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// YAML parsing errors
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yml::Error),
}

impl Error {
    /// Create a configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }

    /// Create a test specification error
    pub fn spec<S: Into<String>>(msg: S) -> Self {
        Self::Spec(msg.into())
    }

    /// Create a dependency resolution error
    pub fn dependency<S: Into<String>>(msg: S) -> Self {
        Self::Dependency(msg.into())
    }

    /// Create a server connection error
    pub fn connection<S: Into<String>>(msg: S) -> Self {
        Self::Connection(msg.into())
    }

    /// Create a test execution error
    pub fn execution<S: Into<String>>(msg: S) -> Self {
        Self::Execution(msg.into())
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Connection(_) | Self::Mcp(_))
    }

    /// Get error category for reporting
    pub fn category(&self) -> &'static str {
        match self {
            Self::Mcp(_) => "protocol",
            Self::Config(_) => "configuration",
            Self::Spec(_) => "specification",
            Self::Dependency(_) => "dependency",
            Self::Connection(_) => "connection",
            Self::Execution(_) => "execution",
            Self::Validation(_) => "validation",
            Self::Io(_) => "io",
            Self::Serialization(_) => "serialization",
            Self::Yaml(_) => "yaml",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_display() {
        let err = Error::config("invalid port number");
        assert_eq!(err.to_string(), "Configuration error: invalid port number");
    }

    #[test]
    fn test_error_constructors() {
        let config_err = Error::config("test config error");
        assert!(matches!(config_err, Error::Config(_)));

        let spec_err = Error::spec("test spec error");
        assert!(matches!(spec_err, Error::Spec(_)));

        let connection_err = Error::connection("test connection error");
        assert!(matches!(connection_err, Error::Connection(_)));

        let execution_err = Error::execution("test execution error");
        assert!(matches!(execution_err, Error::Execution(_)));

        let validation_err = Error::validation("test validation error");
        assert!(matches!(validation_err, Error::Validation(_)));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = Error::from(io_err);
        assert!(matches!(err, Error::Io(_)));
        assert_eq!(err.to_string(), "I/O error: file not found");
    }

    #[test]
    fn test_error_from_serde_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let err = Error::from(json_err);
        assert!(matches!(err, Error::Serialization(_)));
    }

    #[test]
    fn test_error_from_serde_yml() {
        let yaml_err = serde_yml::from_str::<serde_yml::Value>("invalid: yaml: [").unwrap_err();
        let err = Error::from(yaml_err);
        assert!(matches!(err, Error::Yaml(_)));
    }

    #[test]
    fn test_error_is_retryable() {
        assert!(Error::connection("test").is_retryable());
        assert!(!Error::config("test").is_retryable());
        assert!(!Error::spec("test").is_retryable());
        assert!(!Error::validation("test").is_retryable());
    }

    #[test]
    fn test_error_category() {
        assert_eq!(Error::config("test").category(), "configuration");
        assert_eq!(Error::spec("test").category(), "specification");
        assert_eq!(Error::connection("test").category(), "connection");
        assert_eq!(Error::execution("test").category(), "execution");
        assert_eq!(Error::validation("test").category(), "validation");

        let io_err = Error::from(io::Error::new(io::ErrorKind::NotFound, "test"));
        assert_eq!(io_err.category(), "io");

        let json_err = Error::from(serde_json::from_str::<serde_json::Value>("bad").unwrap_err());
        assert_eq!(json_err.category(), "serialization");
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<String> {
            Ok("success".to_string())
        }

        let result = returns_result();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }
}
