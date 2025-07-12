# [Issue 250] Design Document: Implement ScriptConfig, ScriptContext, ScriptResult, ScriptError Types

## Problem Statement

The multi-language script validation system (issues #248 and #249) requires core types and context API to enable script execution with proper configuration, context passing, result handling, and error management. These foundational types will be used by the script engines implemented in issue #249 to provide a consistent interface across Lua, JavaScript, and Python execution environments.

## Requirements

### Functional Requirements
- **ScriptConfig**: Configuration for script execution (timeouts, memory limits, security settings)
- **ScriptContext**: Runtime context passed to scripts (request data, response data, metadata)
- **ScriptResult**: Standardized result type for script execution outcomes
- **ScriptError**: Comprehensive error handling for script execution failures
- **Serialization**: All types must support JSON serialization/deserialization
- **Documentation**: Comprehensive rustdoc with working examples

### Non-Functional Requirements
- **Performance**: Context creation and serialization <1ms
- **Memory Safety**: No memory leaks or unsafe operations
- **Thread Safety**: All types must be Send + Sync for concurrent execution
- **Extensibility**: Design for future enhancements without breaking changes

## Proposed Solution

### Type Hierarchy

```rust
// Configuration for script execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptConfig {
    pub timeout_ms: u64,
    pub memory_limit_mb: Option<u64>,
    pub max_output_size: usize,
    pub allow_network: bool,
    pub allow_filesystem: bool,
    pub environment_variables: HashMap<String, String>,
}

// Runtime context passed to scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptContext {
    pub request: serde_json::Value,
    pub response: Option<serde_json::Value>,
    pub metadata: ContextMetadata,
    pub config: ScriptConfig,
}

// Metadata about the execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    pub test_name: String,
    pub execution_id: uuid::Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub tool_name: String,
    pub server_info: ServerInfo,
}

// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
}

// Result of script execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResult {
    pub success: bool,
    pub output: serde_json::Value,
    pub logs: Vec<LogEntry>,
    pub duration_ms: u64,
    pub memory_used_mb: Option<f64>,
    pub error: Option<ScriptError>,
}

// Log entry from script execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// Log levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

// Comprehensive error handling
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
}
```

## API Design

### ScriptConfig Implementation

```rust
impl ScriptConfig {
    /// Creates a new ScriptConfig with default security settings
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
    fn default() -> Self {
        Self::new()
    }
}
```

### ScriptContext Implementation

```rust
impl ScriptContext {
    /// Creates a new script context
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
    pub fn with_response(mut self, response: serde_json::Value) -> Self {
        self.response = Some(response);
        self
    }
    
    /// Sets the server information
    pub fn with_server_info(mut self, server_info: ServerInfo) -> Self {
        self.metadata.server_info = server_info;
        self
    }
    
    /// Gets a value from the request by JSONPath
    pub fn get_request_value(&self, path: &str) -> Result<serde_json::Value, ScriptError> {
        // Implementation would use jsonpath_lib
        todo!("Implement JSONPath extraction")
    }
    
    /// Gets a value from the response by JSONPath
    pub fn get_response_value(&self, path: &str) -> Result<serde_json::Value, ScriptError> {
        let response = self.response.as_ref().ok_or_else(|| ScriptError::ExecutionError {
            message: "No response data available".to_string(),
        })?;
        
        // Implementation would use jsonpath_lib
        todo!("Implement JSONPath extraction")
    }
}
```

### ScriptResult Implementation

```rust
impl ScriptResult {
    /// Creates a successful result
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
    pub fn add_log(mut self, level: LogLevel, message: String) -> Self {
        self.logs.push(LogEntry {
            level,
            message,
            timestamp: chrono::Utc::now(),
        });
        self
    }
    
    /// Sets memory usage information
    pub fn with_memory_usage(mut self, memory_mb: f64) -> Self {
        self.memory_used_mb = Some(memory_mb);
        self
    }
}
```

## Implementation Plan

### Phase 1: Core Types (TDD Red)
1. **Write failing tests** for each type:
   - `test_script_config_creation_and_validation()`
   - `test_script_context_construction()`
   - `test_script_result_success_and_failure()`
   - `test_script_error_variants()`
   - `test_serialization_round_trip()`

2. **Define type signatures** without implementation

### Phase 2: Basic Implementation (TDD Green)
1. **Implement core functionality**:
   - Basic constructors and methods
   - Serialization support
   - Error type definitions

2. **Make all tests pass**

### Phase 3: Enhanced Features (TDD Refactor)
1. **Add advanced functionality**:
   - JSONPath support in ScriptContext
   - Validation methods
   - Builder patterns
   - Performance optimizations

2. **Add comprehensive documentation**
3. **Add integration tests**

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_script_config_creation_and_validation() {
        let config = ScriptConfig::new();
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.memory_limit_mb, Some(100));
        assert!(!config.allow_network);
        assert!(!config.allow_filesystem);
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_script_config_invalid_timeout() {
        let mut config = ScriptConfig::new();
        config.timeout_ms = 0;
        assert!(config.validate().is_err());
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
        assert!(context.response.is_none());
        assert_eq!(context.metadata.test_name, "test_case");
        assert_eq!(context.metadata.tool_name, "test_tool");
    }
    
    #[test]
    fn test_script_result_success() {
        let output = serde_json::json!({"result": "success"});
        let result = ScriptResult::success(output.clone(), 100);
        
        assert!(result.success);
        assert_eq!(result.output, output);
        assert_eq!(result.duration_ms, 100);
        assert!(result.error.is_none());
    }
    
    #[test]
    fn test_script_result_failure() {
        let error = ScriptError::RuntimeError {
            message: "Test error".to_string(),
        };
        let result = ScriptResult::failure(error.clone(), 50);
        
        assert!(!result.success);
        assert_eq!(result.duration_ms, 50);
        assert!(result.error.is_some());
    }
    
    #[test]
    fn test_serialization_round_trip() {
        let config = ScriptConfig::new();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: ScriptConfig = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(config.timeout_ms, deserialized.timeout_ms);
        assert_eq!(config.memory_limit_mb, deserialized.memory_limit_mb);
    }
    
    #[test]
    fn test_script_error_display() {
        let error = ScriptError::TimeoutError { timeout_ms: 5000 };
        let message = error.to_string();
        assert!(message.contains("5000ms"));
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
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_context_with_response() {
    let request = serde_json::json!({"input": "test"});
    let response = serde_json::json!({"output": "result"});
    
    let context = ScriptContext::new(
        request,
        "integration_test".to_string(),
        "test_tool".to_string(),
        ScriptConfig::new(),
    ).with_response(response.clone());
    
    assert_eq!(context.response, Some(response));
}

#[test]
fn test_performance_context_creation() {
    let start = std::time::Instant::now();
    
    for _ in 0..1000 {
        let _context = ScriptContext::new(
            serde_json::json!({"test": "data"}),
            "perf_test".to_string(),
            "tool".to_string(),
            ScriptConfig::new(),
        );
    }
    
    let duration = start.elapsed();
    assert!(duration.as_millis() < 100, "Context creation too slow: {}ms", duration.as_millis());
}
```

## Integration Points

### With Issue #248 (validation_scripts)
- `ValidationScript` will reference these types for execution
- Script language determines which engine processes the `ScriptContext`

### With Issue #249 (dependencies)
- Each engine (mlua, rquickjs, pyo3) will consume `ScriptContext`
- Engines will produce `ScriptResult` with appropriate error handling
- Configuration will control engine behavior

### With Issue #247 (multi-language validation)
- These types form the foundation for the complete validation system
- Results will be aggregated into test execution reports

## Serialization Examples

### ScriptConfig JSON
```json
{
  "timeout_ms": 5000,
  "memory_limit_mb": 100,
  "max_output_size": 1048576,
  "allow_network": false,
  "allow_filesystem": false,
  "environment_variables": {}
}
```

### ScriptContext JSON
```json
{
  "request": {"tool": "test", "params": {"input": "data"}},
  "response": {"result": "success", "data": "output"},
  "metadata": {
    "test_name": "test_tool_execution",
    "execution_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2024-01-15T10:30:00Z",
    "tool_name": "test_tool",
    "server_info": {
      "name": "Test Server",
      "version": "1.0.0",
      "capabilities": ["tools", "resources"]
    }
  },
  "config": { /* ScriptConfig */ }
}
```

### ScriptResult JSON
```json
{
  "success": true,
  "output": {"validation": "passed", "score": 100},
  "logs": [
    {
      "level": "Info",
      "message": "Script execution started",
      "timestamp": "2024-01-15T10:30:00Z"
    }
  ],
  "duration_ms": 150,
  "memory_used_mb": 2.5,
  "error": null
}
```

## Success Criteria

### Compilation Success
- [ ] All types compile without warnings
- [ ] All traits properly implemented (Debug, Clone, Serialize, Deserialize)
- [ ] No circular dependencies

### Functional Verification
- [ ] ScriptConfig validates properly with appropriate error messages
- [ ] ScriptContext constructs with all required metadata
- [ ] ScriptResult handles both success and failure cases
- [ ] ScriptError variants cover all expected failure modes
- [ ] Serialization round-trip preserves all data

### Performance Benchmarks
- [ ] Context creation: <1ms for typical use cases
- [ ] Serialization: <5ms for complex contexts
- [ ] Memory usage: <1MB for typical contexts
- [ ] No memory leaks in repeated operations

### Documentation Quality
- [ ] All public types have comprehensive rustdoc
- [ ] Working examples for all major use cases
- [ ] Doc tests compile and pass
- [ ] Integration examples demonstrate real usage

### Integration Readiness
- [ ] Types are compatible with existing validation framework
- [ ] Error types integrate with existing error handling
- [ ] Serialization format is stable and versioned
- [ ] Thread safety verified for concurrent usage

## Risk Assessment

### High Risk
- **Serialization format stability**: Changes could break compatibility
- **Performance under load**: Complex contexts might be slow
- **Memory usage growth**: Large contexts could consume significant memory

### Medium Risk
- **Error message quality**: Generic errors might be hard to debug
- **JSONPath implementation**: Complex path expressions might fail
- **Thread safety**: Concurrent access to mutable context data

### Low Risk
- **Type safety**: Rust's type system provides strong guarantees
- **Basic functionality**: Simple data structures are well-understood

## Mitigation Strategies

### For Serialization Stability
- Use semantic versioning for format changes
- Add format version field to all serialized types
- Provide migration utilities for format upgrades

### For Performance
- Benchmark with realistic data sizes
- Use lazy evaluation for expensive operations
- Implement caching for repeated operations

### For Memory Usage
- Use reference counting for shared data
- Implement memory limits and monitoring
- Profile memory usage under load

---

**Implementation Priority**: High (blocks issue #247)
**Estimated Complexity**: Medium (type definitions with serialization)
**Breaking Changes**: None (new types only) 