//! Custom validation script execution engine
//!
//! Provides secure, sandboxed execution of custom validation scripts in multiple languages:
//! - JavaScript/Node.js for flexible validation logic
//! - Python for data analysis and complex validation
//! - Lua for lightweight, embedded validation scripts
//!
//! Features:
//! - Secure execution environment with resource limits
//! - Rich context passing with full MCP response data
//! - Comprehensive error handling and reporting
//! - Performance monitoring and optimization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub mod context;
pub mod executor;
pub mod sandbox;

// Re-export key types for easier access
pub use context::ScriptContext;
pub use executor::{MultiLanguageExecutor, ScriptExecutorFactory};

/// Script execution errors
#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("Script compilation failed: {0}")]
    CompilationFailed(String),
    #[error("Script execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Script timeout after {timeout}ms")]
    Timeout { timeout: u64 },
    #[error("Resource limit exceeded: {resource} ({limit})")]
    ResourceLimitExceeded { resource: String, limit: String },
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    #[error("Unsupported script language: {0}")]
    UnsupportedLanguage(String),
    #[error("Context error: {0}")]
    ContextError(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Lua error: {0}")]
    LuaError(String),
}

impl From<mlua::Error> for ScriptError {
    fn from(err: mlua::Error) -> Self {
        ScriptError::LuaError(err.to_string())
    }
}

/// Supported scripting languages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScriptLanguage {
    /// JavaScript using embedded V8 or QuickJS
    JavaScript,
    /// Python using embedded interpreter
    Python,
    /// Lua using embedded mlua
    Lua,
}

impl std::fmt::Display for ScriptLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptLanguage::JavaScript => write!(f, "javascript"),
            ScriptLanguage::Python => write!(f, "python"),
            ScriptLanguage::Lua => write!(f, "lua"),
        }
    }
}

/// Script execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptConfig {
    /// Script language
    pub language: ScriptLanguage,
    /// Script source code
    pub source: String,
    /// Script name/identifier
    pub name: String,
    /// Execution timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum memory usage in MB
    pub max_memory_mb: u64,
    /// Whether to allow network access
    pub allow_network: bool,
    /// Allowed file system paths (read-only)
    pub allowed_paths: Vec<String>,
    /// Environment variables to pass to script
    pub env_vars: HashMap<String, String>,
    /// Script arguments
    pub args: Vec<String>,
}

impl Default for ScriptConfig {
    fn default() -> Self {
        Self {
            language: ScriptLanguage::JavaScript,
            source: String::new(),
            name: "unnamed_script".to_string(),
            timeout_ms: 5000,   // 5 seconds
            max_memory_mb: 128, // 128 MB
            allow_network: false,
            allowed_paths: Vec::new(),
            env_vars: HashMap::new(),
            args: Vec::new(),
        }
    }
}

/// Script execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResult {
    /// Whether the script executed successfully
    pub success: bool,
    /// Script output/return value
    pub output: serde_json::Value,
    /// Error message if execution failed
    pub error: Option<String>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Memory usage in MB
    pub memory_used_mb: u64,
    /// Exit code (for process-based execution)
    pub exit_code: Option<i32>,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Custom metrics collected during execution
    pub metrics: HashMap<String, serde_json::Value>,
}

impl Default for ScriptResult {
    fn default() -> Self {
        Self {
            success: false,
            output: serde_json::Value::Null,
            error: None,
            duration_ms: 0,
            memory_used_mb: 0,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            metrics: HashMap::new(),
        }
    }
}

/// Validation script definition for test cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationScript {
    /// Script configuration
    pub config: ScriptConfig,
    /// When to execute this script (before, after, or both)
    pub execution_phase: ScriptExecutionPhase,
    /// Whether script failure should fail the test
    pub required: bool,
    /// Description of what this script validates
    pub description: Option<String>,
}

/// Script execution phase
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptExecutionPhase {
    /// Execute before the main test
    Before,
    /// Execute after the main test
    After,
    /// Execute both before and after
    Both,
}

/// Script execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptStatistics {
    /// Total scripts executed
    pub total_executed: u64,
    /// Successful executions
    pub successful: u64,
    /// Failed executions
    pub failed: u64,
    /// Total execution time in milliseconds
    pub total_duration_ms: u64,
    /// Average execution time in milliseconds
    pub average_duration_ms: f64,
    /// Peak memory usage in MB
    pub peak_memory_mb: u64,
    /// Timeout occurrences
    pub timeouts: u64,
    /// Security violations
    pub security_violations: u64,
}

impl Default for ScriptStatistics {
    fn default() -> Self {
        Self {
            total_executed: 0,
            successful: 0,
            failed: 0,
            total_duration_ms: 0,
            average_duration_ms: 0.0,
            peak_memory_mb: 0,
            timeouts: 0,
            security_violations: 0,
        }
    }
}

impl ScriptStatistics {
    /// Update statistics with a script result
    pub fn update(&mut self, result: &ScriptResult) {
        self.total_executed += 1;
        if result.success {
            self.successful += 1;
        } else {
            self.failed += 1;

            // Check for specific error types
            if let Some(error) = &result.error {
                if error.contains("timeout") {
                    self.timeouts += 1;
                } else if error.contains("security") || error.contains("violation") {
                    self.security_violations += 1;
                }
            }
        }

        self.total_duration_ms += result.duration_ms;
        self.average_duration_ms = self.total_duration_ms as f64 / self.total_executed as f64;

        if result.memory_used_mb > self.peak_memory_mb {
            self.peak_memory_mb = result.memory_used_mb;
        }
    }
}

/// Script execution engine trait
pub trait ScriptEngine: Send + Sync {
    /// Execute a script with the given context
    fn execute(
        &self,
        config: &ScriptConfig,
        context: &context::ScriptContext,
    ) -> Result<ScriptResult, ScriptError>;

    /// Check if the engine supports the given language
    fn supports(&self, language: &ScriptLanguage) -> bool;

    /// Get engine name for debugging
    fn name(&self) -> &'static str;

    /// Validate script syntax without execution
    fn validate_syntax(&self, config: &ScriptConfig) -> Result<(), ScriptError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_config_default() {
        let config = ScriptConfig::default();
        assert_eq!(config.language, ScriptLanguage::JavaScript);
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.max_memory_mb, 128);
        assert!(!config.allow_network);
    }

    #[test]
    fn test_script_language_display() {
        assert_eq!(ScriptLanguage::JavaScript.to_string(), "javascript");
        assert_eq!(ScriptLanguage::Python.to_string(), "python");
        assert_eq!(ScriptLanguage::Lua.to_string(), "lua");
    }

    #[test]
    fn test_script_statistics_update() {
        let mut stats = ScriptStatistics::default();

        let result = ScriptResult {
            success: true,
            duration_ms: 100,
            memory_used_mb: 64,
            ..Default::default()
        };

        stats.update(&result);

        assert_eq!(stats.total_executed, 1);
        assert_eq!(stats.successful, 1);
        assert_eq!(stats.failed, 0);
        assert_eq!(stats.total_duration_ms, 100);
        assert_eq!(stats.average_duration_ms, 100.0);
        assert_eq!(stats.peak_memory_mb, 64);
    }

    #[test]
    fn test_script_execution_phase_serialization() {
        let phase = ScriptExecutionPhase::After;
        let serialized = serde_json::to_string(&phase).unwrap();
        assert_eq!(serialized, "\"after\"");

        let deserialized: ScriptExecutionPhase = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, phase);
    }

    #[test]
    fn test_validation_script_structure() {
        let script = ValidationScript {
            config: ScriptConfig {
                name: "test_script".to_string(),
                source: "console.log('hello')".to_string(),
                ..Default::default()
            },
            execution_phase: ScriptExecutionPhase::After,
            required: true,
            description: Some("Test validation script".to_string()),
        };

        assert_eq!(script.config.name, "test_script");
        assert_eq!(script.execution_phase, ScriptExecutionPhase::After);
        assert!(script.required);
    }
}
