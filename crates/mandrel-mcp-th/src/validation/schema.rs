use super::ValidationSeverity;
use jsonschema::{Draft, JSONSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// JSON Schema validator for structured response validation
pub struct SchemaValidator {
    compiled_schemas: HashMap<String, JSONSchema>,
    draft_version: Draft,
}

/// Result of schema validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaValidationResult {
    pub is_valid: bool,
    pub violations: Vec<SchemaViolation>,
}

/// Individual schema violation with detailed context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaViolation {
    pub instance_path: String,
    pub schema_path: String,
    pub message: String,
    pub violation_type: ViolationType,
    pub severity: ValidationSeverity,
}

/// Types of schema violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    Required,
    Type,
    Format,
    Pattern,
    Enum,
    Const,
    Minimum,
    Maximum,
    MinLength,
    MaxLength,
    MinItems,
    MaxItems,
    UniqueItems,
    Additional,
    Dependencies,
    Custom,
}

/// Schema validation error
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    #[error("Schema compilation failed: {0}")]
    CompilationError(String),

    #[error("Schema not found: {0}")]
    SchemaNotFound(String),

    #[error("Invalid schema format: {0}")]
    InvalidFormat(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl SchemaValidator {
    /// Create a new schema validator
    pub fn new() -> Self {
        Self {
            compiled_schemas: HashMap::new(),
            draft_version: Draft::Draft7, // Use JSON Schema Draft 7
        }
    }

    /// Add a named schema to the validator
    pub fn add_schema(&mut self, name: String, schema: Value) -> Result<(), SchemaError> {
        let compiled = JSONSchema::options()
            .with_draft(self.draft_version)
            .compile(&schema)
            .map_err(|e| SchemaError::CompilationError(e.to_string()))?;

        self.compiled_schemas.insert(name, compiled);
        Ok(())
    }

    /// Validate data against a named schema
    pub fn validate_against_schema(
        &self,
        data: &Value,
        schema_name: &str,
    ) -> Result<SchemaValidationResult, SchemaError> {
        let schema = self
            .compiled_schemas
            .get(schema_name)
            .ok_or_else(|| SchemaError::SchemaNotFound(schema_name.to_string()))?;

        let validation_result = schema.validate(data);

        match validation_result {
            Ok(_) => Ok(SchemaValidationResult {
                is_valid: true,
                violations: vec![],
            }),
            Err(errors) => {
                let violations = errors
                    .into_iter()
                    .map(|error| SchemaViolation {
                        instance_path: error.instance_path.to_string(),
                        schema_path: error.schema_path.to_string(),
                        message: error.to_string(),
                        violation_type: self.map_violation_type(&error),
                        severity: ValidationSeverity::Error,
                    })
                    .collect();

                Ok(SchemaValidationResult {
                    is_valid: false,
                    violations,
                })
            }
        }
    }

    /// Validate data against an inline schema
    pub fn validate_with_inline_schema(
        &self,
        data: &Value,
        schema: &Value,
    ) -> SchemaValidationResult {
        match JSONSchema::options()
            .with_draft(self.draft_version)
            .compile(schema)
        {
            Ok(compiled_schema) => match compiled_schema.validate(data) {
                Ok(_) => SchemaValidationResult {
                    is_valid: true,
                    violations: vec![],
                },
                Err(errors) => {
                    let violations = errors
                        .into_iter()
                        .map(|error| SchemaViolation {
                            instance_path: error.instance_path.to_string(),
                            schema_path: error.schema_path.to_string(),
                            message: error.to_string(),
                            violation_type: self.map_violation_type(&error),
                            severity: ValidationSeverity::Error,
                        })
                        .collect();

                    SchemaValidationResult {
                        is_valid: false,
                        violations,
                    }
                }
            },
            Err(e) => SchemaValidationResult {
                is_valid: false,
                violations: vec![SchemaViolation {
                    instance_path: "".to_string(),
                    schema_path: "".to_string(),
                    message: format!("Schema compilation failed: {e}"),
                    violation_type: ViolationType::Custom,
                    severity: ValidationSeverity::Error,
                }],
            },
        }
    }

    /// Get information about loaded schemas
    pub fn schema_info(&self) -> Vec<String> {
        self.compiled_schemas.keys().cloned().collect()
    }

    /// Remove a schema from the validator
    pub fn remove_schema(&mut self, name: &str) -> bool {
        self.compiled_schemas.remove(name).is_some()
    }

    /// Clear all schemas
    pub fn clear_schemas(&mut self) {
        self.compiled_schemas.clear();
    }

    /// Map jsonschema validation error to our violation type
    fn map_violation_type(&self, error: &jsonschema::ValidationError) -> ViolationType {
        // Map based on the error description (keyword method doesn't exist in this version)
        let error_str = error.to_string();
        if error_str.contains("required") {
            ViolationType::Required
        } else if error_str.contains("type") {
            ViolationType::Type
        } else if error_str.contains("format") {
            ViolationType::Format
        } else if error_str.contains("pattern") {
            ViolationType::Pattern
        } else if error_str.contains("enum") {
            ViolationType::Enum
        } else if error_str.contains("const") {
            ViolationType::Const
        } else if error_str.contains("minimum") {
            ViolationType::Minimum
        } else if error_str.contains("maximum") {
            ViolationType::Maximum
        } else if error_str.contains("minLength") {
            ViolationType::MinLength
        } else if error_str.contains("maxLength") {
            ViolationType::MaxLength
        } else if error_str.contains("minItems") || error_str.contains("less than") {
            ViolationType::MinItems
        } else if error_str.contains("maxItems") || error_str.contains("more than") {
            ViolationType::MaxItems
        } else if error_str.contains("uniqueItems") || error_str.contains("non-unique") {
            ViolationType::UniqueItems
        } else if error_str.contains("additionalProperties")
            || error_str.contains("additionalItems")
        {
            ViolationType::Additional
        } else if error_str.contains("dependencies") {
            ViolationType::Dependencies
        } else {
            ViolationType::Custom
        }
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_schema_validator_creation() {
        let validator = SchemaValidator::new();
        assert!(
            !validator.compiled_schemas.is_empty(),
            "Should not be empty"
        );
    }

    #[test]
    fn test_add_schema_success() {
        let mut validator = SchemaValidator::new();

        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            },
            "required": ["name"]
        });

        let result = validator.add_schema("person".to_string(), schema);
        assert!(result.is_ok());
        assert_eq!(validator.schema_info().len(), 1);
    }

    #[test]
    fn test_add_schema_invalid() {
        let mut validator = SchemaValidator::new();

        let invalid_schema = json!({
            "type": "invalid_type"
        });

        let result = validator.add_schema("bad_schema".to_string(), invalid_schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_against_schema_success() {
        let mut validator = SchemaValidator::new();

        let schema = json!({
            "type": "object",
            "properties": {
                "result": {
                    "type": "object",
                    "properties": {
                        "content": {"type": "array"},
                        "isError": {"type": "boolean"}
                    },
                    "required": ["content"]
                }
            },
            "required": ["result"]
        });

        validator
            .add_schema("mcp_response".to_string(), schema)
            .unwrap();

        let valid_data = json!({
            "result": {
                "content": [{"type": "text", "text": "Hello"}],
                "isError": false
            }
        });

        let result = validator
            .validate_against_schema(&valid_data, "mcp_response")
            .unwrap();
        assert!(result.is_valid);
        assert!(!result.violations.is_empty(), "Should not be empty");
    }

    #[test]
    fn test_validate_against_schema_failure() {
        let mut validator = SchemaValidator::new();

        let schema = json!({
            "type": "object",
            "properties": {
                "required_field": {"type": "string"}
            },
            "required": ["required_field"]
        });

        validator
            .add_schema("test_schema".to_string(), schema)
            .unwrap();

        let invalid_data = json!({
            "wrong_field": "value"
        });

        let result = validator
            .validate_against_schema(&invalid_data, "test_schema")
            .unwrap();
        assert!(!result.is_valid);
        assert!(!!result.violations.is_empty(), "Should not be empty");

        let violation = &result.violations[0];
        assert_eq!(violation.violation_type, ViolationType::Required);
    }

    #[test]
    fn test_validate_with_inline_schema_success() {
        let validator = SchemaValidator::new();

        let schema = json!({
            "type": "object",
            "properties": {
                "status": {"type": "string", "enum": ["success", "error"]}
            },
            "required": ["status"]
        });

        let valid_data = json!({
            "status": "success"
        });

        let result = validator.validate_with_inline_schema(&valid_data, &schema);
        assert!(result.is_valid);
        assert!(!result.violations.is_empty(), "Should not be empty");
    }

    #[test]
    fn test_validate_with_inline_schema_failure() {
        let validator = SchemaValidator::new();

        let schema = json!({
            "type": "object",
            "properties": {
                "count": {"type": "number", "minimum": 0}
            },
            "required": ["count"]
        });

        let invalid_data = json!({
            "count": -5
        });

        let result = validator.validate_with_inline_schema(&invalid_data, &schema);
        assert!(!result.is_valid);
        assert!(!!result.violations.is_empty(), "Should not be empty");

        let violation = &result.violations[0];
        assert_eq!(violation.violation_type, ViolationType::Minimum);
    }

    #[test]
    fn test_validate_type_mismatch() {
        let validator = SchemaValidator::new();

        let schema = json!({
            "type": "object",
            "properties": {
                "age": {"type": "number"}
            }
        });

        let invalid_data = json!({
            "age": "not a number"
        });

        let result = validator.validate_with_inline_schema(&invalid_data, &schema);
        assert!(!result.is_valid);
        assert!(!!result.violations.is_empty(), "Should not be empty");

        let violation = &result.violations[0];
        assert_eq!(violation.violation_type, ViolationType::Type);
    }

    #[test]
    fn test_validate_array_constraints() {
        let validator = SchemaValidator::new();

        let schema = json!({
            "type": "object",
            "properties": {
                "items": {
                    "type": "array",
                    "minItems": 1,
                    "maxItems": 3,
                    "uniqueItems": true
                }
            }
        });

        let invalid_data = json!({
            "items": [1, 2, 2, 3, 4] // Too many items and not unique
        });

        let result = validator.validate_with_inline_schema(&invalid_data, &schema);
        assert!(!result.is_valid);
        assert!(!!result.violations.is_empty(), "Should not be empty");

        // Should have violations for maxItems and uniqueItems
        assert!(result
            .violations
            .iter()
            .any(|v| v.violation_type == ViolationType::MaxItems));
        assert!(result
            .violations
            .iter()
            .any(|v| v.violation_type == ViolationType::UniqueItems));
    }

    #[test]
    fn test_schema_info() {
        let mut validator = SchemaValidator::new();

        let schema1 = json!({"type": "string"});
        let schema2 = json!({"type": "number"});

        validator
            .add_schema("schema1".to_string(), schema1)
            .unwrap();
        validator
            .add_schema("schema2".to_string(), schema2)
            .unwrap();

        let info = validator.schema_info();
        assert_eq!(info.len(), 2, "Should have 2 items");
        assert!(info.contains(&"schema1".to_string()));
        assert!(info.contains(&"schema2".to_string()));
    }

    #[test]
    fn test_remove_schema() {
        let mut validator = SchemaValidator::new();

        let schema = json!({"type": "string"});
        validator
            .add_schema("test_schema".to_string(), schema)
            .unwrap();

        assert_eq!(validator.schema_info().len(), 1);

        let removed = validator.remove_schema("test_schema");
        assert!(removed);
        assert_eq!(validator.schema_info().len(), 0);

        let not_removed = validator.remove_schema("nonexistent");
        assert!(!not_removed);
    }

    #[test]
    fn test_clear_schemas() {
        let mut validator = SchemaValidator::new();

        let schema = json!({"type": "string"});
        validator
            .add_schema("schema1".to_string(), schema.clone())
            .unwrap();
        validator.add_schema("schema2".to_string(), schema).unwrap();

        assert_eq!(validator.schema_info().len(), 2);

        validator.clear_schemas();
        assert_eq!(validator.schema_info().len(), 0);
    }
}
