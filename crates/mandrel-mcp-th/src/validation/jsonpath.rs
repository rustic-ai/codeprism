use super::{FieldValidationResult, JsonType, ValidationEngineError, ValidationSeverity};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// JSONPath expression evaluator for field validation
pub struct JsonPathEvaluator {
    expression_cache: HashMap<String, String>, // Simplified cache for now
}

/// JSONPath validation rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonPathRule {
    pub path: String,
    pub constraint: PathConstraint,
    pub description: String,
    pub severity: ValidationSeverity,
}

/// Path constraint types for JSONPath validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathConstraint {
    Exists,
    NotExists,
    Equals(Value),
    NotEquals(Value),
    Contains(String),
    MatchesPattern(String),
    HasType(JsonType),
    InRange(f64, f64),
    ArrayLength(usize, Option<usize>), // min_length, max_length
    Custom(String),                    // Custom expression for future extensibility
}

/// JSONPath evaluation error
#[derive(Debug, thiserror::Error)]
pub enum JsonPathError {
    #[error("Invalid JSONPath expression: {0}")]
    InvalidExpression(String),

    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("Type mismatch at path {path}: expected {expected}, got {actual}")]
    TypeMismatch {
        path: String,
        expected: String,
        actual: String,
    },

    #[error("Constraint evaluation failed: {0}")]
    ConstraintError(String),
}

impl JsonPathEvaluator {
    /// Create a new JSONPath evaluator
    pub fn new() -> Result<Self, ValidationEngineError> {
        Ok(Self {
            expression_cache: HashMap::new(),
        })
    }

    /// Evaluate a JSONPath rule against the provided data
    pub fn evaluate_rule(&self, data: &Value, rule: &JsonPathRule) -> FieldValidationResult {
        // Extract values using JSONPath
        match self.extract_values(data, &rule.path) {
            Ok(values) => {
                // Apply constraint to extracted values
                let (is_valid, error_message) =
                    self.apply_constraint(&values, &rule.constraint, &rule.path);

                FieldValidationResult {
                    field_path: rule.path.clone(),
                    validation_type: crate::spec::FieldValidationType::Exists, // Will be determined by constraint type
                    is_valid,
                    actual_value: Some(values.first().cloned().unwrap_or(Value::Null)),
                    expected_value: self.constraint_to_expected_value(&rule.constraint),
                    error_message,
                }
            }
            Err(e) => FieldValidationResult {
                field_path: rule.path.clone(),
                validation_type: crate::spec::FieldValidationType::Exists,
                is_valid: false,
                actual_value: Some(Value::Null),
                expected_value: None,
                error_message: Some(e.to_string()),
            },
        }
    }

    /// Extract values from data using JSONPath expression
    pub fn extract_values(&self, data: &Value, path: &str) -> Result<Vec<Value>, JsonPathError> {
        // Simple JSONPath implementation for common patterns
        // ENHANCEMENT: Use jsonpath_lib crate for full JSONPath support in Phase 2

        if path == "$" {
            return Ok(vec![data.clone()]);
        }

        if let Some(stripped) = path.strip_prefix("$.") {
            let path_parts: Vec<&str> = stripped.split('.').collect();
            let mut current = data;

            for part in path_parts {
                if part.contains('[') && part.ends_with(']') {
                    // Handle array access like "content[0]"
                    let field_name = part.split('[').next().unwrap();
                    let index_str = part.split('[').nth(1).unwrap().trim_end_matches(']');

                    current = current.get(field_name).ok_or_else(|| {
                        JsonPathError::PathNotFound(format!("Field {field_name} not found"))
                    })?;

                    if let Ok(index) = index_str.parse::<usize>() {
                        current = current.get(index).ok_or_else(|| {
                            JsonPathError::PathNotFound(format!("Array index {index} not found"))
                        })?;
                    } else {
                        return Err(JsonPathError::InvalidExpression(format!(
                            "Invalid array index: {}",
                            index_str
                        )));
                    }
                } else {
                    current = current.get(part).ok_or_else(|| {
                        JsonPathError::PathNotFound(format!("Field {part} not found"))
                    })?;
                }
            }

            Ok(vec![current.clone()])
        } else {
            Err(JsonPathError::InvalidExpression(format!(
                "Unsupported JSONPath: {}",
                path
            )))
        }
    }

    /// Apply a constraint to extracted values
    fn apply_constraint(
        &self,
        values: &[Value],
        constraint: &PathConstraint,
        path: &str,
    ) -> (bool, Option<String>) {
        match constraint {
            PathConstraint::Exists => {
                let exists = !values.is_empty() && !values[0].is_null();
                if exists {
                    (true, None)
                } else {
                    (
                        false,
                        Some(format!("Path {path} does not exist or is null")),
                    )
                }
            }

            PathConstraint::NotExists => {
                let exists = !values.is_empty() && !values[0].is_null();
                if !exists {
                    (true, None)
                } else {
                    (
                        false,
                        Some(format!("Path {path} should not exist but was found")),
                    )
                }
            }

            PathConstraint::Equals(expected) => {
                if values.is_empty() {
                    (
                        false,
                        Some(format!("Path {path} not found for equality check")),
                    )
                } else if values[0] == *expected {
                    (true, None)
                } else {
                    (
                        false,
                        Some(format!("Expected {}, got {}", expected, values[0])),
                    )
                }
            }

            PathConstraint::NotEquals(expected) => {
                if values.is_empty() || values[0] != *expected {
                    (true, None) // Not found or not equal means success
                } else {
                    (false, Some(format!("Value should not equal {expected}")))
                }
            }

            PathConstraint::Contains(substring) => {
                if values.is_empty() {
                    (
                        false,
                        Some(format!("Path {path} not found for contains check")),
                    )
                } else if let Value::String(s) = &values[0] {
                    if s.contains(substring) {
                        (true, None)
                    } else {
                        (
                            false,
                            Some(format!("String '{s}' does not contain '{substring}'")),
                        )
                    }
                } else {
                    (
                        false,
                        Some("Contains constraint only works on strings".to_string()),
                    )
                }
            }

            PathConstraint::MatchesPattern(pattern) => {
                if values.is_empty() {
                    (
                        false,
                        Some(format!("Path {path} not found for pattern check")),
                    )
                } else if let Value::String(s) = &values[0] {
                    // Simple pattern matching - just check if pattern is contained
                    // ENHANCEMENT: Use regex crate for proper pattern matching in Phase 2
                    let matches = s.contains(&pattern.replace(".*", ""));
                    if matches {
                        (true, None)
                    } else {
                        (
                            false,
                            Some(format!(
                                "String '{}' does not match pattern '{}'",
                                s, pattern
                            )),
                        )
                    }
                } else {
                    (
                        false,
                        Some("Pattern constraint only works on strings".to_string()),
                    )
                }
            }

            PathConstraint::HasType(expected_type) => {
                if values.is_empty() {
                    (false, Some(format!("Path {path} not found for type check")))
                } else {
                    let actual_type = self.value_to_json_type(&values[0]);
                    if actual_type == *expected_type {
                        (true, None)
                    } else {
                        (
                            false,
                            Some(format!(
                                "Expected type {:?}, got {:?}",
                                expected_type, actual_type
                            )),
                        )
                    }
                }
            }

            PathConstraint::InRange(min, max) => {
                if values.is_empty() {
                    (
                        false,
                        Some(format!("Path {path} not found for range check")),
                    )
                } else if let Value::Number(n) = &values[0] {
                    if let Some(value) = n.as_f64() {
                        if value >= *min && value <= *max {
                            (true, None)
                        } else {
                            (
                                false,
                                Some(format!("Value {value} not in range {min}..{max}")),
                            )
                        }
                    } else {
                        (false, Some("Could not convert number to f64".to_string()))
                    }
                } else {
                    (
                        false,
                        Some("Range constraint only works on numbers".to_string()),
                    )
                }
            }

            PathConstraint::ArrayLength(min_len, max_len) => {
                if values.is_empty() {
                    (
                        false,
                        Some(format!("Path {path} not found for array length check")),
                    )
                } else if let Value::Array(arr) = &values[0] {
                    let len = arr.len();
                    let min_valid = len >= *min_len;
                    let max_valid = max_len.is_none_or(|max| len <= max);

                    if min_valid && max_valid {
                        (true, None)
                    } else {
                        let max_str = max_len.map_or("∞".to_string(), |m| m.to_string());
                        (
                            false,
                            Some(format!(
                                "Array length {} not in range {}..{}",
                                len, min_len, max_str
                            )),
                        )
                    }
                } else {
                    (
                        false,
                        Some("Array length constraint only works on arrays".to_string()),
                    )
                }
            }

            PathConstraint::Custom(_expr) => {
                // PLANNED: Implement custom expression evaluation in Phase 4
                (
                    false,
                    Some("Custom constraints not yet implemented".to_string()),
                )
            }
        }
    }

    /// Convert a JSON value to JsonType
    fn value_to_json_type(&self, value: &Value) -> JsonType {
        match value {
            Value::Object(_) => JsonType::Object,
            Value::Array(_) => JsonType::Array,
            Value::String(_) => JsonType::String,
            Value::Number(_) => JsonType::Number,
            Value::Bool(_) => JsonType::Boolean,
            Value::Null => JsonType::Null,
        }
    }

    /// Convert constraint to expected value for display
    fn constraint_to_expected_value(&self, constraint: &PathConstraint) -> Option<Value> {
        match constraint {
            PathConstraint::Equals(value) => Some(value.clone()),
            PathConstraint::HasType(json_type) => Some(Value::String(format!("{json_type:?}"))),
            _ => None,
        }
    }

    /// Clear the expression cache
    pub fn clear_cache(&mut self) {
        self.expression_cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.expression_cache.len(), 0) // (entries, capacity)
    }
}

impl Default for JsonPathEvaluator {
    fn default() -> Self {
        Self::new().expect("Failed to create default JSONPath evaluator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_jsonpath_evaluator_creation() {
        let evaluator = JsonPathEvaluator::new();
        assert!(evaluator.is_ok());
    }

    #[test]
    fn test_extract_values_simple_path() {
        let evaluator = JsonPathEvaluator::new().unwrap();
        let data = json!({
            "result": {
                "content": [{"text": "Hello"}]
            }
        });

        let values = evaluator.extract_values(&data, "$.result").unwrap();
        assert_eq!(values.len(), 1);
        assert!(values[0].is_object());
    }

    #[test]
    fn test_extract_values_array_access() {
        let evaluator = JsonPathEvaluator::new().unwrap();
        let data = json!({
            "result": {
                "content": [{"text": "Hello"}, {"text": "World"}]
            }
        });

        let values = evaluator
            .extract_values(&data, "$.result.content[0]")
            .unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0]["text"], "Hello");
    }

    #[test]
    fn test_extract_values_nested_path() {
        let evaluator = JsonPathEvaluator::new().unwrap();
        let data = json!({
            "result": {
                "content": [{"text": "Hello"}]
            }
        });

        let values = evaluator
            .extract_values(&data, "$.result.content[0].text")
            .unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], "Hello");
    }

    #[test]
    fn test_extract_values_path_not_found() {
        let evaluator = JsonPathEvaluator::new().unwrap();
        let data = json!({"other": "value"});

        let result = evaluator.extract_values(&data, "$.result");
        assert!(result.is_err());
    }

    #[test]
    fn test_evaluate_rule_exists_constraint() {
        let evaluator = JsonPathEvaluator::new().unwrap();
        let data = json!({
            "result": {
                "content": [{"text": "Hello"}]
            }
        });

        let rule = JsonPathRule {
            path: "$.result.content[0].text".to_string(),
            constraint: PathConstraint::Exists,
            description: "Text field must exist".to_string(),
            severity: ValidationSeverity::Error,
        };

        let result = evaluator.evaluate_rule(&data, &rule);
        assert!(result.is_valid);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_evaluate_rule_equals_constraint() {
        let evaluator = JsonPathEvaluator::new().unwrap();
        let data = json!({"status": "success"});

        let rule = JsonPathRule {
            path: "$.status".to_string(),
            constraint: PathConstraint::Equals(json!("success")),
            description: "Status must be success".to_string(),
            severity: ValidationSeverity::Error,
        };

        let result = evaluator.evaluate_rule(&data, &rule);
        assert!(result.is_valid);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_evaluate_rule_type_constraint() {
        let evaluator = JsonPathEvaluator::new().unwrap();
        let data = json!({"count": 42});

        let rule = JsonPathRule {
            path: "$.count".to_string(),
            constraint: PathConstraint::HasType(JsonType::Number),
            description: "Count must be a number".to_string(),
            severity: ValidationSeverity::Error,
        };

        let result = evaluator.evaluate_rule(&data, &rule);
        assert!(result.is_valid);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_evaluate_rule_range_constraint() {
        let evaluator = JsonPathEvaluator::new().unwrap();
        let data = json!({"score": 85.5});

        let rule = JsonPathRule {
            path: "$.score".to_string(),
            constraint: PathConstraint::InRange(0.0, 100.0),
            description: "Score must be in valid range".to_string(),
            severity: ValidationSeverity::Error,
        };

        let result = evaluator.evaluate_rule(&data, &rule);
        assert!(result.is_valid);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_evaluate_rule_array_length_constraint() {
        let evaluator = JsonPathEvaluator::new().unwrap();
        let data = json!({
            "items": ["a", "b", "c"]
        });

        let rule = JsonPathRule {
            path: "$.items".to_string(),
            constraint: PathConstraint::ArrayLength(1, Some(5)),
            description: "Items array must have 1-5 elements".to_string(),
            severity: ValidationSeverity::Error,
        };

        let result = evaluator.evaluate_rule(&data, &rule);
        assert!(result.is_valid);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_evaluate_rule_contains_constraint() {
        let evaluator = JsonPathEvaluator::new().unwrap();
        let data = json!({"message": "Hello, world!"});

        let rule = JsonPathRule {
            path: "$.message".to_string(),
            constraint: PathConstraint::Contains("world".to_string()),
            description: "Message must contain 'world'".to_string(),
            severity: ValidationSeverity::Warning,
        };

        let result = evaluator.evaluate_rule(&data, &rule);
        assert!(result.is_valid);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_evaluate_rule_validation_failure() {
        let evaluator = JsonPathEvaluator::new().unwrap();
        let data = json!({"status": "failure"});

        let rule = JsonPathRule {
            path: "$.status".to_string(),
            constraint: PathConstraint::Equals(json!("success")),
            description: "Status must be success".to_string(),
            severity: ValidationSeverity::Error,
        };

        let result = evaluator.evaluate_rule(&data, &rule);
        assert!(!result.is_valid);
        assert!(result.error_message.is_some());
        assert!(result.error_message.unwrap().contains("Expected"));
    }
}
