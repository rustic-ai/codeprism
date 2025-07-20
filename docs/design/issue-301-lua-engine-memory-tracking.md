# Issue #301: Lua Engine Memory Tracking Implementation

## Problem Statement

Currently, the LuaEngine does not track actual memory usage during script execution, limiting resource monitoring and debugging capabilities. The `ScriptResult.memory_used_mb` field is hardcoded to `None`, preventing effective memory limit enforcement and performance optimization.

## Current State Analysis

**Existing Infrastructure:**
- ✅ ScriptResult structure with `memory_used_mb: Option<f64>` field
- ✅ SecurityManager with memory limit enforcement capabilities
- ✅ LuaEngine with comprehensive script execution framework
- ✅ Cross-platform support (Linux, macOS, Windows) in existing codebase

**Missing Implementation:**
- ❌ Platform-specific memory tracking APIs
- ❌ Memory measurement before/after script execution
- ❌ Accurate memory delta calculation
- ❌ Integration with SecurityManager memory limits
- ❌ Error handling for memory measurement failures

## Proposed Solution

### High-Level Architecture

```rust
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  LuaEngine      │───▶│  MemoryTracker  │───▶│ Platform APIs   │
│  (execute)      │    │  (measure)      │    │ (Linux/Mac/Win) │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  ScriptResult   │    │  MemorySnapshot │    │  MemoryDelta    │
│ (memory_used_mb)│    │   (before/after)│    │   (calculation) │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Core Components

1. **MemoryTracker** - Cross-platform memory measurement utility
2. **MemorySnapshot** - Point-in-time memory measurement
3. **MemoryDelta** - Calculated memory usage difference
4. **Platform-specific APIs** - OS-specific memory measurement implementations

### Implementation Plan

#### Phase 1: Memory Tracker Infrastructure
```rust
pub struct MemoryTracker {
    config: MemoryTrackingConfig,
}

pub struct MemorySnapshot {
    pub heap_size_bytes: u64,
    pub rss_bytes: u64,
    pub timestamp: Instant,
}

pub struct MemoryDelta {
    pub heap_delta_bytes: i64,
    pub rss_delta_bytes: i64,
    pub duration: Duration,
}

impl MemoryTracker {
    pub fn new(config: MemoryTrackingConfig) -> Self;
    pub fn snapshot(&self) -> Result<MemorySnapshot, MemoryError>;
    pub fn calculate_delta(&self, before: &MemorySnapshot, after: &MemorySnapshot) -> MemoryDelta;
    pub fn delta_to_mb(&self, delta: &MemoryDelta) -> f64;
}
```

#### Phase 2: Platform-Specific Implementations
```rust
#[cfg(target_os = "linux")]
mod linux_memory {
    use std::fs;
    
    pub fn get_memory_info() -> Result<(u64, u64), MemoryError> {
        // Read /proc/self/status for VmRSS and VmHeap
        let status = fs::read_to_string("/proc/self/status")?;
        // Parse memory values
    }
}

#[cfg(target_os = "macos")]
mod macos_memory {
    use mach2::mach_types::thread_port_t;
    
    pub fn get_memory_info() -> Result<(u64, u64), MemoryError> {
        // Use mach_task_basic_info API
    }
}

#[cfg(target_os = "windows")]
mod windows_memory {
    use winapi::um::psapi::GetProcessMemoryInfo;
    
    pub fn get_memory_info() -> Result<(u64, u64), MemoryError> {
        // Use GetProcessMemoryInfo API
    }
}
```

#### Phase 3: LuaEngine Integration
```rust
impl LuaEngine {
    pub async fn execute_script(
        &self,
        script: &str,
        context: ScriptContext,
    ) -> Result<ScriptResult, ScriptError> {
        let start_time = Instant::now();
        let memory_tracker = MemoryTracker::new(self.config.memory_tracking_config.clone());
        
        // Take memory snapshot before execution
        let memory_before = memory_tracker.snapshot()
            .map_err(|e| ScriptError::MemoryTrackingError { message: e.to_string() })?;
        
        // Execute script (existing logic)
        let result = self.execute_with_monitoring(script, &security_manager).await;
        
        // Take memory snapshot after execution
        let memory_after = memory_tracker.snapshot()
            .map_err(|e| ScriptError::MemoryTrackingError { message: e.to_string() })?;
        
        // Calculate memory usage
        let memory_delta = memory_tracker.calculate_delta(&memory_before, &memory_after);
        let memory_used_mb = Some(memory_tracker.delta_to_mb(&memory_delta));
        
        // Create result with memory tracking
        Ok(ScriptResult {
            success: true,
            output,
            logs: vec![],
            duration_ms,
            memory_used_mb,
            error: None,
        })
    }
}
```

## Technical Considerations

### Memory Measurement Accuracy
- **Challenge**: Distinguishing script memory from engine overhead
- **Solution**: Measure delta during script execution phase only
- **Trade-off**: Accept ±10% accuracy for performance reasons

### Cross-Platform Differences
- **Linux**: Use `/proc/self/status` for RSS and heap measurements
- **macOS**: Use `mach_task_basic_info` for accurate memory statistics
- **Windows**: Use `GetProcessMemoryInfo` for process memory information

### Performance Overhead
- **Target**: < 2% execution time overhead
- **Approach**: Use efficient platform APIs and minimize measurement calls
- **Optimization**: Cache memory tracker instance and reuse snapshots

### Error Handling Strategy
```rust
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Platform memory API unavailable: {message}")]
    PlatformUnavailable { message: String },
    
    #[error("Memory measurement failed: {message}")]
    MeasurementFailed { message: String },
    
    #[error("Invalid memory data: {message}")]
    InvalidData { message: String },
}
```

## Security Integration

### Memory Limit Enforcement
```rust
impl SecurityManager {
    pub fn check_memory_limit(&self, memory_used_mb: f64) -> Result<(), SecurityViolation> {
        if memory_used_mb > self.config.memory_limit_mb {
            return Err(SecurityViolation::MemoryLimitExceeded {
                used: memory_used_mb,
                limit: self.config.memory_limit_mb,
            });
        }
        Ok(())
    }
}
```

## Testing Strategy

### Unit Tests
- Memory tracker creation and configuration
- Platform-specific API calls (mocked for CI)
- Memory delta calculations with various scenarios
- Error handling for measurement failures

### Integration Tests
- End-to-end memory tracking during script execution
- Memory limit enforcement with SecurityManager
- Cross-platform compatibility (conditional compilation)
- Performance overhead validation

### Memory Usage Scenarios
```rust
#[tokio::test]
async fn test_memory_tracking_small_script() {
    // Test minimal memory usage for simple scripts
}

#[tokio::test]
async fn test_memory_tracking_large_allocation() {
    // Test significant memory usage detection
}

#[tokio::test]
async fn test_memory_limit_enforcement() {
    // Test SecurityManager integration
}

#[tokio::test]
async fn test_memory_tracking_error_handling() {
    // Test graceful fallback when tracking fails
}
```

## Implementation Phases

### Phase 1: Foundation (GREEN - TDD)
1. Create MemoryTracker trait and basic implementation
2. Add MemorySnapshot and MemoryDelta structures
3. Write comprehensive failing tests for memory tracking
4. Implement mock memory tracking for initial GREEN phase

### Phase 2: Platform APIs (REFACTOR)
1. Implement Linux memory tracking using /proc/self/status
2. Implement macOS memory tracking using mach APIs
3. Implement Windows memory tracking using Win32 APIs
4. Add platform-specific error handling

### Phase 3: Integration (COMPLETE)
1. Integrate MemoryTracker into LuaEngine execution flow
2. Connect with SecurityManager for memory limit enforcement
3. Add comprehensive error handling and fallback mechanisms
4. Performance optimization and testing

## Success Criteria

### Functional Requirements
- ✅ Real memory usage tracked and returned in ScriptResult.memory_used_mb
- ✅ Cross-platform support (Linux, macOS, Windows)
- ✅ Memory tracking accuracy within ±10%
- ✅ Integration with SecurityManager memory limits
- ✅ Graceful error handling when tracking fails

### Performance Requirements
- ✅ Memory tracking overhead < 2% execution time
- ✅ No memory leaks in tracker implementation
- ✅ Efficient platform API usage

### Quality Requirements
- ✅ All existing tests continue to pass
- ✅ 90%+ test coverage for new memory tracking code
- ✅ Documentation updated with memory tracking examples
- ✅ Error scenarios properly tested

## Dependencies

- **Platform APIs**: Platform-specific memory measurement libraries
- **Error Handling**: Enhanced ScriptError types for memory tracking
- **Security**: Integration with existing SecurityManager
- **Testing**: Mock implementations for CI/cross-platform testing

## Breaking Changes

**None** - This implementation is purely additive and enhances existing functionality without changing public APIs.

## File Locations

- `crates/mandrel-mcp-th/src/script_engines/memory_tracker.rs` - New memory tracking implementation
- `crates/mandrel-mcp-th/src/script_engines/lua_engine.rs` - Integration point (line 479)
- `crates/mandrel-mcp-th/src/script_engines/types.rs` - Enhanced error types
- `crates/mandrel-mcp-th/src/script_engines/mod.rs` - Module exports

---

**This design enables accurate, cross-platform memory tracking for Lua script execution while maintaining performance and providing robust error handling.** 