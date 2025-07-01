//! Validation engine for MCP test responses and configurations
//!
//! Provides comprehensive validation of test responses against expected patterns.

use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;
use tracing::debug;

use crate::config::{ExpectedResponse, TestCase, ValidationConfig, ValidationPattern};
use crate::runner::ValidationResult;

/// Test response validator
#[derive(Debug, Clone)]
pub struct TestValidator {
    custom_validators: HashMap<String, CustomValidator>,
}

#[derive(Debug, Clone)]
pub struct CustomValidator {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub validator_fn: fn(&Value) -> Result<ValidationResult>,
}

impl TestValidator {
    /// Create new test validator
    pub fn new() -> Self {
        let mut validator = Self {
            custom_validators: HashMap::new(),
        };

        // Register built-in validators
        validator.register_builtin_validators();
        validator
    }

    /// Validate test case configuration
    pub fn validate_test_case(&self, test_case: &TestCase) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        // Basic test case validation
        if test_case.id.is_empty() {
            results.push(ValidationResult {
                rule_name: "test_id_required".to_string(),
                passed: false,
                message: "Test case ID cannot be empty".to_string(),
                score: Some(0.0),
            });
        } else {
            results.push(ValidationResult {
                rule_name: "test_id_required".to_string(),
                passed: true,
                message: "Test case ID is valid".to_string(),
                score: Some(1.0),
            });
        }

        if test_case.tool_name.is_empty() {
            results.push(ValidationResult {
                rule_name: "tool_name_required".to_string(),
                passed: false,
                message: "Tool name cannot be empty".to_string(),
                score: Some(0.0),
            });
        } else {
            results.push(ValidationResult {
                rule_name: "tool_name_required".to_string(),
                passed: true,
                message: "Tool name is valid".to_string(),
                score: Some(1.0),
            });
        }

        // Validate input parameters if present
        if let Some(ref params) = test_case.input_params {
            let param_result = self.validate_json_structure(params, "input_params");
            results.push(param_result);
        }

        // Validate expected response configuration
        if let Some(ref expected) = test_case.expected {
            let expected_results = self.validate_expected_response_config(expected);
            results.extend(expected_results);
        }

        results
    }

    /// Validate response against expected patterns
    pub fn validate_response(
        &self,
        test_case: &TestCase,
        response: &Value,
    ) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        let expected = match &test_case.expected {
            Some(expected) => expected,
            None => {
                // No validation patterns specified - just check basic response structure
                return vec![ValidationResult {
                    rule_name: "basic_response_check".to_string(),
                    passed: !response.is_null(),
                    message: if response.is_null() {
                        "Response is null".to_string()
                    } else {
                        "Response received".to_string()
                    },
                    score: Some(if response.is_null() { 0.0 } else { 1.0 }),
                }];
            }
        };

        // Check if empty results are allowed
        if let Some(allow_empty) = expected.allow_empty_results {
            if !allow_empty && self.is_empty_response(response) {
                results.push(ValidationResult {
                    rule_name: "empty_results_check".to_string(),
                    passed: false,
                    message: "Empty results not allowed for this test".to_string(),
                    score: Some(0.0),
                });
                return results;
            }
        }

        // Validate each pattern
        if let Some(ref patterns) = expected.patterns {
            for pattern in patterns {
                let pattern_result = self.validate_pattern(response, pattern);
                results.push(pattern_result);
            }
        }

        // Run custom validation scripts if present
        if let Some(ref custom_scripts) = test_case.custom_scripts {
            for script in custom_scripts {
                let script_result = self.run_custom_validation(response, script);
                results.push(script_result);
            }
        }

        results
    }

    fn register_builtin_validators(&mut self) {
        // Register common MCP protocol validators
        self.custom_validators.insert(
            "mcp_initialize_response".to_string(),
            CustomValidator {
                name: "mcp_initialize_response".to_string(),
                validator_fn: validate_mcp_initialize_response,
            },
        );

        self.custom_validators.insert(
            "mcp_tools_list_response".to_string(),
            CustomValidator {
                name: "mcp_tools_list_response".to_string(),
                validator_fn: validate_mcp_tools_response,
            },
        );

        self.custom_validators.insert(
            "mcp_resources_list_response".to_string(),
            CustomValidator {
                name: "mcp_resources_list_response".to_string(),
                validator_fn: validate_mcp_resources_response,
            },
        );
    }

    fn validate_expected_response_config(
        &self,
        expected: &ExpectedResponse,
    ) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        if let Some(ref patterns) = expected.patterns {
            if patterns.is_empty() {
                results.push(ValidationResult {
                    rule_name: "patterns_not_empty".to_string(),
                    passed: false,
                    message: "Expected response patterns list is empty".to_string(),
                    score: Some(0.0),
                });
            } else {
                results.push(ValidationResult {
                    rule_name: "patterns_not_empty".to_string(),
                    passed: true,
                    message: format!("Found {} validation patterns", patterns.len()),
                    score: Some(1.0),
                });
            }
        }

        results
    }

    fn validate_pattern(&self, response: &Value, pattern: &ValidationPattern) -> ValidationResult {
        debug!(
            "Validating pattern: {} with key: {}",
            serde_json::to_string(&pattern.validation).unwrap_or_default(),
            pattern.key
        );

        // Extract value using JSON path
        let value = self.extract_json_path_value(response, &pattern.key);

        match &pattern.validation {
            ValidationConfig::Exists => {
                let exists = value.is_some();
                ValidationResult {
                    rule_name: format!("exists_{}", pattern.key),
                    passed: exists,
                    message: if exists {
                        format!("Key '{}' exists in response", pattern.key)
                    } else {
                        format!("Key '{}' missing from response", pattern.key)
                    },
                    score: Some(if exists { 1.0 } else { 0.0 }),
                }
            }
            ValidationConfig::Equals {
                value: expected_value,
            } => match value {
                Some(actual_value) => {
                    let matches = actual_value == expected_value;
                    ValidationResult {
                        rule_name: format!("equals_{}", pattern.key),
                        passed: matches,
                        message: if matches {
                            format!("Key '{}' has expected value", pattern.key)
                        } else {
                            format!(
                                "Key '{}' value mismatch. Expected: {}, Got: {}",
                                pattern.key, expected_value, actual_value
                            )
                        },
                        score: Some(if matches { 1.0 } else { 0.0 }),
                    }
                }
                None => ValidationResult {
                    rule_name: format!("equals_{}", pattern.key),
                    passed: false,
                    message: format!("Key '{}' not found for equality check", pattern.key),
                    score: Some(0.0),
                },
            },
            ValidationConfig::Range { min, max } => match value {
                Some(val) => {
                    if let Some(num) = val.as_f64() {
                        let in_range = num >= *min && num <= *max;
                        ValidationResult {
                            rule_name: format!("range_{}", pattern.key),
                            passed: in_range,
                            message: if in_range {
                                format!(
                                    "Key '{}' value {} is within range [{}, {}]",
                                    pattern.key, num, min, max
                                )
                            } else {
                                format!(
                                    "Key '{}' value {} is outside range [{}, {}]",
                                    pattern.key, num, min, max
                                )
                            },
                            score: Some(if in_range { 1.0 } else { 0.0 }),
                        }
                    } else {
                        ValidationResult {
                            rule_name: format!("range_{}", pattern.key),
                            passed: false,
                            message: format!("Key '{}' is not a number", pattern.key),
                            score: Some(0.0),
                        }
                    }
                }
                None => ValidationResult {
                    rule_name: format!("range_{}", pattern.key),
                    passed: false,
                    message: format!("Key '{}' not found for range check", pattern.key),
                    score: Some(0.0),
                },
            },
            ValidationConfig::Array => match value {
                Some(val) => {
                    let is_array = val.is_array();
                    ValidationResult {
                        rule_name: format!("array_{}", pattern.key),
                        passed: is_array,
                        message: if is_array {
                            format!(
                                "Key '{}' is an array with {} elements",
                                pattern.key,
                                val.as_array().unwrap().len()
                            )
                        } else {
                            format!("Key '{}' is not an array", pattern.key)
                        },
                        score: Some(if is_array { 1.0 } else { 0.0 }),
                    }
                }
                None => ValidationResult {
                    rule_name: format!("array_{}", pattern.key),
                    passed: false,
                    message: format!("Key '{}' not found for array check", pattern.key),
                    score: Some(0.0),
                },
            },
            ValidationConfig::ArrayMinLength { min_length } => match value {
                Some(val) => {
                    if let Some(array) = val.as_array() {
                        let meets_min = array.len() >= *min_length;
                        ValidationResult {
                            rule_name: format!("array_min_length_{}", pattern.key),
                            passed: meets_min,
                            message: if meets_min {
                                format!(
                                    "Array '{}' has {} elements (>= {} required)",
                                    pattern.key,
                                    array.len(),
                                    min_length
                                )
                            } else {
                                format!(
                                    "Array '{}' has {} elements (< {} required)",
                                    pattern.key,
                                    array.len(),
                                    min_length
                                )
                            },
                            score: Some(if meets_min { 1.0 } else { 0.0 }),
                        }
                    } else {
                        ValidationResult {
                            rule_name: format!("array_min_length_{}", pattern.key),
                            passed: false,
                            message: format!("Key '{}' is not an array", pattern.key),
                            score: Some(0.0),
                        }
                    }
                }
                None => ValidationResult {
                    rule_name: format!("array_min_length_{}", pattern.key),
                    passed: false,
                    message: format!("Key '{}' not found for array length check", pattern.key),
                    score: Some(0.0),
                },
            },
            ValidationConfig::ArrayMaxLength { max_length } => match value {
                Some(val) => {
                    if let Some(array) = val.as_array() {
                        let meets_max = array.len() <= *max_length;
                        ValidationResult {
                            rule_name: format!("array_max_length_{}", pattern.key),
                            passed: meets_max,
                            message: if meets_max {
                                format!(
                                    "Array '{}' has {} elements (<= {} allowed)",
                                    pattern.key,
                                    array.len(),
                                    max_length
                                )
                            } else {
                                format!(
                                    "Array '{}' has {} elements (> {} allowed)",
                                    pattern.key,
                                    array.len(),
                                    max_length
                                )
                            },
                            score: Some(if meets_max { 1.0 } else { 0.0 }),
                        }
                    } else {
                        ValidationResult {
                            rule_name: format!("array_max_length_{}", pattern.key),
                            passed: false,
                            message: format!("Key '{}' is not an array", pattern.key),
                            score: Some(0.0),
                        }
                    }
                }
                None => ValidationResult {
                    rule_name: format!("array_max_length_{}", pattern.key),
                    passed: false,
                    message: format!("Key '{}' not found for array length check", pattern.key),
                    score: Some(0.0),
                },
            },
            ValidationConfig::Boolean {
                value: expected_value,
            } => {
                match value {
                    Some(val) => {
                        if let Some(bool_val) = val.as_bool() {
                            let matches = match expected_value {
                                Some(expected) => bool_val == *expected,
                                None => true, // Any boolean value is acceptable
                            };
                            ValidationResult {
                                rule_name: format!("boolean_{}", pattern.key),
                                passed: matches,
                                message: if matches {
                                    format!(
                                        "Boolean key '{}' has expected value: {}",
                                        pattern.key, bool_val
                                    )
                                } else {
                                    format!(
                                        "Boolean key '{}' value mismatch. Expected: {:?}, Got: {}",
                                        pattern.key, expected_value, bool_val
                                    )
                                },
                                score: Some(if matches { 1.0 } else { 0.0 }),
                            }
                        } else {
                            ValidationResult {
                                rule_name: format!("boolean_{}", pattern.key),
                                passed: false,
                                message: format!("Key '{}' is not a boolean", pattern.key),
                                score: Some(0.0),
                            }
                        }
                    }
                    None => ValidationResult {
                        rule_name: format!("boolean_{}", pattern.key),
                        passed: false,
                        message: format!("Key '{}' not found for boolean check", pattern.key),
                        score: Some(0.0),
                    },
                }
            }
        }
    }

    fn extract_json_path_value<'a>(&self, response: &'a Value, path: &str) -> Option<&'a Value> {
        // Simple JSON path extraction (supports dot notation)
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = response;

        for part in parts {
            match current {
                Value::Object(obj) => {
                    current = obj.get(part)?;
                }
                Value::Array(arr) => {
                    // Handle array indices
                    if let Ok(index) = part.parse::<usize>() {
                        current = arr.get(index)?;
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        Some(current)
    }

    fn validate_json_structure(&self, value: &Value, context: &str) -> ValidationResult {
        match value {
            Value::Object(_) => ValidationResult {
                rule_name: format!("json_structure_{}", context),
                passed: true,
                message: format!("Valid JSON object structure for {}", context),
                score: Some(1.0),
            },
            Value::Array(_) => ValidationResult {
                rule_name: format!("json_structure_{}", context),
                passed: true,
                message: format!("Valid JSON array structure for {}", context),
                score: Some(1.0),
            },
            _ => ValidationResult {
                rule_name: format!("json_structure_{}", context),
                passed: true,
                message: format!("Valid JSON value for {}", context),
                score: Some(1.0),
            },
        }
    }

    fn is_empty_response(&self, response: &Value) -> bool {
        match response {
            Value::Null => true,
            Value::Object(obj) => obj.is_empty(),
            Value::Array(arr) => arr.is_empty(),
            Value::String(s) => s.is_empty(),
            _ => false,
        }
    }

    fn run_custom_validation(
        &self,
        response: &Value,
        script: &crate::config::CustomScript,
    ) -> ValidationResult {
        // Real custom script validation using the MCP Test Harness script engine
        use mcp_test_harness_lib::script::{
            ScriptConfig, ScriptContext, ScriptExecutorFactory, ScriptLanguage,
        };
        use mcp_test_harness_lib::spec::{schema::ExpectedOutput, TestCase};

        // Convert script configuration to ScriptConfig format
        let script_language = match script.language.as_str() {
            "javascript" => ScriptLanguage::JavaScript,
            "python" => ScriptLanguage::Python,
            "lua" => ScriptLanguage::Lua,
            _ => {
                return ValidationResult {
                    rule_name: format!("custom_validation_{}", script.name),
                    passed: false,
                    message: format!("Unsupported script language: {}", script.language),
                    score: Some(0.0),
                };
            }
        };

        let script_config = ScriptConfig {
            language: script_language,
            source: script.script.clone(),
            name: script.name.clone(),
            timeout_ms: 30_000,        // 30 second timeout
            max_memory_mb: 256,        // 256 MB memory limit
            allow_network: false,      // Secure by default
            allowed_paths: Vec::new(), // No file system access for security
            env_vars: std::collections::HashMap::new(),
            args: Vec::new(), // No additional arguments
        };

        // Create script context with test response data
        let test_case = TestCase {
            name: format!("custom_validation_{}", script.name),
            description: Some("Custom validation script".to_string()),
            input: serde_json::Value::Null,
            expected: ExpectedOutput::default(),
            ..Default::default()
        };

        let context = ScriptContext::new(
            test_case,
            serde_json::json!({}),  // Empty request
            Some(response.clone()), // Test response to validate
            None,                   // No error
        );

        // Execute script using secure sandbox
        let executor = ScriptExecutorFactory::create_secure();

        // Use tokio runtime for async execution in sync context
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                return ValidationResult {
                    rule_name: format!("custom_validation_{}", script.name),
                    passed: false,
                    message: format!("Failed to create runtime for script execution: {}", e),
                    score: Some(0.0),
                };
            }
        };

        match rt.block_on(executor.execute_script(&script_config, &context)) {
            Ok(result) => ValidationResult {
                rule_name: format!("custom_validation_{}", script.name),
                passed: result.success,
                message: if result.success {
                    format!(
                        "Custom {} validation '{}' passed in {}ms",
                        script.language, script.name, result.duration_ms
                    )
                } else {
                    result
                        .error
                        .unwrap_or_else(|| "Script validation failed".to_string())
                },
                score: Some(if result.success { 1.0 } else { 0.0 }),
            },
            Err(e) => ValidationResult {
                rule_name: format!("custom_validation_{}", script.name),
                passed: false,
                message: format!("Script execution failed: {}", e),
                score: Some(0.0),
            },
        }
    }
}

impl Default for TestValidator {
    fn default() -> Self {
        Self::new()
    }
}

// Built-in validator functions
fn validate_mcp_initialize_response(response: &Value) -> Result<ValidationResult> {
    let required_fields = ["protocolVersion", "capabilities", "serverInfo"];
    let mut missing_fields = Vec::new();

    for field in &required_fields {
        if response.get(field).is_none() {
            missing_fields.push(*field);
        }
    }

    if missing_fields.is_empty() {
        Ok(ValidationResult {
            rule_name: "mcp_initialize_response".to_string(),
            passed: true,
            message: "MCP initialize response contains all required fields".to_string(),
            score: Some(1.0),
        })
    } else {
        Ok(ValidationResult {
            rule_name: "mcp_initialize_response".to_string(),
            passed: false,
            message: format!(
                "MCP initialize response missing fields: {}",
                missing_fields.join(", ")
            ),
            score: Some(0.0),
        })
    }
}

fn validate_mcp_tools_response(response: &Value) -> Result<ValidationResult> {
    if let Some(tools) = response.get("tools") {
        if let Some(tools_array) = tools.as_array() {
            let mut tool_validation_score = 1.0;
            let mut messages = Vec::new();

            for (i, tool) in tools_array.iter().enumerate() {
                let tool_obj = tool
                    .as_object()
                    .ok_or_else(|| anyhow!("Tool {} is not an object", i))?;

                if !tool_obj.contains_key("name") {
                    tool_validation_score *= 0.5;
                    messages.push(format!("Tool {} missing 'name' field", i));
                }

                if !tool_obj.contains_key("description") {
                    tool_validation_score *= 0.8;
                    messages.push(format!("Tool {} missing 'description' field", i));
                }
            }

            Ok(ValidationResult {
                rule_name: "mcp_tools_response".to_string(),
                passed: tool_validation_score > 0.7,
                message: if messages.is_empty() {
                    format!("All {} tools are properly formatted", tools_array.len())
                } else {
                    messages.join("; ")
                },
                score: Some(tool_validation_score),
            })
        } else {
            Ok(ValidationResult {
                rule_name: "mcp_tools_response".to_string(),
                passed: false,
                message: "'tools' field is not an array".to_string(),
                score: Some(0.0),
            })
        }
    } else {
        Ok(ValidationResult {
            rule_name: "mcp_tools_response".to_string(),
            passed: false,
            message: "Response missing 'tools' field".to_string(),
            score: Some(0.0),
        })
    }
}

fn validate_mcp_resources_response(response: &Value) -> Result<ValidationResult> {
    if let Some(resources) = response.get("resources") {
        if let Some(resources_array) = resources.as_array() {
            let mut resource_validation_score = 1.0;
            let mut messages = Vec::new();

            for (i, resource) in resources_array.iter().enumerate() {
                let resource_obj = resource
                    .as_object()
                    .ok_or_else(|| anyhow!("Resource {} is not an object", i))?;

                if !resource_obj.contains_key("uri") {
                    resource_validation_score *= 0.3;
                    messages.push(format!("Resource {} missing 'uri' field", i));
                }

                if !resource_obj.contains_key("name") {
                    resource_validation_score *= 0.8;
                    messages.push(format!("Resource {} missing 'name' field", i));
                }
            }

            Ok(ValidationResult {
                rule_name: "mcp_resources_response".to_string(),
                passed: resource_validation_score > 0.7,
                message: if messages.is_empty() {
                    format!(
                        "All {} resources are properly formatted",
                        resources_array.len()
                    )
                } else {
                    messages.join("; ")
                },
                score: Some(resource_validation_score),
            })
        } else {
            Ok(ValidationResult {
                rule_name: "mcp_resources_response".to_string(),
                passed: false,
                message: "'resources' field is not an array".to_string(),
                score: Some(0.0),
            })
        }
    } else {
        Ok(ValidationResult {
            rule_name: "mcp_resources_response".to_string(),
            passed: false,
            message: "Response missing 'resources' field".to_string(),
            score: Some(0.0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validator_creation() {
        let validator = TestValidator::new();
        assert!(validator.custom_validators.len() > 0);
    }

    #[test]
    fn test_json_path_extraction() {
        let validator = TestValidator::new();
        let response = json!({
            "data": {
                "items": [
                    {"name": "test1"},
                    {"name": "test2"}
                ]
            }
        });

        assert_eq!(
            validator.extract_json_path_value(&response, "data.items.0.name"),
            Some(&json!("test1"))
        );

        assert_eq!(
            validator.extract_json_path_value(&response, "data.missing"),
            None
        );
    }

    #[test]
    fn test_validation_patterns() {
        let validator = TestValidator::new();
        let response = json!({
            "status": "ok",
            "count": 5,
            "items": ["a", "b", "c"],
            "enabled": true
        });

        // Test exists validation
        let exists_pattern = ValidationPattern {
            key: "status".to_string(),
            validation: ValidationConfig::Exists,
            required: true,
        };
        let result = validator.validate_pattern(&response, &exists_pattern);
        assert!(result.passed);

        // Test equals validation
        let equals_pattern = ValidationPattern {
            key: "status".to_string(),
            validation: ValidationConfig::Equals { value: json!("ok") },
            required: true,
        };
        let result = validator.validate_pattern(&response, &equals_pattern);
        assert!(result.passed);

        // Test range validation
        let range_pattern = ValidationPattern {
            key: "count".to_string(),
            validation: ValidationConfig::Range {
                min: 1.0,
                max: 10.0,
            },
            required: true,
        };
        let result = validator.validate_pattern(&response, &range_pattern);
        assert!(result.passed);

        // Test array validation
        let array_pattern = ValidationPattern {
            key: "items".to_string(),
            validation: ValidationConfig::Array,
            required: true,
        };
        let result = validator.validate_pattern(&response, &array_pattern);
        assert!(result.passed);

        // Test boolean validation
        let boolean_pattern = ValidationPattern {
            key: "enabled".to_string(),
            validation: ValidationConfig::Boolean { value: Some(true) },
            required: true,
        };
        let result = validator.validate_pattern(&response, &boolean_pattern);
        assert!(result.passed);
    }

    #[test]
    fn test_mcp_initialize_validation() {
        let valid_response = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "serverInfo": {
                "name": "test-server",
                "version": "1.0.0"
            }
        });

        let result = validate_mcp_initialize_response(&valid_response).unwrap();
        assert!(result.passed);

        let invalid_response = json!({
            "protocolVersion": "2024-11-05"
            // Missing capabilities and serverInfo
        });

        let result = validate_mcp_initialize_response(&invalid_response).unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn test_empty_response_detection() {
        let validator = TestValidator::new();

        assert!(validator.is_empty_response(&json!(null)));
        assert!(validator.is_empty_response(&json!({})));
        assert!(validator.is_empty_response(&json!([])));
        assert!(validator.is_empty_response(&json!("")));

        assert!(!validator.is_empty_response(&json!({"key": "value"})));
        assert!(!validator.is_empty_response(&json!([1, 2, 3])));
        assert!(!validator.is_empty_response(&json!("hello")));
        assert!(!validator.is_empty_response(&json!(42)));
    }
}
