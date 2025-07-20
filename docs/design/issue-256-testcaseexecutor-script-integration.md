# Issue #256: Wire TestCaseExecutor to Script Validation

## Problem Statement

Currently, the TestCaseExecutor executes test cases through a validation pipeline, but it doesn't integrate script validation despite having a ScriptValidator implementation (Issue #255). Test cases can reference validation scripts in their YAML specifications, but these scripts are not executed during test case execution. We need to wire the ScriptValidator into the TestCaseExecutor to enable script-based validation as part of the test lifecycle.

## Current Architecture Analysis

### TestCaseExecutor Flow
The current `TestCaseExecutor::execute_test_case` flow:
1. **Prepare Tool Request**: Convert test case input to MCP tool arguments
2. **Execute MCP Call**: Call the tool with timeout handling
3. **Validate Response**: Use ValidationEngine to validate against expected output
4. **Collect Metrics**: Gather performance metrics
5. **Return Result**: Create TestCaseResult with success/failure status

### Script Integration Points
- **TestCase**: Has `validation_scripts: Option<Vec<String>>` field referencing script names
- **TestSpecification**: Has top-level `validation_scripts: Option<Vec<ValidationScript>>` definitions
- **ScriptValidator**: Implements CustomValidator trait with before/after execution phases
- **ValidationEngine**: Already supports CustomValidator integration

### Current Gap
The TestCaseExecutor doesn't:
- Load validation scripts from test specifications
- Create ScriptValidator instances for test cases that reference scripts
- Execute scripts at appropriate phases (before/after validation)
- Integrate script results into test case results

## Proposed Solution

### High-Level Architecture

```
TestCaseExecutor::execute_test_case()
├── 1. Load and prepare validation scripts (NEW)
├── 2. Execute "before" scripts (NEW)
├── 3. Prepare MCP tool request (EXISTING)
├── 4. Execute MCP call (EXISTING)
├── 5. Execute "after" scripts (NEW)
├── 6. Standard validation with script results (MODIFIED)
├── 7. Collect metrics including script metrics (MODIFIED)
└── 8. Return comprehensive result (MODIFIED)
```

### Script Integration Design

#### 1. Script Loading and Management
```rust
pub struct ScriptManager {
    available_scripts: HashMap<String, ValidationScript>,
    script_validators: HashMap<String, Arc<ScriptValidator>>,
}

impl ScriptManager {
    pub fn new(scripts: Vec<ValidationScript>) -> Result<Self, ScriptError>;
    pub fn get_scripts_for_test_case(&self, test_case: &TestCase) -> Vec<&ValidationScript>;
    pub fn create_validators_for_phase(&self, scripts: &[&ValidationScript], phase: ScriptExecutionPhase) -> Result<Vec<Arc<ScriptValidator>>, ScriptError>;
}
```

#### 2. Enhanced TestCaseExecutor
```rust
pub struct TestCaseExecutor {
    client: Arc<Mutex<McpClient>>,
    validation_engine: ValidationEngine,
    config: ExecutorConfig,
    script_manager: Option<ScriptManager>, // NEW
}

impl TestCaseExecutor {
    pub fn with_scripts(client: Arc<Mutex<McpClient>>, config: ExecutorConfig, scripts: Vec<ValidationScript>) -> Result<Self, ExecutorError>;
    
    async fn execute_test_case_with_scripts(&mut self, tool_name: &str, test_case: &TestCase) -> Result<TestCaseResult, ExecutorError>;
    
    async fn execute_script_phase(&self, phase: ScriptExecutionPhase, scripts: &[&ValidationScript], context: &ValidationContext) -> Result<Vec<ScriptValidationResult>, ExecutorError>;
}
```

#### 3. Script Execution Context
```rust
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub method: String,
    pub request_id: Option<serde_json::Value>,
    pub response: Option<serde_json::Value>, // Available in "after" phase
    pub test_metadata: HashMap<String, String>,
    pub tool_name: String,
    pub test_case_name: String,
}

#[derive(Debug, Clone)]
pub struct ScriptValidationResult {
    pub script_name: String,
    pub success: bool,
    pub execution_time: Duration,
    pub errors: Vec<ValidationError>,
    pub logs: Vec<String>,
    pub phase: ScriptExecutionPhase,
}
```

#### 4. Enhanced Test Results
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseResult {
    pub test_name: String,
    pub tool_name: String,
    pub success: bool,
    pub execution_time: Duration,
    pub validation: ValidationResult,
    pub script_results: Vec<ScriptValidationResult>, // NEW
    pub metrics: ExecutionMetrics,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub duration: Duration,
    pub memory_usage: Option<u64>,
    pub network_latency: Option<Duration>,
    pub retry_count: u32,
    pub script_execution_time: Duration, // NEW
    pub script_count: u32, // NEW
}
```

## Implementation Plan

### Phase 1: Script Manager Implementation
1. **Create ScriptManager struct** to manage available scripts and validators
2. **Implement script loading** from TestSpecification validation_scripts
3. **Add script filtering logic** to find scripts referenced by test cases
4. **Implement validator creation** for different execution phases

### Phase 2: TestCaseExecutor Integration
1. **Modify TestCaseExecutor constructor** to accept optional scripts
2. **Update execute_test_case method** to include script execution phases
3. **Implement script execution workflow**:
   - Before phase: Execute before MCP call
   - After phase: Execute after MCP call, before standard validation
4. **Integrate script results** into validation pipeline

### Phase 3: Context and Results Enhancement
1. **Implement ValidationContext creation** with test case and response data
2. **Add ScriptValidationResult** to capture script execution details
3. **Enhance TestCaseResult** to include script results and metrics
4. **Update error handling** to propagate script errors appropriately

### Phase 4: Error Handling and Metrics
1. **Add script-specific error variants** to ExecutorError
2. **Implement script timeout handling** consistent with tool execution
3. **Add script metrics collection** (execution time, success rate)
4. **Ensure script failures** are handled according to required/optional configuration

## Detailed Implementation

### Script Execution Flow
```rust
pub async fn execute_test_case(&mut self, tool_name: &str, test_case: &TestCase) -> Result<TestCaseResult, ExecutorError> {
    let start_time = Instant::now();
    let mut script_results = Vec::new();
    
    // 1. Prepare validation context
    let context = ValidationContext {
        method: "tools/call".to_string(),
        request_id: Some(test_case.input.clone()),
        response: None, // Not available yet
        test_metadata: self.create_test_metadata(test_case),
        tool_name: tool_name.to_string(),
        test_case_name: test_case.name.clone(),
    };
    
    // 2. Execute "before" scripts
    if let Some(script_manager) = &self.script_manager {
        let scripts = script_manager.get_scripts_for_test_case(test_case);
        let before_scripts: Vec<_> = scripts.iter().filter(|s| s.execution_phase.as_deref() == Some("before")).collect();
        
        let before_results = self.execute_script_phase(ScriptExecutionPhase::Before, &before_scripts, &context).await?;
        script_results.extend(before_results);
        
        // Check if any required "before" scripts failed
        if before_scripts.iter().any(|s| s.required.unwrap_or(false)) && 
           script_results.iter().any(|r| r.phase == ScriptExecutionPhase::Before && !r.success) {
            return Ok(TestCaseResult {
                test_name: test_case.name.clone(),
                tool_name: tool_name.to_string(),
                success: false,
                execution_time: start_time.elapsed(),
                validation: ValidationResult { is_valid: false, validation_errors: vec![], warnings: vec![] },
                script_results,
                metrics: self.collect_metrics_with_scripts(start_time, &None, &script_results),
                error: Some("Required 'before' script validation failed".to_string()),
            });
        }
    }
    
    // 3. Prepare and execute MCP tool call (existing logic)
    let (_tool_name, arguments) = self.prepare_tool_request(tool_name, &test_case.input)?;
    let response = self.execute_mcp_call(tool_name, arguments).await?;
    
    // 4. Execute "after" scripts with response data
    if let Some(script_manager) = &self.script_manager {
        let scripts = script_manager.get_scripts_for_test_case(test_case);
        let after_scripts: Vec<_> = scripts.iter().filter(|s| s.execution_phase.as_deref() != Some("before")).collect();
        
        let mut context_with_response = context.clone();
        context_with_response.response = Some(response.clone());
        
        let after_results = self.execute_script_phase(ScriptExecutionPhase::After, &after_scripts, &context_with_response).await?;
        script_results.extend(after_results);
    }
    
    // 5. Standard validation (existing logic)
    let validation_result = self.validate_response(&response, &test_case.expected).await?;
    
    // 6. Determine overall success including script results
    let script_success = script_results.iter().all(|r| r.success || !self.is_script_required(&r.script_name));
    let overall_success = validation_result.is_valid && script_success;
    
    // 7. Collect enhanced metrics
    let metrics = self.collect_metrics_with_scripts(start_time, &Some(response), &script_results);
    
    Ok(TestCaseResult {
        test_name: test_case.name.clone(),
        tool_name: tool_name.to_string(),
        success: overall_success,
        execution_time: metrics.duration,
        validation: validation_result,
        script_results,
        metrics,
        error: if overall_success { None } else { Some("Test case validation failed".to_string()) },
    })
}
```

### Script Phase Execution
```rust
async fn execute_script_phase(
    &self, 
    phase: ScriptExecutionPhase, 
    scripts: &[&ValidationScript], 
    context: &ValidationContext
) -> Result<Vec<ScriptValidationResult>, ExecutorError> {
    let mut results = Vec::new();
    
    for script in scripts {
        let start_time = Instant::now();
        
        // Create ScriptValidator for this script
        let script_config = ScriptValidationConfig {
            timeout_seconds: self.config.timeout.as_secs() as u32,
            memory_limit_mb: 64, // From config
            fail_on_script_error: script.required.unwrap_or(false),
            capture_script_logs: true,
        };
        
        let validator = ScriptValidator::new(vec![script.clone()], phase.clone(), script_config)
            .map_err(|e| ExecutorError::ConfigError(format!("Failed to create script validator: {}", e)))?;
        
        // Execute script validation
        let script_validation_context = crate::validation::ValidationContext {
            method: context.method.clone(),
            request_id: context.request_id.clone(),
            server_capabilities: None, // TODO: Add server capabilities
            test_metadata: context.test_metadata.clone(),
        };
        
        let validation_errors = validator.validate(
            &context.response.clone().unwrap_or(serde_json::Value::Null),
            &script_validation_context
        ).map_err(|e| ExecutorError::ValidationError(format!("Script validation failed: {}", e)))?;
        
        let script_result = ScriptValidationResult {
            script_name: script.name.clone(),
            success: validation_errors.is_empty(),
            execution_time: start_time.elapsed(),
            errors: validation_errors,
            logs: vec![], // TODO: Implement log capture
            phase: phase.clone(),
        };
        
        results.push(script_result);
    }
    
    Ok(results)
}
```

## Testing Strategy

### Unit Tests
1. **ScriptManager Tests**:
   - Script loading from specifications
   - Script filtering by test case references
   - Validator creation for different phases

2. **TestCaseExecutor Script Integration Tests**:
   - Test case execution with "before" scripts
   - Test case execution with "after" scripts
   - Mixed phase script execution
   - Required vs optional script handling

3. **Error Handling Tests**:
   - Script timeout scenarios
   - Required script failure scenarios
   - Invalid script configuration handling

### Integration Tests
1. **End-to-End Test Execution**:
   - Complete test case with script validation
   - Multiple test cases with shared scripts
   - Performance impact measurement

2. **YAML Specification Integration**:
   - Loading scripts from YAML specifications
   - Test case referencing non-existent scripts
   - Script execution phase validation

### Performance Tests
1. **Script Execution Overhead**:
   - Measure impact on test execution time
   - Memory usage with script execution
   - Concurrent script execution performance

## Success Criteria

### Functional Requirements
- ✅ Test cases can reference validation scripts from YAML specifications
- ✅ Scripts execute in correct phases (before/after) during test execution
- ✅ Script results are integrated into test case results
- ✅ Required script failures cause test case failure
- ✅ Optional script failures don't cause test case failure
- ✅ Script execution metrics are collected and reported

### Performance Requirements
- ✅ Script execution adds <20% overhead to test case execution time
- ✅ Memory usage scales linearly with number of scripts
- ✅ Script timeout handling prevents hanging test cases

### Quality Requirements
- ✅ Comprehensive error handling for all script execution scenarios
- ✅ Detailed logging and metrics for script execution
- ✅ Backward compatibility with existing test cases (no scripts)

## Breaking Changes
None expected - this is an additive feature that enhances existing test case execution.

## Alternative Approaches Considered

### 1. Separate Script Execution Service
**Approach**: Create a separate service for script execution outside TestCaseExecutor
**Pros**: Better separation of concerns, easier to test
**Cons**: Complex communication, harder to integrate results
**Decision**: Rejected - integration complexity outweighs benefits

### 2. Script Execution as ValidationEngine Plugin
**Approach**: Integrate scripts as just another CustomValidator in ValidationEngine
**Pros**: Simpler integration, consistent with existing validation
**Cons**: Limited control over execution phases, harder to provide test context
**Decision**: Rejected - doesn't support "before" phase execution

### 3. Script Execution in Test Runner
**Approach**: Handle script execution at the test runner level
**Pros**: Can control entire test lifecycle
**Cons**: Test runner complexity, harder to provide tool-specific context
**Decision**: Rejected - TestCaseExecutor is the right abstraction level

## Future Enhancements

### Script Caching and Optimization
- **Script Compilation Caching**: Cache compiled scripts for performance
- **Script Result Caching**: Cache script results for deterministic scripts
- **Parallel Script Execution**: Execute independent scripts concurrently

### Enhanced Script Features
- **Script Dependencies**: Support scripts that depend on other scripts
- **Conditional Script Execution**: Execute scripts based on test results
- **Script Templates**: Support parameterized scripts with test case data

### Advanced Integration
- **Custom Script Engines**: Support for additional script languages
- **Script Debugging**: Enhanced debugging and profiling for script execution
- **Script Metrics Dashboard**: Visualization of script execution metrics

---

This design provides a comprehensive solution for integrating script validation into the TestCaseExecutor while maintaining backward compatibility and providing enhanced testing capabilities. 