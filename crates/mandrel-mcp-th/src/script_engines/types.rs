//! Core types for script execution: ScriptConfig, ScriptContext, ScriptResult, ScriptError

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for script execution
///
/// Controls security settings, resource limits, and execution parameters for script validation.
///
/// # Examples
///
/// ```
/// # use mandrel_mcp_th::script_engines::ScriptConfig;
/// // Create a secure configuration with default settings
/// let config = ScriptConfig::new();
/// assert_eq!(config.timeout_ms, 5000);
/// assert!(!config.allow_network);
/// assert!(config.validate().is_ok());
///
/// // Create a permissive configuration for testing
/// let test_config = ScriptConfig::permissive();
/// assert!(test_config.allow_network);
/// assert!(test_config.allow_filesystem);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptConfig {
    /// Maximum execution time in milliseconds
    pub timeout_ms: u64,
    /// Memory limit in megabytes (None = unlimited)
    pub memory_limit_mb: Option<u64>,
    /// Maximum output size in bytes
    pub max_output_size: usize,
    /// Whether scripts can access network resources
    pub allow_network: bool,
    /// Whether scripts can access filesystem
    pub allow_filesystem: bool,
    /// Environment variables available to scripts
    pub environment_variables: HashMap<String, String>,
}

/// Runtime context passed to scripts
///
/// Contains all data and metadata needed for script execution, including request/response data,
/// test metadata, and execution configuration.
///
/// # Examples
///
/// ```
/// # use mandrel_mcp_th::script_engines::{ScriptContext, ScriptConfig, ServerInfo};
/// # use serde_json::json;
/// let context = ScriptContext::new(
///     json!({"input": "test_data"}),
///     "test_case".to_string(),
///     "test_tool".to_string(),
///     ScriptConfig::new(),
/// ).with_response(json!({"output": "result"}))
///  .with_server_info(ServerInfo {
///      name: "Test Server".to_string(),
///      version: "1.0.0".to_string(),
///      capabilities: vec!["tools".to_string()],
///  });
///
/// assert_eq!(context.metadata.test_name, "test_case");
/// assert!(context.response.is_some(), "Should have value");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptContext {
    /// The original request data
    pub request: serde_json::Value,
    /// The response data (if available)
    pub response: Option<serde_json::Value>,
    /// Execution metadata
    pub metadata: ContextMetadata,
    /// Execution configuration
    pub config: ScriptConfig,
}

/// Metadata about the execution context
///
/// Contains information about the test case, execution environment, and server details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    /// Name of the test case
    pub test_name: String,
    /// Unique execution identifier
    pub execution_id: uuid::Uuid,
    /// Execution timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Name of the tool being tested
    pub tool_name: String,
    /// Information about the MCP server
    pub server_info: ServerInfo,
}

/// Server information
///
/// Details about the MCP server being tested.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
    /// Server capabilities
    pub capabilities: Vec<String>,
}

/// Result of script execution
///
/// Contains the outcome of script execution including success/failure status,
/// output data, logs, performance metrics, and any errors.
///
/// # Examples
///
/// ```
/// # use mandrel_mcp_th::script_engines::{ScriptResult, ScriptError, LogLevel};
/// # use serde_json::json;
/// // Create a successful result
/// let success = ScriptResult::success(json!({"validated": true}), 150)
///     .add_log(LogLevel::Info, "Validation passed".to_string())
///     .with_memory_usage(2.5);
///
/// assert!(success.success);
/// assert_eq!(success.duration_ms, 150);
/// assert_eq!(success.logs.len(), 1, "Should have 1 items");
///
/// // Create a failure result
/// let failure = ScriptResult::failure(
///     ScriptError::RuntimeError { message: "Script failed".to_string() },
///     75
/// );
///
/// assert!(!failure.success);
/// assert!(failure.error.is_some(), "Should have value");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResult {
    /// Whether the script executed successfully
    pub success: bool,
    /// Output data from the script
    pub output: serde_json::Value,
    /// Log entries from script execution
    pub logs: Vec<LogEntry>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Memory used in megabytes (if measured)
    pub memory_used_mb: Option<f64>,
    /// Error details (if execution failed)
    pub error: Option<ScriptError>,
}

/// Log entry from script execution
///
/// Represents a single log message with level, content, and timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Timestamp when log was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Log levels for script execution
///
/// Standard log levels from most verbose to least verbose.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    /// Debug information
    Debug,
    /// Informational messages
    Info,
    /// Warning messages
    Warn,
    /// Error messages
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// Comprehensive error handling for script execution
///
/// Covers all possible failure modes during script execution including syntax errors,
/// runtime errors, timeouts, resource limits, and security violations.
///
/// # Examples
///
/// ```
/// # use mandrel_mcp_th::script_engines::ScriptError;
/// let syntax_error = ScriptError::SyntaxError {
///     message: "Unexpected token".to_string(),
///     line: 10,
/// };
/// assert!(syntax_error.to_string().contains("line 10"));
///
/// let timeout_error = ScriptError::TimeoutError { timeout_ms: 5000 };
/// assert!(timeout_error.to_string().contains("5000ms"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum ScriptError {
    #[error("Syntax error in script: {message} at line {line}")]
    SyntaxError { message: String, line: u32 },

    #[error("Runtime error: {message}")]
    RuntimeError { message: String },

    #[error("Timeout error: script exceeded {timeout_ms}ms limit")]
    TimeoutError { timeout_ms: u64 },

    #[error("Memory limit exceeded: used {used_mb}MB, limit {limit_mb}MB")]
    MemoryLimitError { used_mb: f64, limit_mb: u64 },

    #[error("Security violation: {operation} not allowed")]
    SecurityError { operation: String },

    #[error("Execution error: {message}")]
    ExecutionError { message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Memory tracking error: {message}")]
    MemoryTrackingError { message: String },
}

impl ScriptConfig {
    /// Creates a new ScriptConfig with default security settings
    ///
    /// Default configuration is designed for security with restrictive settings:
    /// - 5 second timeout
    /// - 100MB memory limit
    /// - 1MB output size limit
    /// - No network access
    /// - No filesystem access
    ///
    /// # Examples
    ///
    /// ```
    /// # use mandrel_mcp_th::script_engines::ScriptConfig;
    /// let config = ScriptConfig::new();
    /// assert_eq!(config.timeout_ms, 5000);
    /// assert_eq!(config.memory_limit_mb, Some(100));
    /// assert!(!config.allow_network);
    /// assert!(!config.allow_filesystem);
    /// ```
    pub fn new() -> Self {
        Self {
            timeout_ms: 5000,
            memory_limit_mb: Some(100),
            max_output_size: 1024 * 1024, // 1MB
            allow_network: false,
            allow_filesystem: false,
            environment_variables: HashMap::new(),
        }
    }

    /// Creates a permissive config for testing
    ///
    /// Permissive configuration allows broader access for testing scenarios:
    /// - 30 second timeout
    /// - 500MB memory limit
    /// - 10MB output size limit
    /// - Network access allowed
    /// - Filesystem access allowed
    ///
    /// # Examples
    ///
    /// ```
    /// # use mandrel_mcp_th::script_engines::ScriptConfig;
    /// let config = ScriptConfig::permissive();
    /// assert_eq!(config.timeout_ms, 30000);
    /// assert_eq!(config.memory_limit_mb, Some(500));
    /// assert!(config.allow_network);
    /// assert!(config.allow_filesystem);
    /// ```
    pub fn permissive() -> Self {
        Self {
            timeout_ms: 30000,
            memory_limit_mb: Some(500),
            max_output_size: 10 * 1024 * 1024, // 10MB
            allow_network: true,
            allow_filesystem: true,
            environment_variables: HashMap::new(),
        }
    }

    /// Validates the configuration
    ///
    /// Ensures all configuration values are valid and consistent.
    ///
    /// # Errors
    ///
    /// Returns `ScriptError::ExecutionError` if:
    /// - Timeout is 0
    /// - Memory limit is 0 (when specified)
    ///
    /// # Examples
    ///
    /// ```
    /// # use mandrel_mcp_th::script_engines::ScriptConfig;
    /// let config = ScriptConfig::new();
    /// assert!(config.validate().is_ok());
    ///
    /// let mut invalid_config = ScriptConfig::new();
    /// invalid_config.timeout_ms = 0;
    /// assert!(invalid_config.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), ScriptError> {
        if self.timeout_ms == 0 {
            return Err(ScriptError::ExecutionError {
                message: "Timeout must be greater than 0".to_string(),
            });
        }

        if let Some(limit) = self.memory_limit_mb {
            if limit == 0 {
                return Err(ScriptError::ExecutionError {
                    message: "Memory limit must be greater than 0".to_string(),
                });
            }
        }

        Ok(())
    }
}

impl Default for ScriptConfig {
    /// Creates a default ScriptConfig with secure settings
    ///
    /// Equivalent to `ScriptConfig::new()`.
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptContext {
    /// Creates a new script context
    ///
    /// Initializes a new execution context with the provided request data and metadata.
    /// The execution ID is automatically generated and the timestamp is set to the current time.
    ///
    /// # Arguments
    ///
    /// * `request` - The request data to pass to the script
    /// * `test_name` - Name of the test case
    /// * `tool_name` - Name of the tool being tested
    /// * `config` - Execution configuration
    ///
    /// # Performance
    ///
    /// Context creation is optimized for speed (typically <1ms) and generates a unique
    /// execution ID for tracking.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mandrel_mcp_th::script_engines::{ScriptContext, ScriptConfig};
    /// # use serde_json::json;
    /// let context = ScriptContext::new(
    ///     json!({"input": "test_data"}),
    ///     "test_case".to_string(),
    ///     "test_tool".to_string(),
    ///     ScriptConfig::new(),
    /// );
    ///
    /// assert_eq!(context.metadata.test_name, "test_case");
    /// assert_eq!(context.metadata.tool_name, "test_tool");
    /// assert!(context.response.is_none(), "Should be none");
    /// ```
    pub fn new(
        request: serde_json::Value,
        test_name: String,
        tool_name: String,
        config: ScriptConfig,
    ) -> Self {
        Self {
            request,
            response: None,
            metadata: ContextMetadata {
                test_name,
                execution_id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                tool_name,
                server_info: ServerInfo {
                    name: "Unknown".to_string(),
                    version: "Unknown".to_string(),
                    capabilities: vec![],
                },
            },
            config,
        }
    }

    /// Sets the response data
    ///
    /// Builder method to add response data to the context. This is typically called
    /// after the MCP tool has been executed and a response is available.
    ///
    /// # Arguments
    ///
    /// * `response` - The response data from the MCP tool
    ///
    /// # Examples
    ///
    /// ```
    /// # use mandrel_mcp_th::script_engines::{ScriptContext, ScriptConfig};
    /// # use serde_json::json;
    /// let context = ScriptContext::new(
    ///     json!({"input": "test"}),
    ///     "test".to_string(),
    ///     "tool".to_string(),
    ///     ScriptConfig::new(),
    /// ).with_response(json!({"output": "result"}));
    ///
    /// assert!(context.response.is_some(), "Should have value");
    /// ```
    pub fn with_response(mut self, response: serde_json::Value) -> Self {
        self.response = Some(response);
        self
    }

    /// Sets the server information
    ///
    /// Builder method to add MCP server information to the context metadata.
    ///
    /// # Arguments
    ///
    /// * `server_info` - Information about the MCP server
    ///
    /// # Examples
    ///
    /// ```
    /// # use mandrel_mcp_th::script_engines::{ScriptContext, ScriptConfig, ServerInfo};
    /// # use serde_json::json;
    /// let server_info = ServerInfo {
    ///     name: "Test Server".to_string(),
    ///     version: "1.0.0".to_string(),
    ///     capabilities: vec!["tools".to_string()],
    /// };
    ///
    /// let context = ScriptContext::new(
    ///     json!({}),
    ///     "test".to_string(),
    ///     "tool".to_string(),
    ///     ScriptConfig::new(),
    /// ).with_server_info(server_info);
    ///
    /// assert_eq!(context.metadata.server_info.name, "Test Server");
    /// ```
    pub fn with_server_info(mut self, server_info: ServerInfo) -> Self {
        self.metadata.server_info = server_info;
        self
    }

    /// Gets a value from the request by JSONPath
    ///
    /// Extracts a value from the request data using JSONPath syntax.
    /// Currently returns the entire request; full JSONPath support planned for future release.
    ///
    /// # Arguments
    ///
    /// * `path` - JSONPath expression (currently unused)
    ///
    /// # Returns
    ///
    /// Returns the request value or a `ScriptError` if extraction fails.
    ///
    /// # Future Enhancement
    ///
    /// Full JSONPath support will be implemented using the `jsonpath_lib` crate.
    pub fn get_request_value(&self, path: &str) -> Result<serde_json::Value, ScriptError> {
        // ENHANCEMENT: Implement JSONPath extraction using jsonpath_lib
        let _ = path; // Suppress unused parameter warning
        Ok(self.request.clone())
    }

    /// Gets a value from the response by JSONPath
    ///
    /// Extracts a value from the response data using JSONPath syntax.
    /// Currently returns the entire response; full JSONPath support planned for future release.
    ///
    /// # Arguments
    ///
    /// * `path` - JSONPath expression (currently unused)
    ///
    /// # Returns
    ///
    /// Returns the response value or a `ScriptError` if no response is available or extraction fails.
    ///
    /// # Errors
    ///
    /// Returns `ScriptError::ExecutionError` if no response data is available.
    ///
    /// # Future Enhancement
    ///
    /// Full JSONPath support will be implemented using the `jsonpath_lib` crate.
    pub fn get_response_value(&self, path: &str) -> Result<serde_json::Value, ScriptError> {
        let response = self
            .response
            .as_ref()
            .ok_or_else(|| ScriptError::ExecutionError {
                message: "No response data available".to_string(),
            })?;

        // ENHANCEMENT: Implement JSONPath extraction using jsonpath_lib
        let _ = path; // Suppress unused parameter warning
        Ok(response.clone())
    }
}

impl ScriptResult {
    /// Creates a successful result
    ///
    /// Constructs a result indicating successful script execution with the provided output.
    ///
    /// # Arguments
    ///
    /// * `output` - The output data from the script
    /// * `duration_ms` - Execution duration in milliseconds
    ///
    /// # Examples
    ///
    /// ```
    /// # use mandrel_mcp_th::script_engines::ScriptResult;
    /// # use serde_json::json;
    /// let result = ScriptResult::success(json!({"validated": true}), 150);
    /// assert!(result.success);
    /// assert_eq!(result.duration_ms, 150);
    /// assert!(result.error.is_none(), "Should be none");
    /// ```
    pub fn success(output: serde_json::Value, duration_ms: u64) -> Self {
        Self {
            success: true,
            output,
            logs: vec![],
            duration_ms,
            memory_used_mb: None,
            error: None,
        }
    }

    /// Creates a failed result
    ///
    /// Constructs a result indicating failed script execution with the provided error.
    ///
    /// # Arguments
    ///
    /// * `error` - The error that caused the failure
    /// * `duration_ms` - Execution duration in milliseconds
    ///
    /// # Examples
    ///
    /// ```
    /// # use mandrel_mcp_th::script_engines::{ScriptResult, ScriptError};
    /// let error = ScriptError::RuntimeError { message: "Script failed".to_string() };
    /// let result = ScriptResult::failure(error, 75);
    /// assert!(!result.success);
    /// assert!(result.error.is_some(), "Should have value");
    /// ```
    pub fn failure(error: ScriptError, duration_ms: u64) -> Self {
        Self {
            success: false,
            output: serde_json::Value::Null,
            logs: vec![],
            duration_ms,
            memory_used_mb: None,
            error: Some(error),
        }
    }

    /// Adds a log entry
    ///
    /// Builder method to add a log entry to the result. Multiple log entries can be chained.
    ///
    /// # Arguments
    ///
    /// * `level` - Log level (Debug, Info, Warn, Error)
    /// * `message` - Log message
    ///
    /// # Examples
    ///
    /// ```
    /// # use mandrel_mcp_th::script_engines::{ScriptResult, LogLevel};
    /// # use serde_json::json;
    /// let result = ScriptResult::success(json!({}), 100)
    ///     .add_log(LogLevel::Info, "Script started".to_string())
    ///     .add_log(LogLevel::Info, "Validation passed".to_string());
    ///
    /// assert_eq!(result.logs.len(), 2, "Should have 2 items");
    /// ```
    pub fn add_log(mut self, level: LogLevel, message: String) -> Self {
        self.logs.push(LogEntry {
            level,
            message,
            timestamp: chrono::Utc::now(),
        });
        self
    }

    /// Sets memory usage information
    ///
    /// Builder method to record memory usage during script execution.
    ///
    /// # Arguments
    ///
    /// * `memory_mb` - Memory usage in megabytes
    ///
    /// # Examples
    ///
    /// ```
    /// # use mandrel_mcp_th::script_engines::ScriptResult;
    /// # use serde_json::json;
    /// let result = ScriptResult::success(json!({}), 100)
    ///     .with_memory_usage(2.5);
    ///
    /// assert_eq!(result.memory_used_mb, Some(2.5));
    /// ```
    pub fn with_memory_usage(mut self, memory_mb: f64) -> Self {
        self.memory_used_mb = Some(memory_mb);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_script_config_creation_and_validation() {
        let config = ScriptConfig::new();
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.memory_limit_mb, Some(100));
        assert!(!config.allow_network);
        assert!(!config.allow_filesystem);
        let validation_result = config.validate();
        assert!(
            validation_result.is_ok(),
            "Valid configuration should pass validation: {:?}",
            validation_result.err()
        );
    }

    #[test]
    fn test_script_config_invalid_timeout() {
        let mut config = ScriptConfig::new();
        config.timeout_ms = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_script_config_invalid_memory_limit() {
        let mut config = ScriptConfig::new();
        config.memory_limit_mb = Some(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_script_config_permissive() {
        let config = ScriptConfig::permissive();
        assert_eq!(config.timeout_ms, 30000);
        assert_eq!(config.memory_limit_mb, Some(500));
        assert!(config.allow_network);
        assert!(config.allow_filesystem);
        let validation_result = config.validate();
        assert!(
            validation_result.is_ok(),
            "Valid configuration should pass validation: {:?}",
            validation_result.err()
        );
    }

    #[test]
    fn test_script_config_default() {
        let config = ScriptConfig::default();
        assert_eq!(config.timeout_ms, 5000);
        let validation_result = config.validate();
        assert!(
            validation_result.is_ok(),
            "Valid configuration should pass validation: {:?}",
            validation_result.err()
        );
    }

    #[test]
    fn test_script_context_construction() {
        let request = serde_json::json!({"test": "data"});
        let context = ScriptContext::new(
            request.clone(),
            "test_case".to_string(),
            "test_tool".to_string(),
            ScriptConfig::new(),
        );

        assert_eq!(context.request, request);
        assert!(context.response.is_none(), "Should be none");
        assert_eq!(context.metadata.test_name, "test_case");
        assert_eq!(context.metadata.tool_name, "test_tool");
        assert_eq!(context.metadata.server_info.name, "Unknown");
    }

    #[test]
    fn test_script_context_with_response() {
        let request = serde_json::json!({"input": "test"});
        let response = serde_json::json!({"output": "result"});

        let context = ScriptContext::new(
            request,
            "test_case".to_string(),
            "test_tool".to_string(),
            ScriptConfig::new(),
        )
        .with_response(response.clone());

        assert_eq!(context.response, Some(response));
    }

    #[test]
    fn test_script_context_with_server_info() {
        let server_info = ServerInfo {
            name: "Test Server".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["tools".to_string(), "resources".to_string()],
        };

        let context = ScriptContext::new(
            serde_json::json!({}),
            "test".to_string(),
            "tool".to_string(),
            ScriptConfig::new(),
        )
        .with_server_info(server_info.clone());

        assert_eq!(context.metadata.server_info.name, server_info.name);
        assert_eq!(context.metadata.server_info.version, server_info.version);
        assert_eq!(
            context.metadata.server_info.capabilities,
            server_info.capabilities
        );
    }

    #[test]
    fn test_script_result_success() {
        let output = serde_json::json!({"result": "success"});
        let result = ScriptResult::success(output.clone(), 100);

        assert!(result.success);
        assert_eq!(result.output, output);
        assert_eq!(result.duration_ms, 100);
        assert!(result.error.is_none(), "Should be none");
        assert!(!result.logs.is_empty(), "Should not be empty");
    }

    #[test]
    fn test_script_result_failure() {
        let error = ScriptError::RuntimeError {
            message: "Test error".to_string(),
        };
        let result = ScriptResult::failure(error.clone(), 50);

        assert!(!result.success);
        assert_eq!(result.duration_ms, 50);
        assert!(result.error.is_some(), "Should have value");
        assert_eq!(result.output, serde_json::Value::Null);
    }

    #[test]
    fn test_script_result_with_logs() {
        let result = ScriptResult::success(serde_json::json!({}), 100)
            .add_log(LogLevel::Info, "Test message".to_string())
            .add_log(LogLevel::Warn, "Warning message".to_string());

        assert_eq!(result.logs.len(), 2, "Should have 2 items");
        assert!(matches!(result.logs[0].level, LogLevel::Info));
        assert_eq!(result.logs[0].message, "Test message");
        assert!(matches!(result.logs[1].level, LogLevel::Warn));
        assert_eq!(result.logs[1].message, "Warning message");
    }

    #[test]
    fn test_script_result_with_memory_usage() {
        let result = ScriptResult::success(serde_json::json!({}), 100).with_memory_usage(2.5);

        assert_eq!(result.memory_used_mb, Some(2.5));
    }

    #[test]
    fn test_script_error_variants() {
        let syntax_error = ScriptError::SyntaxError {
            message: "Unexpected token".to_string(),
            line: 10,
        };
        assert!(syntax_error.to_string().contains("line 10"));

        let timeout_error = ScriptError::TimeoutError { timeout_ms: 5000 };
        assert!(timeout_error.to_string().contains("5000ms"));

        let memory_error = ScriptError::MemoryLimitError {
            used_mb: 150.0,
            limit_mb: 100,
        };
        assert!(memory_error.to_string().contains("150"));
        assert!(memory_error.to_string().contains("100"));

        let security_error = ScriptError::SecurityError {
            operation: "network access".to_string(),
        };
        assert!(security_error.to_string().contains("network access"));
    }

    #[test]
    fn test_serialization_round_trip_config() {
        let config = ScriptConfig::new();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: ScriptConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.timeout_ms, deserialized.timeout_ms);
        assert_eq!(config.memory_limit_mb, deserialized.memory_limit_mb);
        assert_eq!(config.allow_network, deserialized.allow_network);
        assert_eq!(config.allow_filesystem, deserialized.allow_filesystem);
    }

    #[test]
    fn test_serialization_round_trip_context() {
        let context = ScriptContext::new(
            serde_json::json!({"test": "data"}),
            "test".to_string(),
            "tool".to_string(),
            ScriptConfig::new(),
        );

        let serialized = serde_json::to_string(&context).unwrap();
        let deserialized: ScriptContext = serde_json::from_str(&serialized).unwrap();

        assert_eq!(context.request, deserialized.request);
        assert_eq!(context.metadata.test_name, deserialized.metadata.test_name);
        assert_eq!(context.metadata.tool_name, deserialized.metadata.tool_name);
    }

    #[test]
    fn test_serialization_round_trip_result() {
        let result = ScriptResult::success(serde_json::json!({"test": "output"}), 150)
            .add_log(LogLevel::Info, "Test log".to_string())
            .with_memory_usage(3.2);

        let serialized = serde_json::to_string(&result).unwrap();
        let deserialized: ScriptResult = serde_json::from_str(&serialized).unwrap();

        assert_eq!(result.success, deserialized.success);
        assert_eq!(result.output, deserialized.output);
        assert_eq!(result.duration_ms, deserialized.duration_ms);
        assert_eq!(result.memory_used_mb, deserialized.memory_used_mb);
        assert_eq!(result.logs.len(), deserialized.logs.len());
    }

    #[test]
    fn test_serialization_round_trip_error() {
        let error = ScriptError::RuntimeError {
            message: "Test runtime error".to_string(),
        };

        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: ScriptError = serde_json::from_str(&serialized).unwrap();

        match (error, deserialized) {
            (
                ScriptError::RuntimeError { message: m1 },
                ScriptError::RuntimeError { message: m2 },
            ) => {
                assert_eq!(m1, m2);
            }
            _ => panic!("Error types don't match"),
        }
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry {
            level: LogLevel::Info,
            message: "Test log".to_string(),
            timestamp: chrono::Utc::now(),
        };

        assert!(matches!(entry.level, LogLevel::Info));
        assert_eq!(entry.message, "Test log");
    }

    #[test]
    fn test_performance_context_creation() {
        let start = Instant::now();

        for _ in 0..1000 {
            let _context = ScriptContext::new(
                serde_json::json!({"test": "data"}),
                "perf_test".to_string(),
                "tool".to_string(),
                ScriptConfig::new(),
            );
        }

        let duration = start.elapsed();
        assert!(
            duration.as_millis() < 100,
            "Context creation too slow: {}ms",
            duration.as_millis()
        );
    }

    #[test]
    fn test_performance_serialization() {
        let context = ScriptContext::new(
            serde_json::json!({"large": "data".repeat(1000)}),
            "perf_test".to_string(),
            "tool".to_string(),
            ScriptConfig::new(),
        );

        let start = Instant::now();

        for _ in 0..100 {
            let _serialized = serde_json::to_string(&context).unwrap();
        }

        let duration = start.elapsed();
        assert!(
            duration.as_millis() < 50,
            "Serialization too slow: {}ms",
            duration.as_millis()
        );
    }
}
