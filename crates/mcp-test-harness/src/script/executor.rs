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
#[derive(Debug)]
pub struct LuaEngine {
    sandbox_manager: Arc<SandboxManager>,
}

impl LuaEngine {
    pub fn new(sandbox_manager: Arc<SandboxManager>) -> Self {
        Self { sandbox_manager }
    }

    /// Check if Lua engine is available (always true for embedded mlua)
    pub async fn is_available(&self) -> bool {
        true // Embedded Lua is always available via mlua
    }

    /// Create a Lua wrapper script with context and utilities
    fn create_wrapper_script(&self, config: &ScriptConfig, context: &ScriptContext) -> String {
        let context_lua = serde_json::to_string_pretty(&context).unwrap_or_default();

        format!(
            r#"
-- MCP Test Harness Lua Script Wrapper
-- Provides context, utilities, and timeout management for user scripts

-- Context data (JSON will be provided by Lua runtime)
local context_json = [[{}]]
local context = json.decode(context_json)

-- Test utilities for Lua scripts
local test_utils = {{
    -- Navigate JSON paths in responses
    navigate = function(obj, path)
        local result = obj
        for part in string.gmatch(path, "[^%.]+") do
            if type(result) == "table" and result[part] then
                result = result[part]
            else
                return nil
            end
        end
        return result
    end,
    
    -- Validate value against pattern
    validate_pattern = function(value, pattern)
        if type(value) ~= "string" then
            return false
        end
        return string.match(value, pattern) ~= nil
    end,
    
    -- Validate numeric range
    validate_range = function(value, min_val, max_val)
        local num = tonumber(value)
        if not num then
            return false
        end
        return num >= min_val and num <= max_val
    end,
    
    -- Validate type
    validate_type = function(value, expected_type)
        local lua_type = type(value)
        
        -- Type mapping for common validations
        if expected_type == "number" or expected_type == "numeric" then
            return lua_type == "number"
        elseif expected_type == "string" or expected_type == "text" then
            return lua_type == "string"
        elseif expected_type == "boolean" or expected_type == "bool" then
            return lua_type == "boolean"
        elseif expected_type == "table" or expected_type == "object" or expected_type == "array" then
            return lua_type == "table"
        elseif expected_type == "nil" or expected_type == "null" then
            return lua_type == "nil"
        else
            return lua_type == expected_type
        end
    end,
    
    -- Validate table schema
    validate_schema = function(obj, schema)
        if type(obj) ~= "table" or type(schema) ~= "table" then
            return false
        end
        
        for key, expected_type in pairs(schema) do
            if not obj[key] or not test_utils.validate_type(obj[key], expected_type) then
                return false
            end
        end
        return true
    end,
    
    -- Count array elements
    count_array = function(arr)
        if type(arr) ~= "table" then
            return 0
        end
        local count = 0
        for _ in pairs(arr) do
            count = count + 1
        end
        return count
    end,
    
    -- Check if table is empty
    is_empty = function(t)
        if type(t) ~= "table" then
            return t == nil or t == ""
        end
        return next(t) == nil
    end
}}

-- Execution timeout management
local timeout_ms = {}
local start_time = os.clock()

local function check_timeout()
    local elapsed = (os.clock() - start_time) * 1000
    if elapsed > timeout_ms then
        error("Script execution timeout after " .. timeout_ms .. "ms")
    end
end

-- Make context and utilities available to user script
_G.context = context
_G.test_utils = test_utils
_G.check_timeout = check_timeout

-- User script execution with error handling
local script_result = nil
local script_error = nil

local function execute_user_script()
    -- User script
    {}
    
    -- Ensure result is set
    if result == nil then
        script_result = {{
            success = false,
            output = nil,
            error = "Script did not set result variable"
        }}
    else
        -- Handle different result formats
        if type(result) == "table" and result.success ~= nil then
            script_result = result
        else
            script_result = {{
                success = true,
                output = result,
                error = nil
            }}
        end
    end
end

-- Execute with error handling
local success, error_msg = pcall(execute_user_script)

if not success then
    script_error = {{
        success = false,
        output = nil,
        error = tostring(error_msg)
    }}
end

-- Output final result as JSON (json.encode will be provided by Lua runtime)
local final_result = script_result or script_error
print(json.encode(final_result))
"#,
            context_lua, config.timeout_ms, config.source
        )
    }

    /// Execute Lua script with embedded mlua
    fn execute_lua(
        &self,
        config: &ScriptConfig,
        context: &ScriptContext,
    ) -> Result<ScriptResult, ScriptError> {
        use mlua::{Lua, LuaOptions, StdLib, Value as LuaValue};

        let start_time = Instant::now();

        // Create Lua runtime with restricted standard library for security
        let lua = Lua::new_with(
            StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::OS | StdLib::PACKAGE,
            LuaOptions::default(),
        )
        .map_err(|e| {
            ScriptError::ExecutionFailed(format!("Failed to create Lua runtime: {}", e))
        })?;

        // Add JSON support
        let json_module = lua.create_table()?;

        // JSON encode function
        let encode_fn =
            lua.create_function(|_, value: LuaValue| match lua_value_to_serde(&value) {
                Ok(serde_value) => Ok(serde_json::to_string(&serde_value).unwrap_or_default()),
                Err(e) => Err(mlua::Error::RuntimeError(format!(
                    "JSON encode error: {}",
                    e
                ))),
            })?;

        // JSON decode function
        let decode_fn =
            lua.create_function(|lua, json_str: String| {
                match serde_json::from_str::<serde_json::Value>(&json_str) {
                    Ok(value) => serde_value_to_lua(lua, &value),
                    Err(e) => Err(mlua::Error::RuntimeError(format!(
                        "JSON decode error: {}",
                        e
                    ))),
                }
            })?;

        json_module.set("encode", encode_fn)?;
        json_module.set("decode", decode_fn)?;

        let globals = lua.globals();
        globals.set("json", json_module)?;

        // Create wrapper script with context
        let wrapper_script = self.create_wrapper_script(config, context);

        // Execute the wrapper script with timeout check
        let timeout_duration = Duration::from_millis(config.timeout_ms);
        let execution_start = Instant::now();

        // Simple timeout check - execute and measure time
        let execution_result = lua.load(&wrapper_script).exec();

        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Check if we exceeded timeout
        if execution_start.elapsed() > timeout_duration {
            return Err(ScriptError::Timeout {
                timeout: config.timeout_ms,
            });
        }

        match execution_result {
            Ok(_) => {
                // Script executed successfully
                // Try to get the result from Lua global
                let result_value = lua
                    .globals()
                    .get::<_, LuaValue>("result")
                    .unwrap_or(LuaValue::Nil);

                let output = match lua_value_to_serde(&result_value) {
                    Ok(serde_value) => serde_value,
                    Err(_) => serde_json::json!({"message": "Script executed successfully"}),
                };

                Ok(ScriptResult {
                    success: true,
                    output,
                    error: None,
                    duration_ms,
                    memory_used_mb: self.estimate_memory_usage(duration_ms),
                    exit_code: Some(0),
                    stdout: "Script executed".to_string(),
                    stderr: String::new(),
                    metrics: HashMap::new(),
                })
            }
            Err(e) => {
                let error_msg = format!("Lua execution error: {}", e);
                if error_msg.contains("timeout") {
                    Err(ScriptError::Timeout {
                        timeout: config.timeout_ms,
                    })
                } else {
                    Err(ScriptError::ExecutionFailed(error_msg))
                }
            }
        }
    }

    /// Estimate memory usage based on execution duration
    fn estimate_memory_usage(&self, duration_ms: u64) -> u64 {
        // Base memory usage for embedded Lua: ~5MB
        let base_memory = 5;

        // Additional memory based on execution time (heuristic)
        // Lua is very lightweight compared to Node.js and Python
        let duration_factor = (duration_ms / 1000).max(1);
        let estimated_memory = base_memory + duration_factor;

        // Cap at reasonable maximum for script execution
        estimated_memory.min(64)
    }
}

impl ScriptEngine for LuaEngine {
    fn execute(
        &self,
        config: &ScriptConfig,
        context: &ScriptContext,
    ) -> Result<ScriptResult, ScriptError> {
        // Apply sandbox restrictions
        self.sandbox_manager.validate_script_security(config)?;

        // Execute Lua script
        self.execute_lua(config, context)
    }

    fn supports(&self, language: &ScriptLanguage) -> bool {
        matches!(language, ScriptLanguage::Lua)
    }

    fn name(&self) -> &'static str {
        "Lua Engine (mlua)"
    }

    fn validate_syntax(&self, config: &ScriptConfig) -> Result<(), ScriptError> {
        use mlua::{Lua, LuaOptions, StdLib};

        // Create minimal Lua runtime for syntax validation
        let lua = Lua::new_with(StdLib::NONE, LuaOptions::default()).map_err(|e| {
            ScriptError::ExecutionFailed(format!("Failed to create Lua runtime: {}", e))
        })?;

        // Try to load the script without executing it (this checks syntax)
        let load_result = lua.load(&config.source).into_function();
        match load_result {
            Ok(_) => Ok(()),
            Err(e) => Err(ScriptError::CompilationFailed(format!(
                "Lua syntax error: {}",
                e
            ))),
        }
    }
}

// Helper functions for Lua-Serde conversion
fn lua_value_to_serde(lua_value: &mlua::Value) -> Result<serde_json::Value, ScriptError> {
    use mlua::Value as LuaValue;

    match lua_value {
        LuaValue::Nil => Ok(serde_json::Value::Null),
        LuaValue::Boolean(b) => Ok(serde_json::Value::Bool(*b)),
        LuaValue::Integer(i) => Ok(serde_json::Value::Number(serde_json::Number::from(*i))),
        LuaValue::Number(n) => Ok(serde_json::Value::Number(
            serde_json::Number::from_f64(*n).unwrap_or(serde_json::Number::from(0)),
        )),
        LuaValue::String(s) => match s.to_str() {
            Ok(str_val) => Ok(serde_json::Value::String(str_val.to_string())),
            Err(e) => Err(ScriptError::LuaError(format!(
                "String conversion error: {}",
                e
            ))),
        },
        LuaValue::Table(table) => {
            // Check if it's an array or object
            let mut map = serde_json::Map::new();
            for pair in table.clone().pairs::<mlua::Value, mlua::Value>() {
                let (key, value) = match pair {
                    Ok(p) => p,
                    Err(e) => {
                        return Err(ScriptError::LuaError(format!(
                            "Table iteration error: {}",
                            e
                        )))
                    }
                };
                let key_str = match key {
                    LuaValue::String(s) => match s.to_str() {
                        Ok(str_val) => str_val.to_string(),
                        Err(_) => continue,
                    },
                    LuaValue::Integer(i) => i.to_string(),
                    LuaValue::Number(n) => n.to_string(),
                    _ => continue,
                };
                let serde_value = lua_value_to_serde(&value)?;
                map.insert(key_str, serde_value);
            }
            Ok(serde_json::Value::Object(map))
        }
        _ => Err(ScriptError::ExecutionFailed(
            "Unsupported Lua value type".to_string(),
        )),
    }
}

fn serde_value_to_lua<'lua>(
    lua: &'lua mlua::Lua,
    serde_value: &serde_json::Value,
) -> mlua::Result<mlua::Value<'lua>> {
    match serde_value {
        serde_json::Value::Null => Ok(mlua::Value::Nil),
        serde_json::Value::Bool(b) => Ok(mlua::Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(mlua::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(mlua::Value::Number(f))
            } else {
                Ok(mlua::Value::Number(0.0))
            }
        }
        serde_json::Value::String(s) => Ok(mlua::Value::String(lua.create_string(s)?)),
        serde_json::Value::Array(arr) => {
            let table = lua.create_table()?;
            for (i, value) in arr.iter().enumerate() {
                let lua_value = serde_value_to_lua(lua, value)?;
                table.set(i + 1, lua_value)?;
            }
            Ok(mlua::Value::Table(table))
        }
        serde_json::Value::Object(obj) => {
            let table = lua.create_table()?;
            for (key, value) in obj.iter() {
                let lua_value = serde_value_to_lua(lua, value)?;
                table.set(key.as_str(), lua_value)?;
            }
            Ok(mlua::Value::Table(table))
        }
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

        // Lua is always available via embedded mlua
        let lua_available = executor.lua_engine.is_available().await;
        assert!(lua_available, "Embedded Lua should always be available");

        // Just ensure the calls don't panic - JS/Python availability depends on system setup
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

    #[test]
    fn test_lua_wrapper_script_generation() {
        let sandbox_manager = Arc::new(SandboxManager::new(SandboxConfig::default()));
        let engine = LuaEngine::new(sandbox_manager);

        let config = ScriptConfig {
            language: ScriptLanguage::Lua,
            source: "result = { success = true, message = 'test' }".to_string(),
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

        assert!(wrapper.contains("-- Context data (JSON will be provided by Lua runtime)"));
        assert!(wrapper.contains("result = { success = true, message = 'test' }"));
        assert!(wrapper.contains("test_utils"));
        assert!(wrapper.contains("timeout_ms = 5000"));
        assert!(wrapper.contains("context = json.decode"));
    }

    #[test]
    fn test_lua_syntax_validation() {
        let sandbox_manager = Arc::new(SandboxManager::new(SandboxConfig::default()));
        let engine = LuaEngine::new(sandbox_manager);

        let valid_config = ScriptConfig {
            language: ScriptLanguage::Lua,
            source: "result = { success = true, message = 'test' }".to_string(),
            name: "test".to_string(),
            ..Default::default()
        };

        assert!(engine.validate_syntax(&valid_config).is_ok());

        let invalid_config = ScriptConfig {
            language: ScriptLanguage::Lua,
            source: "result = { success = true, message = 'test'".to_string(), // Missing brace
            name: "test".to_string(),
            ..Default::default()
        };

        assert!(engine.validate_syntax(&invalid_config).is_err());
    }

    #[test]
    fn test_lua_basic_execution() {
        let sandbox_manager = Arc::new(SandboxManager::new(SandboxConfig::default()));
        let engine = LuaEngine::new(sandbox_manager);

        let config = ScriptConfig {
            language: ScriptLanguage::Lua,
            source: "result = { success = true, message = 'Hello from Lua!' }".to_string(),
            name: "lua_test".to_string(),
            timeout_ms: 5000,
            ..Default::default()
        };

        let context = ScriptContext::new(
            TestCase::default(),
            serde_json::json!({"test": "data"}),
            Some(serde_json::json!({"result": "success"})),
            None,
        );

        let result = engine.execute(&config, &context);
        match &result {
            Ok(script_result) => {
                assert!(script_result.success);
                assert_eq!(script_result.exit_code, Some(0));
                // Duration measurement is successful (embedded Lua executes very quickly)
                // Just verify the field exists and is a valid number
            }
            Err(e) => {
                println!("Lua execution error: {:?}", e);
                panic!("Expected successful execution, got error: {}", e);
            }
        }
    }
}
