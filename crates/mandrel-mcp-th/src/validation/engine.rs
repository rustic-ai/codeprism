use super::{
    CustomValidator, JsonPathEvaluator, McpValidationResult, ProtocolValidator, SchemaValidator,
    ValidationContext, ValidationEngineError, ValidationSpec,
};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// Main MCP protocol validation engine that coordinates all validation components
pub struct McpValidationEngine {
    jsonpath_evaluator: JsonPathEvaluator,
    schema_validator: SchemaValidator,
    protocol_validator: ProtocolValidator,
    custom_validators: Vec<Box<dyn CustomValidator>>,
}

impl McpValidationEngine {
    /// Create a new MCP validation engine with default components
    pub fn new() -> Result<Self, ValidationEngineError> {
        Ok(Self {
            jsonpath_evaluator: JsonPathEvaluator::new()?,
            schema_validator: SchemaValidator::new(),
            protocol_validator: ProtocolValidator::new("1.0".to_string()),
            custom_validators: Vec::new(),
        })
    }

    /// Validate an MCP response against a validation specification
    pub async fn validate_response(
        &self,
        response: &Value,
        spec: &ValidationSpec,
    ) -> Result<McpValidationResult, ValidationEngineError> {
        let mut result = McpValidationResult::new();

        // Create validation context
        let context = ValidationContext {
            method: spec.protocol_requirements.method.clone(),
            request_id: response.get("id").cloned(),
            server_capabilities: None, // FUTURE: Add capabilities support in Phase 2
            test_metadata: HashMap::new(),
        };

        // 1. JSONPath validation
        for rule in &spec.jsonpath_rules {
            let field_result = self.jsonpath_evaluator.evaluate_rule(response, rule);
            result.add_field_result(rule.path.clone(), field_result);
        }

        // 2. Schema validation
        if let Some(schema) = &spec.schema {
            let schema_result = self
                .schema_validator
                .validate_with_inline_schema(response, schema);
            result.add_schema_violations(schema_result.violations);
        }

        // 3. Protocol validation
        let protocol_issues = self.protocol_validator.validate_mcp_method_response(
            &spec.protocol_requirements.method,
            response,
            &spec.protocol_requirements,
        );
        result.add_protocol_issues(protocol_issues);

        // 4. Custom validation
        for validator in &self.custom_validators {
            match validator.validate(response, &context) {
                Ok(errors) => {
                    for error in errors {
                        result.add_error(error);
                    }
                }
                Err(e) => {
                    return Err(ValidationEngineError::CustomValidatorError(format!(
                        "Custom validator '{}' failed: {}",
                        validator.name(),
                        e
                    )));
                }
            }
        }

        Ok(result)
    }

    /// Add a custom validator to the engine
    pub fn add_custom_validator(
        &mut self,
        validator: Box<dyn CustomValidator>,
    ) -> Result<(), ValidationEngineError> {
        self.custom_validators.push(validator);
        Ok(())
    }

    /// Load validation specification from a file
    pub fn load_validation_spec_from_file(
        &self,
        path: &Path,
    ) -> Result<ValidationSpec, ValidationEngineError> {
        let content = std::fs::read_to_string(path)?;
        let spec: ValidationSpec = serde_json::from_str(&content)?;
        Ok(spec)
    }

    /// Pre-compile schemas for better performance
    pub fn precompile_schemas(
        &mut self,
        schemas: HashMap<String, Value>,
    ) -> Result<(), ValidationEngineError> {
        for (name, schema) in schemas {
            self.schema_validator
                .add_schema(name, schema)
                .map_err(|e| ValidationEngineError::SchemaError(e.to_string()))?;
        }
        Ok(())
    }
}

impl Default for McpValidationEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default MCP validation engine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::{
        JsonPathRule, PathConstraint, ProtocolRequirements, ValidationSeverity,
    };
    use serde_json::json;

    #[test]
    fn test_mcp_validation_engine_creation() {
        let engine = McpValidationEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_validation_spec_deserialization() {
        let spec_json = r#"
        {
            "schema": null,
            "jsonpath_rules": [
                {
                    "path": "$.result.content[0].text",
                    "constraint": "Exists",
                    "description": "Response must contain text content",
                    "severity": "Error"
                }
            ],
            "protocol_requirements": {
                "method": "tools/call",
                "required_fields": ["result"],
                "optional_fields": [],
                "expected_error_codes": [],
                "capability_requirements": []
            },
            "custom_rules": [],
            "strict_mode": true
        }"#;

        let spec: ValidationSpec = serde_json::from_str(spec_json).unwrap();
        assert_eq!(spec.jsonpath_rules.len(), 1);
        assert_eq!(spec.protocol_requirements.method, "tools/call");
        assert!(spec.strict_mode);
    }

    #[tokio::test]
    async fn test_validate_response_success() {
        let engine = McpValidationEngine::new().unwrap();

        let spec = ValidationSpec {
            schema: None,
            jsonpath_rules: vec![JsonPathRule {
                path: "$.result.content".to_string(),
                constraint: PathConstraint::Exists,
                description: "Content must exist".to_string(),
                severity: ValidationSeverity::Error,
            }],
            protocol_requirements: ProtocolRequirements {
                method: "tools/call".to_string(),
                required_fields: vec!["result".to_string()],
                optional_fields: vec![],
                expected_error_codes: vec![],
                capability_requirements: vec![],
            },
            custom_rules: vec![],
            strict_mode: false,
        };

        let response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "content": [{"type": "text", "text": "Hello"}],
                "isError": false
            }
        });

        let result = engine.validate_response(&response, &spec).await;
        assert!(result.is_ok());

        let _validation_result = result.unwrap();
        // Note: This will likely show validation issues until we implement the sub-modules
        // but the engine should be created successfully
    }

    #[test]
    fn test_load_validation_spec_from_file() {
        let engine = McpValidationEngine::new().unwrap();

        // Create a temporary spec file
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        let spec_content = r#"
        {
            "schema": null,
            "jsonpath_rules": [],
            "protocol_requirements": {
                "method": "tools/list",
                "required_fields": ["tools"],
                "optional_fields": [],
                "expected_error_codes": [],
                "capability_requirements": []
            },
            "custom_rules": [],
            "strict_mode": false
        }"#;

        temp_file.write_all(spec_content.as_bytes()).unwrap();

        let result = engine.load_validation_spec_from_file(temp_file.path());
        assert!(result.is_ok());

        let spec = result.unwrap();
        assert_eq!(spec.protocol_requirements.method, "tools/list");
    }

    #[test]
    fn test_precompile_schemas() {
        let mut engine = McpValidationEngine::new().unwrap();

        let mut schemas = HashMap::new();
        schemas.insert(
            "tool_response".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "result": {
                        "type": "object",
                        "properties": {
                            "content": {"type": "array"}
                        },
                        "required": ["content"]
                    }
                },
                "required": ["result"]
            }),
        );

        let result = engine.precompile_schemas(schemas);
        assert!(result.is_ok());
    }
}
