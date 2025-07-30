//! Script Manager for validation script management and execution coordination
//!
//! This module provides ScriptManager that handles loading, filtering, and coordinating
//! validation scripts for test case execution.

use crate::spec::{TestCase, ValidationScript};
use crate::validation::{ScriptExecutionPhase, ScriptValidationConfig, ScriptValidator};
use std::collections::HashMap;
use std::sync::Arc;

/// Error types for script management operations
#[derive(Debug, thiserror::Error)]
pub enum ScriptManagerError {
    #[error("Script not found: {name}")]
    ScriptNotFound { name: String },
    #[error("Script validation error: {0}")]
    ValidationError(String),
    #[error("Script configuration error: {0}")]
    ConfigurationError(String),
}

/// Manages validation scripts and their associated validators
#[derive(Debug)]
pub struct ScriptManager {
    /// Available scripts by name
    pub available_scripts: HashMap<String, ValidationScript>,
    /// Cached script validators for performance
    #[allow(dead_code)]
    script_validators: HashMap<String, Arc<ScriptValidator>>,
}

impl ScriptManager {
    /// Create a new script manager with the given scripts
    pub fn new(scripts: Vec<ValidationScript>) -> Result<Self, ScriptManagerError> {
        let available_scripts: HashMap<String, ValidationScript> = scripts
            .into_iter()
            .map(|script| (script.name.clone(), script))
            .collect();

        Ok(Self {
            available_scripts,
            script_validators: HashMap::new(),
        })
    }

    /// Get scripts referenced by a test case
    pub fn get_scripts_for_test_case(&self, test_case: &TestCase) -> Vec<&ValidationScript> {
        if let Some(script_refs) = &test_case.validation_scripts {
            script_refs
                .iter()
                .filter_map(|script_name| self.available_scripts.get(script_name))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Create validators for scripts in a specific execution phase
    pub fn create_validators_for_phase(
        &self,
        scripts: &[&ValidationScript],
        phase: ScriptExecutionPhase,
    ) -> Result<Vec<Arc<ScriptValidator>>, ScriptManagerError> {
        let mut validators = Vec::new();

        for script in scripts {
            // Filter scripts by phase
            let script_phase = match &script.execution_phase {
                crate::spec::ExecutionPhase::Before => Some("before"),
                crate::spec::ExecutionPhase::After => Some("after"),
                crate::spec::ExecutionPhase::Both => Some("both"),
            };
            let matches_phase = match (script_phase, &phase) {
                (Some("before"), ScriptExecutionPhase::Before) => true,
                (Some("after"), ScriptExecutionPhase::After) => true,
                (None, ScriptExecutionPhase::After) => true, // Default to "after"
                _ => false,
            };

            if matches_phase {
                // Create or get cached validator
                let validator = self.get_or_create_validator(script, phase.clone())?;
                validators.push(validator);
            }
        }

        Ok(validators)
    }

    /// Get or create a validator for a script (with caching)
    fn get_or_create_validator(
        &self,
        script: &ValidationScript,
        phase: ScriptExecutionPhase,
    ) -> Result<Arc<ScriptValidator>, ScriptManagerError> {
        let _cache_key = format!("{}_{:?}", script.name, phase);

        let config = ScriptValidationConfig {
            timeout_seconds: 30,
            memory_limit_mb: 64,
            fail_on_script_error: script.required,
            capture_script_logs: true,
        };

        let validator = ScriptValidator::new(vec![script.clone()], phase, config).map_err(|e| {
            ScriptManagerError::ValidationError(format!("Failed to create validator: {e}"))
        })?;

        Ok(Arc::new(validator))
    }

    /// Check if a script is required
    pub fn is_script_required(&self, script_name: &str) -> bool {
        self.available_scripts
            .get(script_name)
            .map(|script| script.required)
            .unwrap_or(false)
    }

    /// Get all script names
    pub fn get_script_names(&self) -> Vec<&String> {
        self.available_scripts.keys().collect()
    }

    /// Get script count
    pub fn script_count(&self) -> usize {
        self.available_scripts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_script(name: &str, phase: Option<&str>, required: bool) -> ValidationScript {
        ValidationScript {
            name: name.to_string(),
            language: crate::spec::ScriptLanguage::Lua,
            execution_phase: phase.map_or(crate::spec::ExecutionPhase::After, |p| match p {
                "before" => crate::spec::ExecutionPhase::Before,
                "both" => crate::spec::ExecutionPhase::Both,
                _ => crate::spec::ExecutionPhase::After,
            }),
            required,
            source: format!("-- Test script: {name}"),
            timeout_ms: None,
        }
    }

    #[test]
    fn test_script_manager_creation() {
        let scripts = vec![
            create_test_script("script1", Some("before"), true),
            create_test_script("script2", Some("after"), false),
        ];

        let manager = ScriptManager::new(scripts).unwrap();
        assert_eq!(manager.script_count(), 2);
        assert!(manager.available_scripts.contains_key("script1"));
        assert!(manager.available_scripts.contains_key("script2"));
    }

    #[test]
    fn test_get_scripts_for_test_case() {
        let scripts = vec![
            create_test_script("script1", Some("before"), true),
            create_test_script("script2", Some("after"), false),
            create_test_script("script3", Some("after"), true),
        ];

        let manager = ScriptManager::new(scripts).unwrap();

        let test_case = TestCase {
            name: "test".to_string(),
            validation_scripts: Some(vec!["script1".to_string(), "script3".to_string()]),
            ..Default::default()
        };

        let matching_scripts = manager.get_scripts_for_test_case(&test_case);
        assert_eq!(matching_scripts.len(), 2, "Should have 2 items");

        let script_names: Vec<&str> = matching_scripts.iter().map(|s| s.name.as_str()).collect();
        assert!(script_names.contains(&"script1"));
        assert!(script_names.contains(&"script3"));
        assert!(!script_names.contains(&"script2"));
    }

    #[test]
    fn test_script_required_check() {
        let scripts = vec![
            create_test_script("required_script", Some("after"), true),
            create_test_script("optional_script", Some("after"), false),
        ];

        let manager = ScriptManager::new(scripts).unwrap();

        assert!(manager.is_script_required("required_script"));
        assert!(!manager.is_script_required("optional_script"));
        assert!(!manager.is_script_required("nonexistent_script"));
    }
}
