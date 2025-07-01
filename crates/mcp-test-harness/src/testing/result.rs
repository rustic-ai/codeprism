//! Test result types and utilities

use crate::types::ValidationResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Result of a single test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test name/identifier
    pub test_name: String,
    /// Test description
    pub description: Option<String>,
    /// Whether the test passed
    pub passed: bool,
    /// Test execution start time
    pub start_time: DateTime<Utc>,
    /// Test execution duration
    pub duration: Duration,
    /// Input parameters used for the test
    pub input: serde_json::Value,
    /// Server response (if any)
    pub response: Option<serde_json::Value>,
    /// Error message (if test failed)
    pub error: Option<String>,
    /// Validation results
    pub validation: ValidationResult,
    // Performance metrics removed - out of scope for current design
    /// Test tags/categories
    pub tags: Vec<String>,
}

impl TestResult {
    /// Create a new successful test result
    pub fn success(
        test_name: String,
        start_time: DateTime<Utc>,
        duration: Duration,
        input: serde_json::Value,
        response: serde_json::Value,
    ) -> Self {
        Self {
            test_name,
            description: None,
            passed: true,
            start_time,
            duration,
            input,
            response: Some(response),
            error: None,
            validation: ValidationResult::success(),
            tags: Vec::new(),
        }
    }

    /// Create a new failed test result
    pub fn failure(
        test_name: String,
        start_time: DateTime<Utc>,
        duration: Duration,
        input: serde_json::Value,
        error: String,
    ) -> Self {
        Self {
            test_name,
            description: None,
            passed: false,
            start_time,
            duration,
            input,
            response: None,
            error: Some(error),
            validation: ValidationResult::error("Test failed"),
            tags: Vec::new(),
        }
    }

    // Performance metrics functionality removed - out of scope

    /// Add tags to the result
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Add description to the result
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Get the duration in milliseconds
    pub fn duration_ms(&self) -> u128 {
        self.duration.as_millis()
    }

    // Performance metrics functionality removed - out of scope
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;
    use std::time::Duration;

    #[test]
    fn test_success_result() {
        let result = TestResult::success(
            "test_case_1".to_string(),
            Utc::now(),
            Duration::from_millis(100),
            json!({"param": "value"}),
            json!({"result": "success"}),
        );

        assert!(result.passed);
        assert!(result.error.is_none());
        assert!(result.response.is_some());
        assert_eq!(result.duration_ms(), 100);
    }

    #[test]
    fn test_failure_result() {
        let result = TestResult::failure(
            "test_case_2".to_string(),
            Utc::now(),
            Duration::from_millis(50),
            json!({"param": "invalid"}),
            "Test failed: invalid parameter".to_string(),
        );

        assert!(!result.passed);
        assert!(result.error.is_some());
        assert!(result.response.is_none());
        assert_eq!(result.duration_ms(), 50);
    }

    #[test]
    fn test_result_with_metadata() {
        let result = TestResult::success(
            "test_case_3".to_string(),
            Utc::now(),
            Duration::from_millis(200),
            json!({}),
            json!({}),
        )
        .with_description("Test description".to_string())
        .with_tags(vec!["integration".to_string(), "performance".to_string()]);

        assert_eq!(result.description, Some("Test description".to_string()));
        assert_eq!(result.tags.len(), 2);
        assert!(result.tags.contains(&"integration".to_string()));
    }
}
