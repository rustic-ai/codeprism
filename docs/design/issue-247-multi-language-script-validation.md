# Multi-Language Script Validation Integration Design Document

## Problem Statement

We have successfully implemented individual script engines (JavaScript #252, Python #253, and Lua #254), but they are not integrated into the validation pipeline. The goal is to enable sophisticated test validation beyond basic JSONPath and schema validation by integrating multi-language script execution into the `ValidationEngine`.

Currently:
- ✅ Script engines exist but are isolated components
- ❌ No data structure support for `validation_scripts` in YAML specs
- ❌ No integration between ValidationEngine and script execution
- ❌ No ScriptContext API for scripts to access test data
- ❌ Enhanced `everything-server.yaml` scripts are non-functional

## Proposed Solution

### High-Level Architecture

```rust
// Enhanced YAML Spec Support
struct TestSpecification {
    validation_scripts: Option<Vec<ValidationScript>>,
    // ... existing fields
}

struct ValidationScript {
    name: String,
    language: ScriptLanguage,
    execution_phase: ExecutionPhase, // Before, After, Both
    required: bool,
    source: String,
    timeout_ms: Option<u64>,
}

// Integration Layer
struct ScriptValidationEngine {
    js_engine: JavaScriptEngine,
    python_engine: PythonEngine, 
    lua_engine: LuaEngine,
    context_builder: ScriptContextBuilder,
}

// Rich Context API
struct ScriptContext {
    test_case: TestCase,
    request: JsonValue,
    response: Option<JsonValue>,
    error: Option<String>,
    metadata: TestMetadata,
    session: SessionData,
    helpers: ContextHelpers,
}
```

### Component Interactions

```text
ValidationEngine
    ↓
ScriptValidationEngine ← ScriptContextBuilder
    ↓                          ↓
┌─────────────────┬─────────────────┬─────────────────┐
│  JavaScriptEngine │   PythonEngine    │   LuaEngine     │
└─────────────────┴─────────────────┴─────────────────┘
    ↓                          ↓                          ↓
ScriptResult ← [execution] → ScriptResult ← [execution] → ScriptResult
```

## API Design

### Core Types and Enums

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptLanguage {
    JavaScript,
    Python,
    Lua,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionPhase {
    Before,  // Execute before MCP request
    After,   // Execute after MCP response
    Both,    // Execute in both phases
}

#[derive(Debug, Clone)]
pub struct ScriptValidationConfig {
    pub enabled: bool,
    pub default_timeout_ms: u64,
    pub max_concurrent_scripts: usize,
    pub security_level: SecurityLevel,
}
```

### ScriptContext API

```rust
#[derive(Debug, Clone, Serialize)]
pub struct ScriptContext {
    pub test_case: TestCase,
    pub request: Option<JsonValue>,
    pub response: Option<JsonValue>, 
    pub error: Option<String>,
    pub metadata: TestMetadata,
    pub session: SessionData,
    pub previous_results: Vec<TestResult>,
    pub custom_data: JsonValue,
}

impl ScriptContext {
    pub fn new(
        test_case: TestCase,
        request: Option<JsonValue>,
        response: Option<JsonValue>,
        metadata: TestMetadata,
    ) -> Self;
    
    pub fn with_error(mut self, error: String) -> Self;
    pub fn with_session_data(mut self, session: SessionData) -> Self;
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextHelpers {
    // JavaScript context helpers would include functions like:
    // responseContains(path), getResponseValue(path), etc.
}
```

### Script Execution Integration

```rust
pub struct ScriptValidationEngine {
    js_engine: JavaScriptEngine,
    python_engine: PythonEngine,
    lua_engine: LuaEngine,
    config: ScriptValidationConfig,
}

impl ScriptValidationEngine {
    pub fn new(config: ScriptValidationConfig) -> Result<Self, ScriptError>;
    
    pub async fn execute_scripts(
        &self,
        scripts: &[ValidationScript],
        context: ScriptContext,
        phase: ExecutionPhase,
    ) -> Result<Vec<ScriptValidationResult>, ScriptError>;
    
    pub async fn execute_script(
        &self,
        script: &ValidationScript,
        context: &ScriptContext,
    ) -> Result<ScriptValidationResult, ScriptError>;
}

#[derive(Debug)]
pub struct ScriptValidationResult {
    pub script_name: String,
    pub success: bool,
    pub message: Option<String>,
    pub execution_time_ms: u64,
    pub output: JsonValue,
    pub error: Option<String>,
}
```

### ValidationEngine Integration

```rust
impl ValidationEngine {
    pub async fn validate_with_scripts(
        &self,
        test_case: &TestCase,
        request: &JsonValue,
        response: Option<&JsonValue>,
        error: Option<&str>,
        scripts: &[ValidationScript],
    ) -> ValidationResult {
        // 1. Execute "Before" phase scripts
        // 2. Perform existing JSONPath/schema validation  
        // 3. Execute "After" phase scripts
        // 4. Combine all results
    }
}
```

## Implementation Plan

### Phase 1: Data Structure Foundation
**Goal**: Support `validation_scripts` in YAML specifications

1. **Update spec data structures** (`crates/mandrel-mcp-th/src/spec/mod.rs`)
   - Add `ValidationScript` struct with all required fields
   - Add `validation_scripts` field to `TestSpecification`
   - Add script references to `TestCase` and `ExpectedOutput`

2. **Enhance YAML parsing**
   - Update deserialization to handle script definitions
   - Add validation for script configuration
   - Ensure backward compatibility with existing specs

3. **Add script validation**
   - Validate script language is supported
   - Check script source is non-empty
   - Validate execution phases are correct

### Phase 2: Script Context Framework
**Goal**: Rich context API for script access to test data

1. **Implement ScriptContext** (`crates/mandrel-mcp-th/src/validation/script_context.rs`)
   - Core context data structure
   - Context builder with fluent API
   - JSON serialization for cross-language compatibility

2. **Add helper functions generation**
   - JavaScript helpers: `responseContains()`, `getResponseValue()`, etc.
   - Python helpers: context object with validation utilities  
   - Lua helpers: context table with utility functions

3. **Implement context injection**
   - Convert context to language-specific formats
   - Handle JSON serialization edge cases
   - Ensure security of context data

### Phase 3: Integration Layer  
**Goal**: Connect script engines to validation pipeline

1. **Create ScriptValidationEngine** (`crates/mandrel-mcp-th/src/validation/script_engine.rs`)
   - Coordinate execution across all three engines
   - Handle script selection by language
   - Manage concurrent script execution
   - Aggregate results from multiple scripts

2. **Implement script execution workflow**
   - Before/After phase management
   - Error handling and recovery
   - Resource monitoring and timeout enforcement
   - Result aggregation and reporting

### Phase 4: ValidationEngine Integration
**Goal**: Wire script validation into existing validation pipeline

1. **Extend ValidationEngine** (`crates/mandrel-mcp-th/src/validation/engine.rs`)
   - Add script validation phase
   - Integrate with existing JSONPath validation
   - Handle script failures gracefully
   - Maintain performance with script overhead

2. **Update TestCaseExecutor** (`crates/mandrel-mcp-th/src/executor.rs`)
   - Wire script execution into test flow
   - Pass script results to reporting
   - Handle script execution errors

### Phase 5: Advanced Features
**Goal**: Performance and usability improvements

1. **Script caching and optimization**
   - Pre-compile scripts where possible
   - Cache contexts for repeated executions
   - Parallel execution of independent scripts

2. **Enhanced error reporting**
   - Detailed script execution logs
   - Context debugging information
   - Performance metrics collection

## Implementation Steps

### Step 1: TDD Foundation (RED Phase)
```rust
#[tokio::test]
async fn test_validation_script_data_structure_parsing() {
    let yaml = r#"
validation_scripts:
  - name: "precision_validator"
    language: "lua"
    execution_phase: "after"
    required: true
    source: |
      result = { success = true, message = "Test passed" }
"#;
    
    let spec: TestSpecification = serde_yaml::from_str(yaml).unwrap();
    assert!(spec.validation_scripts.is_some());
    let scripts = spec.validation_scripts.unwrap();
    assert_eq!(scripts.len(), 1);
    assert_eq!(scripts[0].name, "precision_validator");
    assert_eq!(scripts[0].language, ScriptLanguage::Lua);
}

#[tokio::test] 
async fn test_script_context_generation() {
    let test_case = create_test_case();
    let request = json!({"tool": "add", "params": {"a": 5, "b": 3}});
    let response = json!([{"text": "8"}]);
    
    let context = ScriptContext::new(test_case, Some(request), Some(response), TestMetadata::default());
    
    assert!(context.request.is_some());
    assert!(context.response.is_some());
    assert_eq!(context.test_case.name, "add_integers");
}

#[tokio::test]
async fn test_script_validation_engine_execution() {
    let engine = ScriptValidationEngine::new(ScriptValidationConfig::default()).unwrap();
    let script = ValidationScript {
        name: "test_script".to_string(),
        language: ScriptLanguage::Lua,
        execution_phase: ExecutionPhase::After,
        required: true,
        source: "result = { success = true }".to_string(),
        timeout_ms: Some(5000),
    };
    
    let context = create_test_context();
    let result = engine.execute_script(&script, &context).await.unwrap();
    
    assert!(result.success);
    assert_eq!(result.script_name, "test_script");
}
```

### Step 2: GREEN Phase Implementation
Implement minimal functionality to make tests pass:

1. Add basic data structures
2. Implement ScriptContext creation  
3. Create ScriptValidationEngine with basic execution
4. Wire into ValidationEngine

### Step 3: REFACTOR Phase
Optimize and enhance:

1. Add comprehensive error handling
2. Implement performance optimizations
3. Add security validations
4. Enhance context API

## Alternatives Considered

### Alternative 1: Embedded Script Engines Only
**Pros**: Better performance, single process
**Cons**: Security risks, limited language support, complex dependency management
**Decision**: Rejected - security is paramount for test execution

### Alternative 2: External Script Execution Service
**Pros**: Ultimate security, language independence
**Cons**: Complex deployment, network dependencies, latency
**Decision**: Rejected - too complex for current requirements

### Alternative 3: Plugin Architecture
**Pros**: Extensible, modular design
**Cons**: Over-engineering for current needs, complex API
**Decision**: Rejected - current approach is sufficient

## Success Criteria

### Functional Requirements
- ✅ YAML specs with `validation_scripts` parse correctly
- ✅ JavaScript, Python, and Lua scripts execute with proper context
- ✅ Script execution integrates seamlessly with existing ValidationEngine
- ✅ Enhanced `everything-server.yaml` scripts become fully functional
- ✅ Error handling provides clear debugging information

### Performance Requirements  
- ✅ Script execution overhead < 10% of total test time
- ✅ Concurrent script execution for independent scripts
- ✅ Memory usage remains bounded during script execution
- ✅ Script timeout enforcement prevents hanging tests

### Security Requirements
- ✅ All script execution uses existing secure engines
- ✅ Context data is safely serialized and injected
- ✅ Script failures don't crash the test harness
- ✅ Resource limits prevent runaway scripts

### Integration Requirements
- ✅ Backward compatibility with existing test specifications
- ✅ ValidationEngine API remains stable
- ✅ TestCaseExecutor continues to work with non-script tests
- ✅ Reporting includes script execution results and metrics

## Breaking Changes

**None expected** - this is a purely additive feature:
- Existing test specifications without scripts continue to work unchanged
- ValidationEngine API remains backward compatible
- All new functionality is opt-in via `validation_scripts` field

## Migration Path

**No migration required** - existing tests continue to work as-is. Users can:
1. Add `validation_scripts` to existing specs incrementally
2. Start with simple scripts and increase complexity over time
3. Use scripts alongside existing JSONPath/schema validation

This implementation will transform the test harness from basic validation to a comprehensive, enterprise-grade testing framework with sophisticated custom validation capabilities. 