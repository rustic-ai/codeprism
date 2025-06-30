//! MCP protocol implementation for communication with servers

use crate::transport::Transport;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, error, warn};

/// MCP protocol error types
#[derive(Debug, Error)]
pub enum McpError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Protocol violation: {0}")]
    Protocol(String),
    #[error("Timeout occurred")]
    Timeout,
    #[error("JSON-RPC error: {code} - {message}")]
    JsonRpc { code: i32, message: String },
    #[error("Server process error: {0}")]
    Process(String),
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// JSON-RPC 2.0 request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

/// Core MCP client functionality
pub struct McpClient {
    request_id_counter: u64,
    timeout_duration: Duration,
    transport: Option<Box<dyn Transport>>,
}

impl std::fmt::Debug for McpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpClient")
            .field("request_id_counter", &self.request_id_counter)
            .field("timeout_duration", &self.timeout_duration)
            .field("transport", &self.transport.as_ref().map(|_| "Transport"))
            .finish()
    }
}

impl McpClient {
    /// Create a new MCP client
    pub fn new() -> Self {
        Self {
            request_id_counter: 0,
            timeout_duration: Duration::from_secs(30),
            transport: None,
        }
    }

    /// Create a new MCP client with a specific transport
    pub fn with_transport(transport: Box<dyn Transport>) -> Self {
        Self {
            request_id_counter: 0,
            timeout_duration: Duration::from_secs(30),
            transport: Some(transport),
        }
    }

    /// Set the timeout duration for requests
    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout_duration = duration;
    }

    /// Connect to an MCP server using stdio transport
    pub async fn connect_stdio(
        &mut self,
        command: String,
        args: Vec<String>,
        working_dir: Option<String>,
    ) -> Result<(), McpError> {
        // Create stdio transport with server configuration
        let mut transport = crate::transport::stdio::StdioTransport::new().with_server_config(
            command,
            args,
            Default::default(),
            working_dir,
        );

        // Connect to the server
        transport
            .connect()
            .await
            .map_err(|e| McpError::Transport(format!("Failed to connect: {}", e)))?;

        self.transport = Some(Box::new(transport));
        Ok(())
    }

    /// Disconnect from the MCP server
    pub async fn disconnect(&mut self) -> Result<(), McpError> {
        if let Some(mut transport) = self.transport.take() {
            transport
                .disconnect()
                .await
                .map_err(|e| McpError::Transport(format!("Failed to disconnect: {}", e)))?;
        }
        Ok(())
    }

    /// Send MCP tool request
    pub async fn call_tool(
        &mut self,
        tool_name: &str,
        parameters: Value,
    ) -> Result<Value, McpError> {
        // Generate unique request ID
        self.request_id_counter += 1;
        let request_id = self.request_id_counter;

        // Create JSON-RPC request for tool call
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(request_id)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": tool_name,
                "arguments": parameters
            })),
        };

        // Send request and get response
        let response = self.send_request(request).await?;

        // Process response
        if let Some(error) = response.error {
            return Err(McpError::JsonRpc {
                code: error.code,
                message: error.message,
            });
        }

        response
            .result
            .ok_or_else(|| McpError::Protocol("Response missing result field".to_string()))
    }

    /// Send JSON-RPC request to server
    pub async fn send_request(
        &mut self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, McpError> {
        debug!("Sending request: {}", request.method);

        let transport = self.transport.as_mut().ok_or_else(|| {
            McpError::Connection("No transport available - call connect_stdio first".to_string())
        })?;

        // Serialize request to JSON
        let request_json = serde_json::to_value(&request)
            .map_err(|e| McpError::Serialization(format!("Failed to serialize request: {}", e)))?;

        // Send request via transport
        transport
            .send(request_json)
            .await
            .map_err(|e| McpError::Transport(format!("Failed to send request: {}", e)))?;

        // Receive response via transport
        let response_json = transport
            .receive()
            .await
            .map_err(|e| McpError::Transport(format!("Failed to receive response: {}", e)))?;

        // Parse response from JSON
        let response: JsonRpcResponse = serde_json::from_value(response_json)
            .map_err(|e| McpError::Serialization(format!("Failed to parse response: {}", e)))?;

        debug!("Received response for request: {}", request.method);
        Ok(response)
    }

    /// Initialize MCP session
    pub async fn initialize(
        &mut self,
        client_info: ClientInfo,
    ) -> Result<InitializeResult, McpError> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(self.next_request_id())),
            method: "initialize".to_string(),
            params: Some(json!(client_info)),
        };

        let response = self.send_request(request).await?;

        if let Some(error) = response.error {
            return Err(McpError::JsonRpc {
                code: error.code,
                message: error.message,
            });
        }

        let result = response
            .result
            .ok_or_else(|| McpError::Protocol("Initialize response missing result".to_string()))?;

        serde_json::from_value(result).map_err(|e| {
            McpError::Serialization(format!("Failed to parse initialize result: {}", e))
        })
    }

    /// Validate that a server implements required MCP protocol methods
    pub async fn validate_protocol_compliance(&mut self) -> Result<bool, McpError> {
        // Test basic MCP capabilities
        let tests = vec![
            (
                "initialize",
                json!({"clientInfo": {"name": "test", "version": "1.0"}}),
            ),
            ("tools/list", json!({})),
        ];

        for (method, params) in tests {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: Some(json!(self.next_request_id())),
                method: method.to_string(),
                params: Some(params),
            };

            match self.send_request(request).await {
                Ok(response) => {
                    if response.error.is_some() {
                        warn!("Protocol compliance test failed for method: {}", method);
                        return Ok(false);
                    }
                }
                Err(e) => {
                    error!(
                        "Protocol compliance test error for method {}: {}",
                        method, e
                    );
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    fn next_request_id(&mut self) -> u64 {
        self.request_id_counter += 1;
        self.request_id_counter
    }
}

impl Default for McpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Client information for MCP initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// MCP initialize result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

/// Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub tools: Option<Value>,
    pub resources: Option<Value>,
    pub prompts: Option<Value>,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// Validate protocol compliance for an MCP server
pub async fn validate_protocol_compliance(mut client: McpClient) -> Result<bool, McpError> {
    client.validate_protocol_compliance().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_client_creation() {
        let client = McpClient::new();
        assert_eq!(client.request_id_counter, 0);
    }

    #[tokio::test]
    async fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "test".to_string(),
            params: Some(json!({"test": "value"})),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.method, deserialized.method);
        assert_eq!(request.jsonrpc, deserialized.jsonrpc);
    }

    #[tokio::test]
    async fn test_tool_call_requires_connection() {
        let mut client = McpClient::new();
        let result = client.call_tool("repository_stats", json!({})).await;

        // Should fail because no transport is connected
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No transport available"));
    }
}
