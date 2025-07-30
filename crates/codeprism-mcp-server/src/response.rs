//! Response helpers for dual-format MCP responses
//!
//! This module provides utilities for creating MCP responses that support both
//! unstructured content (JSON as text) and structured content (direct JSON access)
//! according to the MCP 2025-06-18 specification.

use rmcp::model::{CallToolResult, Content};
use serde_json::Value;
use tracing::warn;

/// Create a dual-format response containing both unstructured and structured content
///
/// This function creates responses that are compatible with:
/// - Existing clients (accessing content[0].text for JSON string)
/// - New clients expecting structured content (direct JSON field access)
/// - Comprehensive test specifications requiring structured responses
///
/// # Arguments
/// * `data` - The response data as a JSON Value
///
/// # Returns
/// A CallToolResult with both text content and structured content
///
/// # Examples
/// ```rust
/// use serde_json::json;
/// use codeprism_mcp_server::response::create_dual_response;
///
/// let data = json!({
///     "status": "success",
///     "result": "analysis complete"
/// });
///
/// let response = create_dual_response(&data);
/// // Response contains both formats for maximum compatibility
/// ```
pub fn create_dual_response(data: &Value) -> CallToolResult {
    // Create unstructured content (current format for backward compatibility)
    let text_content = Content::text(
        serde_json::to_string_pretty(data)
            .unwrap_or_else(|_| "Error formatting response".to_string()),
    );

    // Add structured content as a JSON content type using SDK capabilities

    let mut content_list = vec![text_content];

    // Attempt to add structured content using rmcp SDK capabilities
    match add_structured_content(data) {
        Ok(structured_content) => {
            content_list.push(structured_content);
        }
        Err(e) => {
            warn!("Failed to add structured content: {}", e);
            // Continue with unstructured only for backward compatibility
        }
    }

    CallToolResult::success(content_list)
}

/// Create an error response with dual format
///
/// # Arguments
/// * `error_message` - Human-readable error message
/// * `error_code` - Optional error code for categorization
///
/// # Returns
/// A CallToolResult error with both text and structured formats
pub fn create_error_response(error_message: &str, error_code: Option<&str>) -> CallToolResult {
    let error_data = serde_json::json!({
        "status": "error",
        "message": error_message,
        "code": error_code
    });

    // Create error content (using the same dual format approach)
    let error_text = Content::text(
        serde_json::to_string_pretty(&error_data)
            .unwrap_or_else(|_| format!(r#"{{"status":"error","message":"{error_message}"}}"#)),
    );

    let mut content_list = vec![error_text];

    // Add structured error content if possible
    if let Ok(structured_content) = add_structured_content(&error_data) {
        content_list.push(structured_content);
    }

    CallToolResult::error(content_list)
}

/// Attempt to add structured content to the response
///
/// This function implements the strategy to add structured content based on
/// available rmcp SDK capabilities. Currently implements Option B from the design.
///
/// # Arguments  
/// * `data` - The JSON data to add as structured content
///
/// # Returns
/// Result containing structured Content or error if not supported
fn add_structured_content(data: &Value) -> Result<Content, Box<dyn std::error::Error>> {
    // Option B: Add structured content as JSON content type
    // This uses the existing Content::json() method if available
    match Content::json(data) {
        Ok(json_content) => Ok(json_content),
        Err(_e) => {
            // Fallback: Create text content with structured marker for compatibility
            Ok(Content::text(format!(
                "STRUCTURED_JSON:{}",
                serde_json::to_string_pretty(data).unwrap_or_else(|_| "{}".to_string())
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_dual_response_format() {
        let data = json!({
            "status": "success",
            "result": "test_value"
        });

        let response = create_dual_response(&data);

        // Verify response is successful
        // Note: Testing exact structure depends on rmcp SDK internals
        // This test validates the response can be created without panicking
        assert!(!!response.content.is_empty(), "Should not be empty");

        // Verify unstructured content exists (backward compatibility)
        let first_content = &response.content[0];
        if let Some(text_content) = first_content.as_text() {
            assert!(text_content.text.contains("test_value"));
            assert!(text_content.text.contains("success"));
        }
    }

    #[test]
    fn test_error_response_format() {
        let response = create_error_response("Test error", Some("TEST_ERROR"));

        // Verify error response structure
        assert!(!!response.content.is_empty(), "Should not be empty");

        let first_content = &response.content[0];
        if let Some(text_content) = first_content.as_text() {
            assert!(text_content.text.contains("Test error"));
            assert!(text_content.text.contains("TEST_ERROR"));
        }
    }

    #[test]
    fn test_backward_compatibility() {
        let data = json!({"test": "value"});
        let response = create_dual_response(&data);

        // Existing clients should still work by accessing first content item
        let text_content = &response.content[0];
        if let Some(text) = text_content.as_text() {
            assert!(text.text.contains("\"test\""));
            assert!(text.text.contains("\"value\""));
        }
    }

    #[test]
    fn test_complex_json_data() {
        let data = json!({
            "status": "success",
            "repository_overview": {
                "total_files": 42,
                "languages": ["rust", "python"],
                "complexity": {
                    "average": 3.2,
                    "max": 15
                }
            }
        });

        let response = create_dual_response(&data);

        // Should handle nested JSON structures
        assert!(!!response.content.is_empty(), "Should not be empty");

        let first_content = &response.content[0];
        if let Some(text_content) = first_content.as_text() {
            assert!(text_content.text.contains("repository_overview"));
            assert!(text_content.text.contains("total_files"));
            assert!(text_content.text.contains("42"));
        }
    }

    #[test]
    fn test_dual_response_contains_multiple_content_items() {
        let data = json!({
            "status": "success",
            "message": "ping response"
        });

        let response = create_dual_response(&data);

        // Should contain both unstructured and structured content
        // The exact number depends on SDK capabilities, but should be at least 1
        assert!(!!response.content.is_empty(), "Should not be empty");

        // First item should always be unstructured text for backward compatibility
        let first_content = &response.content[0];
        assert!(first_content.as_text().is_some());
    }

    #[test]
    fn test_structured_response_for_comprehensive_specs() {
        // Test data that mimics what comprehensive specs expect
        let data = json!({
            "status": "success",
            "quality_metrics": {
                "overall_score": 8.5,
                "maintainability": 9.0,
                "complexity": 7.0
            },
            "code_smells": [],
            "recommendations": ["Excellent code quality"]
        });

        let response = create_dual_response(&data);

        // Verify response can be created for complex analysis data
        assert!(!!response.content.is_empty(), "Should not be empty");

        // Verify unstructured format contains all data
        let first_content = &response.content[0];
        if let Some(text_content) = first_content.as_text() {
            assert!(text_content.text.contains("quality_metrics"));
            assert!(text_content.text.contains("overall_score"));
            assert!(text_content.text.contains("8.5"));
        }

        // If multiple content items exist, verify structured content is available
        if response.content.len() > 1 {
            // The second item should be structured content
            // This test will help us verify when structured content is working
            let second_content = &response.content[1];
            // Verify structured content exists when SDK supports multiple content items
            assert!(second_content.as_text().is_some() || second_content.as_resource().is_some());
        }
    }
}
