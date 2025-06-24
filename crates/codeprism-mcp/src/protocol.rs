//! MCP Protocol types and JSON-RPC 2.0 implementation
//!
//! This module implements the core Model Context Protocol types according to the specification.
//! All message types follow JSON-RPC 2.0 format as required by MCP.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::Duration;

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
}
