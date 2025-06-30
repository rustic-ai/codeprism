//! Response validation for MCP test harness

use super::schema::{ExpectedOutput, FieldValidation, SchemaValidator};
use crate::types::ValidationResult;
use jsonpath_lib as jsonpath;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

/// Response validator for test expectations
pub struct ResponseValidator {
    /// Schema validator for JSON schema-based validation
    schema_validator: SchemaValidator,
    /// Compiled regex patterns cache
    regex_cache: HashMap<String, Regex>,
}

impl ResponseValidator {
    /// Create a new response validator
    pub fn new() -> Self {
        Self {
            schema_validator: SchemaValidator::new(),
            regex_cache: HashMap::new(),
        }
    }

    /// Create a response validator with a base directory for schema files
    pub fn with_base_dir<P: AsRef<std::path::Path>>(base_dir: P) -> Self {
        Self {
            schema_validator: SchemaValidator::with_base_dir(base_dir),
            regex_cache: HashMap::new(),
        }
    }

    /// Validate a response against expected output specification
    pub fn validate_response(
        &mut self,
        response: &serde_json::Value,
        expected: &ExpectedOutput,
    ) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 1. Validate error expectation
        let error_validation = self.validate_error_expectation(response, expected);
        if !error_validation.valid {
            errors.extend(error_validation.errors);
        }
        warnings.extend(error_validation.warnings);

        // 2. Validate against JSON schema if provided
        if let Some(schema_validation) = self.validate_schema(response, expected) {
            if !schema_validation.valid {
                errors.extend(schema_validation.errors);
            }
            warnings.extend(schema_validation.warnings);
        }

        // 3. Validate specific fields
        for field in &expected.fields {
            let field_validation = self.validate_field(response, field);
            if !field_validation.valid {
                errors.extend(field_validation.errors);
            }
            warnings.extend(field_validation.warnings);
        }

        // 4. Check for extra fields if not allowed
        if !expected.allow_extra_fields {
            let extra_validation = self.validate_no_extra_fields(response, expected);
            if !extra_validation.valid {
                errors.extend(extra_validation.errors);
            }
            warnings.extend(extra_validation.warnings);
        }

        if errors.is_empty() {
            let mut result = ValidationResult::success();
            for warning in warnings {
                result = result.with_warning(warning);
            }
            result
        } else {
            let mut result = ValidationResult::error(errors.join("; "));
            for warning in warnings {
                result = result.with_warning(warning);
            }
            result
        }
    }

    /// Validate error expectation (whether error should or shouldn't occur)
    fn validate_error_expectation(
        &self,
        response: &Value,
        expected: &ExpectedOutput,
    ) -> ValidationResult {
        let has_error = response.get("error").is_some();

        if expected.error && !has_error {
            return ValidationResult::error("Expected error response but got success");
        }

        if !expected.error && has_error {
            return ValidationResult::error("Expected success response but got error");
        }

        // If error is expected, validate error details
        if expected.error && has_error {
            let error_obj = response.get("error").unwrap();

            // Validate error code if specified
            if let Some(expected_code) = expected.error_code {
                if let Some(code) = error_obj.get("code").and_then(|c| c.as_i64()) {
                    if code != expected_code as i64 {
                        return ValidationResult::error(format!(
                            "Expected error code {} but got {}",
                            expected_code, code
                        ));
                    }
                } else {
                    return ValidationResult::error("Error response missing 'code' field");
                }
            }

            // Validate error message contains expected text
            if let Some(expected_message) = &expected.error_message_contains {
                if let Some(message) = error_obj.get("message").and_then(|m| m.as_str()) {
                    if !message.contains(expected_message) {
                        return ValidationResult::error(format!(
                            "Error message '{}' does not contain expected text '{}'",
                            message, expected_message
                        ));
                    }
                } else {
                    return ValidationResult::error("Error response missing 'message' field");
                }
            }
        }

        ValidationResult::success()
    }

    /// Validate response against JSON schema
    fn validate_schema(
        &mut self,
        response: &Value,
        expected: &ExpectedOutput,
    ) -> Option<ValidationResult> {
        // Try schema file first, then inline schema
        if let Some(schema_file) = &expected.schema_file {
            match self.schema_validator.validate(response, schema_file) {
                Ok(_) => Some(ValidationResult::success()),
                Err(e) => Some(ValidationResult::error(format!(
                    "Schema validation failed: {}",
                    e
                ))),
            }
        } else if let Some(schema) = &expected.schema {
            match self.schema_validator.validate_inline(response, schema) {
                Ok(_) => Some(ValidationResult::success()),
                Err(e) => Some(ValidationResult::error(format!(
                    "Inline schema validation failed: {}",
                    e
                ))),
            }
        } else {
            None
        }
    }

    /// Validate a specific field using JSONPath
    pub fn validate_field(
        &mut self,
        response: &Value,
        field: &FieldValidation,
    ) -> ValidationResult {
        // Extract field value using JSONPath
        let field_values = match jsonpath::select(response, &field.path) {
            Ok(values) => values,
            Err(e) => {
                if field.required {
                    return ValidationResult::error(format!(
                        "Required field '{}' not found: {}",
                        field.path, e
                    ));
                } else {
                    return ValidationResult::success(); // Optional field not found is OK
                }
            }
        };

        // Check if field is required but not found
        if field.required && field_values.is_empty() {
            return ValidationResult::error(format!("Required field '{}' not found", field.path));
        }

        // If optional field not found, validation passes
        if field_values.is_empty() {
            return ValidationResult::success();
        }

        // Validate each found value
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for value in field_values {
            let field_validation = self.validate_field_value(value, field);
            if !field_validation.valid {
                errors.extend(field_validation.errors);
            }
            warnings.extend(field_validation.warnings);
        }

        if errors.is_empty() {
            let mut result = ValidationResult::success();
            for warning in warnings {
                result = result.with_warning(warning);
            }
            result
        } else {
            let mut result = ValidationResult::error(errors.join("; "));
            for warning in warnings {
                result = result.with_warning(warning);
            }
            result
        }
    }

    /// Validate a single field value against field validation rules
    fn validate_field_value(&mut self, value: &Value, field: &FieldValidation) -> ValidationResult {
        // Check exact value match
        if let Some(expected_value) = &field.value {
            if value != expected_value {
                return ValidationResult::error(format!(
                    "Field '{}' expected value {:?} but got {:?}",
                    field.path, expected_value, value
                ));
            }
        }

        // Check field type
        if let Some(expected_type) = &field.field_type {
            if !self.check_field_type(value, expected_type) {
                return ValidationResult::error(format!(
                    "Field '{}' expected type '{}' but got type '{}'",
                    field.path,
                    expected_type,
                    self.get_value_type(value)
                ));
            }
        }

        // Check pattern for strings
        if let Some(pattern) = &field.pattern {
            if let Some(string_value) = value.as_str() {
                match self.get_or_compile_regex(pattern) {
                    Ok(regex) => {
                        if !regex.is_match(string_value) {
                            return ValidationResult::error(format!(
                                "Field '{}' value '{}' does not match pattern '{}'",
                                field.path, string_value, pattern
                            ));
                        }
                    }
                    Err(e) => {
                        return ValidationResult::error(format!(
                            "Invalid regex pattern '{}': {}",
                            pattern, e
                        ));
                    }
                }
            } else {
                return ValidationResult::error(format!(
                    "Field '{}' pattern validation requires string value, got {}",
                    field.path,
                    self.get_value_type(value)
                ));
            }
        }

        // Check numeric ranges
        if let Some(min_val) = field.min {
            if let Some(num_value) = value.as_f64() {
                if num_value < min_val {
                    return ValidationResult::error(format!(
                        "Field '{}' value {} is less than minimum {}",
                        field.path, num_value, min_val
                    ));
                }
            }
        }

        if let Some(max_val) = field.max {
            if let Some(num_value) = value.as_f64() {
                if num_value > max_val {
                    return ValidationResult::error(format!(
                        "Field '{}' value {} is greater than maximum {}",
                        field.path, num_value, max_val
                    ));
                }
            }
        }

        ValidationResult::success()
    }

    /// Check if extra fields are present when they shouldn't be
    fn validate_no_extra_fields(
        &self,
        _response: &Value,
        _expected: &ExpectedOutput,
    ) -> ValidationResult {
        // FUTURE: Implement comprehensive extra field validation (tracked in #124)
        // This would require comparing response fields against expected schema structure.
        // Currently, we rely on JSON schema validation to catch unauthorized extra fields.
        ValidationResult::success()
    }

    /// Check if a value matches the expected type
    fn check_field_type(&self, value: &Value, expected_type: &str) -> bool {
        match expected_type.to_lowercase().as_str() {
            "string" => value.is_string(),
            "number" | "numeric" => value.is_number(),
            "integer" | "int" => value.is_i64() || value.is_u64(),
            "float" | "double" => value.is_f64(),
            "boolean" | "bool" => value.is_boolean(),
            "array" => value.is_array(),
            "object" => value.is_object(),
            "null" => value.is_null(),
            _ => false, // Unknown type
        }
    }

    /// Get the type name of a JSON value
    fn get_value_type(&self, value: &Value) -> &'static str {
        match value {
            Value::String(_) => "string",
            Value::Number(_) => "number",
            Value::Bool(_) => "boolean",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Null => "null",
        }
    }

    /// Get or compile a regex pattern, using cache for performance
    fn get_or_compile_regex(&mut self, pattern: &str) -> Result<&Regex, regex::Error> {
        if !self.regex_cache.contains_key(pattern) {
            let regex = Regex::new(pattern)?;
            self.regex_cache.insert(pattern.to_string(), regex);
        }
        Ok(self.regex_cache.get(pattern).unwrap())
    }
}

impl Default for ResponseValidator {
    fn default() -> Self {
        Self::new()
    }
}
