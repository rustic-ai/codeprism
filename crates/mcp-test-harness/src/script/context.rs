//! Script execution context and APIs
//!
//! Provides rich context information for validation scripts including:
//! - Full MCP request/response data
//! - Test metadata and configuration
//! - Previous test results for stateful validation
//! - Custom metrics collection APIs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::spec::TestCase;
use crate::testing::TestResult;

/// Comprehensive context passed to validation scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptContext {
    /// Current test case being executed
    pub test_case: TestCase,
    /// MCP request that was sent
    pub request: serde_json::Value,
    /// MCP response that was received
    pub response: Option<serde_json::Value>,
    /// Error information if request failed
    pub error: Option<String>,
    /// Test execution metadata
    pub metadata: TestMetadata,
    /// Previous test results for stateful validation
    pub previous_results: Vec<TestResult>,
    /// Global test session data
    pub session: SessionContext,
    /// Custom data passed from configuration
    pub custom_data: HashMap<String, serde_json::Value>,
}

/// Test execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetadata {
    /// Test execution timestamp
    pub timestamp: u64,
    /// Test execution duration in milliseconds
    pub duration_ms: u64,
    /// Test case name/identifier
    pub test_name: String,
    /// Test suite name
    pub suite_name: String,
    /// Server configuration used
    pub server_name: String,
    /// Environment information
    pub environment: HashMap<String, String>,
    /// Test execution attempt number (for retries)
    pub attempt: u32,
    /// Whether this is a retry attempt
    pub is_retry: bool,
}

/// Global test session context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Session ID for correlation
    pub session_id: String,
    /// Total tests in session
    pub total_tests: u32,
    /// Current test index (0-based)
    pub current_test_index: u32,
    /// Session start time
    pub session_start: u64,
    /// Accumulated session metrics
    pub session_metrics: HashMap<String, serde_json::Value>,
    /// Session-wide state for stateful testing
    pub session_state: HashMap<String, serde_json::Value>,
}

/// API functions available to scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptAPI {
    /// Functions for test validation
    pub validation: ValidationAPI,
    /// Functions for metrics collection
    pub metrics: MetricsAPI,
    /// Functions for logging and debugging
    pub logging: LoggingAPI,
    /// Functions for external service integration
    pub external: ExternalAPI,
}

/// Validation helper functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationAPI {
    /// Available validation functions
    pub functions: Vec<String>,
}

/// Metrics collection functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsAPI {
    /// Available metrics functions
    pub functions: Vec<String>,
}

/// Logging and debugging functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingAPI {
    /// Available logging functions
    pub functions: Vec<String>,
}

/// External service integration functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalAPI {
    /// Available external functions
    pub functions: Vec<String>,
}

impl ScriptContext {
    /// Create a new script context from test data
    pub fn new(
        test_case: TestCase,
        request: serde_json::Value,
        response: Option<serde_json::Value>,
        error: Option<String>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            test_case: test_case.clone(),
            request,
            response,
            error,
            metadata: TestMetadata {
                timestamp,
                duration_ms: 0,
                test_name: test_case.name.clone(),
                suite_name: "default".to_string(),
                server_name: "default".to_string(),
                environment: std::env::vars().collect(),
                attempt: 1,
                is_retry: false,
            },
            previous_results: Vec::new(),
            session: SessionContext {
                session_id: uuid::Uuid::new_v4().to_string(),
                total_tests: 1,
                current_test_index: 0,
                session_start: timestamp,
                session_metrics: HashMap::new(),
                session_state: HashMap::new(),
            },
            custom_data: HashMap::new(),
        }
    }

    /// Update context with test execution results
    pub fn update_duration(&mut self, duration: Duration) {
        self.metadata.duration_ms = duration.as_millis() as u64;
    }

    /// Add previous test result for stateful validation
    pub fn add_previous_result(&mut self, result: TestResult) {
        self.previous_results.push(result);
    }

    /// Set custom data for script access
    pub fn set_custom_data(&mut self, key: &str, value: serde_json::Value) {
        self.custom_data.insert(key.to_string(), value);
    }

    /// Get custom data
    pub fn get_custom_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.custom_data.get(key)
    }

    /// Update session state
    pub fn update_session_state(&mut self, key: &str, value: serde_json::Value) {
        self.session.session_state.insert(key.to_string(), value);
    }

    /// Get session state
    pub fn get_session_state(&self, key: &str) -> Option<&serde_json::Value> {
        self.session.session_state.get(key)
    }

    /// Add session metric
    pub fn add_session_metric(&mut self, key: &str, value: serde_json::Value) {
        self.session.session_metrics.insert(key.to_string(), value);
    }

    /// Check if response contains expected data
    pub fn response_contains(&self, path: &str) -> bool {
        if let Some(response) = &self.response {
            self.json_path_exists(response, path)
        } else {
            false
        }
    }

    /// Get value from response using JSONPath-like syntax
    pub fn get_response_value(&self, path: &str) -> Option<&serde_json::Value> {
        if let Some(response) = &self.response {
            self.get_json_path_value(response, path)
        } else {
            None
        }
    }

    /// Check if request contains expected data
    pub fn request_contains(&self, path: &str) -> bool {
        self.json_path_exists(&self.request, path)
    }

    /// Get value from request using JSONPath-like syntax
    pub fn get_request_value(&self, path: &str) -> Option<&serde_json::Value> {
        self.get_json_path_value(&self.request, path)
    }

    /// Count successful tests in previous results
    pub fn count_successful_tests(&self) -> usize {
        self.previous_results.iter().filter(|r| r.passed).count()
    }

    /// Count failed tests in previous results
    pub fn count_failed_tests(&self) -> usize {
        self.previous_results.iter().filter(|r| !r.passed).count()
    }

    /// Get average duration of previous tests
    pub fn average_test_duration(&self) -> f64 {
        if self.previous_results.is_empty() {
            0.0
        } else {
            let total: u128 = self
                .previous_results
                .iter()
                .map(|r| r.duration.as_millis())
                .sum();
            total as f64 / self.previous_results.len() as f64
        }
    }

    /// Simple JSON path resolution (supports basic dot notation)
    fn json_path_exists(&self, value: &serde_json::Value, path: &str) -> bool {
        self.get_json_path_value(value, path).is_some()
    }

    /// Get value using simple JSON path (supports basic dot notation)
    fn get_json_path_value<'a>(
        &self,
        value: &'a serde_json::Value,
        path: &str,
    ) -> Option<&'a serde_json::Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = value;

        for part in parts {
            match current {
                serde_json::Value::Object(map) => {
                    current = map.get(part)?;
                }
                serde_json::Value::Array(arr) => {
                    if let Ok(index) = part.parse::<usize>() {
                        current = arr.get(index)?;
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        Some(current)
    }

    /// Create JavaScript context object for script execution
    pub fn to_javascript_context(&self) -> String {
        // Create a JavaScript object with all context data
        format!(
            r#"
const context = {{
    testCase: {},
    request: {},
    response: {},
    error: {},
    metadata: {},
    previousResults: {},
    session: {},
    customData: {},
    
    // Helper functions
    responseContains: function(path) {{
        return this.response && this._jsonPathExists(this.response, path);
    }},
    
    getResponseValue: function(path) {{
        return this.response ? this._getJsonPathValue(this.response, path) : null;
    }},
    
    requestContains: function(path) {{
        return this._jsonPathExists(this.request, path);
    }},
    
    getRequestValue: function(path) {{
        return this._getJsonPathValue(this.request, path);
    }},
    
    countSuccessfulTests: function() {{
        return this.previousResults.filter(r => r.passed).length;
    }},
    
    countFailedTests: function() {{
        return this.previousResults.filter(r => !r.passed).length;
    }},
    
    averageTestDuration: function() {{
        if (this.previousResults.length === 0) return 0;
        const total = this.previousResults.reduce((sum, r) => {{
            // Duration is in nanoseconds, convert to milliseconds
            return sum + (r.duration.nanos / 1000000) + (r.duration.secs * 1000);
        }}, 0);
        return total / this.previousResults.length;
    }},
    
    // Internal helper functions
    _jsonPathExists: function(obj, path) {{
        return this._getJsonPathValue(obj, path) !== null;
    }},
    
    _getJsonPathValue: function(obj, path) {{
        const parts = path.split('.');
        let current = obj;
        for (const part of parts) {{
            if (current === null || current === undefined) return null;
            if (Array.isArray(current)) {{
                const index = parseInt(part);
                if (isNaN(index) || index < 0 || index >= current.length) return null;
                current = current[index];
            }} else if (typeof current === 'object') {{
                current = current[part];
            }} else {{
                return null;
            }}
        }}
        return current;
    }}
}};
"#,
            serde_json::to_string(&self.test_case).unwrap_or_default(),
            serde_json::to_string(&self.request).unwrap_or_default(),
            serde_json::to_string(&self.response).unwrap_or("null".to_string()),
            serde_json::to_string(&self.error).unwrap_or("null".to_string()),
            serde_json::to_string(&self.metadata).unwrap_or_default(),
            serde_json::to_string(&self.previous_results).unwrap_or_default(),
            serde_json::to_string(&self.session).unwrap_or_default(),
            serde_json::to_string(&self.custom_data).unwrap_or_default(),
        )
    }

    /// Create Python context dictionary for script execution
    pub fn to_python_context(&self) -> String {
        format!(
            r#"
import json

class ValidationContext:
    def __init__(self):
        self.test_case = {}
        self.request = {}
        self.response = {}
        self.error = {}
        self.metadata = {}
        self.previous_results = {}
        self.session = {}
        self.custom_data = {}
    
    def response_contains(self, path):
        return self.response is not None and self._json_path_exists(self.response, path)
    
    def get_response_value(self, path):
        return self._get_json_path_value(self.response, path) if self.response else None
    
    def request_contains(self, path):
        return self._json_path_exists(self.request, path)
    
    def get_request_value(self, path):
        return self._get_json_path_value(self.request, path)
    
    def count_successful_tests(self):
        return len([r for r in self.previous_results if r.get('passed', False)])
    
    def count_failed_tests(self):
        return len([r for r in self.previous_results if not r.get('passed', False)])
    
    def average_test_duration(self):
        if not self.previous_results:
            return 0.0
        # Duration is a struct with secs and nanos fields
        total = 0
        for r in self.previous_results:
            duration = r.get('duration', {{}})
            secs = duration.get('secs', 0) * 1000  # Convert to milliseconds
            nanos = duration.get('nanos', 0) // 1_000_000  # Convert to milliseconds
            total += secs + nanos
        return total / len(self.previous_results)
    
    def _json_path_exists(self, obj, path):
        return self._get_json_path_value(obj, path) is not None
    
    def _get_json_path_value(self, obj, path):
        parts = path.split('.')
        current = obj
        for part in parts:
            if current is None:
                return None
            if isinstance(current, list):
                try:
                    index = int(part)
                    if 0 <= index < len(current):
                        current = current[index]
                    else:
                        return None
                except ValueError:
                    return None
            elif isinstance(current, dict):
                current = current.get(part)
            else:
                return None
        return current

context = ValidationContext()
"#,
            serde_json::to_string(&self.test_case).unwrap_or_default(),
            serde_json::to_string(&self.request).unwrap_or_default(),
            serde_json::to_string(&self.response).unwrap_or("null".to_string()),
            serde_json::to_string(&self.error).unwrap_or("null".to_string()),
            serde_json::to_string(&self.metadata).unwrap_or_default(),
            serde_json::to_string(&self.previous_results).unwrap_or_default(),
            serde_json::to_string(&self.session).unwrap_or_default(),
            serde_json::to_string(&self.custom_data).unwrap_or_default(),
        )
    }
}

impl Default for ScriptContext {
    fn default() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            test_case: TestCase::default(),
            request: serde_json::Value::Null,
            response: None,
            error: None,
            metadata: TestMetadata {
                timestamp,
                duration_ms: 0,
                test_name: "default".to_string(),
                suite_name: "default".to_string(),
                server_name: "default".to_string(),
                environment: HashMap::new(),
                attempt: 1,
                is_retry: false,
            },
            previous_results: Vec::new(),
            session: SessionContext {
                session_id: uuid::Uuid::new_v4().to_string(),
                total_tests: 1,
                current_test_index: 0,
                session_start: timestamp,
                session_metrics: HashMap::new(),
                session_state: HashMap::new(),
            },
            custom_data: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_context_creation() {
        let test_case = TestCase::default();
        let request = serde_json::json!({"method": "test", "params": {}});
        let response = Some(serde_json::json!({"result": "success"}));

        let context = ScriptContext::new(test_case, request, response, None);

        assert_eq!(context.test_case.name, "");
        assert!(context.response.is_some());
        assert!(context.error.is_none());
    }

    #[test]
    fn test_json_path_resolution() {
        let context = ScriptContext {
            response: Some(serde_json::json!({
                "result": {
                    "data": [
                        {"name": "item1", "value": 100},
                        {"name": "item2", "value": 200}
                    ]
                }
            })),
            ..Default::default()
        };

        assert!(context.response_contains("result.data"));
        assert!(context.response_contains("result.data.0.name"));
        assert_eq!(
            context.get_response_value("result.data.0.value"),
            Some(&serde_json::Value::Number(serde_json::Number::from(100)))
        );
        assert!(!context.response_contains("result.nonexistent"));
    }

    #[test]
    fn test_test_statistics() {
        let mut context = ScriptContext::default();

        // Add some test results using proper TestResult constructors
        use chrono::Utc;
        use std::time::Duration;

        context.add_previous_result(TestResult::success(
            "test1".to_string(),
            Utc::now(),
            Duration::from_millis(100),
            serde_json::json!({}),
            serde_json::json!({}),
        ));
        context.add_previous_result(TestResult::failure(
            "test2".to_string(),
            Utc::now(),
            Duration::from_millis(200),
            serde_json::json!({}),
            "Test failed".to_string(),
        ));
        context.add_previous_result(TestResult::success(
            "test3".to_string(),
            Utc::now(),
            Duration::from_millis(300),
            serde_json::json!({}),
            serde_json::json!({}),
        ));

        assert_eq!(context.count_successful_tests(), 2);
        assert_eq!(context.count_failed_tests(), 1);
        assert_eq!(context.average_test_duration(), 200.0);
    }

    #[test]
    fn test_custom_data_management() {
        let mut context = ScriptContext::default();

        context.set_custom_data("key1", serde_json::json!("value1"));
        context.set_custom_data("key2", serde_json::json!(42));

        assert_eq!(
            context.get_custom_data("key1"),
            Some(&serde_json::json!("value1"))
        );
        assert_eq!(
            context.get_custom_data("key2"),
            Some(&serde_json::json!(42))
        );
        assert_eq!(context.get_custom_data("nonexistent"), None);
    }

    #[test]
    fn test_session_state_management() {
        let mut context = ScriptContext::default();

        context.update_session_state("counter", serde_json::json!(5));
        context.update_session_state("flag", serde_json::json!(true));

        assert_eq!(
            context.get_session_state("counter"),
            Some(&serde_json::json!(5))
        );
        assert_eq!(
            context.get_session_state("flag"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(context.get_session_state("nonexistent"), None);
    }

    #[test]
    fn test_javascript_context_generation() {
        let context = ScriptContext::default();
        let js_context = context.to_javascript_context();

        assert!(js_context.contains("const context = {"));
        assert!(js_context.contains("responseContains"));
        assert!(js_context.contains("getResponseValue"));
        assert!(js_context.contains("countSuccessfulTests"));
    }

    #[test]
    fn test_python_context_generation() {
        let context = ScriptContext::default();
        let py_context = context.to_python_context();

        assert!(py_context.contains("class ValidationContext"));
        assert!(py_context.contains("def response_contains"));
        assert!(py_context.contains("def get_response_value"));
        assert!(py_context.contains("context = ValidationContext()"));
    }
}
