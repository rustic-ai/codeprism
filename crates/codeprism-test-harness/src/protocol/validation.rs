//! MCP Protocol Validation
//!
//! This module provides validation utilities for MCP protocol compliance,
//! message validation, and capability verification.

use super::{
    capabilities::{CapabilityValidation, McpCapabilities, ServerCapabilities},
    jsonrpc::JsonRpcMessage,
    McpConfig,
};

/// Protocol compliance validator
pub struct ProtocolValidator {
    config: McpConfig,
}

/// Validation result for protocol compliance
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

impl ProtocolValidator {
    /// Create a new protocol validator
    pub fn new(config: McpConfig) -> Self {
        Self { config }
    }

    /// Validate a JSON-RPC message for compliance
    pub fn validate_message(&self, message: &JsonRpcMessage) -> ValidationResult {
        let mut result = ValidationResult {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Validate JSON-RPC 2.0 compliance if strict mode is enabled
        if self.config.validation.strict_json_rpc {
            if let Err(error) = message.validate() {
                result.valid = false;
                result
                    .errors
                    .push(format!("JSON-RPC validation failed: {}", error));
            }
        }

        // Validate message structure
        if message.jsonrpc != "2.0" {
            result.valid = false;
            result.errors.push("Invalid JSON-RPC version".to_string());
        }

        result
    }

    /// Validate capabilities compatibility
    pub fn validate_capabilities(
        &self,
        client_caps: &McpCapabilities,
        server_caps: &ServerCapabilities,
    ) -> CapabilityValidation {
        client_caps.validate_against_server(server_caps)
    }

    /// Validate protocol version compatibility
    pub fn validate_protocol_version(&self, server_version: &str) -> ValidationResult {
        let mut result = ValidationResult {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        if self.config.validation.check_protocol_version
            && server_version != self.config.protocol_version
        {
            result.warnings.push(format!(
                "Protocol version mismatch: client expects {}, server provides {}",
                self.config.protocol_version, server_version
            ));
        }

        result
    }

    /// Validate message schema (placeholder for future implementation)
    pub fn validate_schema(&self, _message: &JsonRpcMessage) -> ValidationResult {
        ValidationResult {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Perform comprehensive protocol validation
    pub fn comprehensive_validation(
        &self,
        message: &JsonRpcMessage,
        client_caps: Option<&McpCapabilities>,
        server_caps: Option<&ServerCapabilities>,
    ) -> ValidationResult {
        let mut result = self.validate_message(message);

        // Validate capabilities if provided
        if let (Some(client), Some(server)) = (client_caps, server_caps) {
            let cap_validation = self.validate_capabilities(client, server);
            if !cap_validation.valid {
                result.valid = false;
                result.errors.extend(cap_validation.errors);
            }
            result.warnings.extend(cap_validation.warnings);
        }

        // Schema validation
        if self.config.validation.validate_schemas {
            let schema_result = self.validate_schema(message);
            if !schema_result.valid {
                result.valid = false;
                result.errors.extend(schema_result.errors);
            }
            result.warnings.extend(schema_result.warnings);
        }

        result
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn success() -> Self {
        Self::default()
    }

    /// Create a failed validation result with an error
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            valid: false,
            errors: vec![message.into()],
            warnings: Vec::new(),
        }
    }

    /// Create a validation result with a warning
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: vec![message.into()],
        }
    }

    /// Add an error to this validation result
    pub fn add_error(&mut self, message: impl Into<String>) {
        self.valid = false;
        self.errors.push(message.into());
    }

    /// Add a warning to this validation result
    pub fn add_warning(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }

    /// Merge another validation result into this one
    pub fn merge(&mut self, other: ValidationResult) {
        if !other.valid {
            self.valid = false;
        }
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::jsonrpc::JsonRpcMessage;

    #[test]
    fn test_validation_result_creation() {
        let success = ValidationResult::success();
        assert!(success.valid);
        assert!(success.errors.is_empty());
        assert!(success.warnings.is_empty());

        let error = ValidationResult::error("Test error");
        assert!(!error.valid);
        assert_eq!(error.errors.len(), 1);
        assert!(error.warnings.is_empty());

        let warning = ValidationResult::warning("Test warning");
        assert!(warning.valid);
        assert!(warning.errors.is_empty());
        assert_eq!(warning.warnings.len(), 1);
    }

    #[test]
    fn test_validation_result_merge() {
        let mut result = ValidationResult::success();
        let error_result = ValidationResult::error("Test error");

        result.merge(error_result);
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_message_validation() {
        let config = McpConfig::default();
        let validator = ProtocolValidator::new(config);

        let valid_message = JsonRpcMessage::request("test", None);
        let result = validator.validate_message(&valid_message);
        assert!(result.valid);

        let mut invalid_message = JsonRpcMessage::request("test", None);
        invalid_message.jsonrpc = "1.0".to_string();
        let result = validator.validate_message(&invalid_message);
        assert!(!result.valid);
    }
}
