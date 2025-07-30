use super::ValidationSeverity;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// MCP protocol validator for validating protocol-specific requirements
pub struct ProtocolValidator {
    mcp_version: String,
    capabilities: McpCapabilities,
}

/// MCP protocol requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolRequirements {
    pub method: String,
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
    pub expected_error_codes: Vec<i32>,
    pub capability_requirements: Vec<String>,
}

/// Protocol validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolIssue {
    pub category: ProtocolCategory,
    pub severity: ValidationSeverity,
    pub message: String,
    pub field_path: Option<String>,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

/// Categories of protocol issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolCategory {
    JsonRpcStructure,
    McpMethodCompliance,
    CapabilityMismatch,
    FieldMissing,
    FieldType,
    FieldValue,
    ErrorHandling,
    VersionMismatch,
}

/// MCP server capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpCapabilities {
    pub tools: Option<ToolsCapability>,
    pub resources: Option<ResourcesCapability>,
    pub prompts: Option<PromptsCapability>,
    pub logging: Option<LoggingCapability>,
}

/// Tools capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    pub list_changed: Option<bool>,
}

/// Resources capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    pub subscribe: Option<bool>,
    pub list_changed: Option<bool>,
}

/// Prompts capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {
    pub list_changed: Option<bool>,
}

/// Logging capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingCapability {
    pub level: Option<String>,
}

impl ProtocolValidator {
    /// Create a new protocol validator
    pub fn new(mcp_version: String) -> Self {
        Self {
            mcp_version,
            capabilities: McpCapabilities::default(),
        }
    }

    /// Validate JSON-RPC 2.0 structure
    pub fn validate_jsonrpc_structure(&self, response: &Value) -> Vec<ProtocolIssue> {
        let mut issues = Vec::new();

        // Check for required JSON-RPC 2.0 fields
        if response.get("jsonrpc").is_none() {
            issues.push(ProtocolIssue {
                category: ProtocolCategory::JsonRpcStructure,
                severity: ValidationSeverity::Error,
                message: "Missing required 'jsonrpc' field".to_string(),
                field_path: Some("jsonrpc".to_string()),
                expected: Some("2.0".to_string()),
                actual: None,
            });
        } else if response["jsonrpc"] != "2.0" {
            issues.push(ProtocolIssue {
                category: ProtocolCategory::JsonRpcStructure,
                severity: ValidationSeverity::Error,
                message: "Invalid JSON-RPC version".to_string(),
                field_path: Some("jsonrpc".to_string()),
                expected: Some("2.0".to_string()),
                actual: Some(response["jsonrpc"].to_string()),
            });
        }

        // Check for id field (required for responses)
        if response.get("id").is_none() {
            issues.push(ProtocolIssue {
                category: ProtocolCategory::JsonRpcStructure,
                severity: ValidationSeverity::Error,
                message: "Missing required 'id' field in response".to_string(),
                field_path: Some("id".to_string()),
                expected: Some("request ID".to_string()),
                actual: None,
            });
        }

        // Must have either 'result' or 'error' but not both
        let has_result = response.get("result").is_some();
        let has_error = response.get("error").is_some();

        if !has_result && !has_error {
            issues.push(ProtocolIssue {
                category: ProtocolCategory::JsonRpcStructure,
                severity: ValidationSeverity::Error,
                message: "Response must contain either 'result' or 'error' field".to_string(),
                field_path: None,
                expected: Some("result or error".to_string()),
                actual: Some("neither".to_string()),
            });
        } else if has_result && has_error {
            issues.push(ProtocolIssue {
                category: ProtocolCategory::JsonRpcStructure,
                severity: ValidationSeverity::Error,
                message: "Response cannot contain both 'result' and 'error' fields".to_string(),
                field_path: None,
                expected: Some("result or error (not both)".to_string()),
                actual: Some("both".to_string()),
            });
        }

        issues
    }

    /// Validate MCP method-specific response
    pub fn validate_mcp_method_response(
        &self,
        method: &str,
        response: &Value,
        requirements: &ProtocolRequirements,
    ) -> Vec<ProtocolIssue> {
        let mut issues = Vec::new();

        // First validate JSON-RPC structure
        issues.extend(self.validate_jsonrpc_structure(response));

        // If there's an error, validate error structure
        if let Some(error) = response.get("error") {
            issues.extend(self.validate_error_structure(error));
            return issues; // Don't validate result if there's an error
        }

        // Validate result structure for successful responses
        if let Some(result) = response.get("result") {
            issues.extend(self.validate_method_result(method, result, requirements));
        }

        issues
    }

    /// Validate error structure
    fn validate_error_structure(&self, error: &Value) -> Vec<ProtocolIssue> {
        let mut issues = Vec::new();

        // Error must be an object
        if !error.is_object() {
            issues.push(ProtocolIssue {
                category: ProtocolCategory::ErrorHandling,
                severity: ValidationSeverity::Error,
                message: "Error field must be an object".to_string(),
                field_path: Some("error".to_string()),
                expected: Some("object".to_string()),
                actual: Some(format!("{:?}", error.as_str().unwrap_or("unknown"))),
            });
            return issues;
        }

        // Check required error fields
        if error.get("code").is_none() {
            issues.push(ProtocolIssue {
                category: ProtocolCategory::ErrorHandling,
                severity: ValidationSeverity::Error,
                message: "Error object missing required 'code' field".to_string(),
                field_path: Some("error.code".to_string()),
                expected: Some("integer".to_string()),
                actual: None,
            });
        } else if !error["code"].is_number() {
            issues.push(ProtocolIssue {
                category: ProtocolCategory::ErrorHandling,
                severity: ValidationSeverity::Error,
                message: "Error code must be a number".to_string(),
                field_path: Some("error.code".to_string()),
                expected: Some("number".to_string()),
                actual: Some(format!("{:?}", error["code"])),
            });
        }

        if error.get("message").is_none() {
            issues.push(ProtocolIssue {
                category: ProtocolCategory::ErrorHandling,
                severity: ValidationSeverity::Error,
                message: "Error object missing required 'message' field".to_string(),
                field_path: Some("error.message".to_string()),
                expected: Some("string".to_string()),
                actual: None,
            });
        } else if !error["message"].is_string() {
            issues.push(ProtocolIssue {
                category: ProtocolCategory::ErrorHandling,
                severity: ValidationSeverity::Error,
                message: "Error message must be a string".to_string(),
                field_path: Some("error.message".to_string()),
                expected: Some("string".to_string()),
                actual: Some(format!("{:?}", error["message"])),
            });
        }

        issues
    }

    /// Validate method-specific result structure
    fn validate_method_result(
        &self,
        method: &str,
        result: &Value,
        requirements: &ProtocolRequirements,
    ) -> Vec<ProtocolIssue> {
        let mut issues = Vec::new();

        // Check required fields
        for required_field in &requirements.required_fields {
            if result.get(required_field).is_none() {
                issues.push(ProtocolIssue {
                    category: ProtocolCategory::FieldMissing,
                    severity: ValidationSeverity::Error,
                    message: format!(
                        "Required field '{required_field}' missing from {method} result"
                    ),
                    field_path: Some(format!("result.{required_field}")),
                    expected: Some("present".to_string()),
                    actual: Some("missing".to_string()),
                });
            }
        }

        // Validate method-specific structures
        match method {
            "tools/list" => issues.extend(self.validate_tools_list_result(result)),
            "tools/call" => issues.extend(self.validate_tools_call_result(result)),
            "resources/list" => issues.extend(self.validate_resources_list_result(result)),
            "resources/read" => issues.extend(self.validate_resources_read_result(result)),
            "prompts/list" => issues.extend(self.validate_prompts_list_result(result)),
            "prompts/get" => issues.extend(self.validate_prompts_get_result(result)),
            _ => {
                // Unknown method - just validate generic structure
                issues.push(ProtocolIssue {
                    category: ProtocolCategory::McpMethodCompliance,
                    severity: ValidationSeverity::Warning,
                    message: format!("Unknown MCP method: {method}"),
                    field_path: None,
                    expected: Some("known MCP method".to_string()),
                    actual: Some(method.to_string()),
                });
            }
        }

        issues
    }

    /// Validate tools/list result structure
    fn validate_tools_list_result(&self, result: &Value) -> Vec<ProtocolIssue> {
        let mut issues = Vec::new();

        if let Some(tools) = result.get("tools") {
            if !tools.is_array() {
                issues.push(ProtocolIssue {
                    category: ProtocolCategory::FieldType,
                    severity: ValidationSeverity::Error,
                    message: "tools field must be an array".to_string(),
                    field_path: Some("result.tools".to_string()),
                    expected: Some("array".to_string()),
                    actual: Some(format!("{tools:?}")),
                });
            } else {
                // Validate each tool definition
                for (i, tool) in tools.as_array().unwrap().iter().enumerate() {
                    issues.extend(self.validate_tool_definition(tool, i));
                }
            }
        }

        issues
    }

    /// Validate tools/call result structure
    fn validate_tools_call_result(&self, result: &Value) -> Vec<ProtocolIssue> {
        let mut issues = Vec::new();

        // Check for content array
        if let Some(content) = result.get("content") {
            if !content.is_array() {
                issues.push(ProtocolIssue {
                    category: ProtocolCategory::FieldType,
                    severity: ValidationSeverity::Error,
                    message: "content field must be an array".to_string(),
                    field_path: Some("result.content".to_string()),
                    expected: Some("array".to_string()),
                    actual: Some(format!("{content:?}")),
                });
            }
        }

        // Check isError field
        if let Some(is_error) = result.get("isError") {
            if !is_error.is_boolean() {
                issues.push(ProtocolIssue {
                    category: ProtocolCategory::FieldType,
                    severity: ValidationSeverity::Error,
                    message: "isError field must be a boolean".to_string(),
                    field_path: Some("result.isError".to_string()),
                    expected: Some("boolean".to_string()),
                    actual: Some(format!("{is_error:?}")),
                });
            }
        }

        issues
    }

    /// Validate resources/list result structure
    fn validate_resources_list_result(&self, result: &Value) -> Vec<ProtocolIssue> {
        let mut issues = Vec::new();

        if let Some(resources) = result.get("resources") {
            if !resources.is_array() {
                issues.push(ProtocolIssue {
                    category: ProtocolCategory::FieldType,
                    severity: ValidationSeverity::Error,
                    message: "resources field must be an array".to_string(),
                    field_path: Some("result.resources".to_string()),
                    expected: Some("array".to_string()),
                    actual: Some(format!("{resources:?}")),
                });
            }
        }

        issues
    }

    /// Validate resources/read result structure
    fn validate_resources_read_result(&self, result: &Value) -> Vec<ProtocolIssue> {
        let mut issues = Vec::new();

        if let Some(contents) = result.get("contents") {
            if !contents.is_array() {
                issues.push(ProtocolIssue {
                    category: ProtocolCategory::FieldType,
                    severity: ValidationSeverity::Error,
                    message: "contents field must be an array".to_string(),
                    field_path: Some("result.contents".to_string()),
                    expected: Some("array".to_string()),
                    actual: Some(format!("{contents:?}")),
                });
            }
        }

        issues
    }

    /// Validate prompts/list result structure
    fn validate_prompts_list_result(&self, result: &Value) -> Vec<ProtocolIssue> {
        let mut issues = Vec::new();

        if let Some(prompts) = result.get("prompts") {
            if !prompts.is_array() {
                issues.push(ProtocolIssue {
                    category: ProtocolCategory::FieldType,
                    severity: ValidationSeverity::Error,
                    message: "prompts field must be an array".to_string(),
                    field_path: Some("result.prompts".to_string()),
                    expected: Some("array".to_string()),
                    actual: Some(format!("{prompts:?}")),
                });
            }
        }

        issues
    }

    /// Validate prompts/get result structure
    fn validate_prompts_get_result(&self, result: &Value) -> Vec<ProtocolIssue> {
        let mut issues = Vec::new();

        if let Some(messages) = result.get("messages") {
            if !messages.is_array() {
                issues.push(ProtocolIssue {
                    category: ProtocolCategory::FieldType,
                    severity: ValidationSeverity::Error,
                    message: "messages field must be an array".to_string(),
                    field_path: Some("result.messages".to_string()),
                    expected: Some("array".to_string()),
                    actual: Some(format!("{messages:?}")),
                });
            }
        }

        issues
    }

    /// Validate individual tool definition
    fn validate_tool_definition(&self, tool: &Value, index: usize) -> Vec<ProtocolIssue> {
        let mut issues = Vec::new();

        if !tool.is_object() {
            issues.push(ProtocolIssue {
                category: ProtocolCategory::FieldType,
                severity: ValidationSeverity::Error,
                message: format!("Tool at index {index} must be an object"),
                field_path: Some(format!("result.tools[{index}]")),
                expected: Some("object".to_string()),
                actual: Some(format!("{tool:?}")),
            });
            return issues;
        }

        // Check required tool fields
        let required_fields = ["name", "description", "inputSchema"];
        for field in &required_fields {
            if tool.get(field).is_none() {
                issues.push(ProtocolIssue {
                    category: ProtocolCategory::FieldMissing,
                    severity: ValidationSeverity::Error,
                    message: format!("Tool at index {index} missing required field '{field}'"),
                    field_path: Some(format!("result.tools[{index}].{field}")),
                    expected: Some("present".to_string()),
                    actual: Some("missing".to_string()),
                });
            }
        }

        issues
    }

    /// Validate capability announcement
    pub fn validate_capability_announcement(&self, capabilities: &Value) -> Vec<ProtocolIssue> {
        let mut issues = Vec::new();

        if !capabilities.is_object() {
            issues.push(ProtocolIssue {
                category: ProtocolCategory::CapabilityMismatch,
                severity: ValidationSeverity::Error,
                message: "Capabilities must be an object".to_string(),
                field_path: Some("capabilities".to_string()),
                expected: Some("object".to_string()),
                actual: Some(format!("{capabilities:?}")),
            });
            return issues;
        }

        // Validate known capability structures
        if let Some(tools) = capabilities.get("tools") {
            if !tools.is_object() {
                issues.push(ProtocolIssue {
                    category: ProtocolCategory::CapabilityMismatch,
                    severity: ValidationSeverity::Error,
                    message: "tools capability must be an object".to_string(),
                    field_path: Some("capabilities.tools".to_string()),
                    expected: Some("object".to_string()),
                    actual: Some(format!("{tools:?}")),
                });
            }
        }

        issues
    }

    /// Update server capabilities
    pub fn update_capabilities(&mut self, capabilities: McpCapabilities) {
        self.capabilities = capabilities;
    }

    /// Get current MCP version
    pub fn version(&self) -> &str {
        &self.mcp_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_protocol_validator_creation() {
        let validator = ProtocolValidator::new("1.0".to_string());
        assert_eq!(validator.version(), "1.0");
    }

    #[test]
    fn test_validate_jsonrpc_structure_valid() {
        let validator = ProtocolValidator::new("1.0".to_string());

        let valid_response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"status": "success"}
        });

        let issues = validator.validate_jsonrpc_structure(&valid_response);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_validate_jsonrpc_structure_missing_fields() {
        let validator = ProtocolValidator::new("1.0".to_string());

        let invalid_response = json!({
            "result": {"status": "success"}
            // Missing "jsonrpc" and "id" fields
        });

        let issues = validator.validate_jsonrpc_structure(&invalid_response);
        assert_eq!(issues.len(), 2);

        assert!(issues
            .iter()
            .any(|i| i.field_path == Some("jsonrpc".to_string())));
        assert!(issues
            .iter()
            .any(|i| i.field_path == Some("id".to_string())));
    }

    #[test]
    fn test_validate_jsonrpc_structure_both_result_and_error() {
        let validator = ProtocolValidator::new("1.0".to_string());

        let invalid_response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"status": "success"},
            "error": {"code": -1, "message": "Error"}
        });

        let issues = validator.validate_jsonrpc_structure(&invalid_response);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.message.contains("both")));
    }

    #[test]
    fn test_validate_error_structure() {
        let validator = ProtocolValidator::new("1.0".to_string());

        let error_response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        });

        let issues = validator.validate_mcp_method_response(
            "unknown/method",
            &error_response,
            &ProtocolRequirements {
                method: "unknown/method".to_string(),
                required_fields: vec![],
                optional_fields: vec![],
                expected_error_codes: vec![],
                capability_requirements: vec![],
            },
        );

        // Should not have error structure issues (but may have unknown method warning)
        assert!(!issues
            .iter()
            .any(|i| i.category == ProtocolCategory::ErrorHandling));
    }

    #[test]
    fn test_validate_tools_call_result() {
        let validator = ProtocolValidator::new("1.0".to_string());

        let response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "content": [{"type": "text", "text": "Analysis complete"}],
                "isError": false
            }
        });

        let requirements = ProtocolRequirements {
            method: "tools/call".to_string(),
            required_fields: vec!["content".to_string()],
            optional_fields: vec!["isError".to_string()],
            expected_error_codes: vec![],
            capability_requirements: vec![],
        };

        let issues = validator.validate_mcp_method_response("tools/call", &response, &requirements);

        // Should not have any critical issues for valid tools/call response
        assert!(issues
            .iter()
            .all(|i| i.severity != ValidationSeverity::Error));
    }

    #[test]
    fn test_validate_tools_list_result() {
        let validator = ProtocolValidator::new("1.0".to_string());

        let response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "tools": [
                    {
                        "name": "analyze_code",
                        "description": "Analyze code quality",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "code": {"type": "string"}
                            }
                        }
                    }
                ]
            }
        });

        let requirements = ProtocolRequirements {
            method: "tools/list".to_string(),
            required_fields: vec!["tools".to_string()],
            optional_fields: vec![],
            expected_error_codes: vec![],
            capability_requirements: vec![],
        };

        let issues = validator.validate_mcp_method_response("tools/list", &response, &requirements);

        // Should not have any critical issues for valid tools/list response
        assert!(issues
            .iter()
            .all(|i| i.severity != ValidationSeverity::Error));
    }

    #[test]
    fn test_validate_capability_announcement() {
        let validator = ProtocolValidator::new("1.0".to_string());

        let capabilities = json!({
            "tools": {
                "listChanged": true
            },
            "resources": {
                "subscribe": false,
                "listChanged": true
            }
        });

        let issues = validator.validate_capability_announcement(&capabilities);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_validate_invalid_capability_announcement() {
        let validator = ProtocolValidator::new("1.0".to_string());

        let capabilities = json!({
            "tools": "invalid_type" // Should be object
        });

        let issues = validator.validate_capability_announcement(&capabilities);
        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.category == ProtocolCategory::CapabilityMismatch));
    }
}
