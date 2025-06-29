//! MCP Protocol Messages
//!
//! This module defines all standard MCP protocol messages according to the
//! Model Context Protocol specification, including initialization, capability
//! discovery, and all capability-specific messages.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::capabilities::{McpCapabilities, ServerCapabilities};

/// MCP protocol methods for different capabilities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpMethod {
    // Core protocol
    #[serde(rename = "initialize")]
    Initialize,
    #[serde(rename = "initialized")]
    Initialized,
    #[serde(rename = "ping")]
    Ping,

    // Resources capability
    #[serde(rename = "resources/list")]
    ResourcesList,
    #[serde(rename = "resources/read")]
    ResourcesRead,
    #[serde(rename = "resources/subscribe")]
    ResourcesSubscribe,
    #[serde(rename = "resources/unsubscribe")]
    ResourcesUnsubscribe,
    #[serde(rename = "notifications/resources/list_changed")]
    ResourcesListChanged,
    #[serde(rename = "notifications/resources/updated")]
    ResourcesUpdated,

    // Tools capability
    #[serde(rename = "tools/list")]
    ToolsList,
    #[serde(rename = "tools/call")]
    ToolsCall,

    // Prompts capability
    #[serde(rename = "prompts/list")]
    PromptsList,
    #[serde(rename = "prompts/get")]
    PromptsGet,

    // Sampling capability
    #[serde(rename = "sampling/create_message")]
    SamplingCreateMessage,

    // Other method (for unknown methods)
    #[serde(untagged)]
    Other(String),
}

impl McpMethod {
    /// Get the string representation of the method
    pub fn as_str(&self) -> &str {
        match self {
            Self::Initialize => "initialize",
            Self::Initialized => "initialized",
            Self::Ping => "ping",
            Self::ResourcesList => "resources/list",
            Self::ResourcesRead => "resources/read",
            Self::ResourcesSubscribe => "resources/subscribe",
            Self::ResourcesUnsubscribe => "resources/unsubscribe",
            Self::ResourcesListChanged => "notifications/resources/list_changed",
            Self::ResourcesUpdated => "notifications/resources/updated",
            Self::ToolsList => "tools/list",
            Self::ToolsCall => "tools/call",
            Self::PromptsList => "prompts/list",
            Self::PromptsGet => "prompts/get",
            Self::SamplingCreateMessage => "sampling/createMessage",
            Self::Other(method) => method,
        }
    }
}

impl From<&str> for McpMethod {
    fn from(method: &str) -> Self {
        match method {
            "initialize" => Self::Initialize,
            "initialized" => Self::Initialized,
            "ping" => Self::Ping,
            "resources/list" => Self::ResourcesList,
            "resources/read" => Self::ResourcesRead,
            "resources/subscribe" => Self::ResourcesSubscribe,
            "resources/unsubscribe" => Self::ResourcesUnsubscribe,
            "notifications/resources/list_changed" => Self::ResourcesListChanged,
            "notifications/resources/updated" => Self::ResourcesUpdated,
            "tools/list" => Self::ToolsList,
            "tools/call" => Self::ToolsCall,
            "prompts/list" => Self::PromptsList,
            "prompts/get" => Self::PromptsGet,
            "sampling/createMessage" => Self::SamplingCreateMessage,
            other => Self::Other(other.to_string()),
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

/// Resource metadata as returned by resources/list
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    /// The URI of this resource
    pub uri: String,

    /// A human-readable name for this resource
    pub name: String,

    /// A description of what this resource contains
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The MIME type of this resource, if known
    #[serde(skip_serializing_if = "Option::is_none", rename = "mimeType")]
    pub mime_type: Option<String>,
}

/// Parameters for resources/list request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListResourcesParams {
    /// Optional cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Response from resources/list
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListResourcesResult {
    /// List of available resources
    pub resources: Vec<Resource>,

    /// Optional cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none", rename = "nextCursor")]
    pub next_cursor: Option<String>,
}

/// Parameters for resources/read request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadResourceParams {
    /// The URI of the resource to read
    pub uri: String,
}

/// Resource content for resources/read response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceContents {
    /// The URI of this resource
    pub uri: String,

    /// The MIME type of this resource
    #[serde(rename = "mimeType")]
    pub mime_type: String,

    /// The text content of the resource (if text-based)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// The base64-encoded binary content (if binary)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

/// Response from resources/read
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadResourceResult {
    /// List of resource contents (usually one item)
    pub contents: Vec<ResourceContents>,
}

/// Parameters for resources/subscribe request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubscribeResourceParams {
    /// The URI of the resource to subscribe to
    pub uri: String,
}

/// Response from resources/subscribe (empty on success)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubscribeResourceResult {}

/// Parameters for resources/unsubscribe request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnsubscribeResourceParams {
    /// The URI of the resource to unsubscribe from
    pub uri: String,
}

/// Response from resources/unsubscribe (empty on success)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnsubscribeResourceResult {}

/// Notification when resource list changes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceListChangedNotification {
    /// Optional metadata about what changed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
}

/// Notification when a subscribed resource is updated
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceUpdatedNotification {
    /// The URI of the updated resource
    pub uri: String,

    /// Optional metadata about the update
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
}

/// Tool input schema definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolInputSchema {
    /// Schema type (typically "object")
    #[serde(rename = "type")]
    pub schema_type: String,

    /// Schema properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Value>,

    /// Required properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    /// Additional schema properties
    #[serde(flatten)]
    pub additional: HashMap<String, Value>,
}

/// Tool metadata as returned by tools/list
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tool {
    /// The name of the tool
    pub name: String,

    /// A description of what this tool does
    pub description: String,

    /// JSON schema for the tool's input
    #[serde(rename = "inputSchema")]
    pub input_schema: ToolInputSchema,
}

/// Parameters for tools/list request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListToolsParams {
    /// Optional cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Response from tools/list
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListToolsResult {
    /// List of available tools
    pub tools: Vec<Tool>,

    /// Optional cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none", rename = "nextCursor")]
    pub next_cursor: Option<String>,
}

/// Parameters for tools/call request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallToolParams {
    /// The name of the tool to call
    pub name: String,

    /// Arguments to pass to the tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
}

/// Tool call result content
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallContent {
    /// Content type
    #[serde(rename = "type")]
    pub content_type: String,

    /// Text content (for text type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// Additional content properties
    #[serde(flatten)]
    pub additional: HashMap<String, Value>,
}

/// Response from tools/call
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallToolResult {
    /// The result content
    pub content: Vec<ToolCallContent>,

    /// Whether the tool completed successfully
    #[serde(skip_serializing_if = "Option::is_none", rename = "isError")]
    pub is_error: Option<bool>,
}

/// Prompt metadata as returned by prompts/list
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prompt {
    /// The name of the prompt
    pub name: String,

    /// A description of what this prompt does
    pub description: String,

    /// List of arguments this prompt accepts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

/// Parameters for prompts/list request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListPromptsParams {
    /// Optional cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Response from prompts/list
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListPromptsResult {
    /// List of available prompts
    pub prompts: Vec<Prompt>,

    /// Optional cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none", rename = "nextCursor")]
    pub next_cursor: Option<String>,
}

/// Parameters for prompts/get request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetPromptParams {
    /// The name of the prompt to retrieve
    pub name: String,

    /// Arguments to pass to the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<String, String>>,
}

/// Response from prompts/get
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetPromptResult {
    /// Description of the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// List of messages in the prompt
    pub messages: Vec<PromptMessage>,
}

/// Sampling request parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateMessageParams {
    /// Messages to include in sampling
    pub messages: Vec<SamplingMessage>,

    /// Model preferences
    #[serde(skip_serializing_if = "Option::is_none", rename = "modelPreferences")]
    pub model_preferences: Option<ModelPreferences>,

    /// System prompt
    #[serde(skip_serializing_if = "Option::is_none", rename = "systemPrompt")]
    pub system_prompt: Option<String>,

    /// Whether to include context
    #[serde(skip_serializing_if = "Option::is_none", rename = "includeContext")]
    pub include_context: Option<String>,

    /// Temperature for sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// Maximum tokens
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxTokens")]
    pub max_tokens: Option<u32>,

    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none", rename = "stopSequences")]
    pub stop_sequences: Option<Vec<String>>,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Response from sampling/create_message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateMessageResult {
    /// The model used for sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// The role of the response
    pub role: String,

    /// The generated content
    pub content: SamplingContent,

    /// Stop reason
    #[serde(skip_serializing_if = "Option::is_none", rename = "stopReason")]
    pub stop_reason: Option<String>,
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
        assert_eq!(custom, McpMethod::Other("custom/method".to_string()));
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

    #[test]
    fn test_resource_serialization() {
        let resource = Resource {
            uri: "file:///test.txt".to_string(),
            name: "Test File".to_string(),
            description: Some("A test file".to_string()),
            mime_type: Some("text/plain".to_string()),
        };

        let json = serde_json::to_string(&resource).unwrap();
        let deserialized: Resource = serde_json::from_str(&json).unwrap();
        assert_eq!(resource, deserialized);
    }

    #[test]
    fn test_resource_contents_serialization() {
        let contents = ResourceContents {
            uri: "file:///test.txt".to_string(),
            mime_type: "text/plain".to_string(),
            text: Some("Hello, world!".to_string()),
            blob: None,
        };

        let json = serde_json::to_string(&contents).unwrap();
        let deserialized: ResourceContents = serde_json::from_str(&json).unwrap();
        assert_eq!(contents, deserialized);
    }

    #[test]
    fn test_binary_resource_contents() {
        let contents = ResourceContents {
            uri: "file:///test.bin".to_string(),
            mime_type: "application/octet-stream".to_string(),
            text: None,
            blob: Some("SGVsbG8gV29ybGQ=".to_string()), // "Hello World" in base64
        };

        let json = serde_json::to_string(&contents).unwrap();
        let deserialized: ResourceContents = serde_json::from_str(&json).unwrap();
        assert_eq!(contents, deserialized);
    }

    #[test]
    fn test_list_resources_params() {
        let params = ListResourcesParams { cursor: None };
        let json = serde_json::to_string(&params).unwrap();
        assert_eq!(json, "{}");

        let params = ListResourcesParams {
            cursor: Some("next_page".to_string()),
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("cursor"));
    }

    #[test]
    fn test_resource_subscription_flow() {
        // Subscribe
        let subscribe_params = SubscribeResourceParams {
            uri: "file:///watched.txt".to_string(),
        };
        let json = serde_json::to_string(&subscribe_params).unwrap();
        assert!(json.contains("file:///watched.txt"));

        // Notification
        let notification = ResourceUpdatedNotification {
            uri: "file:///watched.txt".to_string(),
            meta: Some(serde_json::json!({"timestamp": "2024-01-01T00:00:00Z"})),
        };
        let json = serde_json::to_string(&notification).unwrap();
        let deserialized: ResourceUpdatedNotification = serde_json::from_str(&json).unwrap();
        assert_eq!(notification, deserialized);
    }

    #[test]
    fn test_tool_messages() {
        let tool = Tool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::json!({
                    "message": {"type": "string"}
                })),
                required: Some(vec!["message".to_string()]),
                additional: HashMap::new(),
            },
        };

        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: Tool = serde_json::from_str(&json).unwrap();
        assert_eq!(tool, deserialized);
    }

    #[test]
    fn test_mcp_methods() {
        assert_eq!(
            serde_json::to_string(&McpMethod::ResourcesList).unwrap(),
            "\"resources/list\""
        );
        assert_eq!(
            serde_json::to_string(&McpMethod::ResourcesRead).unwrap(),
            "\"resources/read\""
        );
        assert_eq!(
            serde_json::to_string(&McpMethod::ToolsList).unwrap(),
            "\"tools/list\""
        );
    }
}
