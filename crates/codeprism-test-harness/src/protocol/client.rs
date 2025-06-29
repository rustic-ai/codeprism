//! MCP Client Implementation
//!
//! This module provides a generic MCP client for communicating with
//! any MCP server implementation.

use std::collections::HashMap;
use std::sync::Arc;
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
}

impl McpClient {
    /// Create a new MCP client with the given configuration
    pub fn new(config: McpConfig) -> Self {
        Self {
            config,
            transport: None,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            capabilities: Arc::new(RwLock::new(None)),
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
        let _params = InitializeParams {
            protocol_version: self.config.protocol_version.clone(),
            capabilities: self.config.capabilities.clone(),
            client_info: ClientInfo {
                name: self.config.client_info.name.clone(),
                version: self.config.client_info.version.clone(),
            },
        };

        // For now, return a mock result
        // TODO: Implement actual transport communication
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: super::capabilities::ServerCapabilities {
                experimental: HashMap::new(),
                logging: None,
                resources: Some(super::capabilities::ServerResourcesCapability {
                    subscribe: true,
                    list_changed: true,
                }),
                tools: Some(super::capabilities::ServerToolsCapability { list_changed: true }),
                prompts: Some(super::capabilities::ServerPromptsCapability { list_changed: true }),
            },
            server_info: super::messages::ServerInfo {
                name: "Mock MCP Server".to_string(),
                version: "1.0.0".to_string(),
            },
        };

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
    async fn test_client_initialization() {
        let config = McpConfig::default();
        let mut client = McpClient::new(config);

        let result = client.initialize().await;
        assert!(result.is_ok());

        let state = client.state().await;
        assert_eq!(state, ConnectionState::Ready);
    }
}
