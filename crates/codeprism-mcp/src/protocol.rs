//! MCP Protocol types and JSON-RPC 2.0 implementation
//!
//! This module implements the core Model Context Protocol types according to the specification.
//! All message types follow JSON-RPC 2.0 format as required by MCP.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::Duration;

/// Supported MCP protocol versions
pub const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &[
    "2024-11-05", // Latest supported version
    "2024-10-07", // Previous stable version
    "2024-09-01", // Legacy compatibility
];

/// Current default protocol version
pub const DEFAULT_PROTOCOL_VERSION: &str = "2024-11-05";

/// Client types we can detect and optimize for
#[derive(Debug, Clone, PartialEq)]
pub enum ClientType {
    Claude,
    Cursor,
    VSCode,
    Unknown(String),
}

impl ClientType {
    /// Detect client type from client info
    pub fn from_client_info(client_info: &ClientInfo) -> Self {
        let name_lower = client_info.name.to_lowercase();

        if name_lower.contains("claude") {
            Self::Claude
        } else if name_lower.contains("cursor") {
            Self::Cursor
        } else if name_lower.contains("vscode") || name_lower.contains("vs code") {
            Self::VSCode
        } else {
            Self::Unknown(client_info.name.clone())
        }
    }

    /// Get client-specific optimizations
    pub fn get_optimizations(&self) -> ClientOptimizations {
        match self {
            Self::Claude => ClientOptimizations {
                max_response_size: 100_000,
                supports_streaming: true,
                preferred_timeout: Duration::from_secs(30),
                batch_size_limit: 10,
            },
            Self::Cursor => ClientOptimizations {
                max_response_size: 50_000,
                supports_streaming: false,
                preferred_timeout: Duration::from_secs(15),
                batch_size_limit: 5,
            },
            Self::VSCode => ClientOptimizations {
                max_response_size: 75_000,
                supports_streaming: true,
                preferred_timeout: Duration::from_secs(20),
                batch_size_limit: 7,
            },
            Self::Unknown(_) => ClientOptimizations::default(),
        }
    }
}

/// Client-specific optimization settings
#[derive(Debug, Clone)]
pub struct ClientOptimizations {
    pub max_response_size: usize,
    pub supports_streaming: bool,
    pub preferred_timeout: Duration,
    pub batch_size_limit: usize,
}

impl Default for ClientOptimizations {
    fn default() -> Self {
        Self {
            max_response_size: 75_000,
            supports_streaming: false,
            preferred_timeout: Duration::from_secs(30),
            batch_size_limit: 5,
        }
    }
}

/// Protocol version negotiation result
#[derive(Debug, Clone)]
pub struct VersionNegotiation {
    pub agreed_version: String,
    pub client_version: String,
    pub server_versions: Vec<String>,
    pub compatibility_level: CompatibilityLevel,
    pub warnings: Vec<String>,
}

/// Compatibility level between client and server
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompatibilityLevel {
    /// Incompatible - connection should be rejected
    Incompatible,
    /// Limited compatibility - some features may not work
    Limited,
    /// Compatible with minor differences
    Compatible,
    /// Full compatibility - same version
    Full,
}

impl VersionNegotiation {
    /// Negotiate protocol version between client and server
    pub fn negotiate(client_version: &str) -> Self {
        let server_versions: Vec<String> = SUPPORTED_PROTOCOL_VERSIONS
            .iter()
            .map(|v| v.to_string())
            .collect();
        let mut warnings = Vec::new();

        // Check if client version is supported
        let (agreed_version, compatibility_level) =
            if SUPPORTED_PROTOCOL_VERSIONS.contains(&client_version) {
                (client_version.to_string(), CompatibilityLevel::Full)
            } else {
                // Try to find a compatible version
                let parsed_client = parse_version(client_version);
                let mut best_match = None;
                let mut best_compatibility = CompatibilityLevel::Incompatible;

                for &server_version in SUPPORTED_PROTOCOL_VERSIONS {
                    let parsed_server = parse_version(server_version);
                    let compatibility = determine_compatibility(&parsed_client, &parsed_server);

                    if compatibility > best_compatibility {
                        best_match = Some(server_version.to_string());
                        best_compatibility = compatibility;
                    }
                }

                match best_match {
                    Some(version) => {
                        warnings.push(format!(
                        "Client version {} not directly supported, using {} with {} compatibility",
                        client_version, version,
                        match best_compatibility {
                            CompatibilityLevel::Full => "full",
                            CompatibilityLevel::Compatible => "high",
                            CompatibilityLevel::Limited => "limited",
                            CompatibilityLevel::Incompatible => "no",
                        }
                    ));
                        (version, best_compatibility)
                    }
                    None => {
                        warnings.push(format!(
                            "Client version {} is incompatible with supported versions: {:?}",
                            client_version, SUPPORTED_PROTOCOL_VERSIONS
                        ));
                        (
                            DEFAULT_PROTOCOL_VERSION.to_string(),
                            CompatibilityLevel::Incompatible,
                        )
                    }
                }
            };

        Self {
            agreed_version,
            client_version: client_version.to_string(),
            server_versions,
            compatibility_level,
            warnings,
        }
    }

    /// Check if this negotiation allows the connection
    pub fn is_acceptable(&self) -> bool {
        self.compatibility_level != CompatibilityLevel::Incompatible
    }
}

/// Parsed version components
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ParsedVersion {
    year: u32,
    month: u32,
    day: u32,
}

/// Parse a version string in YYYY-MM-DD format
fn parse_version(version: &str) -> ParsedVersion {
    let parts: Vec<&str> = version.split('-').collect();
    if parts.len() == 3 {
        ParsedVersion {
            year: parts[0].parse().unwrap_or(0),
            month: parts[1].parse().unwrap_or(0),
            day: parts[2].parse().unwrap_or(0),
        }
    } else {
        ParsedVersion {
            year: 0,
            month: 0,
            day: 0,
        }
    }
}

/// Determine compatibility level between two versions
fn determine_compatibility(client: &ParsedVersion, server: &ParsedVersion) -> CompatibilityLevel {
    if client == server {
        return CompatibilityLevel::Full;
    }

    // Same year and month = compatible
    if client.year == server.year && client.month == server.month {
        return CompatibilityLevel::Compatible;
    }

    // Within 6 months = limited compatibility
    let client_days = client.year * 365 + client.month * 30 + client.day;
    let server_days = server.year * 365 + server.month * 30 + server.day;
    let diff_days = (client_days as i32 - server_days as i32).abs();

    if diff_days <= 180 {
        // ~6 months
        CompatibilityLevel::Limited
    } else {
        CompatibilityLevel::Incompatible
    }
}

/// JSON-RPC 2.0 Request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version, must be "2.0"
    pub jsonrpc: String,
    /// Request ID (number or string)
    pub id: serde_json::Value,
    /// Method name
    pub method: String,
    /// Optional parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 Response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version, must be "2.0"
    pub jsonrpc: String,
    /// Request ID matching the original request
    pub id: serde_json::Value,
    /// Successful result (mutually exclusive with error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error information (mutually exclusive with result)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 Notification message (no response expected)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    /// JSON-RPC version, must be "2.0"
    pub jsonrpc: String,
    /// Method name
    pub method: String,
    /// Optional parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 Error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Optional additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Cancellation notification parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancellationParams {
    /// Request ID being cancelled
    pub id: serde_json::Value,
    /// Optional reason for cancellation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Cancellation token for request cancellation
#[derive(Debug, Clone)]
pub struct CancellationToken {
    /// Notifier for cancellation
    notify: Arc<Notify>,
    /// Whether the token is cancelled
    cancelled: Arc<std::sync::atomic::AtomicBool>,
    /// Request ID associated with this token
    request_id: serde_json::Value,
}

impl CancellationToken {
    /// Create a new cancellation token
    pub fn new(request_id: serde_json::Value) -> Self {
        Self {
            notify: Arc::new(Notify::new()),
            cancelled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            request_id,
        }
    }

    /// Check if cancellation was requested
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Cancel this token
    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.notify.notify_waiters();
    }

    /// Wait for cancellation
    pub async fn cancelled(&self) {
        if self.is_cancelled() {
            return;
        }
        self.notify.notified().await;
    }

    /// Get the request ID
    pub fn request_id(&self) -> &serde_json::Value {
        &self.request_id
    }

    /// Run an operation with timeout and cancellation
    pub async fn with_timeout<F, T>(
        &self,
        timeout: Duration,
        operation: F,
    ) -> Result<T, CancellationError>
    where
        F: std::future::Future<Output = T>,
    {
        tokio::select! {
            result = operation => Ok(result),
            _ = self.cancelled() => Err(CancellationError::Cancelled),
            _ = tokio::time::sleep(timeout) => Err(CancellationError::Timeout),
        }
    }
}

/// Cancellation error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum CancellationError {
    /// Operation was cancelled
    #[error("Operation was cancelled")]
    Cancelled,
    /// Operation timed out
    #[error("Operation timed out")]
    Timeout,
}

/// MCP Initialize request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    /// Protocol version supported by client
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Client capabilities
    pub capabilities: ClientCapabilities,
    /// Client implementation information
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

/// MCP Initialize response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// Protocol version supported by server
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Server capabilities
    pub capabilities: ServerCapabilities,
    /// Server implementation information
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

/// Client capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientCapabilities {
    /// Experimental capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, serde_json::Value>>,
    /// Sampling capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<SamplingCapability>,
}

/// Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerCapabilities {
    /// Experimental capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, serde_json::Value>>,
    /// Resources capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<crate::resources::ResourceCapabilities>,
    /// Tools capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<crate::tools::ToolCapabilities>,
    /// Prompts capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<crate::prompts::PromptCapabilities>,
}

/// Sampling capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingCapability {}

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client name
    pub name: String,
    /// Client version
    pub version: String,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(id: serde_json::Value, method: String, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method,
            params,
        }
    }
}

impl JsonRpcResponse {
    /// Create a successful JSON-RPC response
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error JSON-RPC response
    pub fn error(id: serde_json::Value, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

impl JsonRpcNotification {
    /// Create a new JSON-RPC notification
    pub fn new(method: String, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
            params,
        }
    }
}

impl JsonRpcError {
    /// Standard JSON-RPC error codes
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    /// Create a new JSON-RPC error
    pub fn new(code: i32, message: String, data: Option<serde_json::Value>) -> Self {
        Self {
            code,
            message,
            data,
        }
    }

    /// Create a method not found error
    pub fn method_not_found(method: &str) -> Self {
        Self::new(
            Self::METHOD_NOT_FOUND,
            format!("Method not found: {}", method),
            None,
        )
    }

    /// Create an invalid parameters error
    pub fn invalid_params(message: String) -> Self {
        Self::new(Self::INVALID_PARAMS, message, None)
    }

    /// Create an internal error
    pub fn internal_error(message: String) -> Self {
        Self::new(Self::INTERNAL_ERROR, message, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest::new(
            serde_json::Value::Number(1.into()),
            "test_method".to_string(),
            Some(serde_json::json!({"param": "value"})),
        );

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.jsonrpc, deserialized.jsonrpc);
        assert_eq!(request.id, deserialized.id);
        assert_eq!(request.method, deserialized.method);
        assert_eq!(request.params, deserialized.params);
    }

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse::success(
            serde_json::Value::Number(1.into()),
            serde_json::json!({"success": true}),
        );

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_response_error() {
        let error = JsonRpcError::method_not_found("unknown_method");
        let response = JsonRpcResponse::error(serde_json::Value::Number(1.into()), error);

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }

    #[test]
    fn test_initialize_params() {
        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: InitializeParams = serde_json::from_str(&json).unwrap();

        assert_eq!(params.protocol_version, deserialized.protocol_version);
        assert_eq!(params.client_info.name, deserialized.client_info.name);
    }

    #[test]
    fn test_client_type_detection() {
        let claude_client = ClientInfo {
            name: "Claude Desktop".to_string(),
            version: "1.0.0".to_string(),
        };
        assert_eq!(
            ClientType::from_client_info(&claude_client),
            ClientType::Claude
        );

        let cursor_client = ClientInfo {
            name: "Cursor Editor".to_string(),
            version: "2.0.0".to_string(),
        };
        assert_eq!(
            ClientType::from_client_info(&cursor_client),
            ClientType::Cursor
        );

        let vscode_client = ClientInfo {
            name: "VS Code".to_string(),
            version: "1.80.0".to_string(),
        };
        assert_eq!(
            ClientType::from_client_info(&vscode_client),
            ClientType::VSCode
        );

        let unknown_client = ClientInfo {
            name: "Custom Client".to_string(),
            version: "1.0.0".to_string(),
        };
        assert_eq!(
            ClientType::from_client_info(&unknown_client),
            ClientType::Unknown("Custom Client".to_string())
        );
    }

    #[test]
    fn test_client_optimizations() {
        let claude_opts = ClientType::Claude.get_optimizations();
        assert_eq!(claude_opts.max_response_size, 100_000);
        assert!(claude_opts.supports_streaming);
        assert_eq!(claude_opts.batch_size_limit, 10);

        let cursor_opts = ClientType::Cursor.get_optimizations();
        assert_eq!(cursor_opts.max_response_size, 50_000);
        assert!(!cursor_opts.supports_streaming);
        assert_eq!(cursor_opts.batch_size_limit, 5);
    }

    #[test]
    fn test_version_negotiation_exact_match() {
        let negotiation = VersionNegotiation::negotiate("2024-11-05");

        assert_eq!(negotiation.agreed_version, "2024-11-05");
        assert_eq!(negotiation.compatibility_level, CompatibilityLevel::Full);
        assert!(negotiation.warnings.is_empty());
        assert!(negotiation.is_acceptable());
    }

    #[test]
    fn test_version_negotiation_compatible() {
        let negotiation = VersionNegotiation::negotiate("2024-11-01");

        assert_eq!(
            negotiation.compatibility_level,
            CompatibilityLevel::Compatible
        );
        assert!(negotiation.is_acceptable());
        assert!(!negotiation.warnings.is_empty());
    }

    #[test]
    fn test_version_negotiation_limited() {
        let negotiation = VersionNegotiation::negotiate("2024-08-15");

        assert_eq!(negotiation.compatibility_level, CompatibilityLevel::Limited);
        assert!(negotiation.is_acceptable());
    }

    #[test]
    fn test_version_negotiation_incompatible() {
        let negotiation = VersionNegotiation::negotiate("2023-01-01");

        assert_eq!(
            negotiation.compatibility_level,
            CompatibilityLevel::Incompatible
        );
        assert!(!negotiation.is_acceptable());
    }

    #[test]
    fn test_parse_version() {
        let parsed = parse_version("2024-11-05");
        assert_eq!(parsed.year, 2024);
        assert_eq!(parsed.month, 11);
        assert_eq!(parsed.day, 5);

        let invalid = parse_version("invalid");
        assert_eq!(invalid.year, 0);
        assert_eq!(invalid.month, 0);
        assert_eq!(invalid.day, 0);
    }

    #[test]
    fn test_compatibility_determination() {
        let v1 = parse_version("2024-11-05");
        let v2 = parse_version("2024-11-05");
        assert_eq!(determine_compatibility(&v1, &v2), CompatibilityLevel::Full);

        let v3 = parse_version("2024-11-01");
        assert_eq!(
            determine_compatibility(&v1, &v3),
            CompatibilityLevel::Compatible
        );

        let v4 = parse_version("2024-08-01");
        assert_eq!(
            determine_compatibility(&v1, &v4),
            CompatibilityLevel::Limited
        );

        let v5 = parse_version("2023-01-01");
        assert_eq!(
            determine_compatibility(&v1, &v5),
            CompatibilityLevel::Incompatible
        );
    }

    #[test]
    fn test_cancellation_token() {
        let token = CancellationToken::new(serde_json::Value::Number(1.into()));
        assert!(!token.is_cancelled());

        token.cancel();
        assert!(token.is_cancelled());
    }
}
