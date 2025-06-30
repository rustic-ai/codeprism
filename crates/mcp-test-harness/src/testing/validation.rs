//! Test validation utilities

use crate::spec::schema::ExpectedOutput;
use crate::spec::validator::ResponseValidator;
use crate::types::ValidationResult;
use serde_json::Value;

/// Validate test responses and results
pub struct TestValidator {
    /// Response validator for detailed validation
    response_validator: ResponseValidator,
}

impl TestValidator {
    /// Create a new test validator
    pub fn new() -> Self {
        Self {
            response_validator: ResponseValidator::new(),
        }
    }

    /// Create a test validator with base directory for schema files
    pub fn with_base_dir<P: AsRef<std::path::Path>>(base_dir: P) -> Self {
        Self {
            response_validator: ResponseValidator::with_base_dir(base_dir),
        }
    }

    /// Validate a test response against expected output
    pub fn validate_response(
        &mut self,
        response: &Value,
        expected: &ExpectedOutput,
    ) -> ValidationResult {
        self.response_validator
            .validate_response(response, expected)
    }

    /// Validate a test response with basic JSON-RPC validation
    pub fn validate(&mut self, response: &Value) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 1. Basic JSON structure validation
        if response.is_null() {
            return ValidationResult::error("Response is null");
        }

        if !response.is_object() {
            return ValidationResult::error("Response must be a JSON object");
        }

        // 2. JSON-RPC 2.0 validation
        match response.get("jsonrpc") {
            Some(version) => {
                if version.as_str() != Some("2.0") {
                    errors.push("Invalid JSON-RPC version, expected '2.0'".to_string());
                }
            }
            None => {
                warnings.push("Missing JSON-RPC version field".to_string());
            }
        }

        // 3. Message structure validation
        let _has_id = response.get("id").is_some();
        let has_result = response.get("result").is_some();
        let has_error = response.get("error").is_some();

        // Validate message type structure
        if has_result && has_error {
            errors.push("Response cannot have both 'result' and 'error' fields".to_string());
        }

        if !has_result && !has_error {
            // Could be a notification, but usually test responses should have result or error
            warnings.push("Response has neither 'result' nor 'error' field".to_string());
        }

        // 4. Error structure validation
        if has_error {
            if let Some(error) = response.get("error") {
                if !error.is_object() {
                    errors.push("Error field must be an object".to_string());
                } else {
                    // Validate error structure
                    if error.get("code").is_none() {
                        errors.push("Error object must have 'code' field".to_string());
                    } else if !error.get("code").unwrap().is_number() {
                        errors.push("Error 'code' field must be a number".to_string());
                    }

                    if error.get("message").is_none() {
                        errors.push("Error object must have 'message' field".to_string());
                    } else if !error.get("message").unwrap().is_string() {
                        errors.push("Error 'message' field must be a string".to_string());
                    }
                }
            }
        }

        // Return validation result
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
}

impl Default for TestValidator {
    fn default() -> Self {
        Self::new()
    }
}
