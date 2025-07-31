use super::ValidationError;
use serde_json::Value;
use std::collections::HashMap;

/// Trait for custom validation logic
pub trait CustomValidator: Send + Sync {
    /// Get the name of this validator
    fn name(&self) -> &str;

    /// Validate data using custom logic
    fn validate(
        &self,
        data: &Value,
        context: &ValidationContext,
    ) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>>;

    /// Get validator description
    fn description(&self) -> &str {
        "Custom validator"
    }

    /// Check if validator is enabled
    fn is_enabled(&self) -> bool {
        true
    }
}

/// Context information provided to custom validators
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub method: String,
    pub request_id: Option<Value>,
    pub server_capabilities: Option<Value>,
    pub test_metadata: HashMap<String, String>,
}

/// Example: Content length validator
pub struct ContentLengthValidator {
    max_length: usize,
}

impl ContentLengthValidator {
    pub fn new(max_length: usize) -> Self {
        Self { max_length }
    }
}

impl CustomValidator for ContentLengthValidator {
    fn name(&self) -> &str {
        "content_length"
    }

    fn description(&self) -> &str {
        "Validates that content length does not exceed maximum"
    }

    fn validate(
        &self,
        data: &Value,
        _context: &ValidationContext,
    ) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
        let mut errors = Vec::new();

        // Check if there's content in the response
        if let Some(result) = data.get("result") {
            if let Some(content) = result.get("content") {
                if let Some(content_array) = content.as_array() {
                    for (i, item) in content_array.iter().enumerate() {
                        if let Some(text) = item.get("text") {
                            if let Some(text_str) = text.as_str() {
                                if text_str.len() > self.max_length {
                                    errors.push(ValidationError::FieldError {
                                        field: format!("result.content[{i}].text"),
                                        expected: format!("length <= {}", self.max_length),
                                        actual: format!("length = {}", text_str.len()),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(errors)
    }
}

/// Example: Response time validator
pub struct ResponseTimeValidator {
    max_duration_ms: u64,
}

impl ResponseTimeValidator {
    pub fn new(max_duration_ms: u64) -> Self {
        Self { max_duration_ms }
    }
}

impl CustomValidator for ResponseTimeValidator {
    fn name(&self) -> &str {
        "response_time"
    }

    fn description(&self) -> &str {
        "Validates that response time does not exceed maximum"
    }

    fn validate(
        &self,
        _data: &Value,
        context: &ValidationContext,
    ) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
        let mut errors = Vec::new();

        // Check for duration in metadata
        if let Some(duration_str) = context.test_metadata.get("response_duration_ms") {
            if let Ok(duration) = duration_str.parse::<u64>() {
                if duration > self.max_duration_ms {
                    errors.push(ValidationError::FieldError {
                        field: "response_time".to_string(),
                        expected: format!("<= {} ms", self.max_duration_ms),
                        actual: format!("{duration} ms"),
                    });
                }
            }
        }

        Ok(errors)
    }
}

/// Example: Security validator for sensitive data
pub struct SecurityValidator {
    forbidden_patterns: Vec<String>,
}

impl SecurityValidator {
    pub fn new(forbidden_patterns: Vec<String>) -> Self {
        Self { forbidden_patterns }
    }
}

impl CustomValidator for SecurityValidator {
    fn name(&self) -> &str {
        "security"
    }

    fn description(&self) -> &str {
        "Validates that response does not contain sensitive data patterns"
    }

    fn validate(
        &self,
        data: &Value,
        _context: &ValidationContext,
    ) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
        let mut errors = Vec::new();

        // Convert entire response to string for pattern matching
        let data_str = data.to_string();

        for pattern in &self.forbidden_patterns {
            if data_str.contains(pattern) {
                errors.push(ValidationError::FieldError {
                    field: "response_content".to_string(),
                    expected: format!("no occurrences of '{pattern}'"),
                    actual: "contains forbidden pattern".to_string(),
                });
            }
        }

        Ok(errors)
    }
}

/// Example: Business rule validator
pub struct BusinessRuleValidator {
    rules: Vec<BusinessRule>,
}

#[derive(Debug, Clone)]
pub struct BusinessRule {
    pub name: String,
    pub condition: String, // JSONPath expression
    pub expected_value: Value,
    pub error_message: String,
}

impl BusinessRuleValidator {
    pub fn new(rules: Vec<BusinessRule>) -> Self {
        Self { rules }
    }
}

impl CustomValidator for BusinessRuleValidator {
    fn name(&self) -> &str {
        "business_rules"
    }

    fn description(&self) -> &str {
        "Validates business-specific rules and constraints"
    }

    fn validate(
        &self,
        data: &Value,
        _context: &ValidationContext,
    ) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
        let mut errors = Vec::new();

        // Simple rule evaluation (would use JSONPath library in practice)
        for rule in &self.rules {
            // For demo purposes, just check if a field equals expected value
            if rule.condition.starts_with("$.") {
                let field_path = &rule.condition[2..]; // Remove "$."

                if let Some(actual_value) = self.get_nested_value(data, field_path) {
                    if actual_value != rule.expected_value {
                        errors.push(ValidationError::FieldError {
                            field: rule.condition.clone(),
                            expected: rule.expected_value.to_string(),
                            actual: actual_value.to_string(),
                        });
                    }
                } else {
                    errors.push(ValidationError::MissingFieldError {
                        field: rule.condition.clone(),
                    });
                }
            }
        }

        Ok(errors)
    }
}

impl BusinessRuleValidator {
    fn get_nested_value(&self, data: &Value, path: &str) -> Option<Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = data;

        for part in parts {
            current = current.get(part)?;
        }

        Some(current.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_content_length_validator() {
        let validator = ContentLengthValidator::new(100);

        let context = ValidationContext {
            method: "tools/call".to_string(),
            request_id: Some(json!(1)),
            server_capabilities: None,
            test_metadata: HashMap::new(),
        };

        // Test valid content
        let valid_data = json!({
            "result": {
                "content": [{"text": "Short text"}]
            }
        });

        let errors = validator.validate(&valid_data, &context).unwrap();
        assert!(errors.is_empty(), "Should be empty for valid content");

        // Test invalid content (too long)
        let long_text = "a".repeat(150);
        let invalid_data = json!({
            "result": {
                "content": [{"text": long_text}]
            }
        });

        let errors = validator.validate(&invalid_data, &context).unwrap();
        assert!(!errors.is_empty(), "Should not be empty");
        assert!(matches!(errors[0], ValidationError::FieldError { .. }));
    }

    #[test]
    fn test_response_time_validator() {
        let validator = ResponseTimeValidator::new(1000); // 1 second max

        let mut context = ValidationContext {
            method: "tools/call".to_string(),
            request_id: Some(json!(1)),
            server_capabilities: None,
            test_metadata: HashMap::new(),
        };

        // Test fast response
        context
            .test_metadata
            .insert("response_duration_ms".to_string(), "500".to_string());

        let data = json!({"result": {"status": "success"}});
        let errors = validator.validate(&data, &context).unwrap();
        assert!(errors.is_empty(), "Should be empty for fast response");

        // Test slow response
        context
            .test_metadata
            .insert("response_duration_ms".to_string(), "2000".to_string());

        let errors = validator.validate(&data, &context).unwrap();
        assert!(!errors.is_empty(), "Should not be empty");
        assert!(matches!(errors[0], ValidationError::FieldError { .. }));
    }

    #[test]
    fn test_security_validator() {
        let forbidden_patterns = vec![
            "password".to_string(),
            "secret".to_string(),
            "token".to_string(),
        ];

        let validator = SecurityValidator::new(forbidden_patterns);

        let context = ValidationContext {
            method: "tools/call".to_string(),
            request_id: Some(json!(1)),
            server_capabilities: None,
            test_metadata: HashMap::new(),
        };

        // Test safe content
        let safe_data = json!({
            "result": {
                "content": [{"text": "Safe analysis result"}]
            }
        });

        let errors = validator.validate(&safe_data, &context).unwrap();
        assert!(errors.is_empty(), "Should be empty for safe content");

        // Test content with forbidden pattern
        let unsafe_data = json!({
            "result": {
                "content": [{"text": "User password is 123456"}]
            }
        });

        let errors = validator.validate(&unsafe_data, &context).unwrap();
        assert!(!errors.is_empty(), "Should not be empty");
        assert!(matches!(errors[0], ValidationError::FieldError { .. }));
    }

    #[test]
    fn test_business_rule_validator() {
        let rules = vec![BusinessRule {
            name: "status_check".to_string(),
            condition: "$.result.status".to_string(),
            expected_value: json!("success"),
            error_message: "Status must be success".to_string(),
        }];

        let validator = BusinessRuleValidator::new(rules);

        let context = ValidationContext {
            method: "tools/call".to_string(),
            request_id: Some(json!(1)),
            server_capabilities: None,
            test_metadata: HashMap::new(),
        };

        // Test valid data
        let valid_data = json!({
            "result": {
                "status": "success"
            }
        });

        let errors = validator.validate(&valid_data, &context).unwrap();
        assert!(
            errors.is_empty(),
            "Should be empty for valid business rules"
        );

        // Test invalid data
        let invalid_data = json!({
            "result": {
                "status": "failure"
            }
        });

        let errors = validator.validate(&invalid_data, &context).unwrap();
        assert!(!errors.is_empty(), "Should not be empty");
        assert!(matches!(errors[0], ValidationError::FieldError { .. }));
    }

    #[test]
    fn test_validator_metadata() {
        let validator = ContentLengthValidator::new(100);

        assert_eq!(validator.name(), "content_length");
        assert!(validator.description().contains("content length"));
        assert!(validator.is_enabled());
    }

    #[test]
    fn test_validation_context_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("test_id".to_string(), "test_123".to_string());

        let context = ValidationContext {
            method: "tools/call".to_string(),
            request_id: Some(json!(42)),
            server_capabilities: Some(json!({"tools": {}})),
            test_metadata: metadata,
        };

        assert_eq!(context.method, "tools/call");
        assert_eq!(context.request_id, Some(json!(42)));
        assert!(context.server_capabilities.is_some(), "Should have value");
        assert_eq!(
            context.test_metadata.get("test_id"),
            Some(&"test_123".to_string())
        );
    }
}
