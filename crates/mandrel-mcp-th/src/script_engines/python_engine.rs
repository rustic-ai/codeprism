//! Python script execution engine using subprocess

use crate::script_engines::memory_tracker::{MemoryTracker, MemoryTrackingConfig};
use crate::script_engines::{ScriptConfig, ScriptContext, ScriptError, ScriptResult};
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;
use tempfile::NamedTempFile;
use tokio::process::Command;
use tokio::time::timeout;

/// Python script execution engine using subprocess
pub struct PythonEngine {
    config: ScriptConfig,
    python_path: PathBuf,
}

/// Precompiled Python script with bytecode caching
#[derive(Debug, Clone)]
pub struct PythonScript {
    source: String,
    #[allow(dead_code)] // Reserved for bytecode caching
    bytecode_path: Option<PathBuf>,
    #[allow(dead_code)] // Reserved for function-specific execution
    function_name: Option<String>,
}

impl PythonEngine {
    /// Create new Python engine with configuration
    pub fn new(config: &ScriptConfig) -> Result<Self, ScriptError> {
        // Detect Python interpreter
        let python_path = Self::find_python_interpreter()?;

        Ok(Self {
            config: config.clone(),
            python_path,
        })
    }

    /// Find Python interpreter on the system
    fn find_python_interpreter() -> Result<PathBuf, ScriptError> {
        // Try common Python executable names
        let python_names = ["python3", "python", "python3.11", "python3.10", "python3.9"];

        for name in &python_names {
            if let Ok(path) = which::which(name) {
                return Ok(path);
            }
        }

        // Fallback to python3 path
        Ok(PathBuf::from("python3"))
    }

    /// Execute Python script with context injection
    pub async fn execute_script(
        &self,
        script: &str,
        context: ScriptContext,
    ) -> Result<ScriptResult, ScriptError> {
        let start_time = Instant::now();

        // Initialize memory tracking
        let memory_config = MemoryTrackingConfig::default();
        let memory_tracker = MemoryTracker::new(memory_config);
        let memory_before =
            memory_tracker
                .snapshot()
                .map_err(|e| ScriptError::MemoryTrackingError {
                    message: format!("Failed to take initial memory snapshot: {}", e),
                })?;

        // Create the Python script with context injection
        let full_script = self.create_script_with_context(script, &context)?;

        // Execute with timeout
        let execution_future = self.execute_python_subprocess(&full_script);
        let py_result = timeout(
            Duration::from_millis(self.config.timeout_ms),
            execution_future,
        )
        .await;

        // Take memory snapshot after execution
        let memory_after =
            memory_tracker
                .snapshot()
                .map_err(|e| ScriptError::MemoryTrackingError {
                    message: format!("Failed to take final memory snapshot: {}", e),
                })?;

        let memory_delta = memory_tracker.calculate_delta(&memory_before, &memory_after);
        let memory_used_mb = Some(memory_tracker.delta_to_mb(&memory_delta));
        let duration_ms = start_time.elapsed().as_millis() as u64;

        match py_result {
            Ok(Ok((stdout, stderr, exit_code))) => {
                self.handle_python_result(stdout, stderr, exit_code, duration_ms, memory_used_mb)
            }
            Ok(Err(exec_error)) => Ok(ScriptResult {
                success: false,
                output: serde_json::Value::Null,
                logs: vec![],
                duration_ms,
                memory_used_mb,
                error: Some(ScriptError::ExecutionError {
                    message: format!("Python execution failed: {}", exec_error),
                }),
            }),
            Err(_) => Ok(ScriptResult {
                success: false,
                output: serde_json::Value::Null,
                logs: vec![],
                duration_ms,
                memory_used_mb,
                error: Some(ScriptError::TimeoutError {
                    timeout_ms: self.config.timeout_ms,
                }),
            }),
        }
    }

    /// Create Python script with context injection
    fn create_script_with_context(
        &self,
        script: &str,
        context: &ScriptContext,
    ) -> Result<String, ScriptError> {
        let context_json = serde_json::to_string_pretty(&serde_json::json!({
            "request": context.request,
            "response": context.response,
            "metadata": {
                "test_name": context.metadata.test_name,
                "tool_name": context.metadata.tool_name,
            }
        }))
        .map_err(|e| ScriptError::ExecutionError {
            message: format!("Failed to serialize context: {}", e),
        })?;

        // Convert JSON nulls to Python None for proper syntax
        let python_context = context_json
            .replace("null", "None")
            .replace("true", "True")
            .replace("false", "False");

        Ok(format!(
            r#"#!/usr/bin/env python3
# Context injection for MCP validation
import json
import sys

# Injected context data
context = {}

def log(level, message):
    """Logging function for validation scripts"""
    print(f"[{{level}}] {{message}}", file=sys.stderr)

# User script execution
{}
"#,
            python_context, script
        ))
    }

    /// Execute Python script in subprocess
    async fn execute_python_subprocess(
        &self,
        script: &str,
    ) -> Result<(String, String, Option<i32>), std::io::Error> {
        // Create temporary file for the script
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(script.as_bytes())?;
        temp_file.flush()?;

        let script_path = temp_file.path().to_string_lossy().to_string();

        // Execute Python with the script
        let child = Command::new(&self.python_path)
            .arg(&script_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let output = child.wait_with_output().await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code();

        Ok((stdout, stderr, exit_code))
    }

    /// Handle Python execution result
    fn handle_python_result(
        &self,
        stdout: String,
        stderr: String,
        exit_code: Option<i32>,
        duration_ms: u64,
        memory_used_mb: Option<f64>,
    ) -> Result<ScriptResult, ScriptError> {
        let success = exit_code.unwrap_or(-1) == 0;

        let output = if !stdout.trim().is_empty() {
            // Try to parse stdout as JSON, fallback to string
            serde_json::from_str(stdout.trim())
                .unwrap_or_else(|_| serde_json::Value::String(stdout.trim().to_string()))
        } else {
            serde_json::Value::Null
        };

        let error = if !success || !stderr.trim().is_empty() {
            Some(self.parse_python_error(&stderr, exit_code))
        } else {
            None
        };

        Ok(ScriptResult {
            success,
            output,
            logs: vec![], // TODO: Parse stderr into structured logs
            duration_ms,
            memory_used_mb,
            error,
        })
    }

    /// Parse Python error from stderr
    fn parse_python_error(&self, stderr: &str, exit_code: Option<i32>) -> ScriptError {
        let stderr_lower = stderr.to_lowercase();

        if stderr_lower.contains("syntaxerror") {
            ScriptError::RuntimeError {
                message: format!("Python syntax error: {}", stderr.trim()),
            }
        } else if stderr_lower.contains("importerror")
            || stderr_lower.contains("modulenotfounderror")
        {
            ScriptError::RuntimeError {
                message: format!("Python import error: {}", stderr.trim()),
            }
        } else if stderr_lower.contains("runtimeerror") || stderr_lower.contains("exception") {
            ScriptError::RuntimeError {
                message: format!("Python runtime error: {}", stderr.trim()),
            }
        } else if let Some(code) = exit_code {
            ScriptError::ExecutionError {
                message: format!(
                    "Python process failed with exit code {}: {}",
                    code,
                    stderr.trim()
                ),
            }
        } else {
            ScriptError::ExecutionError {
                message: format!("Python execution error: {}", stderr.trim()),
            }
        }
    }

    /// Precompile Python script to bytecode
    pub fn precompile_script(
        &self,
        script: &str,
        function_name: Option<String>,
    ) -> Result<PythonScript, ScriptError> {
        // For now, just store the source (future: actual bytecode compilation)
        Ok(PythonScript {
            source: script.to_string(),
            bytecode_path: None,
            function_name,
        })
    }

    /// Execute precompiled Python script
    pub async fn execute_precompiled(
        &self,
        script: &PythonScript,
        context: ScriptContext,
    ) -> Result<ScriptResult, ScriptError> {
        // For now, just re-execute the source (future: execute actual bytecode)
        self.execute_script(&script.source, context).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script_engines::ScriptConfig;
    use serde_json::json;
    use std::time::Instant;

    /// Create a basic test context for Python engine tests
    fn create_test_context() -> ScriptContext {
        let config = ScriptConfig::new();
        ScriptContext::new(
            json!({"test": "data"}),
            "test_case".to_string(),
            "python_tool".to_string(),
            config,
        )
    }

    /// Create a test context with response data for comprehensive testing
    fn create_test_context_with_data() -> ScriptContext {
        let config = ScriptConfig::new();
        ScriptContext::new(
            json!({
                "method": "tools/call",
                "params": {
                    "name": "test_tool",
                    "arguments": {"input": "test_value"}
                }
            }),
            "comprehensive_test".to_string(),
            "mcp_tool".to_string(),
            config,
        )
        .with_response(json!({
            "content": [
                {
                    "type": "text",
                    "text": "Test response content"
                }
            ],
            "isError": false
        }))
    }

    #[tokio::test]
    async fn test_python_engine_creation() {
        let config = ScriptConfig::new();
        let result = PythonEngine::new(&config);

        assert!(result.is_ok(), "Should create Python engine successfully");
    }

    #[tokio::test]
    async fn test_python_simple_script_execution() {
        let config = ScriptConfig::new();
        let engine = PythonEngine::new(&config).unwrap();
        let context = create_test_context();

        let script = r#"
# Simple Python script that returns JSON result
import json

result = {
    "success": True,
    "message": "Hello from Python",
    "input_received": "context" in globals()
}

print(json.dumps(result))
        "#;

        let result = engine.execute_script(script, context).await;

        assert!(result.is_ok(), "Should execute simple Python script");
        let script_result = result.unwrap();
        assert!(script_result.success, "Script should execute successfully");
        // duration_ms is u64 so always >= 0, and execution tracking is working
    }

    #[tokio::test]
    async fn test_python_context_injection() {
        let config = ScriptConfig::new();
        let engine = PythonEngine::new(&config).unwrap();
        let context = create_test_context_with_data();

        let script = r#"
# Verify context is properly injected
import json

if "context" not in globals() or not context.get("request") or not context.get("response"):
    raise ValueError("Context not properly injected")

result = {
    "success": True,
    "test_case": context["metadata"]["test_name"],
    "tool": context["metadata"]["tool_name"],
    "has_request": "request" in context,
    "has_response": "response" in context
}

print(json.dumps(result))
        "#;

        let result = engine.execute_script(script, context).await;

        assert!(
            result.is_ok(),
            "Should execute script with context injection"
        );
        let script_result = result.unwrap();
        assert!(
            script_result.success,
            "Script should access context successfully"
        );
    }

    #[tokio::test]
    async fn test_python_error_handling() {
        let config = ScriptConfig::new();
        let engine = PythonEngine::new(&config).unwrap();
        let context = create_test_context();

        let script = r#"
# Script that raises an exception
raise RuntimeError("Test Python error")
        "#;

        let result = engine.execute_script(script, context).await;

        // Should handle Python errors gracefully
        assert!(
            result.is_ok(),
            "Should return ScriptResult even for Python errors"
        );
        let script_result = result.unwrap();
        assert!(!script_result.success, "Script should fail due to error");
        assert!(
            script_result.error.is_some(),
            "Should capture error details"
        );
    }

    #[tokio::test]
    async fn test_python_syntax_error_handling() {
        let config = ScriptConfig::new();
        let engine = PythonEngine::new(&config).unwrap();
        let context = create_test_context();

        let script = r#"
# Invalid Python syntax
def invalid_syntax(
    # Missing closing parenthesis
        "#;

        let result = engine.execute_script(script, context).await;

        // Should handle syntax errors gracefully
        assert!(
            result.is_ok(),
            "Should return ScriptResult even for syntax errors"
        );
        let script_result = result.unwrap();
        assert!(
            !script_result.success,
            "Script should fail due to syntax error"
        );
        assert!(
            script_result.error.is_some(),
            "Should capture syntax error details"
        );
    }

    #[tokio::test]
    async fn test_python_timeout_handling() {
        let mut config = ScriptConfig::new();
        config.timeout_ms = 100; // Very short timeout

        let engine = PythonEngine::new(&config).unwrap();
        let context = create_test_context();

        let script = r#"
# Long-running operation to trigger timeout
import time

# This should be interrupted by timeout before completion
for i in range(1000000):
    time.sleep(0.001)  # 1ms per iteration = 1000 seconds total

print("Should not reach here")
        "#;

        let result = engine.execute_script(script, context).await;

        // Should handle timeouts gracefully
        assert!(
            result.is_ok(),
            "Should return ScriptResult even for timeouts"
        );
        let script_result = result.unwrap();
        assert!(!script_result.success, "Script should fail due to timeout");
        assert!(
            script_result.error.is_some(),
            "Should capture timeout error"
        );
    }

    #[tokio::test]
    async fn test_python_precompile_script() {
        let config = ScriptConfig::new();
        let engine = PythonEngine::new(&config).unwrap();

        let script = r#"
def validate(input_data):
    return {"success": True, "processed": input_data}

# Return the function for execution
validate
        "#;

        let result = engine.precompile_script(script, Some("validate".to_string()));

        assert!(result.is_ok(), "Should precompile Python script");
        let py_script = result.unwrap();
        assert!(!py_script.source.is_empty(), "Should store script source");
        assert_eq!(py_script.function_name, Some("validate".to_string()));
    }

    #[tokio::test]
    async fn test_python_execute_precompiled() {
        let config = ScriptConfig::new();
        let engine = PythonEngine::new(&config).unwrap();
        let context = create_test_context();

        // First precompile the script
        let script = r#"
import json

result = {
    "success": True,
    "message": "Precompiled Python script executed",
    "context_available": "context" in globals()
}

print(json.dumps(result))
        "#;

        let py_script = engine.precompile_script(script, None).unwrap();
        let result = engine.execute_precompiled(&py_script, context).await;

        assert!(result.is_ok(), "Should execute precompiled Python script");
        let script_result = result.unwrap();
        assert!(
            script_result.success,
            "Precompiled script should execute successfully"
        );
    }

    #[tokio::test]
    async fn test_python_memory_tracking() {
        let config = ScriptConfig::new();
        let engine = PythonEngine::new(&config).unwrap();
        let context = create_test_context();

        let script = r#"
# Create some objects to use memory
import json

data = []
for i in range(1000):
    data.append({"id": i, "value": f"item_{i}"})

result = {
    "success": True,
    "items_created": len(data)
}

print(json.dumps(result))
        "#;

        let result = engine.execute_script(script, context).await;

        // Should track memory usage
        assert!(result.is_ok(), "Should execute script with memory tracking");
        let script_result = result.unwrap();
        assert!(script_result.success, "Script should execute successfully");
        assert!(
            script_result.memory_used_mb.is_some(),
            "Should track memory usage"
        );
        assert!(
            script_result.memory_used_mb.unwrap() >= 0.0,
            "Memory usage should be non-negative"
        );
    }

    #[tokio::test]
    async fn test_python_performance_requirements() {
        let config = ScriptConfig::new();
        let engine = PythonEngine::new(&config).unwrap();
        let context = create_test_context();

        let script = r#"
# Simple computation
import json

total = sum(range(1000))

result = {
    "success": True,
    "sum": total,
    "computation_complete": True
}

print(json.dumps(result))
        "#;

        let start = Instant::now();
        let result = engine.execute_script(script, context).await;
        let total_duration = start.elapsed();

        // Should meet performance requirements
        assert!(
            result.is_ok(),
            "Should execute script within performance limits"
        );
        let script_result = result.unwrap();
        assert!(script_result.success, "Script should execute successfully");

        // Performance assertions
        assert!(
            total_duration.as_millis() < 1000,
            "Total execution should be <1000ms"
        );
        assert!(
            script_result.duration_ms < 1000,
            "Script execution should be <1000ms"
        );

        if let Some(memory_mb) = script_result.memory_used_mb {
            assert!(memory_mb < 100.0, "Memory usage should be <100MB");
        }
    }

    #[tokio::test]
    async fn test_python_import_error_handling() {
        let config = ScriptConfig::new();
        let engine = PythonEngine::new(&config).unwrap();
        let context = create_test_context();

        let script = r#"
# Try to import a module that doesn't exist
import nonexistent_module_that_does_not_exist

print("Should not reach here")
        "#;

        let result = engine.execute_script(script, context).await;

        // Should handle import errors gracefully
        assert!(
            result.is_ok(),
            "Should return ScriptResult even for import errors"
        );
        let script_result = result.unwrap();
        assert!(
            !script_result.success,
            "Script should fail due to import error"
        );
        assert!(
            script_result.error.is_some(),
            "Should capture import error details"
        );
    }

    #[tokio::test]
    async fn test_python_mcp_validation_script() {
        let config = ScriptConfig::new();
        let engine = PythonEngine::new(&config).unwrap();
        let context = create_test_context_with_data();

        let validation_script = r#"
# Validate MCP response structure
import json

if "context" not in globals() or "response" not in context:
    raise ValueError("Missing response content")

if "content" not in context["response"]:
    raise ValueError("Response missing content field")

if not isinstance(context["response"]["content"], list):
    raise ValueError("Content should be a list")

# Check for required content structure
content = context["response"]["content"]
if len(content) == 0:
    raise ValueError("Content list is empty")

first_item = content[0]
if "type" not in first_item or "text" not in first_item:
    raise ValueError("Content item missing required fields")

result = {
    "valid": True,
    "content_type": first_item["type"],
    "has_text": len(first_item["text"]) > 0,
    "is_error": context["response"].get("isError", False)
}

print(json.dumps(result))
        "#;

        let result = engine.execute_script(validation_script, context).await;

        assert!(result.is_ok(), "Should execute MCP validation script");
        let script_result = result.unwrap();
        assert!(
            script_result.success,
            "MCP validation should pass for valid response"
        );

        // Verify the validation result contains expected structure
        if let Some(output_obj) = script_result.output.as_object() {
            assert!(
                output_obj.contains_key("valid"),
                "Should contain 'valid' field"
            );
            assert_eq!(
                output_obj["valid"],
                serde_json::Value::Bool(true),
                "Should be valid"
            );
        } else {
            panic!(
                "Output should be a JSON object, got: {:?}",
                script_result.output
            );
        }
    }

    #[tokio::test]
    async fn test_python_concurrent_execution() {
        let config = ScriptConfig::new();
        let engine = PythonEngine::new(&config).unwrap();

        let script = r#"
import json
import time

# Small delay to simulate work
time.sleep(0.01)

result = {
    "success": True,
    "executed": True
}

print(json.dumps(result))
        "#;

        // Execute multiple scripts concurrently
        let context1 = create_test_context();
        let context2 = create_test_context();
        let context3 = create_test_context();

        let start = Instant::now();
        let (result1, result2, result3) = tokio::join!(
            engine.execute_script(script, context1),
            engine.execute_script(script, context2),
            engine.execute_script(script, context3)
        );
        let total_duration = start.elapsed();

        // All executions should succeed
        assert!(result1.is_ok(), "First execution should succeed");
        assert!(result2.is_ok(), "Second execution should succeed");
        assert!(result3.is_ok(), "Third execution should succeed");

        // Concurrent execution should be faster than sequential
        assert!(
            total_duration.as_millis() < 100,
            "Concurrent execution should be <100ms"
        );

        // Verify all results are successful
        assert!(result1.unwrap().success, "First script should succeed");
        assert!(result2.unwrap().success, "Second script should succeed");
        assert!(result3.unwrap().success, "Third script should succeed");
    }
}
