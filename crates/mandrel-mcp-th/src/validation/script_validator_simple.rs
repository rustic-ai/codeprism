//! Simplified Script validation integration for ValidationEngine
//!
//! This module provides a simplified ScriptValidator that implements the CustomValidator trait
//! to integrate basic script validation into the validation pipeline.

use crate::script_engines::{LuaEngine, ScriptConfig, ScriptContext};
use crate::spec::ValidationScript;
use crate::validation::{CustomValidator, ValidationContext, ValidationError};
// Note: serde traits may be needed for future serialization
use serde_json::Value;
use std::collections::HashMap;

/// Script execution phases for validation
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ScriptExecutionPhase {
    Before, // Execute before standard validation
    After,  // Execute after standard validation
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

/// Simplified ScriptValidator implementing CustomValidator trait
#[derive(Debug)]
pub struct ScriptValidator {
    validation_scripts: HashMap<String, ValidationScript>,
    execution_phase: ScriptExecutionPhase,
    config: ScriptValidationConfig,
}

impl ScriptValidator {
    /// Create a new ScriptValidator with the given scripts and configuration
    pub fn new(
        scripts: Vec<ValidationScript>,
        phase: ScriptExecutionPhase,
        config: ScriptValidationConfig,
    ) -> Result<Self, crate::validation::ValidationEngineError> {
        let validation_scripts: HashMap<String, ValidationScript> = scripts
            .into_iter()
            .map(|script| (script.name.clone(), script))
            .collect();

        Ok(Self {
            validation_scripts,
            execution_phase: phase,
            config,
        })
    }

    fn should_execute_script(&self, script: &ValidationScript) -> bool {
        matches!(
            (&script.execution_phase, &self.execution_phase),
            (
                crate::spec::ExecutionPhase::Before,
                ScriptExecutionPhase::Before
            ) | (
                crate::spec::ExecutionPhase::After,
                ScriptExecutionPhase::After
            ) | (crate::spec::ExecutionPhase::Both, _)
        )
    }

    /// Execute a script using the appropriate engine
    fn execute_script(
        &self,
        script: &ValidationScript,
        data: &Value,
        context: &ValidationContext,
        script_name: &str,
    ) -> Result<crate::script_engines::ScriptResult, Box<dyn std::error::Error>> {
        // Create script context with validation data
        let script_context = self.create_script_context(script, data, context, script_name);

        // Execute script based on language (currently only Lua is supported)
        match script.language {
            crate::spec::ScriptLanguage::Lua => {
                // Move all data needed for script execution
                let timeout_ms = script_context.config.timeout_ms;
                let script_source = script.source.clone();
                let script_request = script_context.request.clone();
                let script_response = script_context.response.clone();
                let script_tool_name = script_context.metadata.tool_name.clone();
                let script_test_name = script_context.metadata.test_name.clone();

                // Execute script in a separate thread to avoid runtime conflicts
                std::thread::spawn(
                    move || -> Result<crate::script_engines::ScriptResult, String> {
                        // Create everything fresh inside the thread to avoid Send/Sync issues
                        let script_config = ScriptConfig {
                            timeout_ms,
                            ..ScriptConfig::default()
                        };

                        let lua_engine = LuaEngine::new(&script_config)
                            .map_err(|e| format!("Failed to create LuaEngine: {}", e))?;

                        // Recreate script context inside thread
                        let fresh_script_context = ScriptContext::new(
                            script_request,
                            script_test_name,
                            script_tool_name,
                            script_config,
                        )
                        .with_response(script_response.unwrap_or(serde_json::Value::Null));

                        // Create new runtime and execute script
                        let rt = tokio::runtime::Runtime::new()
                            .map_err(|e| format!("Failed to create runtime: {}", e))?;

                        rt.block_on(async {
                            lua_engine
                                .execute_script(&script_source, fresh_script_context)
                                .await
                        })
                        .map_err(|e| format!("Script execution failed: {}", e))
                    },
                )
                .join()
                .map_err(|_| "Script execution thread panicked")?
                .map_err(|e| e.into())
            }
            _ => Err(format!("Script language '{:?}' not supported", script.language).into()),
        }
    }

    /// Create script context with validation data
    fn create_script_context(
        &self,
        script: &ValidationScript,
        data: &Value,
        context: &ValidationContext,
        script_name: &str,
    ) -> ScriptContext {
        // Use script-specific timeout if specified, otherwise use config timeout
        let timeout_ms = script
            .timeout_ms
            .unwrap_or((self.config.timeout_seconds * 1000) as u64);

        let script_config = ScriptConfig {
            timeout_ms,
            ..ScriptConfig::default()
        };

        // Create script context with request/response data and metadata
        let mut script_data = serde_json::Map::new();
        script_data.insert("response".to_string(), data.clone());
        script_data.insert("method".to_string(), Value::String(context.method.clone()));
        script_data.insert(
            "request_id".to_string(),
            context.request_id.clone().unwrap_or(Value::Null),
        );

        ScriptContext::new(
            Value::Object(script_data),
            script_name.to_string(),
            context.method.clone(),
            script_config,
        )
    }

    /// Parse validation errors from script output
    fn parse_script_validation_output(
        &self,
        result: &crate::script_engines::ScriptResult,
    ) -> Result<Option<Vec<ValidationError>>, Box<dyn std::error::Error>> {
        // If script returned structured validation output, parse it
        if let Ok(output) =
            serde_json::from_value::<serde_json::Map<String, Value>>(result.output.clone())
        {
            if let Some(validation_errors) = output.get("validation_errors") {
                if let Ok(errors) = serde_json::from_value::<Vec<serde_json::Map<String, Value>>>(
                    validation_errors.clone(),
                ) {
                    let mut parsed_errors = Vec::new();

                    for error in errors {
                        if let (Some(field), Some(expected), Some(actual)) = (
                            error.get("field").and_then(|v| v.as_str()),
                            error.get("expected").and_then(|v| v.as_str()),
                            error.get("actual").and_then(|v| v.as_str()),
                        ) {
                            parsed_errors.push(ValidationError::FieldError {
                                field: field.to_string(),
                                expected: expected.to_string(),
                                actual: actual.to_string(),
                            });
                        }
                    }

                    if !parsed_errors.is_empty() {
                        return Ok(Some(parsed_errors));
                    }
                }
            }
        }

        Ok(None)
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
        let mut errors = Vec::new();

        for (script_name, script) in &self.validation_scripts {
            // Check if script should execute in this phase
            if !self.should_execute_script(script) {
                continue;
            }

            // Execute script with real LuaEngine instead of simulation
            match self.execute_script(script, data, context, script_name) {
                Ok(script_result) => {
                    // Process script execution result
                    if !script_result.success
                        && (script.required || self.config.fail_on_script_error)
                    {
                        let error_message = script_result
                            .error
                            .as_ref()
                            .map(|e| format!("{}", e))
                            .unwrap_or_else(|| "script execution failed".to_string());

                        errors.push(ValidationError::FieldError {
                            field: format!("script:{}", script_name),
                            expected: "script execution success".to_string(),
                            actual: error_message,
                        });
                    }

                    // Parse validation errors from script output if script returned validation results
                    if let Some(validation_errors) =
                        self.parse_script_validation_output(&script_result)?
                    {
                        errors.extend(validation_errors);
                    }
                }
                Err(e) => {
                    // Handle script execution errors
                    if script.required || self.config.fail_on_script_error {
                        errors.push(ValidationError::FieldError {
                            field: format!("script:{}", script_name),
                            expected: "script execution success".to_string(),
                            actual: format!("execution error: {}", e),
                        });
                    }
                }
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
            language: crate::spec::ScriptLanguage::Lua,
            execution_phase: phase.map_or(crate::spec::ExecutionPhase::After, |p| match p {
                "before" => crate::spec::ExecutionPhase::Before,
                "both" => crate::spec::ExecutionPhase::Both,
                _ => crate::spec::ExecutionPhase::After,
            }),
            required: true,
            source: "-- test script\nreturn {success = true}".to_string(),
            timeout_ms: None,
        }
    }

    #[test]
    fn test_script_validator_creation_with_before_phase() {
        let scripts = vec![create_test_validation_script("test_script", Some("before"))];
        let config = ScriptValidationConfig::default();

        let result = ScriptValidator::new(scripts, ScriptExecutionPhase::Before, config);

        assert!(result.is_ok());
        let validator = result.unwrap();
        assert_eq!(validator.name(), "script_validator_before");
        assert_eq!(validator.execution_phase, ScriptExecutionPhase::Before);
        assert_eq!(validator.validation_scripts.len(), 1);
    }

    #[test]
    fn test_script_validator_creation_with_after_phase() {
        let scripts = vec![create_test_validation_script("test_script", Some("after"))];
        let config = ScriptValidationConfig::default();

        let result = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config);

        assert!(result.is_ok());
        let validator = result.unwrap();
        assert_eq!(validator.name(), "script_validator_after");
        assert_eq!(validator.execution_phase, ScriptExecutionPhase::After);
    }

    #[test]
    fn test_should_execute_script_before_phase_match() {
        let scripts = vec![create_test_validation_script(
            "before_script",
            Some("before"),
        )];
        let config = ScriptValidationConfig::default();
        let validator =
            ScriptValidator::new(scripts, ScriptExecutionPhase::Before, config).unwrap();

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
        let scripts = vec![create_test_validation_script(
            "before_script",
            Some("before"),
        )];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap();

        let script = create_test_validation_script("before_script", Some("before"));
        assert!(!validator.should_execute_script(&script));
    }

    #[test]
    fn test_custom_validator_validate_success() {
        let script = ValidationScript {
            name: "success_validator".to_string(),
            language: crate::spec::ScriptLanguage::Lua,
            execution_phase: crate::spec::ExecutionPhase::After,
            required: false,
            source: "return {success = true}".to_string(),
            timeout_ms: None,
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

    #[test]
    fn test_custom_validator_validate_script_failure_required() {
        let script = ValidationScript {
            name: "failing_validator".to_string(),
            language: crate::spec::ScriptLanguage::Lua,
            execution_phase: crate::spec::ExecutionPhase::After,
            required: true, // Required script
            source: "error('validation failed')".to_string(),
            timeout_ms: None,
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

    #[test]
    fn test_custom_validator_validate_script_failure_optional() {
        let script = ValidationScript {
            name: "failing_optional_validator".to_string(),
            language: crate::spec::ScriptLanguage::Lua,
            execution_phase: crate::spec::ExecutionPhase::After,
            required: false, // Optional script
            source: "error('validation failed')".to_string(),
            timeout_ms: None,
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

    #[test]
    fn test_custom_validator_validate_wrong_execution_phase() {
        let script = ValidationScript {
            name: "before_script".to_string(),
            language: crate::spec::ScriptLanguage::Lua,
            execution_phase: crate::spec::ExecutionPhase::Before, // Before phase script
            required: true,
            source: "return {success = true}".to_string(),
            timeout_ms: None,
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
        assert_eq!(ScriptExecutionPhase::Before, ScriptExecutionPhase::Before);
        assert_eq!(ScriptExecutionPhase::After, ScriptExecutionPhase::After);
        assert_ne!(ScriptExecutionPhase::Before, ScriptExecutionPhase::After);
    }

    #[test]
    fn test_validator_name_before_phase() {
        let scripts = vec![create_test_validation_script("test", Some("before"))];
        let config = ScriptValidationConfig::default();
        let validator =
            ScriptValidator::new(scripts, ScriptExecutionPhase::Before, config).unwrap();

        assert_eq!(validator.name(), "script_validator_before");
    }

    #[test]
    fn test_validator_name_after_phase() {
        let scripts = vec![create_test_validation_script("test", Some("after"))];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap();

        assert_eq!(validator.name(), "script_validator_after");
    }

    #[test]
    fn test_validator_is_send_sync() {
        // Test that ScriptValidator can be used in async contexts and sent between threads
        let scripts = vec![create_test_validation_script("test", Some("after"))];
        let config = ScriptValidationConfig::default();
        let validator = ScriptValidator::new(scripts, ScriptExecutionPhase::After, config).unwrap();

        // This ensures ScriptValidator implements Send + Sync
        let _: Box<dyn CustomValidator> = Box::new(validator);
    }
}
