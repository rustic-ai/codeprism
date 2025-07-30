//! Script validation integration for ValidationEngine
//!
//! This module provides ScriptValidator that implements the CustomValidator trait
//! to integrate script execution (Lua, JavaScript, Python) into the validation pipeline.

use crate::script_engines::{LuaEngine, ScriptContext, ScriptResult, ScriptConfig, ScriptError, ContextMetadata, ServerInfo};
use crate::validation::{CustomValidator, ValidationContext, ValidationError};
use crate::spec::ValidationScript;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Script execution phases for validation
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptExecutionPhase {
    Before,  // Execute before standard validation
    After,   // Execute after standard validation
}

/// Configuration for script validation behavior
#[derive(Debug, Clone)]
pub struct ScriptValidationConfig {
    pub timeout_seconds: u32,
    pub memory_limit_mb: u32,
    pub fail_on_script_error: bool,
    pub capture_script_logs: bool,
}

impl Default for ScriptValidationConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            memory_limit_mb: 64,
            fail_on_script_error: false,
            capture_script_logs: true,
        }
    }
}

/// ScriptValidator implementing CustomValidator trait for script execution in validation pipeline
pub struct ScriptValidator {
    validation_scripts: HashMap<String, ValidationScript>,
    execution_phase: ScriptExecutionPhase,
    config: ScriptValidationConfig,
    lua_engine: Arc<LuaEngine>,
}

/// Script output structure for validation results
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

impl ScriptValidator {
    /// Create a new ScriptValidator with scripts for the specified execution phase
    pub fn new(
        scripts: HashMap<String, ValidationScript>,
        phase: ScriptExecutionPhase,
        config: ScriptValidationConfig,
    ) -> Result<Self, ScriptError> {
        // Filter scripts by execution phase
        let validation_scripts: HashMap<String, ValidationScript> = scripts
            .into_iter()
            .filter(|(_, script)| {
                if let Some(script_phase) = &script.execution_phase {
                    match phase {
                        ScriptExecutionPhase::Before => script_phase == "before" || script_phase == "both",
                        ScriptExecutionPhase::After => script_phase == "after" || script_phase == "both",
                    }
                } else {
                    // If no phase specified, include in all phases
                    true
                }
            })
            .collect();

        // Create LuaEngine with appropriate configuration
        let script_config = ScriptConfig {
            timeout_seconds: config.timeout_seconds,
            memory_limit_mb: config.memory_limit_mb,
            disable_filesystem: true,
            disable_network: true,
        };
        
        let lua_engine = LuaEngine::new(&script_config)
            .map_err(|e| ScriptError::ConfigurationError {
                message: format!("Failed to create LuaEngine: {e}"),
            })?;

        Ok(Self {
            validation_scripts,
            execution_phase: phase,
            config,
            lua_engine: Arc::new(lua_engine),
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
        // Create proper ContextMetadata
        let metadata = ContextMetadata {
            test_name: script_name.to_string(),
            execution_id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            tool_name: validation_context.method.clone(),
            server_info: ServerInfo {
                name: "validation-engine".to_string(),
                version: "1.0.0".to_string(),
                capabilities: validation_context.server_capabilities.clone().unwrap_or_default(),
            },
        };
        
        let script_config = ScriptConfig {
            timeout_seconds: self.config.timeout_seconds,
            memory_limit_mb: self.config.memory_limit_mb,
            disable_filesystem: true,
            disable_network: true,
        };
        
        ScriptContext {
            request: validation_context.request_id.clone().unwrap_or_default(),
            response: Some(response.clone()),
            metadata,
            config: script_config,
        }
    }

    /// Execute a validation script and return the result
    async fn execute_validation_script(
        &self, 
        script: &ValidationScript, 
        context: ScriptContext
    ) -> Result<ScriptResult, ScriptError> {
        let source = script.source.as_ref()
            .ok_or_else(|| ScriptError::ConfigurationError {
                message: "Script source is empty".to_string(),
            })?;

        self.lua_engine.execute_script(source, context).await
    }

    /// Parse script execution results for validation errors
    fn parse_script_validation_output(&self, script_result: &Result<ScriptResult, ScriptError>) -> Option<Vec<ValidationError>> {
        match script_result {
            Ok(result) => {
                if !result.success {
                    // Script executed but reported failure
                    let error_message = result.error.as_ref()
                        .map(|e| e.to_string())
                        .unwrap_or_else(|| "Script validation failed".to_string());
                    
                    Some(vec![ValidationError::FieldError {
                        field: "script_validation".to_string(),
                        expected: "script validation success".to_string(),
                        actual: error_message,
                    }])
                } else {
                    // Script executed successfully
                    None
                }
            }
            Err(script_error) => {
                // Script execution failed
                if self.config.fail_on_script_error {
                    Some(vec![ValidationError::FieldError {
                        field: "script_execution".to_string(),
                        expected: "successful script execution".to_string(),
                        actual: format!("Script execution error: {script_error}"),
                    }])
                } else {
                    // Don't fail validation on script errors
                    None
                }
            }
        }
    }
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
        // Note: This is a synchronous trait but we need async execution
        // In a real implementation, we'd use block_on or similar
        let mut errors = Vec::new();
        
        for (script_name, script) in &self.validation_scripts {
            // Check if script should execute in this phase
            if !self.should_execute_script(script) {
                continue;
            }

            // Create script execution context
            let script_context = self.create_script_context(data, context, script_name);
            
            // Execute the script using tokio::task::block_on
            let script_result = tokio::task::block_on(self.execute_validation_script(script, script_context));

            // Parse the script output for validation errors
            if let Some(script_errors) = self.parse_script_validation_output(&script_result) {
                errors.extend(script_errors);
            }
        }
        
        Ok(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    fn create_test_validation_context() -> ValidationContext {
        ValidationContext {
            method: "tools/call".to_string(),
            request_id: Some(json!({"id": "test-123"})),
            server_capabilities: Some(json!({"tools": true})),
            test_metadata: HashMap::new(),
        }
    }

    fn create_test_validation_script(name: &str, phase: Option<&str>) -> ValidationScript {
        ValidationScript {
            name: name.to_string(),
            language: "lua".to_string(),
            execution_phase: phase.map(|p| p.to_string()),
            required: Some(true),
            source: Some("-- test script\nreturn {success = true}".to_string()),
        }
    }

    // ========================================================================
    // PHASE 1: ScriptValidator Creation Tests (RED - Should FAIL)
    // ========================================================================

    #[test]
    fn test_script_validator_creation_with_before_phase() {
        let scripts = vec![create_test_validation_script("test_script", Some("before"))];
        let config = ScriptValidationConfig::default();
        
        let result = ScriptValidator::new(scripts, ScriptExecutionPhase::Before, config);
        
        // This should pass once implemented
        assert!(result.is_ok(), "ScriptValidator creation should succeed with valid scripts");
        
        // Verify the validator actually contains the scripts and can function
        let validator = result.unwrap();
        assert_eq!(validator.name(), "script_validator_before", "Validator name should reflect before phase");
        assert_eq!(validator.execution_phase, ScriptExecutionPhase::Before, "Execution phase should be Before");
        assert_eq!(validator.validation_scripts.len(), 1, "Should have 1 validation script");
        
        // Verify the script has the expected properties
        let script = &validator.validation_scripts[0];
        assert_eq!(script.name, "test_script", "Script name should match");
        assert_eq!(script.execution_phase, Some("before".to_string()), "Script should have before phase");
        assert!(script.source.is_some(), "Script should have source code");
    }

    #[test]
    fn test_script_validator_creation_with_after_phase() {
        let scripts = vec![create_test_validation_script("test_script", Some("after"))];
        let config = ScriptValidationConfig::default();
        
        let result = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config);
        
        // This should pass once implemented
        assert!(result.is_ok(), "ScriptValidator creation should succeed for after phase");
        let validator = result.unwrap();
        assert_eq!(validator.name(), "script_validator_after", "Validator name should reflect after phase");
        assert_eq!(validator.execution_phase, ScriptExecutionPhase::After, "Execution phase should be After");
        
        // Verify the validator has functional scripts
        assert!(!validator.validation_scripts.is_empty(), "Validator should have scripts");
        let script = &validator.validation_scripts[0];
        assert_eq!(script.name, "test_script", "Script name should be preserved");
        assert!(script.source.is_some(), "Script should have source code");
    }

    #[test]
    fn test_script_validator_creation_with_multiple_scripts() {
        let scripts = vec![
            create_test_validation_script("script1", Some("after")),
            create_test_validation_script("script2", Some("after")),
            create_test_validation_script("script3", None),
        ];
        let config = ScriptValidationConfig::default();
        
        let result = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config);
        
        assert!(result.is_ok());
        let validator = result.unwrap();
        assert_eq!(validator.validation_scripts.len(), 3);
    }

    #[test]
    fn test_script_validator_creation_with_custom_config() {
        let scripts = vec![create_test_validation_script("test_script", Some("before"))];
        let config = ScriptValidationConfig {
            timeout_seconds: 60,
            memory_limit_mb: 128,
            fail_on_script_error: true,
            capture_script_logs: false,
        };
        
        let result = ScriptValidator::new(scripts, ScriptExecutionPhase::Before, config.clone());
        
        assert!(result.is_ok());
        let validator = result.unwrap();
        // Note: config is private, so we can't test it directly in this implementation
        // In a real scenario, we might add getter methods or make config public
    }

    // ========================================================================
    // PHASE 2: Script Execution Phase Logic Tests (RED - Should FAIL)
    // ========================================================================

    #[test]
    fn test_should_execute_script_before_phase_match() {
        let scripts = vec![create_test_validation_script("before_script", Some("before"))];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::Before, config).unwrap();
        
        let script = create_test_validation_script("before_script", Some("before"));
        assert!(validator.should_execute_script(&script));
    }

    #[test]
    fn test_should_execute_script_after_phase_match() {
        let scripts = vec![create_test_validation_script("after_script", Some("after"))];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap();
        
        let script = create_test_validation_script("after_script", Some("after"));
        assert!(validator.should_execute_script(&script));
    }

    #[test]
    fn test_should_execute_script_phase_mismatch() {
        let scripts = vec![create_test_validation_script("before_script", Some("before"))];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap();
        
        let script = create_test_validation_script("before_script", Some("before"));
        assert!(!validator.should_execute_script(&script));
    }

    #[test]
    fn test_should_execute_script_default_to_after_phase() {
        let scripts = vec![create_test_validation_script("default_script", None)];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap();
        
        let script = create_test_validation_script("default_script", None);
        assert!(validator.should_execute_script(&script));
    }

    // ========================================================================
    // PHASE 3: Context Creation Tests (RED - Should FAIL)
    // ========================================================================

    #[tokio::test]
    async fn test_create_script_context_with_full_validation_context() {
        let scripts = vec![create_test_validation_script("test_script", Some("after"))];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap();
        
        let response = json!({"result": {"content": [{"text": "test response"}]}});
        let mut validation_context = create_test_validation_context();
        validation_context.test_metadata.insert("test_case_name".to_string(), "test_case_1".to_string());
        
        let script_context = validator.create_script_context(&response, &validation_context, "test_script");
        
        assert_eq!(script_context.request, validation_context.request_id);
        assert_eq!(script_context.response, Some(response));
        assert_eq!(script_context.server_info, validation_context.server_capabilities);
        assert_eq!(script_context.metadata.get("script_name"), Some(&"test_script".to_string()));
        assert_eq!(script_context.metadata.get("validation_method"), Some(&"tools/call".to_string()));
        assert_eq!(script_context.metadata.get("test_test_case_name"), Some(&"test_case_1".to_string()));
    }

    #[tokio::test]
    async fn test_create_script_context_with_minimal_validation_context() {
        let scripts = vec![create_test_validation_script("test_script", Some("after"))];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap();
        
        let response = json!(null);
        let validation_context = ValidationContext {
            method: "test_method".to_string(),
            request_id: None,
            server_capabilities: None,
            test_metadata: HashMap::new(),
        };
        
        let script_context = validator.create_script_context(&response, &validation_context, "test_script");
        
        assert_eq!(script_context.request, None);
        assert_eq!(script_context.response, Some(response));
        assert_eq!(script_context.server_info, None);
        assert_eq!(script_context.metadata.get("script_name"), Some(&"test_script".to_string()));
        assert_eq!(script_context.metadata.get("validation_method"), Some(&"test_method".to_string()));
    }

    // ========================================================================
    // PHASE 4: Script Execution Tests (RED - Should FAIL)
    // ========================================================================

    #[tokio::test]
    async fn test_execute_validation_script_success() {
        let scripts = vec![create_test_validation_script("success_script", Some("after"))];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap();
        
        let script = ValidationScript {
            name: "success_script".to_string(),
            language: "lua".to_string(),
            execution_phase: Some("after".to_string()),
            required: Some(true),
            source: Some("return {success = true, message = 'validation passed'}".to_string()),
        };
        
        let context = ScriptContext {
            request: Some(json!({"id": "test"})),
            response: Some(json!({"result": "success"})),
            metadata: HashMap::new(),
            server_info: None,
        };
        
        let result = validator.execute_validation_script(&script, context).await;
        
        assert!(result.is_ok());
        let script_result = result.unwrap();
        assert!(script_result.success);
        assert!(script_result.output.get("message").is_some());
    }

    #[tokio::test]
    async fn test_execute_validation_script_with_missing_source() {
        let scripts = vec![create_test_validation_script("no_source_script", Some("after"))];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap();
        
        let script = ValidationScript {
            name: "no_source_script".to_string(),
            language: "lua".to_string(),
            execution_phase: Some("after".to_string()),
            required: Some(true),
            source: None, // Missing source
        };
        
        let context = ScriptContext {
            request: None,
            response: None,
            metadata: HashMap::new(),
            server_info: None,
        };
        
        let result = validator.execute_validation_script(&script, context).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ScriptError::ConfigurationError { .. }));
    }

    // ========================================================================
    // PHASE 5: Script Output Parsing Tests (RED - Should FAIL)
    // ========================================================================

    #[test]
    fn test_parse_script_validation_output_with_errors() {
        let scripts = vec![create_test_validation_script("parser_script", Some("after"))];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap();
        
        let script_result = ScriptResult {
            success: false,
            output: json!({
                "validation_errors": [
                    {
                        "field": "$.result.value",
                        "expected": "42",
                        "actual": "24",
                        "message": "Value mismatch"
                    }
                ],
                "warnings": [],
                "metadata": {}
            }),
            logs: vec![],
            duration_ms: 100,
            memory_used_mb: None,
            error: None,
        };
        
        let parsed_errors = validator.parse_script_validation_output(&Ok(script_result));
        
        assert!(parsed_errors.is_some());
        let errors = parsed_errors.unwrap();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], ValidationError::FieldError { ref field, .. } if field == "$.result.value"));
    }

    #[test]
    fn test_parse_script_validation_output_invalid_format() {
        let scripts = vec![create_test_validation_script("parser_script", Some("after"))];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap();
        
        let script_result = ScriptResult {
            success: true,
            output: json!("invalid output format"),
            logs: vec![],
            duration_ms: 100,
            memory_used_mb: None,
            error: None,
        };
        
        let parsed_errors = validator.parse_script_validation_output(&Ok(script_result));
        
        assert!(parsed_errors.is_none());
    }

    // ========================================================================
    // PHASE 6: CustomValidator Implementation Tests (RED - Should FAIL)
    // ========================================================================

    #[tokio::test]
    async fn test_custom_validator_validate_success() {
        let script = ValidationScript {
            name: "success_validator".to_string(),
            language: "lua".to_string(),
            execution_phase: Some("after".to_string()),
            required: Some(false),
            source: Some("return {success = true}".to_string()),
        };
        
        let scripts = vec![script];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap();
        
        let response = json!({"result": {"content": [{"text": "test"}]}});
        let validation_context = create_test_validation_context();
        
        let result = validator.validate(&response, &validation_context);
        
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(errors.is_empty()); // Should be empty for successful script
    }

    #[tokio::test]
    async fn test_custom_validator_validate_script_failure_required() {
        let script = ValidationScript {
            name: "failing_validator".to_string(),
            language: "lua".to_string(),
            execution_phase: Some("after".to_string()),
            required: Some(true), // Required script
            source: Some("error('validation failed')".to_string()),
        };
        
        let scripts = vec![script];
        let config = ScriptValidationConfig {
            fail_on_script_error: true,
            ..ScriptValidationConfig::default()
        };
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap();
        
        let response = json!({"result": {"content": [{"text": "test"}]}});
        let validation_context = create_test_validation_context();
        
        let result = validator.validate(&response, &validation_context);
        
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(!errors.is_empty()); // Should contain errors for failed required script
        assert!(errors.iter().any(|e| matches!(e, ValidationError::FieldError { field, .. } if field.starts_with("script:"))));
    }

    #[tokio::test]
    async fn test_custom_validator_validate_script_failure_optional() {
        let script = ValidationScript {
            name: "failing_optional_validator".to_string(),
            language: "lua".to_string(),
            execution_phase: Some("after".to_string()),
            required: Some(false), // Optional script
            source: Some("error('validation failed')".to_string()),
        };
        
        let scripts = vec![script];
        let config = ScriptValidationConfig {
            fail_on_script_error: false, // Don't fail on script errors
            ..ScriptValidationConfig::default()
        };
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap();
        
        let response = json!({"result": {"content": [{"text": "test"}]}});
        let validation_context = create_test_validation_context();
        
        let result = validator.validate(&response, &validation_context);
        
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(errors.is_empty()); // Should be empty for optional failing script with fail_on_script_error=false
    }

    #[tokio::test]
    async fn test_custom_validator_validate_wrong_execution_phase() {
        let script = ValidationScript {
            name: "before_script".to_string(),
            language: "lua".to_string(),
            execution_phase: Some("before".to_string()), // Before phase script
            required: Some(true),
            source: Some("return {success = true}".to_string()),
        };
        
        let scripts = vec![script];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap(); // After phase validator
        
        let response = json!({"result": {"content": [{"text": "test"}]}});
        let validation_context = create_test_validation_context();
        
        let result = validator.validate(&response, &validation_context);
        
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(errors.is_empty()); // Should be empty because script shouldn't execute in wrong phase
    }

    #[tokio::test]
    async fn test_custom_validator_validate_with_script_generated_errors() {
        let script = ValidationScript {
            name: "error_generating_validator".to_string(),
            language: "lua".to_string(),
            execution_phase: Some("after".to_string()),
            required: Some(false),
            source: Some(r#"
                return {
                    success = false,
                    validation_errors = {
                        {
                            field = "$.result.code",
                            expected = "200",
                            actual = "404",
                            message = "Status code mismatch"
                        }
                    },
                    warnings = {},
                    metadata = {}
                }
            "#.to_string()),
        };
        
        let scripts = vec![script];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap();
        
        let response = json!({"result": {"code": 404}});
        let validation_context = create_test_validation_context();
        
        let result = validator.validate(&response, &validation_context);
        
        assert!(result.is_ok());
        let errors = result.unwrap();
        assert!(!errors.is_empty()); // Should contain script-generated validation errors
        assert!(errors.iter().any(|e| matches!(e, ValidationError::FieldError { field, .. } if field == "$.result.code")));
    }

    // ========================================================================
    // PHASE 7: Configuration Tests (RED - Should FAIL)
    // ========================================================================

    #[test]
    fn test_script_validation_config_default() {
        let config = ScriptValidationConfig::default();
        
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.memory_limit_mb, 64);
        assert!(!config.fail_on_script_error);
        assert!(config.capture_script_logs);
    }

    #[test]
    fn test_script_execution_phase_equality() {
        // Test creating different phase instances and comparing them
        let before_phase1 = ScriptExecutionPhase::Before;
        let before_phase2 = ScriptExecutionPhase::Before;
        let after_phase = ScriptExecutionPhase::After;
        
        // Test equality between same phases
        assert_eq!(before_phase1, before_phase2, "Two Before phases should be equal");
        assert_eq!(after_phase, ScriptExecutionPhase::After, "After phase should equal itself");
        
        // Test inequality between different phases
        assert_ne!(before_phase1, after_phase, "Before and After phases should not be equal");
        
        // Test Debug trait implementation
        let before_debug = format!("{:?}", before_phase1);
        assert!(before_debug.contains("Before"), "Debug output should contain phase name: {}", before_debug);
        
        let after_debug = format!("{:?}", after_phase);
        assert!(after_debug.contains("After"), "Debug output should contain phase name: {}", after_debug);
    }

    // ========================================================================
    // PHASE 8: Integration Readiness Tests (RED - Should FAIL)
    // ========================================================================

    #[test]
    fn test_validator_name_before_phase() {
        let scripts = vec![create_test_validation_script("test", Some("before"))];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::Before, config).unwrap();
        
        assert_eq!(validator.name(), "script_validator_before");
    }

    #[test]
    fn test_validator_name_after_phase() {
        let scripts = vec![create_test_validation_script("test", Some("after"))];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap();
        
        assert_eq!(validator.name(), "script_validator_after");
    }

    #[tokio::test]
    async fn test_validator_is_send_sync() {
        // Test that ScriptValidator can be used in async contexts and sent between threads
        let scripts = vec![create_test_validation_script("test", Some("after"))];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap();
        
        // This ensures ScriptValidator implements Send + Sync
        let _: Box<dyn CustomValidator> = Box::new(validator);
    }
} 