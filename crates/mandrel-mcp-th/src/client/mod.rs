//! MCP client module
//!
//! This module provides a comprehensive wrapper around the official rmcp SDK
//! for connecting to and interacting with MCP servers.

use crate::error::{Error, Result};
use rmcp::model::*;
use rmcp::service::{RoleClient, ServiceExt};
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
    /// Timeout for individual operations
    pub operation_timeout: Duration,
    /// Maximum number of retries for failed operations
    pub max_retries: u32,
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
    /// Server capabilities
    pub capabilities: ServerCapabilities,
}

/// Main MCP client for test harness
pub struct McpClient {
    /// rmcp running service instance
    service: Option<rmcp::service::RunningService<RoleClient, ()>>,
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
            operation_timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }
}

impl From<crate::spec::ServerConfig> for ServerConfig {
    fn from(spec_config: crate::spec::ServerConfig) -> Self {
        Self {
            command: spec_config.command,
            args: spec_config.args,
            env: spec_config.env,
            working_dir: spec_config.working_dir.map(PathBuf::from),
            transport: Transport::Stdio, // NOTE: Hardcoded for now
            startup_timeout: Duration::from_secs(spec_config.startup_timeout_seconds as u64),
            shutdown_timeout: Duration::from_secs(spec_config.shutdown_timeout_seconds as u64),
            operation_timeout: Duration::from_secs(30), // Default value
            max_retries: 3,                             // Default value
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

        // Clone transport config to avoid borrowing issues
        let transport = self.config.transport.clone();
        match transport {
            Transport::Stdio => {
                self.connect_stdio().await?;
            }
            #[cfg(feature = "transport-streamable-http-client")]
            Transport::Http { url } => {
                self.connect_http(&url).await?;
            }
            #[cfg(not(feature = "transport-streamable-http-client"))]
            Transport::Http { url: _ } => {
                return Err(Error::connection(
                    "HTTP transport not supported - enable 'transport-streamable-http-client' feature".to_string(),
                ));
            }
            #[cfg(feature = "transport-sse-client")]
            Transport::Sse { url } => {
                self.connect_sse(&url).await?;
            }
            #[cfg(not(feature = "transport-sse-client"))]
            Transport::Sse { url: _ } => {
                return Err(Error::connection(
                    "SSE transport not supported - enable 'transport-sse-client' feature"
                        .to_string(),
                ));
            }
        }

        // After connection, get server info and capabilities
        self.discover_capabilities().await?;

        self.connection_state = ConnectionState::Connected;
        info!("Successfully connected to MCP server");
        Ok(())
    }

    /// Connect using stdio transport
    async fn connect_stdio(&mut self) -> Result<()> {
        // Start server process
        let process = ServerProcess::start(&self.config).await?;
        self.server_process = Some(process);

        // Create transport
        let mut cmd = Command::new(&self.config.command);
        for arg in &self.config.args {
            cmd.arg(arg);
        }

        for (key, value) in &self.config.env {
            cmd.env(key, value);
        }

        if let Some(working_dir) = &self.config.working_dir {
            cmd.current_dir(working_dir);
        }

        let transport = rmcp::transport::TokioChildProcess::new(cmd)
            .map_err(|e| Error::connection(format!("Failed to create stdio transport: {e}")))?;

        // Create service using the correct pattern
        let service = ()
            .serve(transport)
            .await
            .map_err(|e| Error::connection(format!("Failed to create MCP service: {e}")))?;

        self.service = Some(service);
        Ok(())
    }

    /// Connect using HTTP transport
    #[cfg(feature = "transport-streamable-http-client")]
    async fn connect_http(&mut self, url: &str) -> Result<()> {
        use rmcp::transport::streamable_http_client::StreamableHttpClientTransport;

        let transport = StreamableHttpClientTransport::from_uri(url.to_string());

        let service = ()
            .serve(transport)
            .await
            .map_err(|e| Error::connection(format!("Failed to create MCP service: {e}")))?;

        self.service = Some(service);
        Ok(())
    }

    /// Connect using SSE transport
    #[cfg(feature = "transport-sse-client")]
    async fn connect_sse(&mut self, _url: &str) -> Result<()> {
        // SSE client requires reqwest::Client implementation for HTTP client interface
        // The RMCP SSE transport expects a type implementing SseClient trait
        // HTTP transport is recommended as it's simpler to configure
        Err(Error::connection(
            "SSE transport requires reqwest::Client implementation - use HTTP transport instead"
                .to_string(),
        ))
    }

    /// Discover server capabilities after connection
    async fn discover_capabilities(&mut self) -> Result<()> {
        if let Some(service) = &self.service {
            let peer_info = service.peer_info();

            if let Some(peer_info) = peer_info {
                self.server_info = Some(ServerInfo {
                    name: peer_info.server_info.name.clone(),
                    version: peer_info.server_info.version.clone(),
                    protocol_version: format!("{}", peer_info.protocol_version),
                    capabilities: peer_info.capabilities.clone(),
                });

                info!(
                    "Connected to {} v{} (protocol: {})",
                    peer_info.server_info.name,
                    peer_info.server_info.version,
                    peer_info.protocol_version
                );
                debug!("Server capabilities: {:?}", peer_info.capabilities);
            }
        }

        Ok(())
    }

    /// Disconnect from the MCP server
    pub async fn disconnect(&mut self) -> Result<()> {
        if self.connection_state == ConnectionState::Disconnected {
            return Ok(());
        }

        info!("Disconnecting from MCP server");

        // Cancel service first
        if let Some(service) = self.service.take() {
            service
                .cancel()
                .await
                .map_err(|e| Error::connection(format!("Failed to cancel service: {e}")))?;
        }

        // Stop server process if running
        if let Some(mut process) = self.server_process.take() {
            process.stop().await?;
        }

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

        let service = self.service.as_ref().unwrap();
        let tools = service
            .list_all_tools()
            .await
            .map_err(|e| Error::execution(format!("Failed to list tools: {e}")))?;

        debug!("Listed {} tools", tools.len());
        Ok(tools)
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

        // Convert arguments from Value to Map if needed
        let arguments_map = arguments.and_then(|v| {
            if let serde_json::Value::Object(map) = v {
                Some(map)
            } else {
                None
            }
        });

        let service = self.service.as_ref().unwrap();
        let result = timeout(
            self.config.operation_timeout,
            service.call_tool(CallToolRequestParam {
                name: name.to_string().into(),
                arguments: arguments_map,
            }),
        )
        .await
        .map_err(|_| Error::execution("Tool call timeout"))?
        .map_err(|e| Error::execution(format!("Tool call failed: {e}")))?;

        debug!("Tool call result: {:?}", result);
        Ok(result)
    }

    /// Call a tool with retry logic
    pub async fn call_tool_with_retry(
        &self,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<CallToolResult> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            match self.call_tool(name, arguments.clone()).await {
                Ok(result) => return Ok(result),
                Err(Error::Connection(_)) if attempt < self.config.max_retries => {
                    warn!("Tool call attempt {} failed, retrying...", attempt + 1);
                    tokio::time::sleep(Duration::from_millis(1000 * (attempt + 1) as u64)).await;
                    last_error = Some(Error::execution("All retries failed"));
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error.unwrap_or(Error::execution("All retries failed")))
    }

    /// List all available resources from the server
    pub async fn list_resources(&self) -> Result<Vec<Resource>> {
        if !self.is_connected() {
            return Err(Error::connection("Client not connected to server"));
        }

        let service = self.service.as_ref().unwrap();
        let resources = service
            .list_all_resources()
            .await
            .map_err(|e| Error::execution(format!("Failed to list resources: {e}")))?;

        debug!("Listed {} resources", resources.len());
        Ok(resources)
    }

    /// Read a resource by URI
    pub async fn read_resource(&self, uri: &str) -> Result<ReadResourceResult> {
        if !self.is_connected() {
            return Err(Error::connection("Client not connected to server"));
        }

        debug!("Reading resource: {}", uri);

        let service = self.service.as_ref().unwrap();
        let result = timeout(
            self.config.operation_timeout,
            service.read_resource(ReadResourceRequestParam {
                uri: uri.to_string(),
            }),
        )
        .await
        .map_err(|_| Error::execution("Resource read timeout"))?
        .map_err(|e| Error::execution(format!("Resource read failed: {e}")))?;

        debug!("Resource read result: {:?}", result);
        Ok(result)
    }

    /// List all available prompts from the server
    pub async fn list_prompts(&self) -> Result<Vec<Prompt>> {
        if !self.is_connected() {
            return Err(Error::connection("Client not connected to server"));
        }

        let service = self.service.as_ref().unwrap();
        let prompts = service
            .list_all_prompts()
            .await
            .map_err(|e| Error::execution(format!("Failed to list prompts: {e}")))?;

        debug!("Listed {} prompts", prompts.len());
        Ok(prompts)
    }

    /// Get a prompt with the given parameters
    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<GetPromptResult> {
        if !self.is_connected() {
            return Err(Error::connection("Client not connected to server"));
        }

        debug!("Getting prompt '{}' with arguments: {:?}", name, arguments);

        // Convert arguments from Value to Map if needed
        let arguments_map = arguments.and_then(|v| {
            if let serde_json::Value::Object(map) = v {
                Some(map)
            } else {
                None
            }
        });

        let service = self.service.as_ref().unwrap();
        let result = timeout(
            self.config.operation_timeout,
            service.get_prompt(GetPromptRequestParam {
                name: name.to_string(),
                arguments: arguments_map,
            }),
        )
        .await
        .map_err(|_| Error::execution("Prompt get timeout"))?
        .map_err(|e| Error::execution(format!("Prompt get failed: {e}")))?;

        debug!("Prompt get result: {:?}", result);
        Ok(result)
    }

    /// Check if the server is healthy and responsive
    pub async fn health_check(&self) -> Result<bool> {
        if !self.is_connected() {
            return Ok(false);
        }

        // Try to list tools as a health check
        match self.list_tools().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get server capabilities
    pub fn get_capabilities(&self) -> Option<&ServerCapabilities> {
        self.server_info.as_ref().map(|info| &info.capabilities)
    }

    /// Check if server supports tools
    pub fn supports_tools(&self) -> bool {
        self.get_capabilities()
            .map(|caps| caps.tools.is_some())
            .unwrap_or(false)
    }

    /// Check if server supports resources
    pub fn supports_resources(&self) -> bool {
        self.get_capabilities()
            .map(|caps| caps.resources.is_some())
            .unwrap_or(false)
    }

    /// Check if server supports prompts
    pub fn supports_prompts(&self) -> bool {
        self.get_capabilities()
            .map(|caps| caps.prompts.is_some())
            .unwrap_or(false)
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
            .map_err(|e| Error::connection(format!("Failed to start server process: {e}")))?;

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
                    "Error waiting for process to exit: {e}"
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
            operation_timeout: Duration::from_secs(30),
            max_retries: 3,
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
            operation_timeout: Duration::from_secs(30),
            max_retries: 3,
        };
        (config, temp_dir)
    }

    #[tokio::test]
    async fn test_client_creation() {
        let config = create_test_config();
        let client = McpClient::new(config).await;

        assert!(client.is_ok(), "Client creation should succeed");
        let client = client.unwrap();
        assert_eq!(client.connection_state(), &ConnectionState::Disconnected);
        assert!(!client.is_connected(), "New client should not be connected");
        assert!(client.server_info().is_none());
    }

    #[tokio::test]
    #[ignore = "Requires real MCP server - echo command is not an MCP server"]
    async fn test_connection_lifecycle() {
        let config = create_test_config();
        let mut client = McpClient::new(config).await.unwrap();

        // Initial state
        assert_eq!(client.connection_state(), &ConnectionState::Disconnected);
        assert!(!client.is_connected(), "New client should not be connected");

        // Connect - NOTE: This will fail with echo command as it's not an MCP server
        let result = client.connect().await;
        assert!(result.is_ok(), "Operation should succeed");
        assert_eq!(client.connection_state(), &ConnectionState::Connected);
        assert!(client.is_connected());

        // Disconnect
        let result = client.disconnect().await;
        assert!(result.is_ok(), "Operation should succeed");
        assert_eq!(client.connection_state(), &ConnectionState::Disconnected);
        assert!(!client.is_connected(), "New client should not be connected");
    }

    #[tokio::test]
    #[ignore = "Requires real MCP server - echo command is not an MCP server"]
    async fn test_double_connect() {
        let config = create_test_config();
        let mut client = McpClient::new(config).await.unwrap();

        // First connection - NOTE: This will fail with echo command as it's not an MCP server
        client.connect().await.unwrap();
        assert!(client.is_connected());

        // Second connection should succeed (no-op)
        let result = client.connect().await;
        assert!(result.is_ok(), "Operation should succeed");
        assert!(client.is_connected());

        client.disconnect().await.unwrap();
    }

    #[tokio::test]
    #[ignore = "Requires real MCP server - echo command is not an MCP server"]
    async fn test_double_disconnect() {
        let config = create_test_config();
        let mut client = McpClient::new(config).await.unwrap();

        // Connect first - NOTE: This will fail with echo command as it's not an MCP server
        client.connect().await.unwrap();

        // First disconnect
        client.disconnect().await.unwrap();
        assert!(!client.is_connected(), "New client should not be connected");

        // Second disconnect should succeed (no-op)
        let result = client.disconnect().await;
        assert!(result.is_ok(), "Operation should succeed");
        assert!(!client.is_connected(), "New client should not be connected");
    }

    #[tokio::test]
    async fn test_operations_while_disconnected() {
        let config = create_test_config();
        let client = McpClient::new(config).await.unwrap();

        // All operations should fail when disconnected
        assert!(!client.is_connected(), "New client should not be connected");

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
        assert!(result.is_ok(), "Operation should succeed");
        assert!(!result.unwrap()); // Should return false when disconnected
    }

    #[tokio::test]
    #[ignore = "Requires real MCP server - echo command is not an MCP server"]
    async fn test_operations_while_connected() {
        let config = create_test_config();
        let mut client = McpClient::new(config).await.unwrap();

        // NOTE: This will fail with echo command as it's not an MCP server
        client.connect().await.unwrap();
        assert!(client.is_connected());

        // All operations succeed with connection validated - PLANNED(#189): actual MCP communication
        let result = client.list_tools().await;
        assert!(result.is_ok(), "Operation should succeed");

        let result = client.call_tool("test", None).await;
        assert!(result.is_ok(), "Operation should succeed");

        let result = client.list_resources().await;
        assert!(result.is_ok(), "Operation should succeed");

        let result = client.read_resource("test://uri").await;
        assert!(result.is_ok(), "Operation should succeed");

        let result = client.health_check().await;
        assert!(result.is_ok(), "Operation should succeed");
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
        assert!(result.is_ok(), "Operation should succeed");

        // Process should no longer be running
        assert!(!process.is_running());
    }

    #[tokio::test]
    async fn test_server_process_with_working_dir() {
        let (config, _temp_dir) = create_test_config_with_working_dir();
        let mut process = ServerProcess::start(&config).await.unwrap();

        assert!(process.is_running());

        let result = process.stop().await;
        assert!(result.is_ok(), "Operation should succeed");
    }

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.command, "echo");
        assert_eq!(config.args, vec!["mcp"]);
        assert!(!config.env.is_empty(), "Should not be empty");
        assert!(config.working_dir.is_none(), "Should be none");
        assert!(matches!(config.transport, Transport::Stdio));
        assert_eq!(config.startup_timeout, Duration::from_secs(10));
        assert_eq!(config.shutdown_timeout, Duration::from_secs(5));
        assert_eq!(config.operation_timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_connection_state_equality_and_transitions() {
        // Test meaningful inequality between different states
        assert_ne!(ConnectionState::Disconnected, ConnectionState::Connected);
        assert_ne!(ConnectionState::Disconnected, ConnectionState::Connecting);
        assert_ne!(ConnectionState::Connecting, ConnectionState::Connected);

        // Test Error state equality with same message
        assert_eq!(
            ConnectionState::Error("test".to_string()),
            ConnectionState::Error("test".to_string()),
            "Error states with same message should be equal"
        );

        // Test Error state inequality with different messages
        assert_ne!(
            ConnectionState::Error("error1".to_string()),
            ConnectionState::Error("error2".to_string()),
            "Error states with different messages should not be equal"
        );

        // Test Error vs other states
        assert_ne!(
            ConnectionState::Error("test".to_string()),
            ConnectionState::Connected
        );
        assert_ne!(
            ConnectionState::Error("test".to_string()),
            ConnectionState::Disconnected
        );
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
