//! RMCP-based MCP Server implementation
//!
//! This module provides an MCP server implementation using the official RMCP SDK.
//! It replaces our custom server.rs by implementing the RMCP Service trait
//! and delegating to our existing resource, tool, and prompt managers.

use anyhow::Result;
use rmcp::{model::*, Error as McpError, Service, ServiceExt};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::{
    rmcp_bridge::CodePrismRmcpBridge,
    tools::{ListToolsParams, ToolRegistry},
    CodePrismMcpServer,
};

/// RMCP-based MCP server that implements the official Service trait
pub struct RmcpMcpServer {
    /// Core CodePrism server instance
    codeprism_server: Arc<RwLock<CodePrismMcpServer>>,
    /// RMCP bridge for tool delegation
    rmcp_bridge: CodePrismRmcpBridge,
    /// Tool registry
    tool_registry: ToolRegistry,
    /// Server information
    server_info: ServerInfo,
}

impl RmcpMcpServer {
    /// Create a new RMCP MCP server
    pub fn new() -> Result<Self> {
        let codeprism_server = Arc::new(RwLock::new(CodePrismMcpServer::new()?));
        let rmcp_bridge = CodePrismRmcpBridge::new(codeprism_server.clone());
        let tool_registry = ToolRegistry::new(codeprism_server.clone());

        let server_info = ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: Self::create_server_capabilities(),
            server_info: Implementation {
                name: env!("CARGO_PKG_NAME").into(),
                version: env!("CARGO_PKG_VERSION").into(),
            },
            instructions: Some("CodePrism MCP Server: Advanced code analysis and repository exploration tools for AI assistants. Provides 26+ tools for code intelligence, dependency analysis, and architectural insights.".to_string()),
        };

        Ok(Self {
            codeprism_server,
            rmcp_bridge,
            tool_registry,
            server_info,
        })
    }

    /// Initialize with repository path
    pub async fn initialize_with_repository<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<()> {
        let mut server = self.codeprism_server.write().await;
        server.initialize_with_repository(path).await
    }

    /// Create server capabilities
    fn create_server_capabilities() -> ServerCapabilities {
        ServerCapabilities {
            experimental: Some(BTreeMap::new()),
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
            ..Default::default()
        }
    }
}

impl Default for RmcpMcpServer {
    fn default() -> Self {
        Self::new().expect("Failed to create RMCP MCP server")
    }
}

/// Implement the RMCP Service trait for our server
impl Service<rmcp::RoleServer> for RmcpMcpServer {
    async fn handle_request(
        &self,
        request: ClientRequest,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ServerResult, McpError> {
        debug!("Handling RMCP request: {:?}", request);

        match request {
            ClientRequest::InitializeRequest(_init_req) => {
                let result = InitializeResult {
                    protocol_version: ProtocolVersion::LATEST,
                    capabilities: Self::create_server_capabilities(),
                    server_info: Implementation {
                        name: env!("CARGO_PKG_NAME").into(),
                        version: env!("CARGO_PKG_VERSION").into(),
                    },
                    instructions: Some("CodePrism MCP Server: Advanced code analysis and repository exploration tools for AI assistants. Provides 26+ tools for code intelligence, dependency analysis, and architectural insights.".to_string()),
                };
                Ok(ServerResult::InitializeResult(result))
            }

            ClientRequest::ListToolsRequest(list_req) => {
                let params = ListToolsParams {
                    cursor: list_req.params.as_ref().and_then(|p| p.cursor.clone()),
                };

                let result = self.tool_registry.list_tools(params).await.map_err(|e| {
                    McpError::new(
                        ErrorCode::INTERNAL_ERROR,
                        format!("Tool list error: {}", e),
                        None,
                    )
                })?;

                // Convert to RMCP result type
                let rmcp_result = ListToolsResult {
                    tools: result
                        .tools
                        .into_iter()
                        .map(|t| Tool {
                            name: t.name.into(),
                            description: Some(t.description.into()),
                            input_schema: Arc::new(match t.input_schema {
                                serde_json::Value::Object(obj) => obj,
                                _ => serde_json::Map::new(),
                            }),
                            annotations: None,
                        })
                        .collect(),
                    next_cursor: result.next_cursor,
                };

                Ok(ServerResult::ListToolsResult(rmcp_result))
            }

            ClientRequest::CallToolRequest(call_req) => {
                // Use RMCP bridge for tool calls to maintain existing functionality
                let args = call_req
                    .params
                    .arguments
                    .map(serde_json::Value::Object)
                    .unwrap_or(serde_json::Value::Null);

                let legacy_result = self
                    .rmcp_bridge
                    .call_tool(&call_req.params.name, args)
                    .await
                    .map_err(|e| {
                        McpError::new(
                            ErrorCode::INTERNAL_ERROR,
                            format!("Tool call error: {}", e),
                            None,
                        )
                    })?;

                // Convert legacy result to RMCP result
                // NOTE: Content conversion will be enhanced in future version
                let rmcp_result = rmcp::model::CallToolResult {
                    content: vec![], // FUTURE: Add rich content type conversion
                    is_error: legacy_result.is_error,
                };

                Ok(ServerResult::CallToolResult(rmcp_result))
            }

            // Handle other request types as needed
            _ => {
                warn!("Unhandled request type: {:?}", request);
                Err(McpError::new(
                    ErrorCode::METHOD_NOT_FOUND,
                    "Method not implemented".to_string(),
                    None,
                ))
            }
        }
    }

    async fn handle_notification(
        &self,
        notification: ClientNotification,
        _context: rmcp::service::NotificationContext<rmcp::RoleServer>,
    ) -> Result<(), McpError> {
        debug!("Handling RMCP notification: {:?}", notification);

        match notification {
            ClientNotification::InitializedNotification(_) => {
                info!("Client reported initialization complete");
                Ok(())
            }

            ClientNotification::CancelledNotification(cancel_notif) => {
                info!(
                    "Request cancellation received: {:?}",
                    cancel_notif.params.request_id
                );
                if let Some(reason) = &cancel_notif.params.reason {
                    debug!("Cancellation reason: {}", reason);
                }
                Ok(())
            }

            _ => {
                warn!("Unhandled notification type: {:?}", notification);
                Ok(())
            }
        }
    }

    fn get_info(&self) -> ServerInfo {
        self.server_info.clone()
    }
}

/// Easy-to-use server runner function
pub async fn run_rmcp_server() -> Result<()> {
    info!("Starting CodePrism RMCP MCP server");

    let server = RmcpMcpServer::new()?;
    let transport = rmcp::transport::io::stdio();

    // Use RMCP's serve function to run the server
    let running_service = server
        .serve(transport)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start RMCP server: {}", e))?;

    info!("RMCP server started successfully, waiting for requests...");

    // Wait for the server to complete
    match running_service.waiting().await {
        Ok(quit_reason) => {
            info!("RMCP server stopped: {:?}", quit_reason);
            Ok(())
        }
        Err(e) => {
            warn!("RMCP server error: {}", e);
            Err(anyhow::anyhow!("Server error: {}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rmcp_server_creation() {
        let server = RmcpMcpServer::new().expect("Failed to create RMCP server");

        let info = server.get_info();
        assert_eq!(info.server_info.name, env!("CARGO_PKG_NAME"));
        assert_eq!(info.server_info.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_rmcp_server_default() {
        let server = RmcpMcpServer::default();
        let info = server.get_info();
        assert!(info.instructions.is_some());
        assert!(info.instructions.as_ref().unwrap().contains("CodePrism"));
    }
}
