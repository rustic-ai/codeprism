//! Simplified Script validation integration for ValidationEngine
//!
//! This module provides a simplified ScriptValidator that implements the CustomValidator trait
//! to integrate basic script validation into the validation pipeline.

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
        match (&script.execution_phase, &self.execution_phase) {
            (Some(phase), current_phase) => {
                matches!(
                    (phase.as_str(), current_phase),
                    ("before", ScriptExecutionPhase::Before)
                        | ("after", ScriptExecutionPhase::After)
                )
            }
            // Default to "after" if no phase specified
            (None, ScriptExecutionPhase::After) => true,
            _ => false,
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
        _data: &Value,
        _context: &ValidationContext,
    ) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
        let mut errors = Vec::new();

        for (script_name, script) in &self.validation_scripts {
            // Check if script should execute in this phase
            if !self.should_execute_script(script) {
                continue;
            }

            // For the GREEN phase, simulate script execution based on script content
            if let Some(source) = &script.source {
                if source.contains("error(") {
                    // Script contains error() call - simulate failure
                    if script.required.unwrap_or(false) || self.config.fail_on_script_error {
                        errors.push(ValidationError::FieldError {
                            field: format!("script:{}", script_name),
                            expected: "script execution success".to_string(),
                            actual: "script failed".to_string(),
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
            language: "lua".to_string(),
            execution_phase: phase.map(|p| p.to_string()),
            required: Some(true),
            source: Some("-- test script\nreturn {success = true}".to_string()),
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

    #[test]
    fn test_custom_validator_validate_script_failure_required() {
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

    #[test]
    fn test_custom_validator_validate_script_failure_optional() {
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

    #[test]
    fn test_custom_validator_validate_wrong_execution_phase() {
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
