# Issue #252: JavaScript Engine Implementation

## Problem Statement

The Mandrel MCP Test Harness needs a JavaScriptEngine to execute JavaScript validation scripts alongside the existing Lua engine. This will enable testing MCP servers with JavaScript-based validation logic and expand the scripting capabilities of the test harness.

## Current State Analysis

**Existing Infrastructure:**
- ✅ ScriptConfig, ScriptContext, ScriptResult, ScriptError types in types.rs
- ✅ Memory tracking system implemented and working
- ✅ LuaEngine as reference implementation with all features
- ✅ QuickJS dependency available (rquickjs) with basic tests in mod.rs
- ✅ Sandbox and security infrastructure available

**Missing Implementation:**
- ❌ JavaScriptEngine struct and implementation
- ❌ JavaScript context injection
- ❌ JavaScript-specific error handling and mapping
- ❌ Integration with existing timeout and memory tracking
- ❌ Comprehensive test coverage

## Technology Stack

**QuickJS (via rquickjs):**
- Lightweight, fast JavaScript engine
- Good sandboxing capabilities
- No Node.js dependencies - pure JS execution
- Suitable for validation scripts and testing logic

## Proposed Solution

### 1. Core Architecture

```rust
/// JavaScript execution engine using QuickJS
pub struct JavaScriptEngine {
    runtime: rquickjs::Runtime,
    config: ScriptConfig,
}

/// Precompiled JavaScript script (future enhancement)
pub struct JavaScriptScript {
    source: String,
    function_name: Option<String>, // For callable scripts
}
```

### 2. API Design

```rust
impl JavaScriptEngine {
    /// Create new JavaScript engine with configuration
    pub fn new(config: &ScriptConfig) -> Result<Self, ScriptError>;
    
    /// Execute JavaScript code with context injection
    pub async fn execute_script(
        &self,
        script: &str,
        context: ScriptContext,
    ) -> Result<ScriptResult, ScriptError>;
    
    /// Precompile JavaScript for future execution (placeholder)
    pub fn precompile_script(
        &self,
        script: &str,
        function_name: Option<String>,
    ) -> Result<JavaScriptScript, ScriptError>;
    
    /// Execute precompiled JavaScript script
    pub async fn execute_precompiled(
        &self,
        script: &JavaScriptScript,
        context: ScriptContext,
    ) -> Result<ScriptResult, ScriptError>;
}
```

### 3. Context Injection Strategy

**JavaScript Context Object:**
```javascript
// Injected global object 'context'
{
    request: { /* MCP request data */ },
    response: { /* MCP response data */ },
    test_case: "test_case_name",
    tool: "tool_name", 
    metadata: {
        server_info: { /* server details */ },
        execution_id: "uuid",
        timestamp: "2025-01-20T10:30:00Z"
    },
    // Utility functions
    log: function(level, message) { /* logging */ },
    assert: function(condition, message) { /* assertions */ }
}
```

**Example Script:**
```javascript
// Access injected context
if (context.response.error) {
    context.log('error', 'Unexpected error in response');
    return { success: false, error: 'Response contains error' };
}

context.assert(context.response.result, 'Response must have result field');

return {
    success: true,
    message: `Tool ${context.tool} executed successfully`,
    result_type: typeof context.response.result
};
```

### 4. Error Handling Strategy

**JavaScript Error Mapping:**
```rust
fn map_js_error(js_error: rquickjs::Error) -> ScriptError {
    match js_error {
        rquickjs::Error::Exception => {
            // Extract JS exception details
            ScriptError::RuntimeError { message: extract_js_exception() }
        }
        rquickjs::Error::Syntax => {
            ScriptError::SyntaxError { 
                message: extract_syntax_error(),
                line: extract_line_number() 
            }
        }
        _ => ScriptError::ExecutionError { message: js_error.to_string() }
    }
}
```

### 5. Security and Sandboxing

**Security Features:**
- No file system access by default
- No network access (unless explicitly enabled)
- Memory limits via existing ResourceMonitor
- Execution timeout via tokio::time::timeout
- No access to dangerous JavaScript APIs

**Implementation:**
```rust
impl JavaScriptEngine {
    fn create_secure_context(&self) -> Result<rquickjs::Context, ScriptError> {
        let context = rquickjs::Context::full(&self.runtime)?;
        
        // Disable dangerous globals
        context.with(|ctx| {
            // Remove or restrict dangerous APIs
            ctx.globals().delete("eval")?;
            ctx.globals().delete("Function")?;
            // Add safe utility functions
            self.inject_safe_utilities(ctx)?;
            Ok(())
        })?;
        
        Ok(context)
    }
}
```

### 6. Performance and Memory Integration

**Memory Tracking Integration:**
```rust
pub async fn execute_script(
    &self,
    script: &str,
    context: ScriptContext,
) -> Result<ScriptResult, ScriptError> {
    let start_time = Instant::now();
    
    // Initialize memory tracking
    let memory_tracker = MemoryTracker::new(MemoryTrackingConfig::default());
    let memory_before = memory_tracker.snapshot()?;
    
    // Create secure context and execute
    let js_context = self.create_secure_context()?;
    let result = js_context.with(|ctx| {
        self.inject_context(ctx, &context)?;
        self.execute_with_timeout(ctx, script)
    });
    
    // Calculate metrics
    let memory_after = memory_tracker.snapshot()?;
    let memory_delta = memory_tracker.calculate_delta(&memory_before, &memory_after);
    let memory_used_mb = memory_tracker.delta_to_mb(&memory_delta);
    let duration_ms = start_time.elapsed().as_millis() as u64;
    
    // Process result with metrics
    self.build_script_result(result, duration_ms, Some(memory_used_mb))
}
```

## Implementation Plan

### Phase 1: Core Engine Implementation
1. **Create JavaScriptEngine struct** with QuickJS runtime
2. **Implement basic script execution** without context injection
3. **Add timeout and error handling** following Lua engine patterns
4. **Basic tests** for script execution and error scenarios

### Phase 2: Context Injection and Integration  
1. **Implement context injection** with MCP data
2. **Add utility functions** (log, assert) for script convenience
3. **Integrate memory tracking** and performance monitoring
4. **Comprehensive error mapping** from QuickJS to ScriptError

### Phase 3: Advanced Features and Testing
1. **Implement precompile_script** (basic source storage)
2. **Add execute_precompiled** method
3. **Performance benchmarks** comparing with Lua engine
4. **Comprehensive test suite** covering all functionality

### Phase 4: Security and Validation
1. **Security hardening** - remove dangerous JavaScript APIs
2. **Resource limiting** integration with existing sandbox
3. **Validation scripts** for real MCP testing scenarios
4. **Integration tests** with actual MCP servers

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_js_simple_script_execution() {
    let engine = JavaScriptEngine::new(&ScriptConfig::new()).unwrap();
    let context = create_test_context();
    
    let script = r#"
        return {
            success: true,
            message: "Hello from JavaScript",
            input_type: typeof context.request
        };
    "#;
    
    let result = engine.execute_script(script, context).await.unwrap();
    assert!(result.success);
    assert!(result.duration_ms > 0);
}

#[tokio::test]
async fn test_js_context_injection() {
    let engine = JavaScriptEngine::new(&ScriptConfig::new()).unwrap();
    let context = create_test_context_with_data();
    
    let script = r#"
        if (!context.request || !context.response) {
            throw new Error("Context not properly injected");
        }
        return { 
            success: true,
            test_case: context.test_case,
            tool: context.tool
        };
    "#;
    
    let result = engine.execute_script(script, context).await.unwrap();
    assert!(result.success);
}

#[tokio::test] 
async fn test_js_error_handling() {
    let engine = JavaScriptEngine::new(&ScriptConfig::new()).unwrap();
    let context = create_test_context();
    
    let script = "throw new Error('Test error');";
    
    let result = engine.execute_script(script, context).await.unwrap();
    assert!(!result.success);
    assert!(result.error.is_some());
}

#[tokio::test]
async fn test_js_timeout_handling() {
    let mut config = ScriptConfig::new();
    config.timeout_ms = 100; // Very short timeout
    
    let engine = JavaScriptEngine::new(&config).unwrap();
    let context = create_test_context();
    
    let script = "while(true) { /* infinite loop */ }";
    
    let result = engine.execute_script(script, context).await.unwrap();
    assert!(!result.success);
    assert!(matches!(result.error, Some(ScriptError::TimeoutError { .. })));
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_js_mcp_validation_script() {
    let engine = JavaScriptEngine::new(&ScriptConfig::new()).unwrap();
    
    // Real MCP server response validation
    let context = ScriptContext::new(
        json!({"tool": "list_files", "arguments": {"path": "/tmp"}}),
        json!({"content": [{"type": "text", "text": "file1.txt\nfile2.txt"}]}),
        "test_mcp_list_files".to_string(),
        "list_files".to_string(),
        config,
    );
    
    let validation_script = r#"
        // Validate MCP response structure
        if (!context.response.content) {
            return { success: false, error: "Missing content field" };
        }
        
        if (!Array.isArray(context.response.content)) {
            return { success: false, error: "Content must be array" };
        }
        
        const textContent = context.response.content.find(c => c.type === 'text');
        if (!textContent) {
            return { success: false, error: "No text content found" };
        }
        
        const lines = textContent.text.split('\n').filter(l => l.trim());
        context.log('info', `Found ${lines.length} files`);
        
        return {
            success: true,
            files_count: lines.length,
            files: lines
        };
    "#;
    
    let result = engine.execute_script(validation_script, context).await.unwrap();
    assert!(result.success);
}
```

### Performance Tests
```rust
#[tokio::test]
async fn test_js_performance_requirements() {
    let engine = JavaScriptEngine::new(&ScriptConfig::new()).unwrap();
    let context = create_test_context();
    
    let script = r#"
        // Simple computation
        let sum = 0;
        for (let i = 0; i < 1000; i++) {
            sum += i;
        }
        return { success: true, sum: sum };
    "#;
    
    let start = Instant::now();
    let result = engine.execute_script(script, context).await.unwrap();
    let duration = start.elapsed();
    
    assert!(result.success);
    assert!(duration.as_millis() < 100, "JS execution should be <100ms");
    assert!(result.memory_used_mb.unwrap_or(0.0) < 10.0, "Memory usage should be <10MB");
}
```

## Success Criteria

### Functional Requirements
- ✅ **Basic Execution**: JavaScript scripts execute successfully with QuickJS
- ✅ **Context Injection**: Scripts can access request/response data and metadata
- ✅ **Error Handling**: Proper error mapping from JS exceptions to ScriptError
- ✅ **Timeout Support**: Scripts respect timeout configuration
- ✅ **Memory Tracking**: Integration with existing memory monitoring

### Quality Requirements  
- ✅ **Test Coverage**: 90%+ test coverage for all JavaScript engine functionality
- ✅ **Performance**: <100ms execution for typical validation scripts
- ✅ **Memory Efficiency**: <10MB memory usage for standard scripts
- ✅ **Security**: Safe execution without access to dangerous APIs

### Integration Requirements
- ✅ **API Compatibility**: Same interface patterns as LuaEngine
- ✅ **Configuration**: Uses existing ScriptConfig infrastructure
- ✅ **Monitoring**: Integrates with ResourceMonitor and memory tracking
- ✅ **Error Reporting**: Consistent error handling across engines

## Future Enhancements

### Phase 2 Considerations
1. **Node.js Integration**: Optional Node.js subprocess execution for npm packages
2. **Module System**: Support for JavaScript modules and imports  
3. **Bytecode Compilation**: QuickJS bytecode precompilation for performance
4. **Debugging Support**: Source maps and debugging capabilities
5. **TypeScript Support**: TypeScript script compilation and execution

### Advanced Features
1. **Async/Await Support**: Full async JavaScript execution patterns
2. **Web APIs**: Selective Web API support (fetch, setTimeout) for testing
3. **ES6+ Features**: Modern JavaScript feature support verification
4. **Performance Profiling**: Detailed execution profiling and optimization

---

**Implementation Priority**: High - Required for issue #252
**Dependencies**: QuickJS (rquickjs), existing script engine infrastructure
**Estimated Effort**: 2-3 days for full implementation with comprehensive testing 