//! MCP client module
//!
//! This module provides a comprehensive wrapper around the official rmcp SDK
//! for connecting to and interacting with MCP servers.

use crate::error::{Error, Result};
use rmcp::model::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Configuration for connecting to an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Command to execute the server
    pub command: String,
    /// Arguments to pass to the server command
    pub args: Vec<String>,
    /// Environment variables for the server process
    pub env: HashMap<String, String>,
    /// Working directory for the server process
    pub working_dir: Option<PathBuf>,
    /// Transport mechanism to use
    pub transport: Transport,
    /// Timeout for server startup
    pub startup_timeout: Duration,
    /// Timeout for server shutdown
    pub shutdown_timeout: Duration,
}

/// Transport mechanisms for MCP communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Transport {
    /// Standard input/output transport
    Stdio,
    /// HTTP transport
    Http { url: String },
    /// Server-Sent Events transport
    Sse { url: String },
}

/// Connection state of the MCP client
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected to any server
    Disconnected,
    /// Currently establishing connection
    Connecting,
    /// Successfully connected and ready
    Connected,
    /// Connection failed or encountered error
    Error(String),
}

/// Information about the connected MCP server
#[derive(Debug, Clone)]
pub struct ServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
    /// Supported MCP protocol version
    pub protocol_version: String,
    /// Server capabilities (as JSON value for now)
    pub capabilities: serde_json::Value,
}

/// Main MCP client for test harness
pub struct McpClient {
    /// rmcp service instance - PLANNED(#189): integrate with rmcp SDK service
    service: Option<Box<dyn std::any::Any + Send + Sync>>,
    /// Server process management
    server_process: Option<ServerProcess>,
    /// Client configuration
    config: ServerConfig,
    /// Current connection state
    connection_state: ConnectionState,
    /// Server information (available after connection)
    server_info: Option<ServerInfo>,
}

/// Manages external MCP server processes
pub struct ServerProcess {
    /// Child process handle
    child: Option<Child>,
    /// Process configuration
    config: ServerConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            command: "echo".to_string(),
            args: vec!["mcp".to_string()],
            env: HashMap::new(),
            working_dir: None,
            transport: Transport::Stdio,
            startup_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(5),
        }
    }
}

impl McpClient {
    /// Create a new MCP client with the given configuration
    pub async fn new(config: ServerConfig) -> Result<Self> {
        debug!("Creating new MCP client with config: {:?}", config);

        Ok(Self {
            service: None,
            server_process: None,
            config,
            connection_state: ConnectionState::Disconnected,
            server_info: None,
        })
    }

    /// Connect to the MCP server
    pub async fn connect(&mut self) -> Result<()> {
        if self.connection_state == ConnectionState::Connected {
            return Ok(());
        }

        self.connection_state = ConnectionState::Connecting;
        info!(
            "Connecting to MCP server: {} {:?}",
            self.config.command, self.config.args
        );

        // Start server process if using stdio transport
        if matches!(self.config.transport, Transport::Stdio) {
            let process = ServerProcess::start(&self.config).await?;
            self.server_process = Some(process);
        }

        // PLANNED(#189): Integrate actual rmcp service connection
        // Current implementation: Process management and connection state tracking
        self.connection_state = ConnectionState::Connected;

        info!("Successfully connected to MCP server");
        Ok(())
    }

    /// Disconnect from the MCP server
    pub async fn disconnect(&mut self) -> Result<()> {
        if self.connection_state == ConnectionState::Disconnected {
            return Ok(());
        }

        info!("Disconnecting from MCP server");

        // Stop server process if running
        if let Some(mut process) = self.server_process.take() {
            process.stop().await?;
        }

        self.service = None;
        self.server_info = None;
        self.connection_state = ConnectionState::Disconnected;

        info!("Successfully disconnected from MCP server");
        Ok(())
    }

    /// Get current connection state
    pub fn connection_state(&self) -> &ConnectionState {
        &self.connection_state
    }

    /// Get server information (if connected)
    pub fn server_info(&self) -> Option<&ServerInfo> {
        self.server_info.as_ref()
    }

    /// Check if client is connected
    pub fn is_connected(&self) -> bool {
        matches!(self.connection_state, ConnectionState::Connected)
    }

    /// List all available tools from the server
    pub async fn list_tools(&self) -> Result<Vec<Tool>> {
        if !self.is_connected() {
            return Err(Error::connection("Client not connected to server"));
        }

        // PLANNED(#189): Implement actual tool listing with rmcp SDK
        warn!("Tool listing not yet implemented");
        Ok(vec![])
    }

    /// Call a tool with the given parameters
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<CallToolResult> {
        if !self.is_connected() {
            return Err(Error::connection("Client not connected to server"));
        }

        debug!("Calling tool '{}' with arguments: {:?}", name, arguments);

        // PLANNED(#189): Implement actual tool calling with rmcp SDK
        warn!("Tool calling not yet implemented");

        // Return mock result for testing
        Ok(CallToolResult {
            content: vec![],
            is_error: Some(false),
        })
    }

    /// List all available resources from the server
    pub async fn list_resources(&self) -> Result<Vec<Resource>> {
        if !self.is_connected() {
            return Err(Error::connection("Client not connected to server"));
        }

        // PLANNED(#189): Implement actual resource listing with rmcp SDK
        warn!("Resource listing not yet implemented");
        Ok(vec![])
    }

    /// Read a resource by URI
    pub async fn read_resource(&self, uri: &str) -> Result<ReadResourceResult> {
        if !self.is_connected() {
            return Err(Error::connection("Client not connected to server"));
        }

        debug!("Reading resource: {}", uri);

        // PLANNED(#189): Implement actual resource reading with rmcp SDK
        warn!("Resource reading not yet implemented");

        // Return mock result for testing
        Ok(ReadResourceResult { contents: vec![] })
    }

    /// Check if the server is healthy and responsive
    pub async fn health_check(&self) -> Result<bool> {
        if !self.is_connected() {
            return Ok(false);
        }

        // PLANNED(#189): Implement actual health check with ping or tool listing
        warn!("Health check not yet implemented");
        Ok(true)
    }
}

impl ServerProcess {
    /// Start a new server process with the given configuration
    pub async fn start(config: &ServerConfig) -> Result<Self> {
        debug!(
            "Starting server process: {} {:?}",
            config.command, config.args
        );

        let mut cmd = Command::new(&config.command);

        // Add arguments
        for arg in &config.args {
            cmd.arg(arg);
        }

        // Set environment variables
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        // Set working directory
        if let Some(working_dir) = &config.working_dir {
            cmd.current_dir(working_dir);
        }

        // Configure stdio for MCP communication
        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        // Spawn the process
        let child = cmd
            .spawn()
            .map_err(|e| Error::connection(format!("Failed to start server process: {}", e)))?;

        info!("Server process started with PID: {:?}", child.id());

        Ok(Self {
            child: Some(child),
            config: config.clone(),
        })
    }

    /// Stop the server process gracefully
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            debug!("Stopping server process with PID: {:?}", child.id());

            // Send termination signal
            let kill_result = child.kill().await;
            if let Err(e) = kill_result {
                warn!("Failed to send kill signal to process: {}", e);
            }

            // Wait for process to exit with timeout
            match timeout(self.config.shutdown_timeout, child.wait()).await {
                Ok(Ok(status)) => {
                    info!("Server process exited with status: {:?}", status);
                    Ok(())
                }
                Ok(Err(e)) => Err(Error::connection(format!(
                    "Error waiting for process to exit: {}",
                    e
                ))),
                Err(_) => {
                    warn!("Server process did not exit within timeout, forcing termination");
                    let _ = child.kill().await;
                    Err(Error::connection("Server process shutdown timeout"))
                }
            }
        } else {
            debug!("No server process to stop");
            Ok(())
        }
    }

    /// Check if the server process is still running
    pub fn is_running(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(_)) => false, // Process has exited
                Ok(None) => true,     // Process is still running
                Err(_) => false,      // Error checking status, assume not running
            }
        } else {
            false
        }
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        if self.is_connected() {
            warn!(
                "McpClient dropped while still connected - this may leave server processes running"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;

    fn create_test_config() -> ServerConfig {
        ServerConfig {
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            env: HashMap::new(),
            working_dir: None,
            transport: Transport::Stdio,
            startup_timeout: Duration::from_secs(5),
            shutdown_timeout: Duration::from_secs(2),
        }
    }

    fn create_test_config_with_working_dir() -> (ServerConfig, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = ServerConfig {
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            env: [("TEST_VAR".to_string(), "test_value".to_string())].into(),
            working_dir: Some(temp_dir.path().to_path_buf()),
            transport: Transport::Stdio,
            startup_timeout: Duration::from_secs(5),
            shutdown_timeout: Duration::from_secs(2),
        };
        (config, temp_dir)
    }

    #[tokio::test]
    async fn test_client_creation() {
        let config = create_test_config();
        let client = McpClient::new(config).await;

        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.connection_state(), &ConnectionState::Disconnected);
        assert!(!client.is_connected());
        assert!(client.server_info().is_none());
    }

    #[tokio::test]
    async fn test_connection_lifecycle() {
        let config = create_test_config();
        let mut client = McpClient::new(config).await.unwrap();

        // Initial state
        assert_eq!(client.connection_state(), &ConnectionState::Disconnected);
        assert!(!client.is_connected());

        // Connect
        let result = client.connect().await;
        assert!(result.is_ok());
        assert_eq!(client.connection_state(), &ConnectionState::Connected);
        assert!(client.is_connected());

        // Disconnect
        let result = client.disconnect().await;
        assert!(result.is_ok());
        assert_eq!(client.connection_state(), &ConnectionState::Disconnected);
        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn test_double_connect() {
        let config = create_test_config();
        let mut client = McpClient::new(config).await.unwrap();

        // First connection
        client.connect().await.unwrap();
        assert!(client.is_connected());

        // Second connection should succeed (no-op)
        let result = client.connect().await;
        assert!(result.is_ok());
        assert!(client.is_connected());

        client.disconnect().await.unwrap();
    }

    #[tokio::test]
    async fn test_double_disconnect() {
        let config = create_test_config();
        let mut client = McpClient::new(config).await.unwrap();

        // Connect first
        client.connect().await.unwrap();

        // First disconnect
        client.disconnect().await.unwrap();
        assert!(!client.is_connected());

        // Second disconnect should succeed (no-op)
        let result = client.disconnect().await;
        assert!(result.is_ok());
        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn test_operations_while_disconnected() {
        let config = create_test_config();
        let client = McpClient::new(config).await.unwrap();

        // All operations should fail when disconnected
        assert!(!client.is_connected());

        let result = client.list_tools().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Connection(_)));

        let result = client.call_tool("test", None).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Connection(_)));

        let result = client.list_resources().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Connection(_)));

        let result = client.read_resource("test://uri").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Connection(_)));

        let result = client.health_check().await;
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false when disconnected
    }

    #[tokio::test]
    async fn test_operations_while_connected() {
        let config = create_test_config();
        let mut client = McpClient::new(config).await.unwrap();

        client.connect().await.unwrap();
        assert!(client.is_connected());

        // All operations succeed with connection validated - PLANNED(#189): actual MCP communication
        let result = client.list_tools().await;
        assert!(result.is_ok());

        let result = client.call_tool("test", None).await;
        assert!(result.is_ok());

        let result = client.list_resources().await;
        assert!(result.is_ok());

        let result = client.read_resource("test://uri").await;
        assert!(result.is_ok());

        let result = client.health_check().await;
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should return true when connected

        client.disconnect().await.unwrap();
    }

    #[tokio::test]
    async fn test_server_process_lifecycle() {
        let config = create_test_config();
        let mut process = ServerProcess::start(&config).await.unwrap();

        // Process should be running initially
        assert!(process.is_running());

        // Stop the process
        let result = process.stop().await;
        assert!(result.is_ok());

        // Process should no longer be running
        assert!(!process.is_running());
    }

    #[tokio::test]
    async fn test_server_process_with_working_dir() {
        let (config, _temp_dir) = create_test_config_with_working_dir();
        let mut process = ServerProcess::start(&config).await.unwrap();

        assert!(process.is_running());

        let result = process.stop().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.command, "echo");
        assert_eq!(config.args, vec!["mcp"]);
        assert!(config.env.is_empty());
        assert!(config.working_dir.is_none());
        assert!(matches!(config.transport, Transport::Stdio));
        assert_eq!(config.startup_timeout, Duration::from_secs(10));
        assert_eq!(config.shutdown_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_connection_state_equality() {
        assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
        assert_eq!(ConnectionState::Connecting, ConnectionState::Connecting);
        assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
        assert_eq!(
            ConnectionState::Error("test".to_string()),
            ConnectionState::Error("test".to_string())
        );

        assert_ne!(ConnectionState::Disconnected, ConnectionState::Connected);
        assert_ne!(
            ConnectionState::Error("test1".to_string()),
            ConnectionState::Error("test2".to_string())
        );
    }

    #[test]
    fn test_transport_serialization() {
        // Test that Transport can be serialized/deserialized
        let transports = vec![
            Transport::Stdio,
            Transport::Http {
                url: "http://localhost:8080".to_string(),
            },
            Transport::Sse {
                url: "http://localhost:8080/events".to_string(),
            },
        ];

        for transport in transports {
            let json = serde_json::to_string(&transport).unwrap();
            let deserialized: Transport = serde_json::from_str(&json).unwrap();

            match (&transport, &deserialized) {
                (Transport::Stdio, Transport::Stdio) => {}
                (Transport::Http { url: url1 }, Transport::Http { url: url2 }) => {
                    assert_eq!(url1, url2);
                }
                (Transport::Sse { url: url1 }, Transport::Sse { url: url2 }) => {
                    assert_eq!(url1, url2);
                }
                _ => panic!("Transport serialization mismatch"),
            }
        }
    }
}
