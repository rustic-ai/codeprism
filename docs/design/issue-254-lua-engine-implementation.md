# Issue #254: LuaEngine (mlua) Implementation Design Document

## Problem Statement

Implement a complete Lua script execution engine for the Mandrel MCP Test Harness to enable sophisticated test validation beyond basic JSONPath and schema validation. The LuaEngine will be the first of three script engines (Lua, JavaScript, Python) that provide comprehensive custom validation capabilities for MCP server testing.

## Current State Analysis

**Existing Infrastructure:**
- ✅ Core types defined: `ScriptConfig`, `ScriptContext`, `ScriptResult`, `ScriptError`
- ✅ Dependency available: `mlua` with lua54, async, and vendored features
- ✅ YAML parsing support: `validation_scripts` field exists in `TestSpecification` and `TestCase`
- ✅ Sandboxing foundation: Security configuration options in `ScriptConfig`

**Missing Implementation:**
- ❌ Lua engine implementation (`lua_engine.rs` is placeholder)
- ❌ Context injection and result extraction
- ❌ Timeout and resource limit enforcement
- ❌ Security sandboxing and environment isolation
- ❌ Integration with ValidationEngine

## Proposed Solution

### High-Level Architecture

```rust
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  ScriptContext  │───▶│   LuaEngine     │───▶│  ScriptResult   │
│  (Test Data)    │    │ (mlua Runtime)  │    │ (Validation)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  ScriptConfig   │    │  SecurityMgr    │    │   LogEntry      │
│ (Limits/Rules)  │    │ (Sandboxing)    │    │ (Debug Info)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Core Components

1. **LuaEngine**: Main execution engine with mlua integration
2. **SecurityManager**: Enforces timeouts, memory limits, and access controls
3. **ContextInjector**: Converts ScriptContext to Lua table
4. **ResultExtractor**: Converts Lua result back to ScriptResult
5. **ErrorHandler**: Maps Lua errors to ScriptError types

## API Design

### LuaEngine Interface

```rust
/// Lua script execution engine using mlua
pub struct LuaEngine {
    lua: mlua::Lua,
    security_manager: SecurityManager,
    performance_monitor: PerformanceMonitor,
}

impl LuaEngine {
    /// Creates a new Lua engine with security restrictions
    pub fn new(config: &ScriptConfig) -> Result<Self, ScriptError>;
    
    /// Executes a Lua script with provided context
    pub async fn execute_script(
        &self,
        script: &str,
        context: ScriptContext,
    ) -> Result<ScriptResult, ScriptError>;
    
    /// Validates script syntax without execution
    pub fn validate_syntax(&self, script: &str) -> Result<(), ScriptError>;
    
    /// Precompiles script for better performance
    pub fn precompile_script(&self, script: &str) -> Result<LuaScript, ScriptError>;
    
    /// Executes precompiled script
    pub async fn execute_precompiled(
        &self,
        script: LuaScript,
        context: ScriptContext,
    ) -> Result<ScriptResult, ScriptError>;
}
```

### Context Injection Strategy

```rust
/// Converts ScriptContext to Lua table accessible in scripts
impl ContextInjector {
    pub fn inject_context(lua: &Lua, context: &ScriptContext) -> mlua::Result<()> {
        // Create global 'context' table with:
        // - context.request (request data)
        // - context.response (response data if available)
        // - context.metadata (test execution metadata)
        // - context.config (execution configuration)
        // - Helper functions: log(), validate(), assert_eq()
    }
}
```

**Lua Context API Example:**
```lua
-- Available in all scripts
local request = context.request
local response = context.response
local metadata = context.metadata

-- Helper functions
context.log("info", "Starting validation")
context.assert_eq(expected, actual, "Values should match")
context.validate(condition, "Validation failed")

-- Result object (must be set by script)
result = {
    success = true,
    message = "Validation passed",
    data = { custom = "validation_data" }
}
```

## Implementation Plan

### TDD Phase 1: Core Engine and Context Injection (6 hours)

**Tests to Write First (Red Phase):**
```rust
#[tokio::test]
async fn test_lua_engine_creation() {
    // Should create engine with default security config
    // Should fail with invalid config
}

#[tokio::test]
async fn test_context_injection() {
    // Should inject request, response, metadata into Lua context
    // Should provide helper functions
    // Should handle missing response gracefully
}

#[tokio::test]
async fn test_simple_script_execution() {
    // Should execute basic Lua script
    // Should return ScriptResult with output
    // Should capture logs from script
}

#[tokio::test]
async fn test_script_syntax_validation() {
    // Should validate correct Lua syntax
    // Should reject invalid syntax with line numbers
    // Should provide helpful error messages
}
```

**Implementation (Green Phase):**
```rust
// crates/mandrel-mcp-th/src/script_engines/lua_engine.rs
use mlua::{Lua, LuaSerdeExt, Result as LuaResult};
use std::time::{Duration, Instant};
use tokio::time::timeout;

pub struct LuaEngine {
    lua: Lua,
    config: ScriptConfig,
}

impl LuaEngine {
    pub fn new(config: &ScriptConfig) -> Result<Self, ScriptError> {
        config.validate()?;
        
        let lua = Lua::new();
        
        // Configure Lua sandbox restrictions
        if !config.allow_filesystem {
            // Remove file I/O functions
            lua.load("io = nil; os.remove = nil; os.execute = nil").exec()?;
        }
        
        if !config.allow_network {
            // Remove network-related functions
            lua.load("socket = nil; http = nil").exec()?;
        }
        
        Ok(Self {
            lua,
            config: config.clone(),
        })
    }
    
    pub async fn execute_script(
        &self,
        script: &str,
        context: ScriptContext,
    ) -> Result<ScriptResult, ScriptError> {
        let start_time = Instant::now();
        
        // Inject context into Lua environment
        self.inject_context(&context)?;
        
        // Execute with timeout
        let lua_result = timeout(
            Duration::from_millis(self.config.timeout_ms),
            self.execute_with_monitoring(script)
        ).await;
        
        let duration = start_time.elapsed();
        
        match lua_result {
            Ok(Ok(lua_value)) => self.extract_result(lua_value, duration),
            Ok(Err(lua_error)) => self.handle_lua_error(lua_error, duration),
            Err(_) => Err(ScriptError::TimeoutError { 
                timeout_ms: self.config.timeout_ms 
            }),
        }
    }
}
```

**Refactor Phase:**
- Extract helper methods for better organization
- Optimize context injection performance
- Improve error handling and diagnostics

### TDD Phase 2: Security and Resource Management (4 hours)

**Tests to Write First:**
```rust
#[tokio::test]
async fn test_timeout_enforcement() {
    // Should timeout long-running scripts
    // Should return TimeoutError with correct timeout value
    // Should not block engine after timeout
}

#[tokio::test]
async fn test_memory_limit_enforcement() {
    // Should monitor memory usage during execution
    // Should stop execution when memory limit exceeded
    // Should return MemoryLimitError with usage details
}

#[tokio::test]
async fn test_security_restrictions() {
    // Should block file operations when filesystem disabled
    // Should block network operations when network disabled
    // Should sanitize environment variables
}

#[tokio::test]
async fn test_output_size_limits() {
    // Should limit output size to configured maximum
    // Should truncate oversized output gracefully
    // Should log when truncation occurs
}
```

**Implementation:**
```rust
pub struct SecurityManager {
    config: ScriptConfig,
    start_memory: Option<usize>,
}

impl SecurityManager {
    pub fn new(config: &ScriptConfig) -> Self {
        Self {
            config: config.clone(),
            start_memory: Self::get_memory_usage(),
        }
    }
    
    pub fn check_memory_limit(&self) -> Result<(), ScriptError> {
        if let (Some(limit), Some(start)) = (self.config.memory_limit_mb, self.start_memory) {
            if let Some(current) = Self::get_memory_usage() {
                let used_mb = (current - start) as f64 / (1024.0 * 1024.0);
                if used_mb > limit as f64 {
                    return Err(ScriptError::MemoryLimitError {
                        used_mb,
                        limit_mb: limit,
                    });
                }
            }
        }
        Ok(())
    }
    
    fn get_memory_usage() -> Option<usize> {
        // Implementation depends on platform
        // Could use /proc/self/status on Linux
        // Could use Windows APIs on Windows
        // Return None if not measurable
        None
    }
}
```

### TDD Phase 3: Advanced Features and Integration (4 hours)

**Tests to Write First:**
```rust
#[tokio::test]
async fn test_precompiled_scripts() {
    // Should precompile scripts for better performance
    // Should execute precompiled scripts multiple times
    // Should maintain isolation between executions
}

#[tokio::test]
async fn test_complex_validation_scenarios() {
    // Should handle real-world validation scripts
    // Should support complex data structure validation
    // Should provide detailed error diagnostics
}

#[tokio::test]
async fn test_logging_and_debugging() {
    // Should capture script logs with levels
    // Should provide debugging information
    // Should handle script errors gracefully
}

#[tokio::test]
async fn test_performance_monitoring() {
    // Should measure execution time accurately
    // Should track memory usage when possible
    // Should provide performance metrics in results
}
```

**Implementation:**
```rust
pub struct LuaScript {
    compiled: mlua::Function<'static>,
    source_hash: String,
}

impl LuaEngine {
    pub fn precompile_script(&self, script: &str) -> Result<LuaScript, ScriptError> {
        let function = self.lua.load(script).into_function()
            .map_err(|e| ScriptError::SyntaxError {
                message: e.to_string(),
                line: self.extract_line_number(&e),
            })?;
        
        Ok(LuaScript {
            compiled: function,
            source_hash: self.hash_script(script),
        })
    }
    
    pub async fn execute_precompiled(
        &self,
        script: LuaScript,
        context: ScriptContext,
    ) -> Result<ScriptResult, ScriptError> {
        let start_time = Instant::now();
        
        // Inject fresh context for each execution
        self.inject_context(&context)?;
        
        // Execute precompiled script
        let result = timeout(
            Duration::from_millis(self.config.timeout_ms),
            self.call_precompiled_function(script.compiled)
        ).await;
        
        let duration = start_time.elapsed();
        self.process_execution_result(result, duration)
    }
}
```

## Error Handling Strategy

### Lua Error Mapping

```rust
impl LuaEngine {
    fn handle_lua_error(
        &self, 
        lua_error: mlua::Error, 
        duration: Duration
    ) -> Result<ScriptResult, ScriptError> {
        let script_error = match lua_error {
            mlua::Error::SyntaxError { message, incomplete_input: _ } => {
                ScriptError::SyntaxError {
                    message: message.to_string(),
                    line: self.extract_line_number(&lua_error),
                }
            }
            mlua::Error::RuntimeError(msg) => {
                ScriptError::RuntimeError { message: msg.to_string() }
            }
            mlua::Error::MemoryError(msg) => {
                ScriptError::MemoryLimitError {
                    used_mb: 0.0, // Estimate if possible
                    limit_mb: self.config.memory_limit_mb.unwrap_or(100),
                }
            }
            _ => {
                ScriptError::ExecutionError {
                    message: lua_error.to_string(),
                }
            }
        };
        
        Ok(ScriptResult {
            success: false,
            output: serde_json::Value::Null,
            logs: vec![],
            duration_ms: duration.as_millis() as u64,
            memory_used_mb: None,
            error: Some(script_error),
        })
    }
}
```

## Performance Considerations

### Memory Management
- **Lua State Reuse**: Reuse Lua state for multiple script executions
- **Context Injection Optimization**: Cache context table structure
- **String Interning**: Reuse common strings in Lua environment
- **Garbage Collection**: Trigger Lua GC between script executions

### Execution Optimization
- **Script Precompilation**: Compile scripts once, execute multiple times
- **Bytecode Caching**: Cache compiled Lua bytecode for repeated scripts
- **Context Preparation**: Prepare context tables efficiently
- **Result Extraction**: Optimize conversion from Lua values to Rust types

### Resource Monitoring
```rust
pub struct PerformanceMonitor {
    execution_count: usize,
    total_duration: Duration,
    average_memory_usage: f64,
}

impl PerformanceMonitor {
    pub fn record_execution(&mut self, duration: Duration, memory_mb: Option<f64>) {
        self.execution_count += 1;
        self.total_duration += duration;
        if let Some(memory) = memory_mb {
            self.average_memory_usage = 
                (self.average_memory_usage * (self.execution_count - 1) as f64 + memory) 
                / self.execution_count as f64;
        }
    }
    
    pub fn get_performance_stats(&self) -> PerformanceStats {
        PerformanceStats {
            total_executions: self.execution_count,
            average_duration_ms: self.total_duration.as_millis() as f64 / self.execution_count as f64,
            average_memory_mb: self.average_memory_usage,
        }
    }
}
```

## Security Implementation

### Sandbox Configuration
```lua
-- Restricted Lua environment (applied when config.allow_filesystem = false)
io = nil
os.execute = nil
os.remove = nil
os.rename = nil
require = nil  -- Prevent loading external modules

-- Restricted when config.allow_network = false
socket = nil
http = nil
net = nil

-- Memory and CPU protection
debug.sethook(function() 
    if check_execution_limits() then
        error("Resource limit exceeded")
    end
end, "", 1000) -- Check every 1000 instructions
```

### Environment Variable Sanitization
```rust
impl LuaEngine {
    fn sanitize_environment(&self, context: &ScriptContext) -> HashMap<String, String> {
        let mut safe_env = HashMap::new();
        
        for (key, value) in &context.config.environment_variables {
            // Only allow safe environment variables
            if self.is_safe_env_var(key) {
                safe_env.insert(key.clone(), value.clone());
            }
        }
        
        safe_env
    }
    
    fn is_safe_env_var(&self, key: &str) -> bool {
        // Block sensitive environment variables
        !matches!(key.to_uppercase().as_str(), 
            "PATH" | "HOME" | "USER" | "PASSWORD" | "SECRET" | "TOKEN" | "KEY"
        )
    }
}
```

## Integration Points

### ValidationEngine Integration
```rust
// In validation/engine.rs
impl ValidationEngine {
    pub async fn execute_validation_scripts(
        &self,
        scripts: &[String],
        context: ScriptContext,
    ) -> Result<Vec<ScriptResult>, ValidationError> {
        let mut results = Vec::new();
        
        for script_name in scripts {
            if let Some(script) = self.script_registry.get(script_name) {
                let result = self.lua_engine.execute_script(script, context.clone()).await?;
                results.push(result);
                
                // Fail fast if script validation fails and strict mode is enabled
                if !result.success && self.config.fail_fast {
                    break;
                }
            }
        }
        
        Ok(results)
    }
}
```

### TestCaseExecutor Integration
```rust
// In executor.rs
impl TestCaseExecutor {
    async fn execute_validation_scripts(
        &self,
        test_case: &TestCase,
        context: ScriptContext,
    ) -> Result<ValidationResult, ExecutorError> {
        if let Some(script_names) = &test_case.validation_scripts {
            let results = self.validation_engine
                .execute_validation_scripts(script_names, context)
                .await?;
            
            let overall_success = results.iter().all(|r| r.success);
            
            Ok(ValidationResult {
                success: overall_success,
                script_results: results,
            })
        } else {
            Ok(ValidationResult::default())
        }
    }
}
```

## Success Criteria

### Functional Requirements
- [ ] Lua scripts execute with injected context (request, response, metadata)
- [ ] Timeout enforcement prevents infinite loops and long-running scripts
- [ ] Memory limits prevent resource exhaustion
- [ ] Security restrictions prevent file and network access when disabled
- [ ] Error handling provides clear diagnostics with line numbers
- [ ] Script precompilation improves performance for repeated execution
- [ ] Integration with ValidationEngine and TestCaseExecutor works end-to-end

### Quality Requirements
- [ ] All tests pass with 95%+ code coverage
- [ ] Performance: <10ms overhead for simple scripts
- [ ] Memory: <5MB base memory usage for Lua engine
- [ ] Security: All sandbox restrictions enforced correctly
- [ ] Error messages provide actionable debugging information

### Integration Requirements
- [ ] YAML specifications with Lua validation scripts parse correctly
- [ ] Scripts execute during test case validation phase
- [ ] Results integrate with reporting system
- [ ] Error handling integrates with overall test execution flow

## Testing Strategy

### Unit Tests
- Individual component testing (SecurityManager, ContextInjector, etc.)
- Error handling verification for all error types
- Performance monitoring accuracy
- Security restriction enforcement

### Integration Tests
- End-to-end script execution with real test cases
- Integration with ValidationEngine and TestCaseExecutor
- YAML specification parsing and execution
- Complex validation scenario testing

### Performance Tests
- Script execution time benchmarks
- Memory usage measurements
- Precompilation performance benefits
- Concurrent script execution handling

### Security Tests
- Sandbox escape attempt prevention
- Resource limit enforcement verification
- Environment variable sanitization
- File and network access restriction testing

## Risk Mitigation

### Performance Risks
**Risk**: Lua script execution adds significant overhead
**Mitigation**: 
- Precompilation for repeated scripts
- Context preparation optimization
- Performance monitoring and alerting

### Security Risks
**Risk**: Scripts could access restricted resources
**Mitigation**:
- Comprehensive sandbox implementation
- Environment variable sanitization
- Resource limit enforcement
- Security testing coverage

### Memory Risks
**Risk**: Scripts could cause memory leaks or excessive usage
**Mitigation**:
- Memory monitoring and limits
- Lua garbage collection management
- Resource cleanup after execution

## Implementation Deliverables

1. **LuaEngine Implementation** (`lua_engine.rs`): Complete Lua script execution engine
2. **Security Infrastructure**: SecurityManager and sandbox implementation
3. **Integration Code**: ValidationEngine and TestCaseExecutor integration
4. **Comprehensive Tests**: Unit, integration, performance, and security tests
5. **Documentation**: API documentation and usage examples
6. **Performance Benchmarks**: Execution time and memory usage metrics

## References

- [mlua Documentation](https://docs.rs/mlua/latest/mlua/) - Official mlua crate documentation
- [Lua 5.4 Reference](https://www.lua.org/manual/5.4/) - Lua language reference
- [Script Security Best Practices](https://owasp.org/www-community/vulnerabilities/Server_Side_Template_Injection) - Security considerations
- [Existing Script Infrastructure](mdc:crates/mandrel-mcp-th/src/script_engines/types.rs) - Current type definitions

---

**This design provides a comprehensive foundation for implementing a secure, performant, and well-integrated Lua script execution engine for the Mandrel MCP Test Harness.** 