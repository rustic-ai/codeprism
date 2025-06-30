//! Custom validation script execution for the CodePrism Test Harness
//!
//! This module provides secure execution of custom validation scripts,
//! primarily Python scripts, with sandboxing and built-in validation utilities.

use crate::types::CustomScript;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, error};

/// Result of script execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResult {
    /// Whether the validation passed
    pub passed: bool,
    /// Score or confidence (0.0 to 1.0)
    pub score: Option<f64>,
    /// Detailed message from the validation
    pub message: String,
    /// Additional data returned by the script
    pub data: HashMap<String, serde_json::Value>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Memory usage in MB
    pub memory_usage_mb: Option<f64>,
}

/// Errors that can occur during script execution
#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    #[error("Script execution timeout after {timeout_seconds} seconds")]
    Timeout { timeout_seconds: u64 },
    #[error("Script execution failed: {message}")]
    ExecutionFailed { message: String },
    #[error("Script syntax error: {message}")]
    SyntaxError { message: String },
    #[error("Script security violation: {message}")]
    SecurityViolation { message: String },
    #[error("Resource limit exceeded: {message}")]
    ResourceLimitExceeded { message: String },
    #[error("Python interpreter not found: {message}")]
    InterpreterNotFound { message: String },
    #[error("Script file not found: {path}")]
    ScriptFileNotFound { path: String },
    #[error("IO error: {message}")]
    IoError { message: String },
}

/// Configuration for script execution sandboxing
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Maximum execution time in seconds
    pub max_execution_time: u64,
    /// Maximum memory usage in MB
    pub max_memory_mb: f64,
    /// Maximum CPU time in seconds
    pub max_cpu_time: u64,
    /// Whether to allow network access
    pub allow_network: bool,
    /// Allowed file system paths for reading
    pub allowed_read_paths: Vec<PathBuf>,
    /// Allowed file system paths for writing
    pub allowed_write_paths: Vec<PathBuf>,
    /// Environment variables to set
    pub env_vars: HashMap<String, String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_execution_time: 30,
            max_memory_mb: 256.0,
            max_cpu_time: 30,
            allow_network: false,
            allowed_read_paths: vec![],
            allowed_write_paths: vec![],
            env_vars: HashMap::new(),
        }
    }
}

/// Custom validation script executor
pub struct ScriptExecutor {
    /// Path to Python interpreter
    python_interpreter: String,
    /// Sandbox configuration
    sandbox_config: SandboxConfig,
    /// Built-in validation utilities
    builtin_utils: BuiltinValidationUtils,
    /// Working directory for script execution
    working_dir: PathBuf,
}

impl ScriptExecutor {
    /// Create a new script executor
    pub fn new(python_interpreter: Option<String>, sandbox_config: SandboxConfig) -> Self {
        let python_interpreter = python_interpreter.unwrap_or_else(|| "python3".to_string());

        Self {
            python_interpreter,
            sandbox_config,
            builtin_utils: BuiltinValidationUtils::new(),
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    /// Execute a custom validation script
    pub async fn execute_script(
        &self,
        script: &CustomScript,
        input_data: &serde_json::Value,
    ) -> Result<ScriptResult, ScriptError> {
        let start_time = Instant::now();

        debug!("Executing custom script: {}", script.name);

        match script.language.as_str() {
            "python" | "python3" => {
                self.execute_python_script(script, input_data, start_time)
                    .await
            }
            "bash" | "sh" => {
                self.execute_bash_script(script, input_data, start_time)
                    .await
            }
            _ => Err(ScriptError::ExecutionFailed {
                message: format!("Unsupported script language: {}", script.language),
            }),
        }
    }

    /// Execute a Python validation script
    async fn execute_python_script(
        &self,
        script: &CustomScript,
        input_data: &serde_json::Value,
        start_time: Instant,
    ) -> Result<ScriptResult, ScriptError> {
        // Check if Python interpreter is available
        if !self.is_python_available().await {
            return Err(ScriptError::InterpreterNotFound {
                message: format!("Python interpreter '{}' not found", self.python_interpreter),
            });
        }

        // Create the Python script with built-in utilities
        let full_script = self.create_python_script_with_utils(script, input_data)?;

        // Execute the script with sandboxing
        let output = self
            .execute_sandboxed_command(
                &self.python_interpreter,
                &["-c", &full_script],
                &script.env,
                script.timeout_seconds,
            )
            .await?;

        // Parse the script output
        self.parse_script_output(&output, start_time)
    }

    /// Execute a Bash validation script
    async fn execute_bash_script(
        &self,
        script: &CustomScript,
        input_data: &serde_json::Value,
        start_time: Instant,
    ) -> Result<ScriptResult, ScriptError> {
        // For bash scripts, we pass the input data as JSON via environment variable
        let mut env = script.env.clone();
        env.insert(
            "INPUT_DATA".to_string(),
            serde_json::to_string(input_data).unwrap(),
        );

        // Execute the script
        let output = self
            .execute_sandboxed_command(
                "bash",
                &["-c", &script.content],
                &env,
                script.timeout_seconds,
            )
            .await?;

        // Parse the script output
        self.parse_script_output(&output, start_time)
    }

    /// Check if Python interpreter is available
    async fn is_python_available(&self) -> bool {
        match Command::new(&self.python_interpreter)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
        {
            Ok(status) => status.success(),
            Err(_) => false,
        }
    }

    /// Create Python script with built-in utilities
    fn create_python_script_with_utils(
        &self,
        script: &CustomScript,
        input_data: &serde_json::Value,
    ) -> Result<String, ScriptError> {
        let builtin_code = self.builtin_utils.get_python_utils_code();
        let input_json =
            serde_json::to_string(input_data).map_err(|e| ScriptError::ExecutionFailed {
                message: format!("Failed to serialize input data: {}", e),
            })?;

        let full_script = format!(
            r#"
import json
import sys
import re
import math
import statistics
from typing import Dict, List, Any, Optional

# Input data from test harness
INPUT_DATA = json.loads(r'''{}''')

# Built-in validation utilities
{}

# User script
{}

# Ensure we have a result
if 'result' not in locals():
    result = {{"passed": False, "message": "Script did not set result variable"}}

# Output result as JSON
print(json.dumps(result))
"#,
            input_json, builtin_code, script.content
        );

        Ok(full_script)
    }

    /// Execute a command with sandboxing
    async fn execute_sandboxed_command(
        &self,
        command: &str,
        args: &[&str],
        env: &HashMap<String, String>,
        timeout_seconds: u64,
    ) -> Result<String, ScriptError> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .current_dir(&self.working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        // Set environment variables
        for (key, value) in env {
            cmd.env(key, value);
        }

        // Add sandbox environment variables
        cmd.env("PYTHONDONTWRITEBYTECODE", "1");
        cmd.env("PYTHONUNBUFFERED", "1");

        // Disable network access (best effort)
        if !self.sandbox_config.allow_network {
            cmd.env("no_proxy", "*");
            cmd.env("NO_PROXY", "*");
        }

        debug!("Executing command: {} {:?}", command, args);

        let child = cmd.spawn().map_err(|e| ScriptError::IoError {
            message: format!("Failed to spawn process: {}", e),
        })?;

        // Set up timeout
        let execution_timeout =
            Duration::from_secs(timeout_seconds.max(self.sandbox_config.max_execution_time));

        let output = timeout(execution_timeout, async { child.wait_with_output().await })
            .await
            .map_err(|_| ScriptError::Timeout {
                timeout_seconds: execution_timeout.as_secs(),
            })?
            .map_err(|e| ScriptError::IoError {
                message: format!("Failed to get process output: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ScriptError::ExecutionFailed {
                message: format!(
                    "Command failed with exit code {}: {}",
                    output.status.code().unwrap_or(-1),
                    stderr
                ),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    /// Parse script output into ScriptResult
    fn parse_script_output(
        &self,
        output: &str,
        start_time: Instant,
    ) -> Result<ScriptResult, ScriptError> {
        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // Try to parse as JSON first
        if let Ok(json_result) = serde_json::from_str::<serde_json::Value>(output.trim()) {
            if let Some(obj) = json_result.as_object() {
                let passed = obj.get("passed").and_then(|v| v.as_bool()).unwrap_or(false);

                let message = obj
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No message provided")
                    .to_string();

                let score = obj.get("score").and_then(|v| v.as_f64());

                let mut data = HashMap::new();
                for (key, value) in obj {
                    if !["passed", "message", "score"].contains(&key.as_str()) {
                        data.insert(key.clone(), value.clone());
                    }
                }

                return Ok(ScriptResult {
                    passed,
                    score,
                    message,
                    data,
                    execution_time_ms,
                    memory_usage_mb: None, // TODO: Implement memory tracking
                });
            }
        }

        // If not JSON, treat as simple text output
        let output = output.trim();
        let passed = !output.contains("FAIL") && !output.contains("ERROR") && !output.is_empty();

        Ok(ScriptResult {
            passed,
            score: None,
            message: if output.is_empty() {
                "Script produced no output".to_string()
            } else {
                output.to_string()
            },
            data: HashMap::new(),
            execution_time_ms,
            memory_usage_mb: None,
        })
    }
}

/// Built-in validation utilities for custom scripts
pub struct BuiltinValidationUtils {
    /// Security patterns for validation
    security_patterns: HashMap<String, String>,
}

impl BuiltinValidationUtils {
    /// Create new built-in validation utilities
    pub fn new() -> Self {
        let mut security_patterns = HashMap::new();

        // SQL Injection patterns
        security_patterns.insert(
            "sql_injection".to_string(),
            r"(?i)(union\s+select|drop\s+table|insert\s+into|delete\s+from|update\s+set|select\s+.*\s+from|;\s*drop|'\s*or\s*'1'\s*=\s*'1)".to_string(),
        );

        // XSS patterns
        security_patterns.insert(
            "xss".to_string(),
            r"(?i)(<script|javascript:|onload\s*=|onerror\s*=|onclick\s*=|<iframe|<object|<embed)"
                .to_string(),
        );

        // Command injection patterns
        security_patterns.insert(
            "command_injection".to_string(),
            r"(?i)(;\s*rm\s|;\s*cat\s|;\s*ls\s|&&\s*rm\s|&&\s*cat\s|`\s*rm\s|`\s*cat\s|\|\s*rm\s|\|\s*cat\s)".to_string(),
        );

        Self { security_patterns }
    }

    /// Get Python code for built-in validation utilities
    pub fn get_python_utils_code(&self) -> String {
        let security_patterns_json = serde_json::to_string(&self.security_patterns).unwrap();

        format!(
            r#"
# Built-in validation utilities for CodePrism Test Harness

import re
import json
from typing import Dict, List, Any, Optional, Union

# Security patterns for validation
SECURITY_PATTERNS = json.loads(r'{}')

def validate_security_patterns(text: str, pattern_names: Optional[List[str]] = None) -> Dict[str, Any]:
    """Validate text against security vulnerability patterns."""
    if pattern_names is None:
        pattern_names = list(SECURITY_PATTERNS.keys())
    
    results = {{}}
    violations = []
    
    for pattern_name in pattern_names:
        if pattern_name in SECURITY_PATTERNS:
            pattern = SECURITY_PATTERNS[pattern_name]
            matches = re.findall(pattern, text, re.IGNORECASE)
            if matches:
                violations.append({{
                    "type": pattern_name,
                    "matches": matches,
                    "count": len(matches)
                }})
    
    return {{
        "passed": len(violations) == 0,
        "violations": violations,
        "total_violations": len(violations)
    }}

def calculate_complexity_score(data: Dict[str, Any]) -> Dict[str, Any]:
    """Calculate code complexity score from analysis data."""
    cyclomatic = data.get("cyclomatic_complexity", 0)
    cognitive = data.get("cognitive_complexity", 0)
    nesting_depth = data.get("max_nesting_depth", 0)
    
    # Simple scoring algorithm
    complexity_score = (cyclomatic * 0.4) + (cognitive * 0.4) + (nesting_depth * 0.2)
    
    if complexity_score <= 5:
        rating = "low"
    elif complexity_score <= 15:
        rating = "medium"
    elif complexity_score <= 25:
        rating = "high"
    else:
        rating = "very_high"
    
    return {{
        "complexity_score": complexity_score,
        "rating": rating,
        "passed": complexity_score <= 15,  # Threshold for acceptable complexity
        "recommendations": get_complexity_recommendations(rating, complexity_score)
    }}

def get_complexity_recommendations(rating: str, score: float) -> List[str]:
    """Get recommendations based on complexity rating."""
    if rating == "low":
        return ["Code complexity is within acceptable limits"]
    elif rating == "medium":
        return ["Consider refactoring some functions to reduce complexity"]
    elif rating == "high":
        return [
            "Refactor complex functions into smaller functions",
            "Reduce nesting depth using early returns",
            "Consider using design patterns to simplify logic"
        ]
    else:
        return [
            "Immediate refactoring required - complexity is too high",
            "Break down large functions into smaller, focused functions",
            "Consider architectural changes to reduce complexity",
            "Add comprehensive unit tests before refactoring"
        ]

def validate_performance_metrics(metrics: Dict[str, Any], thresholds: Dict[str, float]) -> Dict[str, Any]:
    """Validate performance metrics against thresholds."""
    violations = []
    
    for metric_name, threshold in thresholds.items():
        if metric_name in metrics:
            value = metrics[metric_name]
            if isinstance(value, (int, float)) and value > threshold:
                violations.append({{
                    "metric": metric_name,
                    "value": value,
                    "threshold": threshold,
                    "exceeded_by": value - threshold
                }})
    
    return {{
        "passed": len(violations) == 0,
        "violations": violations,
        "total_violations": len(violations),
        "metrics_checked": len(thresholds)
    }}
"#,
            security_patterns_json
        )
    }
}

impl Default for BuiltinValidationUtils {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert_eq!(config.max_execution_time, 30);
        assert_eq!(config.max_memory_mb, 256.0);
        assert!(!config.allow_network);
    }

    #[test]
    fn test_script_result_creation() {
        let result = ScriptResult {
            passed: true,
            score: Some(0.95),
            message: "Test passed".to_string(),
            data: HashMap::new(),
            execution_time_ms: 150,
            memory_usage_mb: None,
        };

        assert!(result.passed);
        assert_eq!(result.score, Some(0.95));
        assert_eq!(result.execution_time_ms, 150);
    }

    #[test]
    fn test_builtin_utils_creation() {
        let utils = BuiltinValidationUtils::new();
        assert!(utils.security_patterns.contains_key("sql_injection"));
        assert!(utils.security_patterns.contains_key("xss"));
        assert!(utils.security_patterns.contains_key("command_injection"));
    }

    #[tokio::test]
    async fn test_script_executor_creation() {
        let config = SandboxConfig::default();
        let executor = ScriptExecutor::new(Some("python3".to_string()), config);
        assert_eq!(executor.python_interpreter, "python3");
    }

    #[tokio::test]
    async fn test_python_script_creation() {
        let config = SandboxConfig::default();
        let executor = ScriptExecutor::new(Some("python3".to_string()), config);

        let script = CustomScript {
            name: "test_script".to_string(),
            language: "python".to_string(),
            content: "result = {'passed': True, 'message': 'Test'}".to_string(),
            env: HashMap::new(),
            timeout_seconds: 10,
        };

        let input_data = json!({"test": "data"});
        let full_script = executor
            .create_python_script_with_utils(&script, &input_data)
            .unwrap();

        assert!(full_script.contains("INPUT_DATA"));
        assert!(full_script.contains("result = {'passed': True, 'message': 'Test'}"));
        assert!(full_script.contains("validate_security_patterns"));
    }
}
