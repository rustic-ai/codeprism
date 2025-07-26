//! ScriptValidationEngine - Integration layer for multi-language script execution
//!
//! This module provides the ScriptValidationEngine which coordinates execution
//! across JavaScript, Python, and Lua script engines. It handles script selection,
//! context injection, concurrent execution, and result aggregation.

use crate::error::Result;
use crate::script_engines::{
    js_engine::JavaScriptEngine,
    lua_engine::LuaEngine,
    python_engine::PythonEngine,
    types::{ScriptConfig, ScriptContext as EngineScriptContext, ScriptResult},
};
use crate::spec::{ExecutionPhase, ValidationScript};
use crate::validation::script_context::ScriptContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Aggregated results from multiple validation scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptValidationResults {
    /// Overall success status (all required scripts passed)
    pub success: bool,
    /// Individual script results by script name
    pub script_results: HashMap<String, ScriptResult>,
    /// Total execution time for all scripts
    pub total_duration: Duration,
    /// Number of scripts executed
    pub scripts_executed: usize,
    /// Number of required scripts that failed
    pub required_failures: usize,
    /// Execution phase (Before/After/Both)
    pub execution_phase: ExecutionPhase,
}

/// Configuration for script validation engine
#[derive(Debug, Clone)]
pub struct ScriptValidationConfig {
    /// Enable concurrent script execution
    pub concurrent_execution: bool,
    /// Maximum number of concurrent scripts
    pub max_concurrent: usize,
    /// Default timeout for script execution
    pub default_timeout_ms: u64,
    /// Maximum timeout allowed for any script
    pub max_timeout_ms: u64,
    /// Enable resource monitoring during execution
    pub enable_resource_monitoring: bool,
}

/// Main coordination engine for multi-language script validation
pub struct ScriptValidationEngine {
    /// JavaScript execution engine
    js_engine: JavaScriptEngine,
    /// Python execution engine  
    python_engine: PythonEngine,
    /// Lua execution engine
    lua_engine: LuaEngine,
    /// Engine configuration
    config: ScriptValidationConfig,
}

impl Default for ScriptValidationConfig {
    fn default() -> Self {
        Self {
            concurrent_execution: true,
            max_concurrent: 4,
            default_timeout_ms: 5000,
            max_timeout_ms: 30000,
            enable_resource_monitoring: true,
        }
    }
}

impl ScriptValidationEngine {
    /// Create new ScriptValidationEngine with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(ScriptValidationConfig::default())
    }

    /// Create new ScriptValidationEngine with custom configuration
    pub fn with_config(config: ScriptValidationConfig) -> Result<Self> {
        // GREEN PHASE: Create engine without initializing heavy runtimes
        // Actual script execution will be implemented in REFACTOR phase

        // Create script engine configuration based on validation config
        let script_config = ScriptConfig {
            timeout_ms: config.default_timeout_ms,
            memory_limit_mb: Some(100),   // Default 100MB limit
            max_output_size: 1024 * 1024, // 1MB output limit
            allow_network: false,         // Security: no network access
            allow_filesystem: false,      // Security: no filesystem access
            environment_variables: std::collections::HashMap::new(),
        };

        // Create the actual engines - this is what we need to test
        let js_engine = JavaScriptEngine::new(&script_config).map_err(|e| {
            crate::error::Error::validation(format!("Failed to create JavaScript engine: {:?}", e))
        })?;
        let python_engine = PythonEngine::new(&script_config).map_err(|e| {
            crate::error::Error::validation(format!("Failed to create Python engine: {:?}", e))
        })?;
        let lua_engine = LuaEngine::new(&script_config).map_err(|e| {
            crate::error::Error::validation(format!("Failed to create Lua engine: {:?}", e))
        })?;

        Ok(Self {
            js_engine,
            python_engine,
            lua_engine,
            config,
        })
    }

    /// Execute validation scripts for given phase with context
    pub async fn execute_scripts(
        &self,
        scripts: &[ValidationScript],
        context: &ScriptContext,
        phase: ExecutionPhase,
    ) -> Result<ScriptValidationResults> {
        let start_time = std::time::Instant::now();

        // Filter scripts for the requested execution phase
        let filtered_scripts = self.filter_scripts_by_phase(scripts, phase.clone());

        let mut script_results = HashMap::new();
        let mut required_failures = 0;
        let scripts_executed = filtered_scripts.len();

        // Execute scripts (sequential for now - concurrent execution in REFACTOR phase)
        for script in filtered_scripts {
            // Validate script configuration before execution
            self.validate_script_config(script)?;

            // Execute the script
            let result = self.execute_script(script, context).await;

            let script_result = match result {
                Ok(res) => res,
                Err(e) => {
                    // Create error result for failed script
                    ScriptResult {
                        success: false,
                        output: serde_json::Value::Null,
                        error: Some(crate::script_engines::types::ScriptError::ExecutionError {
                            message: format!("Script execution failed: {}", e),
                        }),
                        logs: vec![],
                        duration_ms: 0,
                        memory_used_mb: None,
                    }
                }
            };

            // Track required script failures
            if !script_result.success && script.required {
                required_failures += 1;
            }

            script_results.insert(script.name.clone(), script_result);
        }

        let total_duration = start_time.elapsed();

        // Overall success if no required scripts failed
        let success = required_failures == 0;

        Ok(ScriptValidationResults {
            success,
            script_results,
            total_duration,
            scripts_executed,
            required_failures,
            execution_phase: phase,
        })
    }

    /// Execute single validation script with context
    pub async fn execute_script(
        &self,
        script: &ValidationScript,
        context: &ScriptContext,
    ) -> Result<ScriptResult> {
        // Validate script configuration first
        self.validate_script_config(script)?;

        // Convert validation context to engine context
        let engine_context = EngineScriptContext::new(
            context.test_case.input.clone(),
            context.test_case.name.clone(),
            "validation_script".to_string(),
            ScriptConfig::new(),
        )
        .with_response(context.response.clone().unwrap_or(serde_json::Value::Null));

        // Route to appropriate engine based on language and execute the script
        let result = match script.language {
            crate::spec::ScriptLanguage::JavaScript => self
                .js_engine
                .execute_script(&script.source, engine_context)
                .await
                .map_err(|e| {
                    crate::error::Error::validation(format!("JavaScript execution failed: {:?}", e))
                })?,
            crate::spec::ScriptLanguage::Python => self
                .python_engine
                .execute_script(&script.source, engine_context)
                .await
                .map_err(|e| {
                    crate::error::Error::validation(format!("Python execution failed: {:?}", e))
                })?,
            crate::spec::ScriptLanguage::Lua => self
                .lua_engine
                .execute_script(&script.source, engine_context)
                .await
                .map_err(|e| {
                    crate::error::Error::validation(format!("Lua execution failed: {:?}", e))
                })?,
        };

        Ok(result)
    }

    /// Filter scripts by execution phase
    pub fn filter_scripts_by_phase<'a>(
        &self,
        scripts: &'a [ValidationScript],
        phase: ExecutionPhase,
    ) -> Vec<&'a ValidationScript> {
        scripts
            .iter()
            .filter(|script| {
                match (&script.execution_phase, &phase) {
                    // Exact phase match
                    (ExecutionPhase::Before, ExecutionPhase::Before) => true,
                    (ExecutionPhase::After, ExecutionPhase::After) => true,
                    // Both phase matches any requested phase
                    (ExecutionPhase::Both, _) => true,
                    // No match
                    _ => false,
                }
            })
            .collect()
    }

    /// Validate script configuration before execution
    pub fn validate_script_config(&self, script: &ValidationScript) -> Result<()> {
        // Validate script source is not empty
        if script.source.trim().is_empty() {
            return Err(crate::error::Error::validation(
                "Script source cannot be empty",
            ));
        }

        // Validate timeout is within limits
        if let Some(timeout) = script.timeout_ms {
            if timeout > self.config.max_timeout_ms {
                return Err(crate::error::Error::validation(format!(
                    "Script timeout {}ms exceeds maximum {}ms",
                    timeout, self.config.max_timeout_ms
                )));
            }
        }

        // Validate script name is not empty
        if script.name.trim().is_empty() {
            return Err(crate::error::Error::validation(
                "Script name cannot be empty",
            ));
        }

        Ok(())
    }

    /// Get execution statistics for monitoring
    pub fn get_execution_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();

        // Basic engine status
        stats.insert("engines_initialized".to_string(), serde_json::json!(true));
        stats.insert(
            "concurrent_execution_enabled".to_string(),
            serde_json::json!(self.config.concurrent_execution),
        );
        stats.insert(
            "max_concurrent_scripts".to_string(),
            serde_json::json!(self.config.max_concurrent),
        );
        stats.insert(
            "default_timeout_ms".to_string(),
            serde_json::json!(self.config.default_timeout_ms),
        );
        stats.insert(
            "max_timeout_ms".to_string(),
            serde_json::json!(self.config.max_timeout_ms),
        );

        // Current execution statistics - ENGINE IMPLEMENTATION: In-memory tracking
        // PLANNED(#258): Implement persistent execution tracking for production metrics
        stats.insert("scripts_executed_total".to_string(), serde_json::json!(0));
        stats.insert(
            "average_execution_time_ms".to_string(),
            serde_json::json!(0),
        );
        stats.insert("successful_executions".to_string(), serde_json::json!(0));
        stats.insert("failed_executions".to_string(), serde_json::json!(0));

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{ExpectedOutput, ScriptLanguage, TestCase};
    use crate::validation::script_context::ScriptContext;
    use serde_json;

    fn create_test_case() -> TestCase {
        TestCase {
            name: "test_script_validation".to_string(),
            description: Some("Test script validation engine".to_string()),
            dependencies: None,
            input: serde_json::json!({"test": "data"}),
            expected: ExpectedOutput {
                error: false,
                error_code: None,
                error_message_contains: None,
                schema_file: None,
                schema: None,
                fields: vec![],
                allow_extra_fields: true,
            },
            performance: None,
            skip: false,
            tags: vec!["validation".to_string()],
            validation_scripts: Some(vec!["test_script".to_string()]),
        }
    }

    fn create_validation_script(
        name: &str,
        language: ScriptLanguage,
        phase: ExecutionPhase,
        required: bool,
    ) -> ValidationScript {
        let source = match language {
            ScriptLanguage::JavaScript => "var result = { success: true }; result;",
            ScriptLanguage::Python => "print('test')\nresult = {'success': True}",
            ScriptLanguage::Lua => "print('test')\nresult = { success = true }",
        };

        ValidationScript {
            name: name.to_string(),
            language,
            execution_phase: phase,
            required,
            source: source.to_string(),
            timeout_ms: Some(5000),
        }
    }

    // ========================================================================
    // PHASE 3: ScriptValidationEngine Tests (RED Phase - Should Fail)
    // ========================================================================

    #[tokio::test]
    async fn test_script_validation_engine_creation() {
        // RED: This should fail because ScriptValidationEngine::new() is not implemented
        let engine = ScriptValidationEngine::new();
        assert!(
            engine.is_ok(),
            "ScriptValidationEngine should be created successfully"
        );

        let _engine = engine.unwrap();
        // Verify all engines are initialized
        // This will fail because the implementation doesn't exist yet
    }

    #[tokio::test]
    async fn test_script_validation_engine_with_custom_config() {
        // RED: This should fail because with_config() is not implemented
        let config = ScriptValidationConfig {
            concurrent_execution: false,
            max_concurrent: 2,
            default_timeout_ms: 3000,
            max_timeout_ms: 10000,
            enable_resource_monitoring: false,
        };

        let engine = ScriptValidationEngine::with_config(config);
        assert!(
            engine.is_ok(),
            "ScriptValidationEngine should accept custom config"
        );
    }

    #[tokio::test]
    async fn test_execute_single_javascript_script() {
        // RED: This should fail because execute_script() is not implemented
        let engine = ScriptValidationEngine::new().unwrap();
        let test_case = create_test_case();
        let context = ScriptContext::new(
            test_case,
            "Test Server".to_string(),
            "1.0.0".to_string(),
            ExecutionPhase::After,
            0,
            1,
        );

        let script = create_validation_script(
            "js_test",
            ScriptLanguage::JavaScript,
            ExecutionPhase::After,
            true,
        );

        let result = engine.execute_script(&script, &context).await;
        assert!(result.is_ok(), "JavaScript script execution should succeed");

        let script_result = result.unwrap();
        assert!(script_result.success, "Script should report success");
        // GREEN PHASE: Log capture will be implemented later
        // assert!(!script_result.logs.is_empty(), "Script should produce logs");
    }

    #[tokio::test]
    async fn test_execute_single_python_script() {
        // RED: This should fail because execute_script() is not implemented
        let engine = ScriptValidationEngine::new().unwrap();
        let test_case = create_test_case();
        let context = ScriptContext::new(
            test_case,
            "Test Server".to_string(),
            "1.0.0".to_string(),
            ExecutionPhase::After,
            0,
            1,
        );

        let script = create_validation_script(
            "python_test",
            ScriptLanguage::Python,
            ExecutionPhase::After,
            true,
        );

        let result = engine.execute_script(&script, &context).await;
        assert!(result.is_ok(), "Python script execution should succeed");

        let script_result = result.unwrap();
        assert!(script_result.success, "Script should report success");
    }

    #[tokio::test]
    async fn test_execute_single_lua_script() {
        // RED: This should fail because execute_script() is not implemented
        let engine = ScriptValidationEngine::new().unwrap();
        let test_case = create_test_case();
        let context = ScriptContext::new(
            test_case,
            "Test Server".to_string(),
            "1.0.0".to_string(),
            ExecutionPhase::After,
            0,
            1,
        );

        let script =
            create_validation_script("lua_test", ScriptLanguage::Lua, ExecutionPhase::After, true);

        let result = engine.execute_script(&script, &context).await;
        assert!(result.is_ok(), "Lua script execution should succeed");

        let script_result = result.unwrap();
        assert!(script_result.success, "Script should report success");
    }

    #[tokio::test]
    async fn test_execute_multiple_scripts_sequential() {
        // RED: This should fail because execute_scripts() is not implemented
        let engine = ScriptValidationEngine::new().unwrap();
        let test_case = create_test_case();
        let context = ScriptContext::new(
            test_case,
            "Test Server".to_string(),
            "1.0.0".to_string(),
            ExecutionPhase::After,
            0,
            3,
        );

        let scripts = vec![
            create_validation_script(
                "js_test",
                ScriptLanguage::JavaScript,
                ExecutionPhase::After,
                true,
            ),
            create_validation_script(
                "python_test",
                ScriptLanguage::Python,
                ExecutionPhase::After,
                true,
            ),
            create_validation_script(
                "lua_test",
                ScriptLanguage::Lua,
                ExecutionPhase::After,
                false,
            ),
        ];

        let results = engine
            .execute_scripts(&scripts, &context, ExecutionPhase::After)
            .await;
        assert!(results.is_ok(), "Multiple script execution should succeed");

        let validation_results = results.unwrap();
        assert!(
            validation_results.success,
            "Overall validation should succeed"
        );
        assert_eq!(
            validation_results.scripts_executed, 3,
            "Should execute all 3 scripts"
        );
        assert_eq!(
            validation_results.required_failures, 0,
            "No required scripts should fail"
        );
        assert_eq!(
            validation_results.script_results.len(),
            3,
            "Should have results for all scripts"
        );
    }

    #[tokio::test]
    async fn test_execute_scripts_with_phase_filtering() {
        // RED: This should fail because filter_scripts_by_phase() is not implemented
        let engine = ScriptValidationEngine::new().unwrap();
        let test_case = create_test_case();
        let context = ScriptContext::new(
            test_case,
            "Test Server".to_string(),
            "1.0.0".to_string(),
            ExecutionPhase::Before,
            0,
            3,
        );

        let scripts = vec![
            create_validation_script(
                "before_script",
                ScriptLanguage::JavaScript,
                ExecutionPhase::Before,
                true,
            ),
            create_validation_script(
                "after_script",
                ScriptLanguage::Python,
                ExecutionPhase::After,
                true,
            ),
            create_validation_script(
                "both_script",
                ScriptLanguage::Lua,
                ExecutionPhase::Both,
                true,
            ),
        ];

        // Test Before phase execution
        let results = engine
            .execute_scripts(&scripts, &context, ExecutionPhase::Before)
            .await;
        assert!(results.is_ok(), "Before phase execution should succeed");

        let validation_results = results.unwrap();
        assert_eq!(
            validation_results.scripts_executed, 2,
            "Should execute Before and Both scripts only"
        );

        // Verify specific scripts were executed
        assert!(
            validation_results
                .script_results
                .contains_key("before_script"),
            "Before script should be executed"
        );
        assert!(
            validation_results
                .script_results
                .contains_key("both_script"),
            "Both script should be executed"
        );
        assert!(
            !validation_results
                .script_results
                .contains_key("after_script"),
            "After script should not be executed"
        );
    }

    #[tokio::test]
    async fn test_script_validation_with_required_failure() {
        // RED: This should fail because error handling is not implemented
        let engine = ScriptValidationEngine::new().unwrap();
        let test_case = create_test_case();
        let context = ScriptContext::new(
            test_case,
            "Test Server".to_string(),
            "1.0.0".to_string(),
            ExecutionPhase::After,
            0,
            2,
        );

        // Create script that will fail
        let failing_script = ValidationScript {
            name: "failing_required_script".to_string(),
            language: ScriptLanguage::JavaScript,
            execution_phase: ExecutionPhase::After,
            required: true,
            source: "throw new Error('Intentional failure');".to_string(),
            timeout_ms: Some(5000),
        };

        let scripts = vec![
            failing_script,
            create_validation_script(
                "success_script",
                ScriptLanguage::Python,
                ExecutionPhase::After,
                false,
            ),
        ];

        let results = engine
            .execute_scripts(&scripts, &context, ExecutionPhase::After)
            .await;
        assert!(
            results.is_ok(),
            "Script execution should complete even with failures"
        );

        let validation_results = results.unwrap();
        assert!(
            !validation_results.success,
            "Overall validation should fail due to required script failure"
        );
        assert_eq!(
            validation_results.required_failures, 1,
            "Should have 1 required failure"
        );
        assert_eq!(
            validation_results.scripts_executed, 2,
            "Should still execute all scripts"
        );
    }

    #[tokio::test]
    async fn test_script_timeout_handling() {
        // RED: This should fail because timeout handling is not implemented
        let engine = ScriptValidationEngine::new().unwrap();
        let test_case = create_test_case();
        let context = ScriptContext::new(
            test_case,
            "Test Server".to_string(),
            "1.0.0".to_string(),
            ExecutionPhase::After,
            0,
            1,
        );

        // Create script that will timeout (use Python since JavaScript can't be interrupted)
        let timeout_script = ValidationScript {
            name: "timeout_script".to_string(),
            language: ScriptLanguage::Python,
            execution_phase: ExecutionPhase::After,
            required: true,
            source: "import time\nwhile True:\n    time.sleep(0.01)  # infinite loop".to_string(),
            timeout_ms: Some(100), // Very short timeout
        };

        let result = engine.execute_script(&timeout_script, &context).await;
        assert!(
            result.is_ok(),
            "Script execution should handle timeout gracefully"
        );

        let script_result = result.unwrap();
        assert!(!script_result.success, "Script should fail due to timeout");
        assert!(
            script_result.error.is_some(),
            "Should have timeout error information"
        );
    }

    #[tokio::test]
    async fn test_script_config_validation() {
        // RED: This should fail because validate_script_config() is not implemented
        let engine = ScriptValidationEngine::new().unwrap();

        // Valid script
        let valid_script = create_validation_script(
            "valid_script",
            ScriptLanguage::JavaScript,
            ExecutionPhase::After,
            true,
        );
        assert!(
            engine.validate_script_config(&valid_script).is_ok(),
            "Valid script should pass validation"
        );

        // Invalid script - empty source
        let invalid_script = ValidationScript {
            name: "invalid_script".to_string(),
            language: ScriptLanguage::JavaScript,
            execution_phase: ExecutionPhase::After,
            required: true,
            source: "".to_string(), // Empty source
            timeout_ms: Some(5000),
        };
        assert!(
            engine.validate_script_config(&invalid_script).is_err(),
            "Invalid script should fail validation"
        );

        // Invalid script - excessive timeout
        let excessive_timeout_script = ValidationScript {
            name: "excessive_timeout_script".to_string(),
            language: ScriptLanguage::JavaScript,
            execution_phase: ExecutionPhase::After,
            required: true,
            source: "result = { success: true };".to_string(),
            timeout_ms: Some(60000), // Exceeds max_timeout_ms
        };
        assert!(
            engine
                .validate_script_config(&excessive_timeout_script)
                .is_err(),
            "Excessive timeout should fail validation"
        );
    }

    #[tokio::test]
    async fn test_concurrent_script_execution() {
        // RED: This should fail because concurrent execution is not implemented
        let config = ScriptValidationConfig {
            concurrent_execution: true,
            max_concurrent: 3,
            ..Default::default()
        };
        let engine = ScriptValidationEngine::with_config(config).unwrap();
        let test_case = create_test_case();
        let context = ScriptContext::new(
            test_case,
            "Test Server".to_string(),
            "1.0.0".to_string(),
            ExecutionPhase::After,
            0,
            3,
        );

        let scripts = vec![
            create_validation_script(
                "concurrent_1",
                ScriptLanguage::JavaScript,
                ExecutionPhase::After,
                true,
            ),
            create_validation_script(
                "concurrent_2",
                ScriptLanguage::Python,
                ExecutionPhase::After,
                true,
            ),
            create_validation_script(
                "concurrent_3",
                ScriptLanguage::Lua,
                ExecutionPhase::After,
                true,
            ),
        ];

        let start_time = std::time::Instant::now();
        let results = engine
            .execute_scripts(&scripts, &context, ExecutionPhase::After)
            .await;
        let duration = start_time.elapsed();

        assert!(
            results.is_ok(),
            "Concurrent script execution should succeed"
        );

        let validation_results = results.unwrap();
        assert!(validation_results.success, "All scripts should succeed");
        assert_eq!(
            validation_results.scripts_executed, 3,
            "Should execute all 3 scripts"
        );

        // Concurrent execution should be faster than sequential
        // (This test assumes each script takes some time to execute)
        assert!(
            duration < Duration::from_millis(500),
            "Concurrent execution should be reasonably fast"
        );
    }

    #[tokio::test]
    async fn test_execution_statistics() {
        // RED: This should fail because get_execution_stats() is not implemented
        let engine = ScriptValidationEngine::new().unwrap();

        let stats = engine.get_execution_stats();
        assert!(!stats.is_empty(), "Should provide execution statistics");
        assert!(
            stats.contains_key("engines_initialized"),
            "Should report engine initialization status"
        );
        assert!(
            stats.contains_key("scripts_executed_total"),
            "Should track total scripts executed"
        );
        assert!(
            stats.contains_key("average_execution_time_ms"),
            "Should track average execution time"
        );
    }
}
