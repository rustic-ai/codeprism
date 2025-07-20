# Issue #300: Lua Print Statement Capture Implementation

## Problem Statement

Currently, the LuaEngine does not capture logs from Lua print statements or other output, limiting debugging and monitoring capabilities for script execution. The `ScriptResult.logs` field is hardcoded to an empty vector, preventing effective script diagnostics and audit trails.

## Current State Analysis

**Existing Infrastructure:**
- ✅ ScriptResult structure with `logs: Vec<LogEntry>` field
- ✅ LogEntry and LogLevel types defined in types.rs
- ✅ LuaEngine with comprehensive script execution framework
- ✅ Thread-safe execution environment
- ✅ Memory tracking system (recently implemented)

**Missing Implementation:**
- ❌ Custom Lua print function to capture output
- ❌ Thread-safe log buffer for collecting output
- ❌ Integration with LuaEngine execution pipeline
- ❌ Log extraction and formatting system
- ❌ Performance optimization for minimal overhead

## Proposed Solution

### High-Level Architecture

```rust
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  LuaEngine      │───▶│  LogBuffer      │───▶│   LogEntry      │
│  (execute)      │    │  (capture)      │    │   (format)      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Custom Print   │    │ Thread-Safe     │    │  ScriptResult   │
│  Function       │    │ Buffer Storage  │    │  (logs field)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Core Components

1. **LogBuffer** - Thread-safe storage for captured print output
2. **Custom Print Function** - Replacement for Lua's default print
3. **Log Extraction** - Converting captured output to LogEntry structs
4. **Integration Points** - Seamless integration with existing execution flow

### Implementation Plan

#### Phase 1: Log Buffer Infrastructure
```rust
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Thread-safe buffer for capturing Lua print statements
#[derive(Debug, Clone)]
pub struct LogBuffer {
    entries: Arc<Mutex<Vec<CapturedLogEntry>>>,
    start_time: SystemTime,
}

#[derive(Debug, Clone)]
struct CapturedLogEntry {
    message: String,
    timestamp: SystemTime,
    level: LogLevel,
}

impl LogBuffer {
    pub fn new() -> Self;
    pub fn capture(&self, message: String, level: LogLevel);
    pub fn extract_logs(&self) -> Vec<LogEntry>;
    pub fn clear(&self);
}
```

#### Phase 2: Custom Lua Print Function
```rust
impl LuaEngine {
    /// Setup custom print function for log capture
    fn setup_log_capture(&self, lua: &Lua, log_buffer: Arc<LogBuffer>) -> Result<(), ScriptError> {
        let buffer_clone = Arc::clone(&log_buffer);
        
        let custom_print = lua.create_function(move |_lua, args: mlua::MultiValue| {
            let messages: Vec<String> = args
                .into_iter()
                .map(|val| format!("{}", val))
                .collect();
            
            let combined_message = messages.join("\t");
            buffer_clone.capture(combined_message, LogLevel::Info);
            
            Ok(())
        })?;
        
        let globals = lua.globals();
        globals.set("print", custom_print)?;
        
        Ok(())
    }
}
```

#### Phase 3: Integration with LuaEngine Execution
```rust
impl LuaEngine {
    pub async fn execute_script(
        &self,
        script: &str,
        context: ScriptContext,
    ) -> Result<ScriptResult, ScriptError> {
        let start_time = Instant::now();
        let security_manager = SecurityManager::new(&self.config);
        
        // Initialize logging infrastructure
        let log_buffer = Arc::new(LogBuffer::new());
        
        // Setup custom print function for log capture
        self.setup_log_capture(&self.lua, Arc::clone(&log_buffer))?;
        
        // Initialize memory tracking (existing implementation)
        let memory_tracker = MemoryTracker::new(MemoryTrackingConfig::default());
        let memory_before = memory_tracker.snapshot()?;

        // Inject context into Lua environment
        self.inject_context(&context)?;

        // Execute with timeout and monitoring
        let execution_future = self.execute_with_monitoring(script, &security_manager);
        let lua_result = timeout(
            Duration::from_millis(self.config.timeout_ms),
            execution_future,
        ).await;

        // Extract logs before processing results
        let captured_logs = log_buffer.extract_logs();
        
        // Memory tracking (existing implementation)
        let memory_after = memory_tracker.snapshot()?;
        let memory_delta = memory_tracker.calculate_delta(&memory_before, &memory_after);
        let memory_used_mb = Some(memory_tracker.delta_to_mb(&memory_delta));

        let duration = start_time.elapsed();
        let duration_ms = duration.as_millis() as u64;

        match lua_result {
            Ok(Ok(lua_value)) => self.extract_result(lua_value, duration_ms, memory_used_mb, captured_logs),
            Ok(Err(lua_error)) => self.handle_lua_error(lua_error, duration_ms, memory_used_mb, captured_logs),
            Err(_) => Ok(ScriptResult {
                success: false,
                output: serde_json::Value::Null,
                logs: captured_logs,
                duration_ms,
                memory_used_mb,
                error: Some(ScriptError::TimeoutError {
                    timeout_ms: self.config.timeout_ms,
                }),
            }),
        }
    }
}
```

## Technical Considerations

### Thread Safety
- **Challenge**: Multiple scripts executing concurrently
- **Solution**: Use `Arc<Mutex<Vec<CapturedLogEntry>>>` for thread-safe access
- **Trade-off**: Small performance cost for safety guarantees

### Performance Optimization
- **Target**: < 5% execution time overhead
- **Approach**: Minimize allocations and use efficient string handling
- **Optimization**: Pre-allocate log buffer capacity based on expected output

### Log Level Detection
- **Basic Implementation**: All print statements default to LogLevel::Info
- **Future Enhancement**: Parse log level from message format (e.g., "[ERROR]", "[DEBUG]")
- **Fallback**: Configurable default log level

### Memory Management
```rust
impl LogBuffer {
    /// Efficient capture with pre-allocated capacity
    pub fn capture(&self, message: String, level: LogLevel) {
        let mut entries = self.entries.lock().unwrap();
        
        // Prevent unbounded growth
        if entries.len() >= MAX_LOG_ENTRIES {
            entries.remove(0); // Remove oldest entry
        }
        
        entries.push(CapturedLogEntry {
            message,
            timestamp: SystemTime::now(),
            level,
        });
    }
}
```

## Error Handling Strategy

```rust
#[derive(Debug, thiserror::Error)]
pub enum LogCaptureError {
    #[error("Failed to setup log capture: {message}")]
    SetupFailed { message: String },
    
    #[error("Log buffer access failed: {message}")]
    BufferAccessFailed { message: String },
    
    #[error("Log extraction failed: {message}")]
    ExtractionFailed { message: String },
}
```

## Integration with Existing Types

### Enhanced ScriptResult
```rust
impl ScriptResult {
    /// Create result with captured logs
    pub fn with_logs(mut self, logs: Vec<LogEntry>) -> Self {
        self.logs = logs;
        self
    }
}
```

### LogEntry Conversion
```rust
impl From<CapturedLogEntry> for LogEntry {
    fn from(captured: CapturedLogEntry) -> Self {
        LogEntry {
            level: captured.level,
            message: captured.message,
            timestamp: captured.timestamp.duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
}
```

## Testing Strategy

### Unit Tests
- Log buffer creation and thread safety
- Custom print function setup and execution
- Log extraction and conversion
- Performance overhead measurement

### Integration Tests
- End-to-end log capture during script execution
- Multiple print statements in single script
- Concurrent script execution with separate log buffers
- Error scenarios and edge cases

### Log Capture Scenarios
```rust
#[tokio::test]
async fn test_lua_print_capture() {
    let engine = LuaEngine::new(&ScriptConfig::new()).unwrap();
    let context = create_test_context();

    let script = r#"
        print("Hello from Lua")
        print("Debug info:", 42)
        print("Multiple", "arguments", "test")
        result = { success = true }
    "#;

    let result = engine.execute_script(script, context).await.unwrap();
    
    assert!(result.success);
    assert_eq!(result.logs.len(), 3);
    assert_eq!(result.logs[0].message, "Hello from Lua");
    assert_eq!(result.logs[1].message, "Debug info:\t42");
    assert_eq!(result.logs[2].message, "Multiple\targuments\ttest");
    assert!(result.logs.iter().all(|log| log.level == LogLevel::Info));
}

#[tokio::test]
async fn test_log_capture_performance_overhead() {
    let engine = LuaEngine::new(&ScriptConfig::new()).unwrap();
    let context = create_test_context();

    let script_with_logs = r#"
        for i = 1, 100 do
            print("Log entry " .. i)
        end
        result = { success = true }
    "#;

    let script_without_logs = r#"
        for i = 1, 100 do
            -- No print statements
        end
        result = { success = true }
    "#;

    // Measure execution times
    let start = Instant::now();
    let _result_with_logs = engine.execute_script(script_with_logs, context.clone()).await.unwrap();
    let time_with_logs = start.elapsed();

    let start = Instant::now();
    let _result_without_logs = engine.execute_script(script_without_logs, context).await.unwrap();
    let time_without_logs = start.elapsed();

    // Verify overhead is < 5%
    let overhead_ratio = time_with_logs.as_millis() as f64 / time_without_logs.as_millis() as f64;
    assert!(overhead_ratio < 1.05, "Log capture overhead too high: {}%", (overhead_ratio - 1.0) * 100.0);
}
```

## Implementation Phases

### Phase 1: Foundation (TDD - RED)
1. Create LogBuffer structure with thread-safe storage
2. Write comprehensive failing tests for log capture
3. Add log capture error types
4. Implement basic log entry conversion

### Phase 2: Custom Print Function (TDD - GREEN)
1. Implement custom Lua print function
2. Setup log capture in LuaEngine initialization
3. Test single and multiple print statement capture
4. Verify thread safety with concurrent execution

### Phase 3: Integration (TDD - REFACTOR)
1. Integrate log capture into execute_script method
2. Update extract_result and handle_lua_error methods
3. Add performance optimization for minimal overhead
4. Comprehensive error handling and edge cases

## Success Criteria

### Functional Requirements
- ✅ Lua print statements captured and returned in ScriptResult.logs
- ✅ Multiple print calls in single script all captured
- ✅ Logs properly formatted with timestamps and levels
- ✅ Thread-safe for concurrent script execution
- ✅ All existing tests continue to pass

### Performance Requirements
- ✅ Log capture overhead < 5% execution time
- ✅ Memory usage remains bounded (MAX_LOG_ENTRIES limit)
- ✅ No memory leaks in log buffer implementation

### Quality Requirements
- ✅ Comprehensive test coverage (90%+ for new code)
- ✅ Error scenarios properly tested and handled
- ✅ Integration with existing memory tracking system
- ✅ Documentation updated with log capture examples

## Dependencies

- **Existing LogEntry/LogLevel types**: Already defined in types.rs
- **Thread Safety**: Rust's Arc<Mutex<T>> for concurrent access
- **mlua Integration**: Custom function creation and global replacement
- **Performance**: Efficient string handling and buffer management

## Breaking Changes

**None** - This implementation is purely additive and enhances existing functionality without changing public APIs.

## File Locations

- `crates/mandrel-mcp-th/src/script_engines/lua_engine.rs` - Primary integration point
- `crates/mandrel-mcp-th/src/script_engines/types.rs` - LogEntry and LogLevel types (existing)
- `crates/mandrel-mcp-th/src/script_engines/mod.rs` - Module exports (if needed)

---

**This design enables comprehensive log capture for Lua script execution while maintaining performance and providing robust error handling, completing the debugging and monitoring capabilities alongside the recently implemented memory tracking.** 