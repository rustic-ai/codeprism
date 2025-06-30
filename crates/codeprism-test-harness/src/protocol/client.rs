//! MCP Client Implementation
//!
//! This module provides a generic MCP client for communicating with
//! any MCP server implementation.

use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use super::{
    capabilities::McpCapabilities,
    jsonrpc::JsonRpcMessage,
    messages::{ClientInfo, InitializeParams, InitializeResult},
    ConnectionState, McpConfig, McpResult, Transport,
};

/// Generic MCP client for communicating with MCP servers
pub struct McpClient {
    config: McpConfig,
    transport: Option<Box<dyn Transport>>,
    state: Arc<RwLock<ConnectionState>>,
    #[allow(dead_code)] // Will be used for capability negotiation
    capabilities: Arc<RwLock<Option<McpCapabilities>>>,
    request_id_counter: Arc<RwLock<u64>>,
}

impl McpClient {
    /// Create a new MCP client with the given configuration
    pub fn new(config: McpConfig) -> Self {
        Self {
            config,
            transport: None,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            capabilities: Arc::new(RwLock::new(None)),
            request_id_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Set the transport for this client
    pub fn with_transport(mut self, transport: Box<dyn Transport>) -> Self {
        self.transport = Some(transport);
        self
    }

    /// Get the current connection state
    pub async fn state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }

    /// Initialize the MCP connection
    pub async fn initialize(&mut self) -> McpResult<InitializeResult> {
        // Update state to connecting
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Connecting;
        }

        // Create initialize request
        let params = InitializeParams {
            protocol_version: self.config.protocol_version.clone(),
            capabilities: self.config.capabilities.clone(),
            client_info: ClientInfo {
                name: self.config.client_info.name.clone(),
                version: self.config.client_info.version.clone(),
            },
        };

        // Send initialize request and wait for response
        let response = self
            .send_request_with_timeout(
                "initialize",
                Some(serde_json::to_value(params)?),
                Duration::from_secs(self.config.timeouts.initialization),
            )
            .await?;

        // Parse initialize result
        let result: InitializeResult = serde_json::from_value(response)?;

        // Store server capabilities (converting from ServerCapabilities to McpCapabilities)
        {
            let mut capabilities = self.capabilities.write().await;
            *capabilities = Some(McpCapabilities::default()); // Simplified in this implementation
        }

        // Update state to ready
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Ready;
        }

        Ok(result)
    }

    /// Close the MCP connection
    pub async fn close(&mut self) -> McpResult<()> {
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Closing;
        }

        if let Some(ref mut transport) = self.transport {
            transport
                .close()
                .await
                .map_err(|e| super::McpError::Transport(e.to_string()))?;
        }

        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Closed;
        }

        Ok(())
    }

    /// Send a JSON-RPC message
    pub async fn send_message(&mut self, message: JsonRpcMessage) -> McpResult<()> {
        if let Some(ref mut transport) = self.transport {
            transport
                .send(message)
                .await
                .map_err(|e| super::McpError::Transport(e.to_string()))?;
        }
        Ok(())
    }

    /// Receive a JSON-RPC message
    pub async fn receive_message(&mut self) -> McpResult<JsonRpcMessage> {
        if let Some(ref mut transport) = self.transport {
            let message = transport
                .receive()
                .await
                .map_err(|e| super::McpError::Transport(e.to_string()))?;
            Ok(message)
        } else {
            Err(super::McpError::Transport(
                "No transport configured".to_string(),
            ))
        }
    }

    /// Send a request and return the response (real implementation)
    pub async fn send_request(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> McpResult<serde_json::Value> {
        self.send_request_with_timeout(
            method,
            params,
            Duration::from_secs(self.config.timeouts.request),
        )
        .await
    }

    /// Send a request with custom timeout and return the response
    pub async fn send_request_with_timeout(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
        timeout: Duration,
    ) -> McpResult<serde_json::Value> {
        // Check if we're in a valid state for requests
        // Special case: allow "initialize" method when in Connecting state
        let current_state = self.state().await;
        if !(matches!(current_state, ConnectionState::Ready)
            || (method == "initialize" && matches!(current_state, ConnectionState::Connecting)))
        {
            return Err(super::McpError::InvalidState {
                expected: ConnectionState::Ready,
                actual: current_state,
            });
        }

        // Generate unique request ID
        let request_id = {
            let mut counter = self.request_id_counter.write().await;
            *counter += 1;
            *counter
        };

        // Create JSON-RPC request message using the correct constructor
        let request_message = JsonRpcMessage::request_with_id(
            Value::Number(serde_json::Number::from(request_id)),
            method,
            params,
        );

        // Send the request
        if let Some(ref mut transport) = self.transport {
            transport.send(request_message).await.map_err(|e| {
                super::McpError::Transport(format!("Failed to send request: {}", e))
            })?;
        } else {
            return Err(super::McpError::Transport(
                "No transport configured".to_string(),
            ));
        }

        // Wait for response with timeout
        let response_result =
            tokio::time::timeout(timeout, self.wait_for_response(request_id)).await;

        match response_result {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(super::McpError::Timeout),
        }
    }

    /// Wait for a specific response by request ID
    async fn wait_for_response(&mut self, expected_id: u64) -> McpResult<serde_json::Value> {
        loop {
            let message = self.receive_message().await?;

            // Check if this is a response message
            if message.is_response() {
                // Check if this is the response we're waiting for
                if let Some(id) = &message.id {
                    if let Some(id_num) = id.as_u64() {
                        if id_num == expected_id {
                            // Handle the response
                            if let Some(result) = message.result {
                                return Ok(result);
                            } else if let Some(error) = message.error {
                                return Err(super::McpError::ServerRejected {
                                    reason: format!(
                                        "Server error {}: {}",
                                        error.code, error.message
                                    ),
                                });
                            } else {
                                return Err(super::McpError::Protocol(
                                    "Response has neither result nor error".to_string(),
                                ));
                            }
                        }
                    }
                }
                // Not our response, continue waiting
            } else if message.is_notification() {
                // Handle notifications if needed, but continue waiting for our response
                continue;
            } else if message.is_request() {
                // We received a request from the server, but we're waiting for a response
                // This shouldn't happen in normal MCP flow, but we'll ignore it
                continue;
            }
        }
    }

    /// List available tools from the MCP server
    pub async fn list_tools(&mut self) -> McpResult<Vec<serde_json::Value>> {
        let response = self.send_request("tools/list", None).await?;

        if let Some(tools) = response.get("tools").and_then(|t| t.as_array()) {
            Ok(tools.clone())
        } else {
            Ok(vec![])
        }
    }

    /// Call a specific tool on the MCP server
    pub async fn call_tool(
        &mut self,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> McpResult<serde_json::Value> {
        let params = serde_json::json!({
            "name": name,
            "arguments": arguments.unwrap_or(serde_json::json!({}))
        });

        self.send_request("tools/call", Some(params)).await
    }

    /// List available resources from the MCP server
    pub async fn list_resources(&mut self) -> McpResult<Vec<serde_json::Value>> {
        let response = self.send_request("resources/list", None).await?;

        if let Some(resources) = response.get("resources").and_then(|r| r.as_array()) {
            Ok(resources.clone())
        } else {
            Ok(vec![])
        }
    }

    /// Read a specific resource from the MCP server
    pub async fn read_resource(&mut self, uri: &str) -> McpResult<serde_json::Value> {
        let params = serde_json::json!({
            "uri": uri
        });

        self.send_request("resources/read", Some(params)).await
    }

    /// List available prompts from the MCP server
    pub async fn list_prompts(&mut self) -> McpResult<Vec<serde_json::Value>> {
        let response = self.send_request("prompts/list", None).await?;

        if let Some(prompts) = response.get("prompts").and_then(|p| p.as_array()) {
            Ok(prompts.clone())
        } else {
            Ok(vec![])
        }
    }

    /// Get a specific prompt from the MCP server
    pub async fn get_prompt(
        &mut self,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> McpResult<serde_json::Value> {
        let params = serde_json::json!({
            "name": name,
            "arguments": arguments.unwrap_or(serde_json::json!({}))
        });

        self.send_request("prompts/get", Some(params)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let config = McpConfig::default();
        let client = McpClient::new(config);

        let state = client.state().await;
        assert_eq!(state, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_request_id_generation() {
        let config = McpConfig::default();
        let client = McpClient::new(config);

        let mut counter = client.request_id_counter.write().await;
        *counter += 1;
        let id1 = *counter;
        *counter += 1;
        let id2 = *counter;

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_ne!(id1, id2);
    }
}
