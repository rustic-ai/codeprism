//! MCP Capabilities System
//!
//! This module defines the capability system for the Model Context Protocol,
//! including all standard capabilities (Resources, Tools, Prompts, Sampling)
//! and capability negotiation during client-server initialization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete MCP capabilities structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpCapabilities {
    /// Experimental capabilities
    #[serde(default)]
    pub experimental: HashMap<String, serde_json::Value>,

    /// Resources capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,

    /// Tools capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,

    /// Prompts capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,

    /// Sampling capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<SamplingCapability>,
}

/// Resources capability configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourcesCapability {
    /// Whether the client supports resource subscriptions
    #[serde(default)]
    pub subscribe: bool,

    /// Whether the client supports listChanged notifications
    #[serde(default, rename = "listChanged")]
    pub list_changed: bool,
}

/// Tools capability configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolsCapability {
    /// Whether the client supports tool list changed notifications
    #[serde(default, rename = "listChanged")]
    pub list_changed: bool,
}

/// Prompts capability configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptsCapability {
    /// Whether the client supports prompt list changed notifications
    #[serde(default, rename = "listChanged")]
    pub list_changed: bool,
}

/// Sampling capability configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SamplingCapability {
    /// Additional sampling configuration
    #[serde(flatten)]
    pub config: HashMap<String, serde_json::Value>,
}

/// Resource information from server capabilities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceInfo {
    /// URI template for the resource
    pub uri: String,

    /// Human-readable name for the resource
    pub name: String,

    /// Optional description of what the resource contains
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// MIME type of the resource if known
    #[serde(skip_serializing_if = "Option::is_none", rename = "mimeType")]
    pub mime_type: Option<String>,
}

/// Tool information from server capabilities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// JSON Schema for tool parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
}

/// Prompt information from server capabilities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptInfo {
    /// Prompt name
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Prompt arguments schema
    #[serde(default)]
    pub arguments: Vec<PromptArgument>,
}

/// Prompt argument definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptArgument {
    /// Argument name
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Whether this argument is required
    #[serde(default)]
    pub required: bool,
}

/// Server capability information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Experimental capabilities
    #[serde(default)]
    pub experimental: HashMap<String, serde_json::Value>,

    /// Logging configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<LoggingCapability>,

    /// Resources capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ServerResourcesCapability>,

    /// Tools capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ServerToolsCapability>,

    /// Prompts capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<ServerPromptsCapability>,
}

/// Server logging capability
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoggingCapability {
    /// Additional logging configuration
    #[serde(flatten)]
    pub config: HashMap<String, serde_json::Value>,
}

/// Server resources capability
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerResourcesCapability {
    /// Whether the server supports resource subscriptions
    #[serde(default)]
    pub subscribe: bool,

    /// Whether the server supports list changed notifications
    #[serde(default, rename = "listChanged")]
    pub list_changed: bool,
}

/// Server tools capability
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerToolsCapability {
    /// Whether the server supports list changed notifications
    #[serde(default, rename = "listChanged")]
    pub list_changed: bool,
}

/// Server prompts capability
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerPromptsCapability {
    /// Whether the server supports list changed notifications
    #[serde(default, rename = "listChanged")]
    pub list_changed: bool,
}

/// Capability validation results
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityValidation {
    /// Whether all capabilities are valid
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Warnings about capability mismatches
    pub warnings: Vec<String>,
}

impl Default for McpCapabilities {
    fn default() -> Self {
        Self {
            experimental: HashMap::new(),
            resources: Some(ResourcesCapability {
                subscribe: true,
                list_changed: true,
            }),
            tools: Some(ToolsCapability { list_changed: true }),
            prompts: Some(PromptsCapability { list_changed: true }),
            sampling: None, // Sampling is optional and typically server-initiated
        }
    }
}

impl Default for ResourcesCapability {
    fn default() -> Self {
        Self {
            subscribe: true,
            list_changed: true,
        }
    }
}

impl Default for ToolsCapability {
    fn default() -> Self {
        Self { list_changed: true }
    }
}

impl Default for PromptsCapability {
    fn default() -> Self {
        Self { list_changed: true }
    }
}

impl McpCapabilities {
    /// Create capabilities with all features enabled
    pub fn all() -> Self {
        Self {
            experimental: HashMap::new(),
            resources: Some(ResourcesCapability::default()),
            tools: Some(ToolsCapability::default()),
            prompts: Some(PromptsCapability::default()),
            sampling: Some(SamplingCapability {
                config: HashMap::new(),
            }),
        }
    }

    /// Create minimal capabilities (no optional features)
    pub fn minimal() -> Self {
        Self {
            experimental: HashMap::new(),
            resources: None,
            tools: None,
            prompts: None,
            sampling: None,
        }
    }

    /// Check if resources capability is enabled
    pub fn has_resources(&self) -> bool {
        self.resources.is_some()
    }

    /// Check if tools capability is enabled
    pub fn has_tools(&self) -> bool {
        self.tools.is_some()
    }

    /// Check if prompts capability is enabled
    pub fn has_prompts(&self) -> bool {
        self.prompts.is_some()
    }

    /// Check if sampling capability is enabled
    pub fn has_sampling(&self) -> bool {
        self.sampling.is_some()
    }

    /// Validate capabilities against server capabilities
    pub fn validate_against_server(
        &self,
        server_caps: &ServerCapabilities,
    ) -> CapabilityValidation {
        let mut validation = CapabilityValidation {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Validate resources capability
        if let Some(client_resources) = &self.resources {
            if let Some(server_resources) = &server_caps.resources {
                if client_resources.subscribe && !server_resources.subscribe {
                    validation.warnings.push(
                        "Client requests resource subscriptions but server doesn't support them"
                            .to_string(),
                    );
                }
                if client_resources.list_changed && !server_resources.list_changed {
                    validation.warnings.push(
                        "Client expects resource list changes but server doesn't support them"
                            .to_string(),
                    );
                }
            } else {
                validation.errors.push(
                    "Client requests resources capability but server doesn't provide it"
                        .to_string(),
                );
                validation.valid = false;
            }
        }

        // Validate tools capability
        if let Some(client_tools) = &self.tools {
            if let Some(server_tools) = &server_caps.tools {
                if client_tools.list_changed && !server_tools.list_changed {
                    validation.warnings.push(
                        "Client expects tool list changes but server doesn't support them"
                            .to_string(),
                    );
                }
            } else {
                validation.errors.push(
                    "Client requests tools capability but server doesn't provide it".to_string(),
                );
                validation.valid = false;
            }
        }

        // Validate prompts capability
        if let Some(client_prompts) = &self.prompts {
            if let Some(server_prompts) = &server_caps.prompts {
                if client_prompts.list_changed && !server_prompts.list_changed {
                    validation.warnings.push(
                        "Client expects prompt list changes but server doesn't support them"
                            .to_string(),
                    );
                }
            } else {
                validation.errors.push(
                    "Client requests prompts capability but server doesn't provide it".to_string(),
                );
                validation.valid = false;
            }
        }

        validation
    }

    /// Get a list of all requested capabilities
    pub fn requested_capabilities(&self) -> Vec<String> {
        let mut caps = Vec::new();

        if self.has_resources() {
            caps.push("resources".to_string());
        }
        if self.has_tools() {
            caps.push("tools".to_string());
        }
        if self.has_prompts() {
            caps.push("prompts".to_string());
        }
        if self.has_sampling() {
            caps.push("sampling".to_string());
        }

        for key in self.experimental.keys() {
            caps.push(format!("experimental.{}", key));
        }

        caps
    }
}

impl ServerCapabilities {
    /// Get a list of all provided capabilities
    pub fn provided_capabilities(&self) -> Vec<String> {
        let mut caps = Vec::new();

        if self.resources.is_some() {
            caps.push("resources".to_string());
        }
        if self.tools.is_some() {
            caps.push("tools".to_string());
        }
        if self.prompts.is_some() {
            caps.push("prompts".to_string());
        }
        if self.logging.is_some() {
            caps.push("logging".to_string());
        }

        for key in self.experimental.keys() {
            caps.push(format!("experimental.{}", key));
        }

        caps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_capabilities() {
        let caps = McpCapabilities::default();
        assert!(caps.has_resources());
        assert!(caps.has_tools());
        assert!(caps.has_prompts());
        assert!(!caps.has_sampling());
    }

    #[test]
    fn test_all_capabilities() {
        let caps = McpCapabilities::all();
        assert!(caps.has_resources());
        assert!(caps.has_tools());
        assert!(caps.has_prompts());
        assert!(caps.has_sampling());
    }

    #[test]
    fn test_minimal_capabilities() {
        let caps = McpCapabilities::minimal();
        assert!(!caps.has_resources());
        assert!(!caps.has_tools());
        assert!(!caps.has_prompts());
        assert!(!caps.has_sampling());
    }

    #[test]
    fn test_capability_validation() {
        let client_caps = McpCapabilities::default();
        let server_caps = ServerCapabilities {
            experimental: HashMap::new(),
            logging: None,
            resources: Some(ServerResourcesCapability {
                subscribe: true,
                list_changed: true,
            }),
            tools: Some(ServerToolsCapability {
                list_changed: false,
            }),
            prompts: None,
        };

        let validation = client_caps.validate_against_server(&server_caps);
        assert!(!validation.valid); // Should fail because prompts not supported
        assert!(!validation.errors.is_empty());
        assert!(!validation.warnings.is_empty());
    }

    #[test]
    fn test_requested_capabilities() {
        let caps = McpCapabilities::default();
        let requested = caps.requested_capabilities();
        assert!(requested.contains(&"resources".to_string()));
        assert!(requested.contains(&"tools".to_string()));
        assert!(requested.contains(&"prompts".to_string()));
        assert!(!requested.contains(&"sampling".to_string()));
    }

    #[test]
    fn test_serialization() {
        let caps = McpCapabilities::default();
        let json_str = serde_json::to_string(&caps).unwrap();
        let parsed: McpCapabilities = serde_json::from_str(&json_str).unwrap();
        assert_eq!(caps, parsed);
    }
}
