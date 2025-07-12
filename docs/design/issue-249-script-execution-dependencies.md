# [Issue 249] Design Document: Add Script Execution Dependencies

## Problem Statement

The `mandrel-mcp-th` crate currently lacks the necessary dependencies to support multi-language script execution for validation purposes. To enable JavaScript, Python, and Lua script validation (as defined in issue #248), we need to add and verify the following dependencies:

- `mlua` - Lua script execution engine
- `quickjs` - JavaScript execution engine  
- `pyo3` - Python binding and execution
- `regex` - Regular expression support for validation
- `uuid` - UUID generation for script contexts

## Requirements

### Functional Requirements
- Add all five dependencies to `mandrel-mcp-th/Cargo.toml`
- Ensure dependencies compile successfully
- Verify basic functionality with minimal "hello world" examples
- Provide unit tests for each dependency

### Non-Functional Requirements
- Dependencies should be compatible with existing crate versions
- Minimize compilation time impact
- Use stable, well-maintained crate versions
- Consider optional features to reduce binary size

## Proposed Solution

### Dependency Specifications

```toml
[dependencies]
# Lua execution engine
mlua = { version = "0.9", features = ["lua54", "async"] }

# JavaScript execution engine
quickjs = { version = "0.4", features = ["async"] }

# Python bindings and execution
pyo3 = { version = "0.21", features = ["auto-initialize"] }

# Regular expression support
regex = "1.10"

# UUID generation
uuid = { version = "1.7", features = ["v4", "serde"] }
```

### Feature Justification

**mlua features:**
- `lua54`: Use Lua 5.4 for latest features and performance
- `async`: Enable async/await support for non-blocking execution

**quickjs features:**
- `async`: Enable async JavaScript execution

**pyo3 features:**
- `auto-initialize`: Automatically initialize Python interpreter

**uuid features:**
- `v4`: Generate random UUIDs for script contexts
- `serde`: Serialize/deserialize UUID types

## Implementation Plan

### Phase 1: Dependency Addition (TDD Red)
1. **Write failing tests** for each dependency:
   - `test_mlua_basic_execution()` - Execute simple Lua script
   - `test_quickjs_basic_execution()` - Execute simple JavaScript
   - `test_pyo3_basic_execution()` - Execute simple Python script
   - `test_regex_pattern_matching()` - Test regex functionality
   - `test_uuid_generation()` - Test UUID generation

2. **Add dependencies** to `Cargo.toml` without implementation

### Phase 2: Basic Implementation (TDD Green)
1. **Implement minimal hello world** for each engine:
   - Lua: `return "Hello from Lua"`
   - JavaScript: `"Hello from JavaScript"`
   - Python: `"Hello from Python"`
   - Regex: Match simple patterns
   - UUID: Generate and validate UUIDs

2. **Create integration module** `src/script_engines/mod.rs` with:
   ```rust
   pub mod lua_engine;
   pub mod js_engine;
   pub mod python_engine;
   pub mod utilities;
   ```

### Phase 3: Refactoring (TDD Refactor)
1. **Optimize compilation** with feature flags
2. **Add comprehensive error handling**
3. **Document engine capabilities and limitations**
4. **Performance benchmarks** for each engine

## API Design

### Test Structure
```rust
#[cfg(test)]
mod dependency_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mlua_basic_execution() {
        let lua = mlua::Lua::new();
        let result: String = lua.load("return 'Hello from Lua'").eval().unwrap();
        assert_eq!(result, "Hello from Lua");
    }
    
    #[tokio::test] 
    async fn test_quickjs_basic_execution() {
        let context = quickjs::Context::new().unwrap();
        let result = context.eval("'Hello from JavaScript'").unwrap();
        assert_eq!(result.as_str().unwrap(), "Hello from JavaScript");
    }
    
    #[test]
    fn test_pyo3_basic_execution() {
        pyo3::Python::with_gil(|py| {
            let result = py.eval("'Hello from Python'", None, None).unwrap();
            assert_eq!(result.to_string(), "Hello from Python");
        });
    }
    
    #[test]
    fn test_regex_pattern_matching() {
        let re = regex::Regex::new(r"hello").unwrap();
        assert!(re.is_match("hello world"));
        assert!(!re.is_match("goodbye world"));
    }
    
    #[test]
    fn test_uuid_generation() {
        let id = uuid::Uuid::new_v4();
        assert_eq!(id.get_version(), Some(uuid::Version::Random));
        
        // Test serialization
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: uuid::Uuid = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }
}
```

## Integration Points

### With Issue #248 (validation_scripts)
- These dependencies will be used by the script execution engines
- `ValidationScript` structs will specify which engine to use
- Engine selection based on `language` field

### With Issue #250 (ScriptContext/ScriptResult)
- Engines will populate `ScriptContext` with execution environment
- Results will be returned as `ScriptResult` with standardized format
- Error handling will use `ScriptError` types

## Alternatives Considered

### Alternative 1: Single JavaScript Engine
- **Pros**: Simpler dependency management, smaller binary
- **Cons**: Limited to JavaScript only, less flexible for users
- **Rejected**: Multi-language support is a key requirement

### Alternative 2: Runtime Engine Selection
- **Pros**: Dynamic engine loading, optional dependencies
- **Cons**: Complex build system, runtime overhead
- **Rejected**: Adds unnecessary complexity for this phase

### Alternative 3: WASM-based Execution
- **Pros**: Sandboxed execution, language-agnostic
- **Cons**: Performance overhead, complex integration
- **Deferred**: Consider for future security enhancements

## Success Criteria

### Compilation Success
- [ ] `cargo build` succeeds without errors
- [ ] `cargo test` runs all dependency tests
- [ ] No version conflicts with existing dependencies

### Functional Verification
- [ ] Each engine executes simple "hello world" scripts
- [ ] Regex patterns match and non-match correctly
- [ ] UUID generation produces valid v4 UUIDs
- [ ] Serialization/deserialization works for all types

### Performance Benchmarks
- [ ] Lua execution: <1ms for simple scripts
- [ ] JavaScript execution: <5ms for simple scripts  
- [ ] Python execution: <10ms for simple scripts
- [ ] Regex matching: <0.1ms for simple patterns
- [ ] UUID generation: <0.01ms per UUID

### Integration Readiness
- [ ] Dependencies are accessible from other modules
- [ ] Error types are compatible with existing error handling
- [ ] Async support works with tokio runtime
- [ ] Feature flags allow optional compilation

## Risk Assessment

### High Risk
- **Python dependency complexity**: pyo3 requires Python development headers
- **Compilation time impact**: Multiple language engines increase build time
- **Binary size growth**: Each engine adds significant size

### Medium Risk
- **Version compatibility**: Ensuring all dependencies work together
- **Platform support**: Some engines may not support all platforms
- **Memory usage**: Each engine has runtime overhead

### Low Risk
- **Regex performance**: Well-established, fast library
- **UUID generation**: Minimal overhead, standard library

## Mitigation Strategies

### For Python Dependency Issues
- Provide clear documentation for Python development setup
- Consider making pyo3 an optional feature
- Add CI tests for multiple Python versions

### For Compilation Time
- Use incremental compilation features
- Consider optional feature flags for each engine
- Optimize dependency features to minimize unused code

### For Binary Size
- Enable link-time optimization (LTO)
- Use feature flags to exclude unused engines
- Consider dynamic linking for development builds

## Testing Strategy

### Unit Tests
- Basic functionality test for each dependency
- Error handling tests for malformed scripts
- Performance tests for execution time limits

### Integration Tests
- Cross-engine compatibility tests
- Memory leak detection during repeated execution
- Concurrent execution safety tests

### CI/CD Integration
- Test on multiple platforms (Linux, macOS, Windows)
- Test with different Python versions (3.8, 3.9, 3.10, 3.11)
- Performance regression testing

## Documentation Requirements

### Code Documentation
- Rustdoc comments for all public APIs
- Examples for each engine usage
- Performance characteristics and limitations

### User Documentation
- Setup instructions for Python development environment
- Feature flag documentation
- Troubleshooting guide for common issues

---

**Implementation Priority**: High (blocks issue #247)
**Estimated Complexity**: Medium (dependency integration)
**Breaking Changes**: None (additive only) 