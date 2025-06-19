//! MCP Server implementation
//!
//! This module implements the main MCP server that handles the protocol lifecycle,
//! request routing, and integration with repository components.

use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::{
    prompts::{GetPromptParams, ListPromptsParams, PromptManager},
    protocol::{
        ClientInfo, InitializeParams, InitializeResult, JsonRpcError, JsonRpcNotification,
        JsonRpcRequest, JsonRpcResponse, ServerInfo,
    },
    resources::{ListResourcesParams, ReadResourceParams, ResourceManager},
    tools::{CallToolParams, ListToolsParams, ToolCapabilities, ToolRegistry},
    transport::{StdioTransport, Transport, TransportMessage},
    PrismMcpServer,
};

/// MCP Server state
#[derive(Debug, Clone, PartialEq)]
pub enum ServerState {
    /// Server is not initialized
    Uninitialized,
    /// Server is initialized and ready
    Ready,
    /// Server is shutting down
    Shutdown,
}

/// Main MCP Server implementation
pub struct McpServer {
    /// Current server state
    state: ServerState,
    /// MCP protocol version
    protocol_version: String,
    /// Server information
    server_info: ServerInfo,
    /// Client information (set during initialization)
    client_info: Option<ClientInfo>,
    /// Core Prism server instance
    prism_server: Arc<RwLock<PrismMcpServer>>,
    /// Resource manager
    resource_manager: ResourceManager,
    /// Tool registry
    tool_registry: ToolRegistry,
    /// Prompt manager
    prompt_manager: PromptManager,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new() -> Result<Self> {
        let prism_server = Arc::new(RwLock::new(PrismMcpServer::new()?));

        let resource_manager = ResourceManager::new(prism_server.clone());
        let tool_registry = ToolRegistry::new(prism_server.clone());
        let prompt_manager = PromptManager::new(prism_server.clone());

        Ok(Self {
            state: ServerState::Uninitialized,
            protocol_version: "2024-11-05".to_string(),
            server_info: ServerInfo {
                name: "prism-mcp".to_string(),
                version: "0.1.0".to_string(),
            },
            client_info: None,
            prism_server,
            resource_manager,
            tool_registry,
            prompt_manager,
        })
    }

    /// Create a new MCP server with custom configuration
    pub fn new_with_config(
        memory_limit_mb: usize,
        batch_size: usize,
        max_file_size_mb: usize,
        disable_memory_limit: bool,
        exclude_dirs: Vec<String>,
        include_extensions: Option<Vec<String>>,
        dependency_mode: Option<String>,
    ) -> Result<Self> {
        let prism_server = Arc::new(RwLock::new(PrismMcpServer::new_with_config(
            memory_limit_mb,
            batch_size,
            max_file_size_mb,
            disable_memory_limit,
            exclude_dirs,
            include_extensions,
            dependency_mode,
        )?));

        let resource_manager = ResourceManager::new(prism_server.clone());
        let tool_registry = ToolRegistry::new(prism_server.clone());
        let prompt_manager = PromptManager::new(prism_server.clone());

        Ok(Self {
            state: ServerState::Uninitialized,
            protocol_version: "2024-11-05".to_string(),
            server_info: ServerInfo {
                name: "prism-mcp".to_string(),
                version: "0.1.0".to_string(),
            },
            client_info: None,
            prism_server,
            resource_manager,
            tool_registry,
            prompt_manager,
        })
    }

    /// Initialize with repository path
    pub async fn initialize_with_repository<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<()> {
        let mut server = self.prism_server.write().await;
        server.initialize_with_repository(path).await
    }

    /// Run the MCP server with stdio transport
    pub async fn run_stdio(self) -> Result<()> {
        info!("Starting Prism MCP server with stdio transport");

        let mut transport = StdioTransport::new();
        transport.start().await?;

        self.run_with_transport(transport).await
    }

    /// Run the MCP server with a custom transport
    pub async fn run_with_transport<T: Transport>(mut self, mut transport: T) -> Result<()> {
        info!("Starting Prism MCP server");

        loop {
            match transport.receive().await? {
                Some(message) => {
                    if let Some(response) = self.handle_message(message).await? {
                        transport.send(response).await?;
                    }
                }
                None => {
                    debug!("Transport closed, shutting down server");
                    break;
                }
            }
        }

        transport.close().await?;
        info!("Prism MCP server stopped");
        Ok(())
    }

    /// Handle an incoming message
    async fn handle_message(
        &mut self,
        message: TransportMessage,
    ) -> Result<Option<TransportMessage>> {
        match message {
            TransportMessage::Request(request) => {
                let response = self.handle_request(request).await;
                Ok(Some(TransportMessage::Response(response)))
            }
            TransportMessage::Notification(notification) => {
                self.handle_notification(notification).await?;
                Ok(None) // Notifications don't get responses
            }
            TransportMessage::Response(_) => {
                warn!("Received unexpected response message");
                Ok(None)
            }
        }
    }

    /// Handle a JSON-RPC request
    async fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        debug!(
            "Handling request: method={}, id={:?}",
            request.method, request.id
        );

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "resources/list" => self.handle_resources_list(request.params).await,
            "resources/read" => self.handle_resources_read(request.params).await,
            "tools/list" => self.handle_tools_list(request.params).await,
            "tools/call" => self.handle_tools_call(request.params).await,
            "prompts/list" => self.handle_prompts_list(request.params).await,
            "prompts/get" => self.handle_prompts_get(request.params).await,
            _ => Err(JsonRpcError::method_not_found(&request.method)),
        };

        match result {
            Ok(result) => JsonRpcResponse::success(request.id, result),
            Err(error) => JsonRpcResponse::error(request.id, error),
        }
    }

    /// Handle a JSON-RPC notification
    async fn handle_notification(&mut self, notification: JsonRpcNotification) -> Result<()> {
        debug!("Handling notification: method={}", notification.method);

        match notification.method.as_str() {
            "initialized" => {
                info!("Client reported initialization complete");
                self.state = ServerState::Ready;
            }
            "notifications/cancelled" => {
                debug!("Received cancellation notification");
                // TODO: Handle request cancellation
            }
            _ => {
                warn!("Unknown notification method: {}", notification.method);
            }
        }

        Ok(())
    }

    /// Handle initialize request
    async fn handle_initialize(&mut self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params: InitializeParams = params
            .ok_or_else(|| JsonRpcError::invalid_params("Missing parameters".to_string()))?
            .try_into_type()
            .map_err(|e| JsonRpcError::invalid_params(format!("Invalid parameters: {}", e)))?;

        info!(
            "Initializing MCP server with client: {} v{}",
            params.client_info.name, params.client_info.version
        );

        // Store client info
        self.client_info = Some(params.client_info);

        // Check protocol version compatibility
        if params.protocol_version != self.protocol_version {
            warn!(
                "Protocol version mismatch: client={}, server={}",
                params.protocol_version, self.protocol_version
            );
        }

        // Create initialize result
        let server = self.prism_server.read().await;
        let result = InitializeResult {
            protocol_version: self.protocol_version.clone(),
            capabilities: server.capabilities().clone(),
            server_info: self.server_info.clone(),
        };

        serde_json::to_value(result)
            .map_err(|e| JsonRpcError::internal_error(format!("Serialization error: {}", e)))
    }

    /// Handle resources/list request
    async fn handle_resources_list(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params = params
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| JsonRpcError::invalid_params(format!("Invalid parameters: {}", e)))?
            .unwrap_or(ListResourcesParams { cursor: None });

        let result = self
            .resource_manager
            .list_resources(params)
            .await
            .map_err(|e| JsonRpcError::internal_error(format!("Resource list error: {}", e)))?;

        serde_json::to_value(result)
            .map_err(|e| JsonRpcError::internal_error(format!("Serialization error: {}", e)))
    }

    /// Handle resources/read request
    async fn handle_resources_read(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params: ReadResourceParams = params
            .ok_or_else(|| JsonRpcError::invalid_params("Missing parameters".to_string()))?
            .try_into_type()
            .map_err(|e| JsonRpcError::invalid_params(format!("Invalid parameters: {}", e)))?;

        let result = self
            .resource_manager
            .read_resource(params)
            .await
            .map_err(|e| JsonRpcError::internal_error(format!("Resource read error: {}", e)))?;

        serde_json::to_value(result)
            .map_err(|e| JsonRpcError::internal_error(format!("Serialization error: {}", e)))
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params = params
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| JsonRpcError::invalid_params(format!("Invalid parameters: {}", e)))?
            .unwrap_or(ListToolsParams { cursor: None });

        let result = self
            .tool_registry
            .list_tools(params)
            .await
            .map_err(|e| JsonRpcError::internal_error(format!("Tool list error: {}", e)))?;

        serde_json::to_value(result)
            .map_err(|e| JsonRpcError::internal_error(format!("Serialization error: {}", e)))
    }

    /// Handle tools/call request
    async fn handle_tools_call(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params: CallToolParams = params
            .ok_or_else(|| JsonRpcError::invalid_params("Missing parameters".to_string()))?
            .try_into_type()
            .map_err(|e| JsonRpcError::invalid_params(format!("Invalid parameters: {}", e)))?;

        let result = self
            .tool_registry
            .call_tool(params)
            .await
            .map_err(|e| JsonRpcError::internal_error(format!("Tool call error: {}", e)))?;

        serde_json::to_value(result)
            .map_err(|e| JsonRpcError::internal_error(format!("Serialization error: {}", e)))
    }

    /// Handle prompts/list request
    async fn handle_prompts_list(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params = params
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| JsonRpcError::invalid_params(format!("Invalid parameters: {}", e)))?
            .unwrap_or(ListPromptsParams { cursor: None });

        let result = self
            .prompt_manager
            .list_prompts(params)
            .await
            .map_err(|e| JsonRpcError::internal_error(format!("Prompt list error: {}", e)))?;

        serde_json::to_value(result)
            .map_err(|e| JsonRpcError::internal_error(format!("Serialization error: {}", e)))
    }

    /// Handle prompts/get request
    async fn handle_prompts_get(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params: GetPromptParams = params
            .ok_or_else(|| JsonRpcError::invalid_params("Missing parameters".to_string()))?
            .try_into_type()
            .map_err(|e| JsonRpcError::invalid_params(format!("Invalid parameters: {}", e)))?;

        let result = self
            .prompt_manager
            .get_prompt(params)
            .await
            .map_err(|e| JsonRpcError::internal_error(format!("Prompt get error: {}", e)))?;

        serde_json::to_value(result)
            .map_err(|e| JsonRpcError::internal_error(format!("Serialization error: {}", e)))
    }

    /// Get current server state
    pub fn state(&self) -> ServerState {
        self.state.clone()
    }

    /// Get server info
    pub fn server_info(&self) -> &ServerInfo {
        &self.server_info
    }

    /// Get client info (if initialized)
    pub fn client_info(&self) -> Option<&ClientInfo> {
        self.client_info.as_ref()
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new().expect("Failed to create default MCP server")
    }
}

// Helper trait for converting JSON values to types
trait TryIntoType<T> {
    fn try_into_type(self) -> Result<T, serde_json::Error>;
}

impl<T> TryIntoType<T> for Value
where
    T: serde::de::DeserializeOwned,
{
    fn try_into_type(self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::ClientCapabilities;

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let server = McpServer::new().expect("Failed to create MCP server");
        assert_eq!(server.state(), ServerState::Uninitialized);
        assert_eq!(server.server_info().name, "prism-mcp");
        assert_eq!(server.server_info().version, "0.1.0");
    }

    #[tokio::test]
    async fn test_initialize_request() {
        let mut server = McpServer::new().expect("Failed to create MCP server");

        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        let params_value = serde_json::to_value(params).unwrap();
        let result = server.handle_initialize(Some(params_value)).await;

        assert!(result.is_ok());
        assert!(server.client_info().is_some());
        assert_eq!(server.client_info().unwrap().name, "test-client");
    }

    #[test]
    fn test_server_states() {
        assert_eq!(ServerState::Uninitialized, ServerState::Uninitialized);
        assert_ne!(ServerState::Uninitialized, ServerState::Ready);
        assert_ne!(ServerState::Ready, ServerState::Shutdown);
    }

    async fn create_test_server_with_repository() -> McpServer {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        // Create test files for server testing
        fs::write(
            repo_path.join("app.py"),
            r#"
"""Main application module."""

import logging
from typing import List, Optional, Dict, Any
from dataclasses import dataclass

@dataclass
class Config:
    """Application configuration."""
    database_url: str
    api_key: str
    debug: bool = False

class ApplicationService:
    """Main application service."""
    
    def __init__(self, config: Config):
        self.config = config
        self.logger = logging.getLogger(__name__)
        self._users: Dict[str, 'User'] = {}
    
    def create_user(self, username: str, email: str) -> 'User':
        """Create a new user."""
        if username in self._users:
            raise ValueError(f"User {username} already exists")
        
        user = User(username=username, email=email)
        self._users[username] = user
        self.logger.info(f"Created user: {username}")
        return user
    
    def get_user(self, username: str) -> Optional['User']:
        """Get a user by username."""
        return self._users.get(username)
    
    def list_users(self) -> List['User']:
        """List all users."""
        return list(self._users.values())
    
    def delete_user(self, username: str) -> bool:
        """Delete a user."""
        if username in self._users:
            del self._users[username]
            self.logger.info(f"Deleted user: {username}")
            return True
        return False

class User:
    """User model."""
    
    def __init__(self, username: str, email: str):
        self.username = username
        self.email = email
        self.created_at = None  # Would be datetime in real app
        self.is_active = True
    
    def deactivate(self) -> None:
        """Deactivate the user."""
        self.is_active = False
    
    def activate(self) -> None:
        """Activate the user."""
        self.is_active = True
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert user to dictionary."""
        return {
            'username': self.username,
            'email': self.email,
            'is_active': self.is_active
        }

def main():
    """Main application entry point."""
    config = Config(
        database_url="postgresql://localhost/myapp",
        api_key="secret-key"
    )
    
    app = ApplicationService(config)
    
    # Create some sample users
    app.create_user("alice", "alice@example.com")
    app.create_user("bob", "bob@example.com")
    
    # List users
    users = app.list_users()
    print(f"Created {len(users)} users")

if __name__ == "__main__":
    main()
"#,
        )
        .unwrap();

        fs::write(
            repo_path.join("utils.py"),
            r#"
"""Utility functions for the application."""

import re
import hashlib
from typing import Optional, Union, List
from datetime import datetime, timedelta

# Constants
EMAIL_REGEX = re.compile(r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$')
PASSWORD_MIN_LENGTH = 8

def validate_email(email: str) -> bool:
    """Validate email address format."""
    return bool(EMAIL_REGEX.match(email))

def validate_password(password: str) -> bool:
    """Validate password strength."""
    if len(password) < PASSWORD_MIN_LENGTH:
        return False
    
    has_upper = any(c.isupper() for c in password)
    has_lower = any(c.islower() for c in password)
    has_digit = any(c.isdigit() for c in password)
    
    return has_upper and has_lower and has_digit

def hash_password(password: str, salt: Optional[str] = None) -> str:
    """Hash a password with salt."""
    if salt is None:
        salt = "default_salt"  # In real app, use random salt
    
    combined = f"{password}{salt}"
    return hashlib.sha256(combined.encode()).hexdigest()

def generate_token(length: int = 32) -> str:
    """Generate a random token."""
    import secrets
    return secrets.token_hex(length)

class DateUtils:
    """Utility class for date operations."""
    
    @staticmethod
    def now() -> datetime:
        """Get current datetime."""
        return datetime.now()
    
    @staticmethod
    def add_days(date: datetime, days: int) -> datetime:
        """Add days to a date."""
        return date + timedelta(days=days)
    
    @staticmethod
    def format_date(date: datetime, format_str: str = "%Y-%m-%d") -> str:
        """Format a date as string."""
        return date.strftime(format_str)

def cleanup_string(text: str) -> str:
    """Clean up a string by removing extra whitespace."""
    return re.sub(r'\s+', ' ', text.strip())

def parse_config_value(value: str) -> Union[str, int, bool]:
    """Parse a configuration value to appropriate type."""
    # Try boolean
    if value.lower() in ('true', 'false'):
        return value.lower() == 'true'
    
    # Try integer
    try:
        return int(value)
    except ValueError:
        pass
    
    # Return as string
    return value
"#,
        )
        .unwrap();

        let server = McpServer::new_with_config(
            2048,  // memory_limit_mb
            20,    // batch_size
            5,     // max_file_size_mb
            false, // disable_memory_limit
            vec!["__pycache__".to_string(), ".pytest_cache".to_string()],
            Some(vec!["py".to_string()]),
            Some("exclude".to_string()),
        )
        .expect("Failed to create MCP server");

        server
            .initialize_with_repository(repo_path)
            .await
            .expect("Failed to initialize repository");

        // Keep temp_dir alive
        std::mem::forget(temp_dir);

        server
    }

    #[tokio::test]
    async fn test_server_with_repository_initialization() {
        let server = create_test_server_with_repository().await;

        // Server should be properly configured
        assert_eq!(server.state(), ServerState::Uninitialized);

        // Should have repository configured
        let prism_server = server.prism_server.read().await;
        assert!(prism_server.repository_path().is_some());
    }

    #[tokio::test]
    async fn test_full_mcp_workflow() {
        let mut server = create_test_server_with_repository().await;

        // 1. Initialize the MCP server
        let init_params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        let init_result = server
            .handle_initialize(Some(serde_json::to_value(init_params).unwrap()))
            .await;
        assert!(init_result.is_ok());

        // Server should have client info
        assert!(server.client_info().is_some());

        // 2. Test resources/list
        let resources_result = server.handle_resources_list(None).await;
        assert!(resources_result.is_ok());

        let resources_value = resources_result.unwrap();
        let resources: crate::resources::ListResourcesResult =
            serde_json::from_value(resources_value).unwrap();
        assert!(!resources.resources.is_empty());

        // Should have various resource types
        let uris: Vec<String> = resources.resources.iter().map(|r| r.uri.clone()).collect();
        assert!(uris.iter().any(|uri| uri == "prism://repository/stats"));
        assert!(uris.iter().any(|uri| uri == "prism://graph/repository"));
        assert!(uris.iter().any(|uri| uri.contains("app.py")));

        // 3. Test resources/read
        let read_params = crate::resources::ReadResourceParams {
            uri: "prism://repository/stats".to_string(),
        };
        let read_result = server
            .handle_resources_read(Some(serde_json::to_value(read_params).unwrap()))
            .await;
        assert!(read_result.is_ok());

        // 4. Test tools/list
        let tools_result = server.handle_tools_list(None).await;
        assert!(tools_result.is_ok());

        let tools_value = tools_result.unwrap();
        let tools: crate::tools::ListToolsResult = serde_json::from_value(tools_value).unwrap();
        assert_eq!(tools.tools.len(), 23); // All 23 tools should be available including all implemented tools

        // 5. Test tools/call with repository_stats
        let tool_params = crate::tools::CallToolParams {
            name: "repository_stats".to_string(),
            arguments: Some(serde_json::json!({})),
        };
        let tool_result = server
            .handle_tools_call(Some(serde_json::to_value(tool_params).unwrap()))
            .await;
        assert!(tool_result.is_ok());

        // 6. Test prompts/list
        let prompts_result = server.handle_prompts_list(None).await;
        assert!(prompts_result.is_ok());

        let prompts_value = prompts_result.unwrap();
        let prompts: crate::prompts::ListPromptsResult =
            serde_json::from_value(prompts_value).unwrap();
        assert_eq!(prompts.prompts.len(), 16); // All 16 prompts should be available (original 8 + 8 new for large codebase understanding)

        // 7. Test prompts/get
        let prompt_params = crate::prompts::GetPromptParams {
            name: "repository_overview".to_string(),
            arguments: Some(serde_json::Map::from_iter([(
                "focus_area".to_string(),
                serde_json::Value::String("architecture".to_string()),
            )])),
        };
        let prompt_result = server
            .handle_prompts_get(Some(serde_json::to_value(prompt_params).unwrap()))
            .await;
        assert!(prompt_result.is_ok());
    }

    #[tokio::test]
    async fn test_request_handling_errors() {
        let mut server = McpServer::new().expect("Failed to create MCP server");

        // Test invalid method
        let invalid_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::Value::Number(1.into()),
            method: "invalid_method".to_string(),
            params: None,
        };

        let response = server.handle_request(invalid_request).await;
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32601); // Method not found

        // Test missing required parameters
        let missing_params_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::Value::Number(2.into()),
            method: "resources/read".to_string(),
            params: None, // Missing required uri parameter
        };

        let response = server.handle_request(missing_params_request).await;
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32602); // Invalid params
    }

    #[tokio::test]
    async fn test_notification_handling() {
        let mut server = McpServer::new().expect("Failed to create MCP server");

        // Test initialized notification
        let initialized_notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "initialized".to_string(),
            params: None,
        };

        assert_eq!(server.state(), ServerState::Uninitialized);

        let result = server.handle_notification(initialized_notification).await;
        assert!(result.is_ok());
        assert_eq!(server.state(), ServerState::Ready);

        // Test unknown notification
        let unknown_notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "unknown_notification".to_string(),
            params: None,
        };

        let result = server.handle_notification(unknown_notification).await;
        assert!(result.is_ok()); // Should not fail, just log warning
    }

    #[tokio::test]
    async fn test_message_handling() {
        let mut server = McpServer::new().expect("Failed to create MCP server");

        // Test request message handling
        let request_message = crate::transport::TransportMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::Value::Number(1.into()),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            })),
        });

        let response = server.handle_message(request_message).await;
        assert!(response.is_ok());
        assert!(response.unwrap().is_some()); // Should return a response

        // Test notification message handling
        let notification_message =
            crate::transport::TransportMessage::Notification(JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "initialized".to_string(),
                params: None,
            });

        let response = server.handle_message(notification_message).await;
        assert!(response.is_ok());
        assert!(response.unwrap().is_none()); // Notifications don't return responses
    }

    #[tokio::test]
    async fn test_server_capabilities_validation() {
        let server = create_test_server_with_repository().await;
        let prism_server = server.prism_server.read().await;
        let capabilities = prism_server.capabilities();

        // Verify all required capabilities are present
        assert!(capabilities.resources.is_some());
        assert!(capabilities.tools.is_some());
        assert!(capabilities.prompts.is_some());

        // Verify resource capabilities
        let resource_caps = capabilities.resources.as_ref().unwrap();
        assert_eq!(resource_caps.subscribe, Some(true));
        assert_eq!(resource_caps.list_changed, Some(true));

        // Verify tool capabilities
        let tool_caps = capabilities.tools.as_ref().unwrap();
        assert_eq!(tool_caps.list_changed, Some(true));

        // Verify prompt capabilities
        let prompt_caps = capabilities.prompts.as_ref().unwrap();
        assert_eq!(prompt_caps.list_changed, Some(false));
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        use std::sync::Arc;
        use tokio::sync::RwLock;

        let server = Arc::new(RwLock::new(create_test_server_with_repository().await));

        // Initialize the server first
        {
            let mut server_lock = server.write().await;
            let init_params = InitializeParams {
                protocol_version: "2024-11-05".to_string(),
                capabilities: ClientCapabilities::default(),
                client_info: ClientInfo {
                    name: "test-client".to_string(),
                    version: "1.0.0".to_string(),
                },
            };

            server_lock
                .handle_initialize(Some(serde_json::to_value(init_params).unwrap()))
                .await
                .unwrap();
        }

        // Run multiple concurrent requests
        let mut handles = Vec::new();

        for i in 0..5 {
            let server_clone = server.clone();
            let handle = tokio::spawn(async move {
                let server_lock = server_clone.write().await;

                // Test resources/list
                let resources_result = server_lock.handle_resources_list(None).await;
                assert!(resources_result.is_ok());

                // Test tools/list
                let tools_result = server_lock.handle_tools_list(None).await;
                assert!(tools_result.is_ok());

                i // Return the task number
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_server_info_serialization() {
        let server_info = ServerInfo {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
        };

        let json = serde_json::to_string(&server_info).unwrap();
        let deserialized: ServerInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(server_info.name, deserialized.name);
        assert_eq!(server_info.version, deserialized.version);
    }

    #[test]
    fn test_client_info_serialization() {
        let client_info = ClientInfo {
            name: "test-client".to_string(),
            version: "2.0.0".to_string(),
        };

        let json = serde_json::to_string(&client_info).unwrap();
        let deserialized: ClientInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(client_info.name, deserialized.name);
        assert_eq!(client_info.version, deserialized.version);
    }
}
