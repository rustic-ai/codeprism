//! Script Context Framework for Multi-Language Validation Scripts
//!
//! This module provides a rich context API that scripts can use to access:
//! - Test case information and metadata
//! - MCP request and response data
//! - Session data and previous test results
//! - Helper functions for common validation tasks
//!
//! The context is serialized to JSON and then converted to language-specific
//! formats for JavaScript, Python, and Lua script execution.

use crate::spec::{ExecutionPhase, TestCase};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Rich context provided to validation scripts during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptContext {
    /// Current test case being executed
    pub test_case: TestCase,
    /// MCP request that was sent (None for before-phase scripts)
    pub request: Option<JsonValue>,
    /// MCP response that was received (None for before-phase scripts)
    pub response: Option<JsonValue>,
    /// Error information if the MCP request failed
    pub error: Option<String>,
    /// Test execution metadata
    pub metadata: TestMetadata,
    /// Global session data shared across test cases
    pub session: SessionData,
    /// Results from previously executed test cases
    pub previous_results: Vec<TestResult>,
    /// Custom configuration data from test specification
    pub custom_data: JsonValue,
    /// Helper functions available to scripts (language-specific)
    pub helpers: ContextHelpers,
}

/// Test execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetadata {
    /// Unique identifier for this test execution
    pub execution_id: String,
    /// Name of the test specification being executed
    pub spec_name: String,
    /// Version of the test specification
    pub spec_version: String,
    /// Current execution phase (before/after)
    pub execution_phase: ExecutionPhase,
    /// Timestamp when test execution started
    pub start_time: u64,
    /// Current timestamp
    pub current_time: u64,
    /// Test case index within the current tool/resource/prompt
    pub test_index: usize,
    /// Total number of test cases for current tool/resource/prompt
    pub total_tests: usize,
}

/// Global session data shared across test cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Session identifier
    pub session_id: String,
    /// Custom session variables set by scripts
    pub variables: HashMap<String, JsonValue>,
    /// Performance metrics collected during session
    pub metrics: SessionMetrics,
    /// Flags and configuration for the session
    pub config: HashMap<String, JsonValue>,
}

/// Performance metrics for the test session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    /// Total test cases executed so far
    pub tests_executed: usize,
    /// Number of successful test cases
    pub tests_passed: usize,
    /// Number of failed test cases
    pub tests_failed: usize,
    /// Average execution time per test case (milliseconds)
    pub avg_execution_time_ms: f64,
    /// Total session execution time (milliseconds)
    pub total_execution_time_ms: u64,
}

/// Result from a completed test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Name of the test case
    pub test_name: String,
    /// Whether the test passed
    pub success: bool,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Any error message if the test failed
    pub error_message: Option<String>,
    /// Custom data returned by the test
    pub output_data: JsonValue,
}

/// Helper functions available to scripts (language-specific implementations)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextHelpers {
    /// JavaScript helper function code
    pub javascript: Option<String>,
    /// Python helper function code
    pub python: Option<String>,
    /// Lua helper function code
    pub lua: Option<String>,
}

impl ScriptContext {
    /// Create a new ScriptContext for test execution
    pub fn new(
        test_case: TestCase,
        spec_name: String,
        spec_version: String,
        execution_phase: ExecutionPhase,
        test_index: usize,
        total_tests: usize,
    ) -> Self {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let execution_id = format!("exec_{}_{}", current_time, rand::random::<u32>());
        let session_id = format!("session_{current_time}");

        Self {
            test_case,
            request: None,
            response: None,
            error: None,
            metadata: TestMetadata {
                execution_id,
                spec_name,
                spec_version,
                execution_phase,
                start_time: current_time,
                current_time,
                test_index,
                total_tests,
            },
            session: SessionData {
                session_id,
                variables: HashMap::new(),
                metrics: SessionMetrics {
                    tests_executed: 0,
                    tests_passed: 0,
                    tests_failed: 0,
                    avg_execution_time_ms: 0.0,
                    total_execution_time_ms: 0,
                },
                config: HashMap::new(),
            },
            previous_results: Vec::new(),
            custom_data: JsonValue::Null,
            helpers: ContextHelpers::default(),
        }
    }

    /// Add MCP request data to the context
    pub fn with_request(mut self, request: JsonValue) -> Self {
        self.request = Some(request);
        self
    }

    /// Add MCP response data to the context
    pub fn with_response(mut self, response: JsonValue) -> Self {
        self.response = Some(response);
        self
    }

    /// Add error information to the context
    pub fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self
    }

    /// Add session data to the context
    pub fn with_session_data(mut self, session: SessionData) -> Self {
        self.session = session;
        self
    }

    /// Add previous test results to the context
    pub fn with_previous_results(mut self, results: Vec<TestResult>) -> Self {
        self.previous_results = results;
        self
    }

    /// Add custom configuration data to the context
    pub fn with_custom_data(mut self, data: JsonValue) -> Self {
        self.custom_data = data;
        self
    }

    /// Generate JavaScript-specific context with helper functions
    pub fn to_javascript(&self) -> Result<String, ScriptContextError> {
        let helpers = self.generate_javascript_helpers();
        let context_json =
            serde_json::to_string_pretty(self).map_err(ScriptContextError::SerializationError)?;

        Ok(format!(
            r#"
// Context object with test data and helper functions
const context = {context_json};

// Helper functions for JavaScript validation scripts
{helpers}

// Make context globally available
globalThis.context = context;
"#
        ))
    }

    /// Generate Python-specific context with helper functions
    pub fn to_python(&self) -> Result<String, ScriptContextError> {
        let helpers = self.generate_python_helpers();
        let context_json =
            serde_json::to_string(self).map_err(ScriptContextError::SerializationError)?;

        // Convert JSON null/true/false to Python None/True/False
        let python_context = context_json
            .replace("null", "None")
            .replace("true", "True")
            .replace("false", "False");

        Ok(format!(
            r#"
# Context object with test data and helper functions
import json
context = {python_context}

# Helper functions for Python validation scripts
{helpers}
"#
        ))
    }

    /// Generate Lua-specific context with helper functions
    pub fn to_lua(&self) -> Result<String, ScriptContextError> {
        let helpers = self.generate_lua_helpers();
        let lua_context = self.to_lua_table()?;

        Ok(format!(
            r#"
-- Context table with test data and helper functions
local context = {lua_context}

-- Helper functions for Lua validation scripts
{helpers}

-- Make context globally available
_G.context = context
"#
        ))
    }

    /// Convert context to Lua table format
    fn to_lua_table(&self) -> Result<String, ScriptContextError> {
        // Serialize to JSON and convert to proper Lua table syntax
        let json =
            serde_json::to_string_pretty(self).map_err(ScriptContextError::SerializationError)?;

        // Convert JSON to Lua table syntax with proper type mappings
        let lua_table = json
            .replace("null", "nil")
            .replace("[", "{")
            .replace("]", "}");

        Ok(lua_table)
    }

    /// Generate JavaScript helper functions
    fn generate_javascript_helpers(&self) -> String {
        r#"
// Helper function to check if response contains a specific path
function responseContains(path) {
    if (!context.response) return false;
    return getResponseValue(path) !== undefined;
}

// Helper function to get value at JSONPath
function getResponseValue(path) {
    if (!context.response) return undefined;
    // Simple path traversal (could be enhanced with JSONPath library)
    const parts = path.replace(/^\$\./, '').split('.');
    let current = context.response;
    for (const part of parts) {
        if (current === null || current === undefined) return undefined;
        current = current[part];
    }
    return current;
}

// Helper function to count successful tests
function countSuccessfulTests() {
    return context.previous_results.filter(r => r.success).length;
}

// Helper function to calculate average test duration
function averageTestDuration() {
    if (context.previous_results.length === 0) return 0;
    const total = context.previous_results.reduce((sum, r) => sum + r.execution_time_ms, 0);
    return total / context.previous_results.length;
}

// Helper function to get session variable
function getSessionVar(name) {
    return context.session.variables[name];
}

// Helper function to set session variable
function setSessionVar(name, value) {
    context.session.variables[name] = value;
}
"#
        .to_string()
    }

    /// Generate Python helper functions
    fn generate_python_helpers(&self) -> String {
        r#"
def response_contains(path):
    """Check if response contains a specific path"""
    if not context.get('response'):
        return False
    return get_response_value(path) is not None

def get_response_value(path):
    """Get value at JSONPath"""
    if not context.get('response'):
        return None
    # Simple path traversal
    parts = path.replace('$.', '').split('.')
    current = context['response']
    for part in parts:
        if current is None or not isinstance(current, dict):
            return None
        current = current.get(part)
    return current

def count_successful_tests():
    """Count successful previous tests"""
    return len([r for r in context['previous_results'] if r['success']])

def average_test_duration():
    """Calculate average test duration"""
    results = context['previous_results']
    if not results:
        return 0
    total = sum(r['execution_time_ms'] for r in results)
    return total / len(results)

def get_session_var(name):
    """Get session variable"""
    return context['session']['variables'].get(name)

def set_session_var(name, value):
    """Set session variable"""
    context['session']['variables'][name] = value
"#
        .to_string()
    }

    /// Generate Lua helper functions
    fn generate_lua_helpers(&self) -> String {
        r#"
-- Helper function to check if response contains a specific path
function response_contains(path)
    if not context.response then return false end
    return get_response_value(path) ~= nil
end

-- Helper function to get value at JSONPath
function get_response_value(path)
    if not context.response then return nil end
    -- Simple path traversal
    local parts = {}
    for part in string.gmatch(path:gsub("^%$%.", ""), "[^%.]+") do
        table.insert(parts, part)
    end
    
    local current = context.response
    for _, part in ipairs(parts) do
        if type(current) ~= "table" then return nil end
        current = current[part]
    end
    return current
end

-- Helper function to count successful tests
function count_successful_tests()
    local count = 0
    for _, result in ipairs(context.previous_results) do
        if result.success then count = count + 1 end
    end
    return count
end

-- Helper function to calculate average test duration
function average_test_duration()
    if #context.previous_results == 0 then return 0 end
    local total = 0
    for _, result in ipairs(context.previous_results) do
        total = total + result.execution_time_ms
    end
    return total / #context.previous_results
end

-- Helper function to get session variable
function get_session_var(name)
    return context.session.variables[name]
end

-- Helper function to set session variable
function set_session_var(name, value)
    context.session.variables[name] = value
end
"#
        .to_string()
    }
}

impl Default for SessionData {
    fn default() -> Self {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            session_id: format!("session_{current_time}"),
            variables: HashMap::new(),
            metrics: SessionMetrics::default(),
            config: HashMap::new(),
        }
    }
}

impl Default for SessionMetrics {
    fn default() -> Self {
        Self {
            tests_executed: 0,
            tests_passed: 0,
            tests_failed: 0,
            avg_execution_time_ms: 0.0,
            total_execution_time_ms: 0,
        }
    }
}

/// Errors that can occur during script context operations
#[derive(Debug, thiserror::Error)]
pub enum ScriptContextError {
    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Context generation error: {0}")]
    GenerationError(String),
    #[error("Invalid context data: {0}")]
    InvalidData(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::ExpectedOutput;

    fn create_test_case() -> TestCase {
        TestCase {
            name: "test_add_integers".to_string(),
            description: Some("Test adding two integers".to_string()),
            dependencies: None,
            input: serde_json::json!({"a": 5, "b": 3}),
            expected: ExpectedOutput {
                ..Default::default()
            },
            performance: None,
            skip: false,
            tags: vec!["math".to_string(), "basic".to_string()],
            validation_scripts: Some(vec!["math_validator".to_string()]),
            test_config: None,
        }
    }

    #[test]
    fn test_script_context_creation() {
        let test_case = create_test_case();
        let context = ScriptContext::new(
            test_case,
            "Test Server".to_string(),
            "1.0.0".to_string(),
            ExecutionPhase::After,
            0,
            5,
        );

        assert_eq!(context.metadata.spec_name, "Test Server");
        assert_eq!(context.metadata.spec_version, "1.0.0");
        assert_eq!(context.metadata.execution_phase, ExecutionPhase::After);
        assert_eq!(context.metadata.test_index, 0);
        assert_eq!(context.metadata.total_tests, 5);
        assert!(context.request.is_none(), "Should be none");
        assert!(context.response.is_none(), "Should be none");
    }

    #[test]
    fn test_script_context_with_request_response() {
        let test_case = create_test_case();
        let request = serde_json::json!({"tool": "add", "params": {"a": 5, "b": 3}});
        let response = serde_json::json!([{"text": "8"}]);

        let context = ScriptContext::new(
            test_case,
            "Test Server".to_string(),
            "1.0.0".to_string(),
            ExecutionPhase::After,
            0,
            1,
        )
        .with_request(request.clone())
        .with_response(response.clone());

        assert_eq!(context.request, Some(request));
        assert_eq!(context.response, Some(response));
    }

    #[test]
    fn test_javascript_context_generation() {
        let test_case = create_test_case();
        let context = ScriptContext::new(
            test_case,
            "Test Server".to_string(),
            "1.0.0".to_string(),
            ExecutionPhase::After,
            0,
            1,
        )
        .with_request(serde_json::json!({"tool": "add"}))
        .with_response(serde_json::json!([{"text": "result"}]));

        let js_code = context.to_javascript().unwrap();

        assert!(js_code.contains("const context ="));
        assert!(js_code.contains("function responseContains"));
        assert!(js_code.contains("function getResponseValue"));
        assert!(js_code.contains("globalThis.context = context"));
    }

    #[test]
    fn test_python_context_generation() {
        let test_case = create_test_case();
        let context = ScriptContext::new(
            test_case,
            "Test Server".to_string(),
            "1.0.0".to_string(),
            ExecutionPhase::After,
            0,
            1,
        )
        .with_request(serde_json::json!({"tool": "add"}))
        .with_response(serde_json::json!([{"text": "result"}]));

        let python_code = context.to_python().unwrap();

        assert!(python_code.contains("context ="));
        assert!(python_code.contains("def response_contains"));
        assert!(python_code.contains("def get_response_value"));
        assert!(python_code.contains("None"));
        assert!(python_code.contains("True"));
        assert!(python_code.contains("False"));
    }

    #[test]
    fn test_lua_context_generation() {
        let test_case = create_test_case();
        let context = ScriptContext::new(
            test_case,
            "Test Server".to_string(),
            "1.0.0".to_string(),
            ExecutionPhase::After,
            0,
            1,
        )
        .with_request(serde_json::json!({"tool": "add"}))
        .with_response(serde_json::json!([{"text": "result"}]));

        let lua_code = context.to_lua().unwrap();

        assert!(lua_code.contains("local context ="));
        assert!(lua_code.contains("function response_contains"));
        assert!(lua_code.contains("function get_response_value"));
        assert!(lua_code.contains("_G.context = context"));
    }

    #[test]
    fn test_session_data_management() {
        let test_case = create_test_case();
        let mut session = SessionData::default();
        session
            .variables
            .insert("test_var".to_string(), serde_json::json!("test_value"));
        session.metrics.tests_executed = 5;
        session.metrics.tests_passed = 4;

        let context = ScriptContext::new(
            test_case,
            "Test Server".to_string(),
            "1.0.0".to_string(),
            ExecutionPhase::After,
            0,
            1,
        )
        .with_session_data(session.clone());

        assert_eq!(
            context.session.variables.get("test_var"),
            Some(&serde_json::json!("test_value"))
        );
        assert_eq!(context.session.metrics.tests_executed, 5);
        assert_eq!(context.session.metrics.tests_passed, 4);
    }
}
