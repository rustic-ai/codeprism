# Python Engine Implementation Design Document

## Problem Statement

Issue #253 requires implementing a Python script execution engine for the MCP test harness. This engine will:

- Execute Python scripts in isolated subprocess environments
- Inject MCP validation context into Python scripts
- Capture output, errors, and performance metrics
- Support timeout handling and memory tracking
- Enable comprehensive testing of MCP server Python implementations

## Proposed Solution

### High-Level Architecture

```
PythonEngine
├── subprocess execution with timeout
├── context injection via stdin/environment
├── output capture (stdout/stderr)
├── error handling and parsing
├── memory tracking integration
└── script precompilation (bytecode caching)
```

### API Design

```rust
/// Python script execution engine using subprocess
pub struct PythonEngine {
    config: ScriptConfig,
    python_path: PathBuf,
}

/// Precompiled Python script with bytecode caching
#[derive(Debug, Clone)]
pub struct PythonScript {
    source: String,
    bytecode_path: Option<PathBuf>,
    function_name: Option<String>,
}

impl PythonEngine {
    /// Create new Python engine with configuration
    pub fn new(config: &ScriptConfig) -> Result<Self, ScriptError>;
    
    /// Execute Python script with context injection
    pub async fn execute_script(
        &self,
        script: &str,
        context: ScriptContext,
    ) -> Result<ScriptResult, ScriptError>;
    
    /// Precompile Python script to bytecode
    pub fn precompile_script(
        &self,
        script: &str,
        function_name: Option<String>,
    ) -> Result<PythonScript, ScriptError>;
    
    /// Execute precompiled Python script
    pub async fn execute_precompiled(
        &self,
        script: &PythonScript,
        context: ScriptContext,
    ) -> Result<ScriptResult, ScriptError>;
}
```

### Implementation Plan

#### Phase 1: Basic Execution
1. **Subprocess Creation**: Use `tokio::process::Command` for async subprocess
2. **Script Execution**: Write script to temp file, execute with `python script.py`
3. **Output Capture**: Capture stdout/stderr with proper encoding handling
4. **Basic Error Handling**: Parse Python exceptions and syntax errors

#### Phase 2: Context Injection
1. **JSON Context**: Inject context via JSON file or stdin
2. **Environment Variables**: Pass metadata via environment
3. **Import System**: Create helper module for context access
4. **Validation Support**: Enable MCP validation script patterns

#### Phase 3: Advanced Features
1. **Timeout Handling**: Use `tokio::time::timeout` with process termination
2. **Memory Tracking**: Integrate with existing MemoryTracker
3. **Performance Monitoring**: Capture execution metrics
4. **Bytecode Caching**: Optional precompilation for performance

#### Phase 4: Production Features
1. **Security Sandbox**: Restrict file system access (future)
2. **Resource Limits**: CPU and memory constraints (future)
3. **Virtual Environment**: Isolated package environments (future)

### Context Injection Strategy

```python
# Injected into Python script environment
import json
import sys

# Context data injected via JSON
context = {
    "request": {...},      # MCP request data
    "response": {...},     # MCP response data (if available)
    "metadata": {
        "test_name": "...",
        "tool_name": "...",
        "server_info": {...}
    }
}

def log(level, message):
    """Logging function for validation scripts"""
    print(f"[{level}] {message}", file=sys.stderr)

# User script executed here
```

### Error Handling Strategy

```rust
#[derive(Debug, thiserror::Error)]
pub enum PythonError {
    #[error("Python interpreter not found at: {path}")]
    InterpreterNotFound { path: String },
    #[error("Syntax error: {message} at line {line}")]
    SyntaxError { message: String, line: u32 },
    #[error("Runtime error: {message}")]
    RuntimeError { message: String },
    #[error("Import error: {module}")]
    ImportError { module: String },
    #[error("Timeout after {timeout_ms}ms")]
    TimeoutError { timeout_ms: u64 },
    #[error("Process failed with exit code: {code}")]
    ProcessError { code: i32 },
}
```

### Performance Requirements

- **Execution Time**: <100ms for simple scripts
- **Memory Usage**: <50MB baseline overhead
- **Throughput**: >100 scripts/second for cached execution
- **Startup Time**: <50ms for subprocess creation

### Testing Strategy

Following TDD principles with comprehensive test coverage:

#### Test Categories
1. **Engine Creation Tests**
   - Python interpreter detection
   - Configuration validation
   - Error handling for missing Python

2. **Script Execution Tests**
   - Simple script execution
   - Context injection verification
   - Output capture (stdout/stderr)
   - Return value handling

3. **Error Handling Tests**
   - Syntax error parsing
   - Runtime exception handling
   - Import error detection
   - Timeout scenarios

4. **Performance Tests**
   - Execution time measurement
   - Memory usage tracking
   - Concurrent execution
   - Resource cleanup

5. **MCP Integration Tests**
   - MCP request/response validation
   - Tool execution scripts
   - Server response verification
   - Complex validation scenarios

### Implementation Files

```
crates/mandrel-mcp-th/src/script_engines/
├── python_engine.rs          # Main implementation
├── python_script.rs          # Script representation (if needed)
└── python_error.rs           # Error types (if needed)
```

### Dependencies

```toml
[dependencies]
tokio = { version = "1.0", features = ["process", "fs", "time"] }
tempfile = "3.0"              # Temporary file creation
serde_json = "1.0"            # JSON context serialization
which = "4.0"                 # Python interpreter detection
```

### Alternatives Considered

1. **PyO3 Integration**: Direct Python C API
   - **Pros**: Better performance, no subprocess overhead
   - **Cons**: Complex setup, version compatibility issues
   - **Decision**: Use subprocess for simplicity and isolation

2. **Docker Containers**: Sandbox execution
   - **Pros**: Strong isolation, reproducible environment
   - **Cons**: Docker dependency, performance overhead
   - **Decision**: Future enhancement, start with subprocess

3. **Virtual Environment**: Per-execution isolation
   - **Pros**: Package isolation, reproducible dependencies
   - **Cons**: Setup complexity, performance impact
   - **Decision**: Future enhancement for production use

## Success Criteria

### Functional Requirements
- ✅ Execute Python scripts with context injection
- ✅ Handle all Python error types gracefully
- ✅ Support timeout and resource management
- ✅ Integrate with memory tracking system
- ✅ Enable MCP validation script patterns

### Performance Requirements
- ✅ <100ms execution for simple scripts
- ✅ <50MB memory overhead
- ✅ Proper resource cleanup
- ✅ Concurrent execution support

### Quality Requirements
- ✅ 90%+ test coverage
- ✅ Comprehensive error scenarios
- ✅ Performance benchmarks
- ✅ Security considerations documented

This design provides a robust foundation for Python script execution while maintaining consistency with the JavaScript engine architecture and following established patterns in the codebase. 