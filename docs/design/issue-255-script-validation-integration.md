# Issue #255: Integrate script execution into ValidationEngine

## Problem Statement

Currently, the ValidationEngine supports JSONPath validation, schema validation, and custom validators, but lacks support for script-based validation. While we have completed the LuaEngine implementation (Issue #254), it's not integrated into the validation pipeline. We need to wire script execution into the validation system to support before/after validation phases as defined in YAML test specifications.

## Current Architecture Analysis

### ValidationEngine Components
- **McpValidationEngine**: Main orchestrator with pluggable CustomValidator system
- **ValidationSpec**: Defines validation rules (JSONPath, schema, protocol, custom)
- **CustomValidator trait**: Interface for custom validation logic  
- **ValidationContext**: Provides execution metadata (method, request_id, server_capabilities, test_metadata)

### LuaEngine Integration Points
- **LuaEngine**: Complete implementation with context injection, sandboxing, and timeout enforcement
- **ScriptContext**: Provides request, response, and metadata to scripts
- **ScriptResult**: Returns success status, output, logs, duration, and memory usage

### YAML Specification Support
- **ValidationScript**: Defined in spec with `name`, `language`, `execution_phase`, `required`, `source`
- **Test case integration**: Can reference validation scripts by name via `validation_scripts` field
- **Execution phases**: Support for "before" and "after" validation phases

## Proposed Solution

### Architecture Overview

```rust
// New ScriptValidator implementing CustomValidator trait
pub struct ScriptValidator {
    lua_engine: LuaEngine,
    validation_scripts: HashMap<String, ValidationScript>,
    execution_phase: ScriptExecutionPhase,
}

// Integration into ValidationEngine
impl McpValidationEngine {
    pub fn add_script_validator(&mut self, scripts: Vec<ValidationScript>, phase: ScriptExecutionPhase);
    pub async fn validate_response_with_scripts(&self, response: &Value, spec: &ValidationSpec) -> Result<McpValidationResult>;
}

// Enhanced ValidationSpec to include script references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSpec {
    // ... existing fields ...
    pub validation_scripts: Option<Vec<String>>, // References to script names
    pub script_execution_phases: Option<ScriptExecutionConfig>,
}
```

### Component Design

#### 1. ScriptValidator Implementation

```rust
use crate::script_engines::{LuaEngine, ScriptContext, ScriptResult, ScriptConfig};
use crate::validation::{CustomValidator, ValidationContext, ValidationError};
use crate::spec::ValidationScript;

pub struct ScriptValidator {
    lua_engine: LuaEngine,
    validation_scripts: HashMap<String, ValidationScript>,
    execution_phase: ScriptExecutionPhase,
    config: ScriptValidationConfig,
}

#[derive(Debug, Clone)]
pub enum ScriptExecutionPhase {
    Before,  // Execute before standard validation
    After,   // Execute after standard validation
}

#[derive(Debug, Clone)]
pub struct ScriptValidationConfig {
    pub timeout_seconds: u32,
    pub memory_limit_mb: u32,
    pub fail_on_script_error: bool,
    pub capture_script_logs: bool,
}

impl CustomValidator for ScriptValidator {
    fn name(&self) -> &str {
        match self.execution_phase {
            ScriptExecutionPhase::Before => "script_validator_before",
            ScriptExecutionPhase::After => "script_validator_after",
        }
    }

    fn validate(
        &self,
        data: &Value,
        context: &ValidationContext,
    ) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
        let mut errors = Vec::new();
        
        for (script_name, script) in &self.validation_scripts {
            // Check if script should execute in this phase
            if !self.should_execute_script(script) {
                continue;
            }

            // Create script execution context
            let script_context = self.create_script_context(data, context, script_name);
            
            // Execute script
            match self.execute_validation_script(script, script_context) {
                Ok(result) => {
                    if !result.success {
                        if script.required.unwrap_or(false) || self.config.fail_on_script_error {
                            errors.push(ValidationError::FieldError {
                                field: format!("script:{}", script_name),
                                expected: "script execution success".to_string(),
                                actual: result.error.unwrap_or("script failed".to_string()),
                            });
                        }
                    }
                    
                    // Parse script output for additional validation errors
                    if let Some(script_errors) = self.parse_script_validation_output(&result) {
                        errors.extend(script_errors);
                    }
                }
                Err(e) => {
                    if script.required.unwrap_or(false) || self.config.fail_on_script_error {
                        errors.push(ValidationError::FieldError {
                            field: format!("script:{}", script_name),
                            expected: "script execution".to_string(),
                            actual: format!("execution error: {}", e),
                        });
                    }
                }
            }
        }
        
        Ok(errors)
    }
}

impl ScriptValidator {
    pub fn new(
        scripts: Vec<ValidationScript>,
        phase: ScriptExecutionPhase,
        config: ScriptValidationConfig,
    ) -> Result<Self, ValidationEngineError> {
        let script_config = ScriptConfig {
            timeout_seconds: config.timeout_seconds,
            memory_limit_mb: config.memory_limit_mb,
            disable_filesystem: true,
            disable_network: true,
        };
        
        let lua_engine = LuaEngine::new(&script_config)?;
        
        let validation_scripts: HashMap<String, ValidationScript> = scripts
            .into_iter()
            .map(|script| (script.name.clone(), script))
            .collect();
        
        Ok(Self {
            lua_engine,
            validation_scripts,
            execution_phase,
            config,
        })
    }
    
    fn should_execute_script(&self, script: &ValidationScript) -> bool {
        match (&script.execution_phase, &self.execution_phase) {
            (Some(phase), current_phase) => {
                match (phase.as_str(), current_phase) {
                    ("before", ScriptExecutionPhase::Before) => true,
                    ("after", ScriptExecutionPhase::After) => true,
                    _ => false,
                }
            }
            // Default to "after" if no phase specified
            (None, ScriptExecutionPhase::After) => true,
            _ => false,
        }
    }
    
    fn create_script_context(
        &self,
        response: &Value,
        validation_context: &ValidationContext,
        script_name: &str,
    ) -> ScriptContext {
        let mut metadata = HashMap::new();
        metadata.insert("script_name".to_string(), script_name.to_string());
        metadata.insert("validation_method".to_string(), validation_context.method.clone());
        
        // Add test metadata from validation context
        for (key, value) in &validation_context.test_metadata {
            metadata.insert(format!("test_{}", key), value.clone());
        }
        
        ScriptContext {
            request: validation_context.request_id.clone(),
            response: Some(response.clone()),
            metadata,
            server_info: validation_context.server_capabilities.clone(),
        }
    }
    
    async fn execute_validation_script(
        &self,
        script: &ValidationScript,
        context: ScriptContext,
    ) -> Result<ScriptResult, ScriptError> {
        let source = script.source.as_ref()
            .ok_or_else(|| ScriptError::ConfigurationError {
                message: format!("Script '{}' has no source code", script.name),
            })?;
        
        self.lua_engine.execute_script(source, context).await
    }
    
    fn parse_script_validation_output(&self, result: &ScriptResult) -> Option<Vec<ValidationError>> {
        // Parse script output for validation errors
        // Scripts can return structured validation results
        if let Ok(output) = serde_json::from_value::<ScriptValidationOutput>(result.output.clone()) {
            let mut errors = Vec::new();
            
            for error in output.validation_errors {
                errors.push(ValidationError::FieldError {
                    field: error.field,
                    expected: error.expected,
                    actual: error.actual,
                });
            }
            
            Some(errors)
        } else {
            None
        }
    }
}

// Script output structure for validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptValidationOutput {
    pub validation_errors: Vec<ScriptValidationError>,
    pub warnings: Vec<ScriptValidationWarning>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptValidationError {
    pub field: String,
    pub expected: String,
    pub actual: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptValidationWarning {
    pub field: String,
    pub message: String,
    pub suggestion: Option<String>,
}
```

#### 2. ValidationEngine Integration

```rust
// Enhanced McpValidationEngine with script support
impl McpValidationEngine {
    /// Add script validators for before/after phases
    pub fn add_script_validators(
        &mut self,
        scripts: Vec<ValidationScript>,
        config: ScriptValidationConfig,
    ) -> Result<(), ValidationEngineError> {
        // Separate scripts by execution phase
        let before_scripts: Vec<ValidationScript> = scripts.iter()
            .filter(|s| s.execution_phase.as_deref() == Some("before"))
            .cloned()
            .collect();
            
        let after_scripts: Vec<ValidationScript> = scripts.iter()
            .filter(|s| s.execution_phase.as_deref().unwrap_or("after") == "after")
            .cloned()
            .collect();
        
        // Create and add before-phase validator
        if !before_scripts.is_empty() {
            let before_validator = ScriptValidator::new(
                before_scripts,
                ScriptExecutionPhase::Before,
                config.clone(),
            )?;
            self.add_custom_validator(Box::new(before_validator))?;
        }
        
        // Create and add after-phase validator  
        if !after_scripts.is_empty() {
            let after_validator = ScriptValidator::new(
                after_scripts,
                ScriptExecutionPhase::After,
                config,
            )?;
            self.add_custom_validator(Box::new(after_validator))?;
        }
        
        Ok(())
    }

    /// Enhanced validation method with script support
    pub async fn validate_response_with_scripts(
        &self,
        response: &Value,
        spec: &ValidationSpec,
        scripts: Option<&[ValidationScript]>,
    ) -> Result<McpValidationResult, ValidationEngineError> {
        // Create a temporary engine with scripts if provided
        if let Some(scripts) = scripts {
            let mut temp_engine = self.clone(); // Would need to make McpValidationEngine cloneable
            
            let script_config = ScriptValidationConfig {
                timeout_seconds: 30,
                memory_limit_mb: 64,
                fail_on_script_error: false,
                capture_script_logs: true,
            };
            
            temp_engine.add_script_validators(scripts.to_vec(), script_config)?;
            temp_engine.validate_response(response, spec).await
        } else {
            self.validate_response(response, spec).await
        }
    }
}
```

#### 3. YAML Integration

```rust
// Enhanced test execution with script support
impl TestCaseExecutor {
    pub async fn execute_test_case_with_scripts(
        &self,
        test_case: &TestCase,
        validation_scripts: Option<&[ValidationScript]>,
    ) -> Result<TestResult, ExecutorError> {
        // Execute MCP tool call
        let response = self.execute_tool_call(&test_case.input).await?;
        
        // Resolve script references from test case
        let scripts_to_execute = if let Some(script_refs) = &test_case.validation_scripts {
            self.resolve_script_references(script_refs, validation_scripts)?
        } else {
            Vec::new()
        };
        
        // Create validation spec from test case
        let validation_spec = self.create_validation_spec_from_test_case(test_case)?;
        
        // Execute validation with scripts
        let validation_result = self.validation_engine
            .validate_response_with_scripts(
                &response,
                &validation_spec,
                if scripts_to_execute.is_empty() { None } else { Some(&scripts_to_execute) },
            )
            .await?;
        
        // Convert validation result to test result
        Ok(self.convert_validation_to_test_result(validation_result, response))
    }
    
    fn resolve_script_references(
        &self,
        script_names: &[String],
        available_scripts: Option<&[ValidationScript]>,
    ) -> Result<Vec<ValidationScript>, ExecutorError> {
        let scripts = available_scripts.ok_or_else(|| {
            ExecutorError::ConfigurationError("No validation scripts available".to_string())
        })?;
        
        let mut resolved_scripts = Vec::new();
        
        for name in script_names {
            if let Some(script) = scripts.iter().find(|s| s.name == *name) {
                resolved_scripts.push(script.clone());
            } else {
                return Err(ExecutorError::ConfigurationError(
                    format!("Validation script '{}' not found", name)
                ));
            }
        }
        
        Ok(resolved_scripts)
    }
}
```

## Implementation Plan

### Phase 1: Core ScriptValidator Implementation
1. **Create ScriptValidator**: Implement CustomValidator trait with LuaEngine integration
2. **Add ScriptExecutionPhase**: Define before/after execution phases
3. **Implement script context mapping**: Map ValidationContext to ScriptContext
4. **Add error handling**: Proper error propagation from script execution to validation results

### Phase 2: ValidationEngine Integration  
1. **Enhance McpValidationEngine**: Add script validator management methods
2. **Implement script resolution**: Resolve script references from YAML specifications
3. **Add configuration support**: ScriptValidationConfig for timeout, memory limits, error handling
4. **Update validation pipeline**: Integrate script execution into standard validation flow

### Phase 3: YAML Specification Support
1. **Extend ValidationSpec**: Add script reference fields
2. **Update test case execution**: Support script execution in TestCaseExecutor
3. **Add script output parsing**: Parse structured validation results from scripts
4. **Implement script lifecycle**: Load, validate, and cache validation scripts

### Phase 4: End-to-End Integration
1. **YAML loading integration**: Load validation scripts from test specifications
2. **Script caching**: Cache compiled scripts for performance
3. **Error aggregation**: Properly aggregate script errors with standard validation errors
4. **Performance optimization**: Minimize script execution overhead

## Error Handling Strategy

### Script Execution Errors
- **Configuration errors**: Missing scripts, invalid references
- **Runtime errors**: Script execution failures, timeouts, memory limits
- **Validation errors**: Script-generated validation failures
- **Network/filesystem errors**: Blocked access attempts from scripts

### Error Propagation
- **Required scripts**: Failures block test execution
- **Optional scripts**: Failures logged but don't block validation
- **Error aggregation**: Combine script errors with standard validation errors
- **Detailed diagnostics**: Include script name, line numbers, execution context

## Performance Considerations

### Script Execution Optimization
- **Script compilation caching**: Cache compiled Lua bytecode for repeated execution
- **Context reuse**: Reuse script contexts where possible
- **Parallel execution**: Execute independent scripts concurrently
- **Memory management**: Proper cleanup of script execution environments

### Resource Limits
- **Timeout enforcement**: Prevent runaway script execution
- **Memory limits**: Sandbox script memory usage
- **Execution isolation**: Prevent scripts from affecting each other
- **Resource monitoring**: Track script resource usage for diagnostics

## Testing Strategy

### Unit Tests
1. **ScriptValidator tests**: Test CustomValidator implementation
2. **Script execution tests**: Test before/after phase execution
3. **Error handling tests**: Test script failure scenarios
4. **Context mapping tests**: Test ValidationContext to ScriptContext conversion

### Integration Tests
1. **End-to-end validation**: Test complete validation pipeline with scripts
2. **YAML integration tests**: Test script loading from YAML specifications
3. **Performance tests**: Test script execution under load
4. **Error scenario tests**: Test validation behavior with script failures

### Test Coverage Requirements
- **Unit tests**: 95%+ coverage of ScriptValidator implementation
- **Integration tests**: Cover all script execution phases and error paths
- **End-to-end tests**: Real YAML specifications with working validation scripts
- **Performance tests**: Validate script execution within latency requirements

## Security Considerations

### Script Sandboxing
- **Filesystem isolation**: Scripts cannot access filesystem
- **Network isolation**: Scripts cannot make network requests
- **Memory limits**: Enforce strict memory usage limits
- **Execution timeouts**: Prevent infinite loops and long-running scripts

### Input Validation
- **Script source validation**: Validate script syntax before execution
- **Context sanitization**: Sanitize data passed to scripts
- **Output validation**: Validate script output before processing
- **Permission enforcement**: Enforce script execution permissions

## Success Criteria

### Functional Requirements
- ✅ Scripts execute in before/after validation phases
- ✅ YAML specifications support validation script references
- ✅ End-to-end tests pass with real validation scripts
- ✅ Error propagation works correctly from scripts to validation results
- ✅ Script context includes request, response, and metadata

### Performance Requirements
- ✅ Script execution adds <10% overhead to validation time
- ✅ Memory usage remains within configured limits
- ✅ Script compilation and caching improves repeated execution performance
- ✅ Concurrent script execution scales with available resources

### Quality Requirements
- ✅ 95%+ test coverage of script integration code
- ✅ Comprehensive error handling for all script failure modes
- ✅ Detailed logging and diagnostics for script execution
- ✅ Security sandboxing prevents malicious script behavior

## Future Enhancements

### Multi-Language Support
- **JavaScript engine**: Add Node.js script execution support (Issue #252)
- **Python engine**: Add Python script execution support (Issue #253)
- **Engine registry**: Abstract script engine selection by language

### Advanced Script Features
- **Script libraries**: Support shared script libraries and imports
- **Async scripts**: Support asynchronous script execution
- **Script debugging**: Add debugging and profiling capabilities
- **Script templates**: Provide common validation script templates

### Performance Optimizations
- **Script compilation pipeline**: Pre-compile scripts at load time
- **Hot path optimization**: Optimize frequently executed validation paths
- **Resource pooling**: Pool script execution contexts for reuse
- **Metrics and monitoring**: Add detailed script execution metrics

## Dependencies

### Completed Dependencies
- ✅ **Issue #254**: LuaEngine implementation with context injection and sandboxing

### Concurrent Dependencies
- **Validation engine enhancements**: May require ValidationEngine improvements
- **YAML specification updates**: May need additional YAML schema extensions

### Future Dependencies
- **Issue #256**: Wire TestCaseExecutor to script validation (builds on this work)
- **Issue #257**: Update documentation and examples (requires this implementation)

---

**This design provides a comprehensive foundation for integrating script execution into the ValidationEngine while maintaining performance, security, and maintainability standards.** 