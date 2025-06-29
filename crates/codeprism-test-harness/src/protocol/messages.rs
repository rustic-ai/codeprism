//! MCP Protocol Messages
//!
//! This module defines all standard MCP protocol messages according to the
//! Model Context Protocol specification, including initialization, capability
//! discovery, and all capability-specific messages.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::capabilities::{McpCapabilities, ServerCapabilities};

/// Standard MCP method names
#[derive(Debug, Clone, PartialEq)]
pub enum McpMethod {
    // Initialization
    Initialize,
    Initialized,

    // Resources
    ResourcesList,
    ResourcesRead,
    ResourcesSubscribe,
    ResourcesUnsubscribe,
    ResourcesListChanged,

    // Tools
    ToolsList,
    ToolsCall,
    ToolsListChanged,

    // Prompts
    PromptsList,
    PromptsGet,
    PromptsListChanged,

    // Sampling
    SamplingCreateMessage,

    // Logging
    LoggingSetLevel,

    // Notifications
    NotificationsCancelled,
    NotificationsProgress,

    // Custom/Extension
    Custom(String),
}

impl McpMethod {
    /// Get the string representation of the method
    pub fn as_str(&self) -> &str {
        match self {
            Self::Initialize => "initialize",
            Self::Initialized => "initialized",
            Self::ResourcesList => "resources/list",
            Self::ResourcesRead => "resources/read",
            Self::ResourcesSubscribe => "resources/subscribe",
            Self::ResourcesUnsubscribe => "resources/unsubscribe",
            Self::ResourcesListChanged => "notifications/resources/list_changed",
            Self::ToolsList => "tools/list",
            Self::ToolsCall => "tools/call",
            Self::ToolsListChanged => "notifications/tools/list_changed",
            Self::PromptsList => "prompts/list",
            Self::PromptsGet => "prompts/get",
            Self::PromptsListChanged => "notifications/prompts/list_changed",
            Self::SamplingCreateMessage => "sampling/createMessage",
            Self::LoggingSetLevel => "logging/setLevel",
            Self::NotificationsCancelled => "notifications/cancelled",
            Self::NotificationsProgress => "notifications/progress",
            Self::Custom(method) => method,
        }
    }
}

impl From<&str> for McpMethod {
    fn from(method: &str) -> Self {
        match method {
            "initialize" => Self::Initialize,
            "initialized" => Self::Initialized,
            "resources/list" => Self::ResourcesList,
            "resources/read" => Self::ResourcesRead,
            "resources/subscribe" => Self::ResourcesSubscribe,
            "resources/unsubscribe" => Self::ResourcesUnsubscribe,
            "notifications/resources/list_changed" => Self::ResourcesListChanged,
            "tools/list" => Self::ToolsList,
            "tools/call" => Self::ToolsCall,
            "notifications/tools/list_changed" => Self::ToolsListChanged,
            "prompts/list" => Self::PromptsList,
            "prompts/get" => Self::PromptsGet,
            "notifications/prompts/list_changed" => Self::PromptsListChanged,
            "sampling/createMessage" => Self::SamplingCreateMessage,
            "logging/setLevel" => Self::LoggingSetLevel,
            "notifications/cancelled" => Self::NotificationsCancelled,
            "notifications/progress" => Self::NotificationsProgress,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// Initialize request parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitializeParams {
    /// Protocol version
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,

    /// Client capabilities
    pub capabilities: McpCapabilities,

    /// Client information
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

/// Initialize response result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitializeResult {
    /// Protocol version supported by server
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,

    /// Server capabilities
    pub capabilities: ServerCapabilities,

    /// Server information
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

/// Client information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client name
    pub name: String,

    /// Client version
    pub version: String,
}

/// Server information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name
    pub name: String,

    /// Server version
    pub version: String,
}

/// Resources list response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourcesListResult {
    /// List of available resources
    pub resources: Vec<ResourceInfo>,
}

/// Resource information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceInfo {
    /// Resource URI
    pub uri: String,

    /// Human-readable name
    pub name: String,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none", rename = "mimeType")]
    pub mime_type: Option<String>,
}

/// Resources read request parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourcesReadParams {
    /// Resource URI to read
    pub uri: String,
}

/// Resources read response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourcesReadResult {
    /// Resource contents
    pub contents: Vec<ResourceContent>,
}

/// Resource content
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceContent {
    /// Resource URI
    pub uri: String,

    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none", rename = "mimeType")]
    pub mime_type: Option<String>,

    /// Text content (for text resources)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// Binary content (base64 encoded, for binary resources)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

/// Tools list response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolsListResult {
    /// List of available tools
    pub tools: Vec<ToolInfo>,
}

/// Tool information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// JSON Schema for input parameters
    #[serde(skip_serializing_if = "Option::is_none", rename = "inputSchema")]
    pub input_schema: Option<Value>,
}

/// Tools call request parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolsCallParams {
    /// Tool name to call
    pub name: String,

    /// Tool arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<String, Value>>,
}

/// Tools call response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolsCallResult {
    /// Tool call result
    pub content: Vec<ToolContent>,

    /// Whether the call was successful
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Tool content result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text {
        /// Text content
        text: String,
    },
    #[serde(rename = "image")]
    Image {
        /// Image data (base64 encoded)
        data: String,
        /// MIME type
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    #[serde(rename = "resource")]
    Resource {
        /// Resource reference
        resource: ResourceReference,
    },
}

/// Resource reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceReference {
    /// Resource URI
    pub uri: String,

    /// Optional resource type
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,
}

/// Prompts list response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptsListResult {
    /// List of available prompts
    pub prompts: Vec<PromptInfo>,
}

/// Prompt information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptInfo {
    /// Prompt name
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Prompt arguments
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

    /// Whether the argument is required
    #[serde(default)]
    pub required: bool,
}

/// Prompts get request parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptsGetParams {
    /// Prompt name
    pub name: String,

    /// Prompt arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<String, String>>,
}

/// Prompts get response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptsGetResult {
    /// Prompt description
    pub description: String,

    /// Prompt messages
    pub messages: Vec<PromptMessage>,
}

/// Prompt message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptMessage {
    /// Message role
    pub role: String,

    /// Message content
    pub content: PromptContent,
}

/// Prompt content
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PromptContent {
    #[serde(rename = "text")]
    Text {
        /// Text content
        text: String,
    },
    #[serde(rename = "image")]
    Image {
        /// Image data (base64 encoded)
        data: String,
        /// MIME type
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    #[serde(rename = "resource")]
    Resource {
        /// Resource reference
        resource: ResourceReference,
    },
}

/// Sampling create message request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SamplingCreateMessageParams {
    /// Messages to send to the LLM
    pub messages: Vec<SamplingMessage>,

    /// Optional model preferences
    #[serde(skip_serializing_if = "Option::is_none", rename = "modelPreferences")]
    pub model_preferences: Option<ModelPreferences>,

    /// System prompt
    #[serde(skip_serializing_if = "Option::is_none", rename = "systemPrompt")]
    pub system_prompt: Option<String>,

    /// Whether to include context
    #[serde(skip_serializing_if = "Option::is_none", rename = "includeContext")]
    pub include_context: Option<String>,

    /// Maximum tokens
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxTokens")]
    pub max_tokens: Option<u32>,

    /// Sampling parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

/// Sampling message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SamplingMessage {
    /// Message role
    pub role: String,

    /// Message content
    pub content: SamplingContent,
}

/// Sampling content
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SamplingContent {
    #[serde(rename = "text")]
    Text {
        /// Text content
        text: String,
    },
    #[serde(rename = "image")]
    Image {
        /// Image data (base64 encoded)
        data: String,
        /// MIME type
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
}

/// Model preferences for sampling
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelPreferences {
    /// Preferred model hints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<Vec<ModelHint>>,

    /// Cost priority
    #[serde(skip_serializing_if = "Option::is_none", rename = "costPriority")]
    pub cost_priority: Option<f64>,

    /// Speed priority
    #[serde(skip_serializing_if = "Option::is_none", rename = "speedPriority")]
    pub speed_priority: Option<f64>,

    /// Intelligence priority
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "intelligencePriority"
    )]
    pub intelligence_priority: Option<f64>,
}

/// Model hint
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelHint {
    /// Hint name
    pub name: String,
}

/// Sampling create message response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SamplingCreateMessageResult {
    /// Role of the response
    pub role: String,

    /// Response content
    pub content: SamplingContent,

    /// Model used for generation
    pub model: String,

    /// Stop reason
    #[serde(skip_serializing_if = "Option::is_none", rename = "stopReason")]
    pub stop_reason: Option<String>,
}

/// Progress notification parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressParams {
    /// Progress token from the original request
    #[serde(rename = "progressToken")]
    pub progress_token: String,

    /// Current progress (0.0 to 1.0)
    pub progress: f64,

    /// Optional progress total
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}

/// Cancellation notification parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelledParams {
    /// Request ID that was cancelled
    #[serde(rename = "requestId")]
    pub request_id: String,

    /// Cancellation reason
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_conversion() {
        assert_eq!(McpMethod::Initialize.as_str(), "initialize");
        assert_eq!(McpMethod::ResourcesList.as_str(), "resources/list");
        assert_eq!(McpMethod::ToolsCall.as_str(), "tools/call");

        let method: McpMethod = "initialize".into();
        assert_eq!(method, McpMethod::Initialize);

        let custom: McpMethod = "custom/method".into();
        assert_eq!(custom, McpMethod::Custom("custom/method".to_string()));
    }

    #[test]
    fn test_initialize_params_serialization() {
        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: McpCapabilities::default(),
            client_info: ClientInfo {
                name: "Test Client".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        let json_str = serde_json::to_string(&params).unwrap();
        let parsed: InitializeParams = serde_json::from_str(&json_str).unwrap();
        assert_eq!(params, parsed);
    }

    #[test]
    fn test_tool_content_variants() {
        let text_content = ToolContent::Text {
            text: "Hello, world!".to_string(),
        };

        let image_content = ToolContent::Image {
            data: "base64encodeddata".to_string(),
            mime_type: "image/png".to_string(),
        };

        let resource_content = ToolContent::Resource {
            resource: ResourceReference {
                uri: "file://test.txt".to_string(),
                resource_type: Some("text".to_string()),
            },
        };

        // Test serialization
        assert!(serde_json::to_string(&text_content).is_ok());
        assert!(serde_json::to_string(&image_content).is_ok());
        assert!(serde_json::to_string(&resource_content).is_ok());
    }

    #[test]
    fn test_resources_read_result() {
        let result = ResourcesReadResult {
            contents: vec![
                ResourceContent {
                    uri: "file://test.txt".to_string(),
                    mime_type: Some("text/plain".to_string()),
                    text: Some("Hello, world!".to_string()),
                    blob: None,
                },
                ResourceContent {
                    uri: "file://image.png".to_string(),
                    mime_type: Some("image/png".to_string()),
                    text: None,
                    blob: Some("base64data".to_string()),
                },
            ],
        };

        let json_str = serde_json::to_string(&result).unwrap();
        let parsed: ResourcesReadResult = serde_json::from_str(&json_str).unwrap();
        assert_eq!(result, parsed);
    }
}
