//! Script execution engine implementations
//!
//! Provides concrete implementations of the ScriptEngine trait for different
//! scripting languages with secure sandboxing and resource monitoring.

use super::context::ScriptContext;
use super::sandbox::{SandboxConfig, SandboxManager};
use super::{ScriptConfig, ScriptEngine, ScriptError, ScriptLanguage, ScriptResult};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{error, info, warn};

/// Multi-language script executor with secure sandboxing
#[derive(Debug)]
pub struct MultiLanguageExecutor {
    /// JavaScript engine using Node.js
    javascript_engine: JavaScriptEngine,
    /// Python engine using embedded interpreter
    python_engine: PythonEngine,
    /// Lua engine using mlua
    lua_engine: LuaEngine,
    /// Sandbox manager for security enforcement
    sandbox_manager: Arc<SandboxManager>,
}

impl MultiLanguageExecutor {
    /// Create a new multi-language executor with sandbox configuration
    pub fn new(sandbox_config: SandboxConfig) -> Self {
        let sandbox_manager = Arc::new(SandboxManager::new(sandbox_config));

        Self {
            javascript_engine: JavaScriptEngine::new(sandbox_manager.clone()),
            python_engine: PythonEngine::new(sandbox_manager.clone()),
            lua_engine: LuaEngine::new(sandbox_manager.clone()),
            sandbox_manager,
        }
    }

    /// Get appropriate engine for the given language
    fn get_engine(&self, language: &ScriptLanguage) -> &dyn ScriptEngine {
        match language {
            ScriptLanguage::JavaScript => &self.javascript_engine,
            ScriptLanguage::Python => &self.python_engine,
            ScriptLanguage::Lua => &self.lua_engine,
        }
    }

    /// Execute script with automatic engine selection
    pub async fn execute_script(
        &self,
        config: &ScriptConfig,
        context: &ScriptContext,
    ) -> Result<ScriptResult, ScriptError> {
        let engine = self.get_engine(&config.language);

        // Pre-execution validation
        engine.validate_syntax(config)?;

        // Security validation
        self.sandbox_manager.validate_script_security(config)?;

        // Execute with monitoring
        let start_time = Instant::now();
        let result = engine.execute(config, context);
        let duration = start_time.elapsed();

        match result {
            Ok(mut script_result) => {
                script_result.duration_ms = duration.as_millis() as u64;
                info!(
                    "Script '{}' executed successfully in {}ms",
                    config.name, script_result.duration_ms
                );
                Ok(script_result)
            }
            Err(e) => {
                error!(
                    "Script '{}' failed after {}ms: {}",
                    config.name,
                    duration.as_millis(),
                    e
                );
                Err(e)
            }
        }
    }

    /// Validate that required interpreters are available
    pub async fn validate_interpreters(&self) -> Result<(), ScriptError> {
        // Check Node.js availability
        if !self.javascript_engine.is_available().await {
            warn!("Node.js not available - JavaScript scripts will fail");
        }

        // Check Python availability
        if !self.python_engine.is_available().await {
            warn!("Python not available - Python scripts will fail");
        }

        // Lua is embedded, so always available (when implemented)

        Ok(())
    }
}

/// JavaScript execution engine using Node.js
#[derive(Debug)]
pub struct JavaScriptEngine {
    sandbox_manager: Arc<SandboxManager>,
    node_path: String,
}

impl JavaScriptEngine {
    pub fn new(sandbox_manager: Arc<SandboxManager>) -> Self {
        Self {
            sandbox_manager,
            node_path: "node".to_string(), // Can be configured
        }
    }

    /// Check if Node.js is available
    pub async fn is_available(&self) -> bool {
        Command::new(&self.node_path)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|status| status.success())
            .unwrap_or(false)
    }

    /// Create JavaScript wrapper script with context and validation functions
    fn create_wrapper_script(&self, config: &ScriptConfig, context: &ScriptContext) -> String {
        let context_js = context.to_javascript_context();

        format!(
            r#"
// MCP Test Harness Script Execution Environment
'use strict';

// Execution timeout handler
const TIMEOUT_MS = {};
const startTime = Date.now();

function checkTimeout() {{
    if (Date.now() - startTime > TIMEOUT_MS) {{
        throw new Error(`Script execution timeout after ${{TIMEOUT_MS}}ms`);
    }}
}}

// Context object with test data and helper functions
{}

// Built-in validation utilities
const testUtils = {{
    validatePattern: function(value, pattern) {{
        const regex = new RegExp(pattern);
        return regex.test(String(value));
    }},
    
    validateRange: function(value, min, max) {{
        const num = parseFloat(value);
        return !isNaN(num) && num >= min && num <= max;
    }},
    
    validateType: function(value, expectedType) {{
        return typeof value === expectedType;
    }},
    
    validateSchema: function(obj, schema) {{
        // Basic JSON schema validation
        for (const [key, type] of Object.entries(schema)) {{
            if (!(key in obj) || typeof obj[key] !== type) {{
                return false;
            }}
        }}
        return true;
    }},
    
    measureDuration: function(func) {{
        const start = Date.now();
        const result = func();
        return {{ result, duration: Date.now() - start }};
    }}
}};

// Security sandbox - restrict dangerous APIs
const originalSetTimeout = setTimeout;
const originalSetInterval = setInterval;
const originalEval = eval;

setTimeout = function() {{ throw new Error('setTimeout is not allowed in scripts'); }};
setInterval = function() {{ throw new Error('setInterval is not allowed in scripts'); }};
eval = function() {{ throw new Error('eval is not allowed in scripts'); }};

// Script execution wrapper
let scriptResult = null;
let scriptError = null;

try {{
    // Periodic timeout checks
    const timeoutChecker = originalSetInterval(checkTimeout, 100);
    
    // User script execution
    (function() {{
        {}
    }})();
    
    clearInterval(timeoutChecker);
    
    // Ensure result is set
    if (typeof result === 'undefined') {{
        scriptResult = {{
            success: false,
            output: null,
            error: 'Script did not set result variable'
        }};
    }} else {{
        scriptResult = {{
            success: result.success !== false,
            output: result,
            error: null
        }};
    }}
    
}} catch (error) {{
    scriptError = {{
        success: false,
        output: null,
        error: error.message || String(error)
    }};
}}

// Output final result
console.log(JSON.stringify(scriptResult || scriptError));
"#,
            config.timeout_ms, context_js, config.source
        )
    }
}

impl ScriptEngine for JavaScriptEngine {
    fn execute(
        &self,
        config: &ScriptConfig,
        context: &ScriptContext,
    ) -> Result<ScriptResult, ScriptError> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            ScriptError::ExecutionFailed(format!("Failed to create runtime: {}", e))
        })?;

        rt.block_on(async { self.execute_async(config, context).await })
    }

    fn supports(&self, language: &ScriptLanguage) -> bool {
        matches!(language, ScriptLanguage::JavaScript)
    }

    fn name(&self) -> &'static str {
        "JavaScript Engine (Node.js)"
    }

    fn validate_syntax(&self, config: &ScriptConfig) -> Result<(), ScriptError> {
        // Basic syntax validation - check for balanced braces and quotes
        let source = &config.source;
        let mut brace_count = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for ch in source.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '"' | '\'' => in_string = !in_string,
                '{' if !in_string => brace_count += 1,
                '}' if !in_string => brace_count -= 1,
                _ => {}
            }
        }

        if brace_count != 0 {
            return Err(ScriptError::CompilationFailed(
                "Unbalanced braces in JavaScript code".to_string(),
            ));
        }

        if in_string {
            return Err(ScriptError::CompilationFailed(
                "Unterminated string in JavaScript code".to_string(),
            ));
        }

        Ok(())
    }
}

impl JavaScriptEngine {
    /// Estimate memory usage based on execution duration and script complexity
    fn estimate_memory_usage(&self, duration_ms: u64) -> u64 {
        // Base memory usage for Node.js runtime: ~20MB
        let base_memory = 20;

        // Additional memory based on execution time (heuristic)
        // Longer execution typically means more memory allocation
        let duration_factor = (duration_ms / 1000).max(1);
        let estimated_memory = base_memory + (duration_factor * 2);

        // Cap at reasonable maximum for script execution
        estimated_memory.min(128)
    }

    async fn execute_async(
        &self,
        config: &ScriptConfig,
        context: &ScriptContext,
    ) -> Result<ScriptResult, ScriptError> {
        let wrapper_script = self.create_wrapper_script(config, context);

        let mut cmd = Command::new(&self.node_path);
        cmd.arg("-e")
            .arg(&wrapper_script)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Apply sandbox restrictions
        self.sandbox_manager.apply_restrictions(&mut cmd, config)?;

        let start_time = Instant::now();

        let child = cmd
            .spawn()
            .map_err(|e| ScriptError::ExecutionFailed(format!("Failed to spawn Node.js: {}", e)))?;

        let timeout_duration = Duration::from_millis(config.timeout_ms);
        let output = timeout(timeout_duration, child.wait_with_output())
            .await
            .map_err(|_| ScriptError::Timeout {
                timeout: config.timeout_ms,
            })?
            .map_err(|e| {
                ScriptError::ExecutionFailed(format!("Node.js execution failed: {}", e))
            })?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ScriptError::ExecutionFailed(format!(
                "Node.js exited with code {}: {}",
                output.status.code().unwrap_or(-1),
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_script_output(&stdout, duration_ms)
    }

    fn parse_script_output(
        &self,
        output: &str,
        duration_ms: u64,
    ) -> Result<ScriptResult, ScriptError> {
        let output = output.trim();

        if output.is_empty() {
            return Ok(ScriptResult {
                success: false,
                output: Value::Null,
                error: Some("No output from script".to_string()),
                duration_ms,
                memory_used_mb: 0,
                exit_code: Some(0),
                stdout: String::new(),
                stderr: String::new(),
                metrics: HashMap::new(),
            });
        }

        match serde_json::from_str::<Value>(output) {
            Ok(result_json) => {
                let success = result_json
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let error = result_json
                    .get("error")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let output_value = result_json.get("output").cloned().unwrap_or(Value::Null);

                Ok(ScriptResult {
                    success,
                    output: output_value,
                    error,
                    duration_ms,
                    memory_used_mb: self.estimate_memory_usage(duration_ms),
                    exit_code: Some(0),
                    stdout: output.to_string(),
                    stderr: String::new(),
                    metrics: HashMap::new(),
                })
            }
            Err(e) => Err(ScriptError::ExecutionFailed(format!(
                "Failed to parse script output as JSON: {}. Output: {}",
                e, output
            ))),
        }
    }
}

/// Python execution engine using subprocess
#[derive(Debug)]
pub struct PythonEngine {
    sandbox_manager: Arc<SandboxManager>,
    python_path: String,
}

impl PythonEngine {
    pub fn new(sandbox_manager: Arc<SandboxManager>) -> Self {
        Self {
            sandbox_manager,
            python_path: "python3".to_string(), // Can be configured
        }
    }

    /// Check if Python is available
    pub async fn is_available(&self) -> bool {
        Command::new(&self.python_path)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|status| status.success())
            .unwrap_or(false)
    }

    /// Create Python wrapper script with context and validation functions
    fn create_wrapper_script(&self, config: &ScriptConfig, context: &ScriptContext) -> String {
        let context_python = context.to_python_context();

        format!(
            r#"
import json
import sys
import time
import re
import signal
from typing import Any, Dict, List, Optional

# Execution timeout handler
TIMEOUT_MS = {}
start_time = time.time() * 1000

def check_timeout():
    if (time.time() * 1000 - start_time) > TIMEOUT_MS:
        raise TimeoutError(f"Script execution timeout after {{TIMEOUT_MS}}ms")

# Set up timeout signal handler
def timeout_handler(signum, frame):
    raise TimeoutError(f"Script execution timeout after {{TIMEOUT_MS}}ms")

signal.signal(signal.SIGALRM, timeout_handler)
signal.alarm(int(TIMEOUT_MS / 1000) + 1)

# Context object with test data and helper functions
{}

# Built-in validation utilities
class TestUtils:
    @staticmethod
    def validate_pattern(value: Any, pattern: str) -> bool:
        return bool(re.match(pattern, str(value)))
    
    @staticmethod
    def validate_range(value: Any, min_val: float, max_val: float) -> bool:
        try:
            num = float(value)
            return min_val <= num <= max_val
        except (ValueError, TypeError):
            return False
    
    @staticmethod
    def validate_type(value: Any, expected_type: str) -> bool:
        type_map = {{
            'str': str, 'string': str,
            'int': int, 'integer': int,
            'float': float, 'number': float,
            'bool': bool, 'boolean': bool,
            'list': list, 'array': list,
            'dict': dict, 'object': dict,
        }}
        return isinstance(value, type_map.get(expected_type, type(None)))
    
    @staticmethod
    def validate_schema(obj: Dict, schema: Dict) -> bool:
        for key, expected_type in schema.items():
            if key not in obj or not TestUtils.validate_type(obj[key], expected_type):
                return False
        return True
    
    @staticmethod
    def measure_duration(func):
        start = time.time()
        result = func()
        return {{'result': result, 'duration': (time.time() - start) * 1000}}

test_utils = TestUtils()

# Script execution wrapper
script_result = None
script_error = None

try:
    # Periodic timeout checks during execution
    def periodic_check():
        check_timeout()
    
    # User script execution
    {}
    
    # Cancel timeout
    signal.alarm(0)
    
    # Ensure result is set
    if 'result' not in locals():
        script_result = {{
            'success': False,
            'output': None,
            'error': 'Script did not set result variable'
        }}
    else:
        script_result = {{
            'success': result.get('success', True) if isinstance(result, dict) else True,
            'output': result,
            'error': None
        }}
        
except Exception as error:
    signal.alarm(0)
    script_error = {{
        'success': False,
        'output': None,
        'error': str(error)
    }}

# Output final result
print(json.dumps(script_result or script_error))
"#,
            config.timeout_ms, context_python, config.source
        )
    }
}

impl ScriptEngine for PythonEngine {
    fn execute(
        &self,
        config: &ScriptConfig,
        context: &ScriptContext,
    ) -> Result<ScriptResult, ScriptError> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            ScriptError::ExecutionFailed(format!("Failed to create runtime: {}", e))
        })?;

        rt.block_on(async { self.execute_async(config, context).await })
    }

    fn supports(&self, language: &ScriptLanguage) -> bool {
        matches!(language, ScriptLanguage::Python)
    }

    fn name(&self) -> &'static str {
        "Python Engine"
    }

    fn validate_syntax(&self, config: &ScriptConfig) -> Result<(), ScriptError> {
        // Basic Python syntax validation using compile
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            ScriptError::ExecutionFailed(format!("Failed to create runtime: {}", e))
        })?;

        rt.block_on(async {
            let mut cmd = Command::new(&self.python_path);
            cmd.arg("-c")
                .arg(format!("compile({:?}, '<string>', 'exec')", config.source))
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let output = cmd.output().await.map_err(|e| {
                ScriptError::ExecutionFailed(format!("Failed to validate Python syntax: {}", e))
            })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ScriptError::CompilationFailed(format!(
                    "Python syntax error: {}",
                    stderr
                )));
            }

            Ok(())
        })
    }
}

impl PythonEngine {
    /// Estimate memory usage based on execution duration and script complexity
    fn estimate_memory_usage(&self, duration_ms: u64) -> u64 {
        // Base memory usage for Python interpreter: ~15MB
        let base_memory = 15;

        // Additional memory based on execution time (heuristic)
        // Python typically uses less memory than Node.js for simple scripts
        let duration_factor = (duration_ms / 1000).max(1);
        let estimated_memory = base_memory + (duration_factor * 3);

        // Cap at reasonable maximum for script execution
        estimated_memory.min(256)
    }

    async fn execute_async(
        &self,
        config: &ScriptConfig,
        context: &ScriptContext,
    ) -> Result<ScriptResult, ScriptError> {
        let wrapper_script = self.create_wrapper_script(config, context);

        let mut cmd = Command::new(&self.python_path);
        cmd.arg("-c")
            .arg(&wrapper_script)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Apply sandbox restrictions
        self.sandbox_manager.apply_restrictions(&mut cmd, config)?;

        let start_time = Instant::now();

        let child = cmd
            .spawn()
            .map_err(|e| ScriptError::ExecutionFailed(format!("Failed to spawn Python: {}", e)))?;

        let timeout_duration = Duration::from_millis(config.timeout_ms);
        let output = timeout(timeout_duration, child.wait_with_output())
            .await
            .map_err(|_| ScriptError::Timeout {
                timeout: config.timeout_ms,
            })?
            .map_err(|e| ScriptError::ExecutionFailed(format!("Python execution failed: {}", e)))?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ScriptError::ExecutionFailed(format!(
                "Python exited with code {}: {}",
                output.status.code().unwrap_or(-1),
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_script_output(&stdout, duration_ms)
    }

    fn parse_script_output(
        &self,
        output: &str,
        duration_ms: u64,
    ) -> Result<ScriptResult, ScriptError> {
        let output = output.trim();

        if output.is_empty() {
            return Ok(ScriptResult {
                success: false,
                output: Value::Null,
                error: Some("No output from script".to_string()),
                duration_ms,
                memory_used_mb: 0,
                exit_code: Some(0),
                stdout: String::new(),
                stderr: String::new(),
                metrics: HashMap::new(),
            });
        }

        match serde_json::from_str::<Value>(output) {
            Ok(result_json) => {
                let success = result_json
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let error = result_json
                    .get("error")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let output_value = result_json.get("output").cloned().unwrap_or(Value::Null);

                Ok(ScriptResult {
                    success,
                    output: output_value,
                    error,
                    duration_ms,
                    memory_used_mb: self.estimate_memory_usage(duration_ms),
                    exit_code: Some(0),
                    stdout: output.to_string(),
                    stderr: String::new(),
                    metrics: HashMap::new(),
                })
            }
            Err(e) => Err(ScriptError::ExecutionFailed(format!(
                "Failed to parse script output as JSON: {}. Output: {}",
                e, output
            ))),
        }
    }
}

/// Lua execution engine using embedded mlua
///
/// FUTURE(#138): Implement Lua engine using mlua crate for embedded execution
#[derive(Debug)]
pub struct LuaEngine {
    #[allow(dead_code)]
    sandbox_manager: Arc<SandboxManager>,
}

impl LuaEngine {
    pub fn new(sandbox_manager: Arc<SandboxManager>) -> Self {
        Self { sandbox_manager }
    }

    /// Check if Lua engine is available
    ///
    /// FUTURE(#138): Implement Lua engine using mlua crate for embedded execution
    pub async fn is_available(&self) -> bool {
        false // Not implemented - requires mlua crate integration
    }
}

impl ScriptEngine for LuaEngine {
    fn execute(
        &self,
        _config: &ScriptConfig,
        _context: &ScriptContext,
    ) -> Result<ScriptResult, ScriptError> {
        // FUTURE(#138): Implement Lua script execution using mlua crate
        Err(ScriptError::UnsupportedLanguage(
            "Lua engine not implemented. Use JavaScript or Python for script validation."
                .to_string(),
        ))
    }

    fn supports(&self, language: &ScriptLanguage) -> bool {
        matches!(language, ScriptLanguage::Lua)
    }

    fn name(&self) -> &'static str {
        "Lua Engine (Future Enhancement)"
    }

    fn validate_syntax(&self, _config: &ScriptConfig) -> Result<(), ScriptError> {
        // FUTURE(#138): Implement Lua syntax validation using mlua crate
        Err(ScriptError::UnsupportedLanguage(
            "Lua syntax validation not implemented".to_string(),
        ))
    }
}

/// Factory for creating script executors
pub struct ScriptExecutorFactory;

impl ScriptExecutorFactory {
    /// Create a new multi-language executor with default sandbox
    pub fn create_default() -> MultiLanguageExecutor {
        MultiLanguageExecutor::new(SandboxConfig::default())
    }

    /// Create a new multi-language executor with custom sandbox
    pub fn create_with_sandbox(sandbox_config: SandboxConfig) -> MultiLanguageExecutor {
        MultiLanguageExecutor::new(sandbox_config)
    }

    /// Create a secure executor for production use
    pub fn create_secure() -> MultiLanguageExecutor {
        let sandbox_config = SandboxConfig {
            strict_mode: true,
            allow_network: false,
            max_execution_time_ms: 10_000, // 10 seconds
            max_memory_mb: 128,            // 128 MB
            ..Default::default()
        };

        MultiLanguageExecutor::new(sandbox_config)
    }

    /// Create a permissive executor for development/testing
    pub fn create_permissive() -> MultiLanguageExecutor {
        let sandbox_config = SandboxConfig {
            strict_mode: false,
            max_execution_time_ms: 60_000, // 60 seconds
            max_memory_mb: 512,            // 512 MB
            ..Default::default()
        };

        MultiLanguageExecutor::new(sandbox_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::TestCase;

    #[test]
    fn test_multi_language_executor_creation() {
        let sandbox_config = SandboxConfig::default();
        let executor = MultiLanguageExecutor::new(sandbox_config);

        assert!(executor
            .javascript_engine
            .supports(&ScriptLanguage::JavaScript));
        assert!(executor.python_engine.supports(&ScriptLanguage::Python));
        assert!(executor.lua_engine.supports(&ScriptLanguage::Lua));
    }

    #[test]
    fn test_javascript_syntax_validation() {
        let sandbox_manager = Arc::new(SandboxManager::new(SandboxConfig::default()));
        let engine = JavaScriptEngine::new(sandbox_manager);

        let valid_config = ScriptConfig {
            language: ScriptLanguage::JavaScript,
            source: "const result = { success: true, message: 'test' };".to_string(),
            name: "test".to_string(),
            ..Default::default()
        };

        assert!(engine.validate_syntax(&valid_config).is_ok());

        let invalid_config = ScriptConfig {
            language: ScriptLanguage::JavaScript,
            source: "const result = { success: true, message: 'test' ".to_string(), // Missing brace
            name: "test".to_string(),
            ..Default::default()
        };

        assert!(engine.validate_syntax(&invalid_config).is_err());
    }

    #[tokio::test]
    async fn test_interpreter_availability() {
        let sandbox_config = SandboxConfig::default();
        let executor = MultiLanguageExecutor::new(sandbox_config);

        // These tests will only pass if interpreters are installed
        // In CI/CD, they might be skipped or mocked
        let _js_available = executor.javascript_engine.is_available().await;
        let _python_available = executor.python_engine.is_available().await;
        let _lua_available = executor.lua_engine.is_available().await;

        // Just ensure the calls don't panic - availability depends on system setup
    }

    #[test]
    fn test_factory_creation() {
        let _default_executor = ScriptExecutorFactory::create_default();
        let _secure_executor = ScriptExecutorFactory::create_secure();
        let _permissive_executor = ScriptExecutorFactory::create_permissive();

        // Ensure all factory methods work without panicking
    }

    #[test]
    fn test_script_execution_flow() {
        let sandbox_config = SandboxConfig::default();
        let executor = MultiLanguageExecutor::new(sandbox_config);

        let config = ScriptConfig {
            language: ScriptLanguage::JavaScript,
            source: "const result = { success: true, message: 'Hello from test!' };".to_string(),
            name: "hello_test".to_string(),
            timeout_ms: 5000,
            ..Default::default()
        };

        let _context = ScriptContext::new(
            TestCase::default(),
            serde_json::json!({"test": "data"}),
            Some(serde_json::json!({"result": "success"})),
            None,
        );

        // Test engine selection and configuration validation
        let engine = executor.get_engine(&config.language);
        assert_eq!(engine.name(), "JavaScript Engine (Node.js)");
        assert!(engine.supports(&ScriptLanguage::JavaScript));

        // Test syntax validation (doesn't require Node.js)
        match engine.validate_syntax(&config) {
            Ok(_) => {
                // Syntax validation passed
            }
            Err(ScriptError::ExecutionFailed(_)) => {
                // Expected if Node.js is not available in test environment
                println!("Node.js not available for syntax validation");
            }
            Err(e) => panic!("Unexpected syntax validation error: {}", e),
        }
    }

    #[test]
    fn test_python_wrapper_script_generation() {
        let sandbox_manager = Arc::new(SandboxManager::new(SandboxConfig::default()));
        let engine = PythonEngine::new(sandbox_manager);

        let config = ScriptConfig {
            language: ScriptLanguage::Python,
            source: "result = {'success': True, 'message': 'test'}".to_string(),
            name: "test".to_string(),
            timeout_ms: 5000,
            ..Default::default()
        };

        let context = ScriptContext::new(
            TestCase::default(),
            serde_json::json!({"test": "data"}),
            Some(serde_json::json!({"result": "success"})),
            None,
        );

        let wrapper = engine.create_wrapper_script(&config, &context);

        assert!(wrapper.contains("import json"));
        assert!(wrapper.contains("result = {'success': True, 'message': 'test'}"));
        assert!(wrapper.contains("TestUtils"));
        assert!(wrapper.contains("TIMEOUT_MS = 5000"));
    }

    #[test]
    fn test_javascript_wrapper_script_generation() {
        let sandbox_manager = Arc::new(SandboxManager::new(SandboxConfig::default()));
        let engine = JavaScriptEngine::new(sandbox_manager);

        let config = ScriptConfig {
            language: ScriptLanguage::JavaScript,
            source: "const result = { success: true, message: 'test' };".to_string(),
            name: "test".to_string(),
            timeout_ms: 5000,
            ..Default::default()
        };

        let context = ScriptContext::new(
            TestCase::default(),
            serde_json::json!({"test": "data"}),
            Some(serde_json::json!({"result": "success"})),
            None,
        );

        let wrapper = engine.create_wrapper_script(&config, &context);

        assert!(wrapper.contains("'use strict'"));
        assert!(wrapper.contains("const result = { success: true, message: 'test' };"));
        assert!(wrapper.contains("testUtils"));
        assert!(wrapper.contains("TIMEOUT_MS = 5000"));
        assert!(wrapper.contains("eval = function()"));
    }
}
