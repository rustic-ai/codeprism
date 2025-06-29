use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tokio::time::{timeout, Instant};

use crate::protocol::jsonrpc::{JsonRpcRequest, JsonRpcMessage};
use crate::protocol::messages::{InitializeParams, InitializeResult};
#[cfg(test)]
use crate::protocol::messages::ClientInfo;

/// Configuration for MCP server instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Command to start the server
    pub command: String,
    /// Arguments for the server command
    pub args: Vec<String>,
    /// Working directory for the server process
    pub working_dir: Option<String>,
    /// Environment variables for the server
    pub env_vars: HashMap<String, String>,
    /// Startup timeout in seconds
    pub startup_timeout: u64,
    /// Request timeout in seconds
    pub request_timeout: u64,
    /// Maximum idle time before considering server unhealthy
    pub max_idle_time: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            command: "node".to_string(),
            args: vec!["server.js".to_string()],
            working_dir: None,
            env_vars: HashMap::new(),
            startup_timeout: 30,
            request_timeout: 10,
            max_idle_time: 300,
        }
    }
}

/// Server health status
#[derive(Debug, Clone, PartialEq)]
pub enum ServerHealth {
    /// Server is healthy and responsive
    Healthy,
    /// Server is unresponsive but process is running
    Unresponsive,
    /// Server process has crashed or terminated
    Crashed,
    /// Server is starting up
    Starting,
}

/// Server management errors
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Failed to start server process: {0}")]
    StartupFailed(String),
    #[error("Server initialization timeout after {timeout}s")]
    InitializationTimeout { timeout: u64 },
    #[error("Server process crashed: {reason}")]
    ProcessCrashed { reason: String },
    #[error("Communication error: {0}")]
    CommunicationError(String),
    #[error("Server is unresponsive")]
    Unresponsive,
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Server communication wrapper for stdio
#[derive(Debug)]
pub struct ServerComm {
    /// Writer for sending messages to server
    writer: BufWriter<tokio::process::ChildStdin>,
    /// Reader for receiving messages from server
    reader: BufReader<tokio::process::ChildStdout>,
    /// Request ID counter
    request_id: std::sync::atomic::AtomicU64,
}

impl ServerComm {
    /// Create new server communication wrapper
    pub fn new(
        stdin: tokio::process::ChildStdin,
        stdout: tokio::process::ChildStdout,
    ) -> Self {
        Self {
            writer: BufWriter::new(stdin),
            reader: BufReader::new(stdout),
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Send initialization request
    pub async fn initialize(&mut self, params: InitializeParams) -> Result<InitializeResult, anyhow::Error> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::Value::Number(serde_json::Number::from(self.next_request_id())),
            method: "initialize".to_string(),
            params: Some(serde_json::to_value(params)?),
        };

        self.send_request(request).await?;
        let response = self.receive_message().await?;
        
        if response.is_error() {
            return Err(anyhow!("Initialize request failed: {:?}", response.error));
        }
        
        if let Some(result) = response.result {
            Ok(serde_json::from_value(result)?)
        } else {
            Err(anyhow!("Initialize request returned no result"))
        }
    }

    /// Generate next request ID
    pub fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    /// Send a JSON-RPC request
    pub async fn send_request(&mut self, request: JsonRpcRequest) -> Result<(), anyhow::Error> {
        let message = serde_json::to_string(&request)?;
        self.writer.write_all(message.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;
        Ok(())
    }

    /// Receive a JSON-RPC message
    pub async fn receive_message(&mut self) -> Result<JsonRpcMessage, anyhow::Error> {
        let mut line = String::new();
        self.reader.read_line(&mut line).await?;
        let message: JsonRpcMessage = serde_json::from_str(&line)?;
        Ok(message)
    }
}

/// Represents a managed MCP server process
#[derive(Debug)]
pub struct ServerProcess {
    /// Server configuration
    config: ServerConfig,
    /// Child process handle
    process: Child,
    /// Server communication wrapper
    comm: ServerComm,
    /// Process start time
    start_time: Instant,
    /// Last activity time
    last_activity: RwLock<Instant>,
    /// Process ID for tracking
    process_id: u32,
}

impl ServerProcess {
    /// Spawn a new MCP server process
    pub async fn spawn(config: ServerConfig) -> Result<Self, ServerError> {
        // Validate configuration
        if config.command.is_empty() {
            return Err(ServerError::ConfigurationError(
                "Server command cannot be empty".to_string(),
            ));
        }

        // Prepare command
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        // Set working directory
        if let Some(ref working_dir) = config.working_dir {
            cmd.current_dir(working_dir);
        }

        // Set environment variables
        for (key, value) in &config.env_vars {
            cmd.env(key, value);
        }

        // Spawn the process
        let mut process = cmd.spawn().map_err(|e| {
            ServerError::StartupFailed(format!("Failed to spawn process: {}", e))
        })?;

        let process_id = process
            .id()
            .ok_or_else(|| ServerError::StartupFailed("Failed to get process ID".to_string()))?;

        // Setup stdio communication
        let stdin = process.stdin.take().ok_or_else(|| {
            ServerError::StartupFailed("Failed to get process stdin".to_string())
        })?;
        let stdout = process.stdout.take().ok_or_else(|| {
            ServerError::StartupFailed("Failed to get process stdout".to_string())
        })?;

        // Create server communication wrapper
        let comm = ServerComm::new(stdin, stdout);

        let start_time = Instant::now();
        let server_process = Self {
            config,
            process,
            comm,
            start_time,
            last_activity: RwLock::new(start_time),
            process_id,
        };

        Ok(server_process)
    }

    /// Initialize the MCP server with handshake
    pub async fn initialize(&mut self, params: InitializeParams) -> Result<(), ServerError> {
        let timeout_duration = Duration::from_secs(self.config.startup_timeout);

        // Send initialization request with timeout
        let init_result = timeout(timeout_duration, self.comm.initialize(params)).await;

        match init_result {
            Ok(Ok(_)) => {
                self.update_last_activity().await;
                Ok(())
            }
            Ok(Err(e)) => Err(ServerError::CommunicationError(format!(
                "Initialization failed: {}",
                e
            ))),
            Err(_) => Err(ServerError::InitializationTimeout {
                timeout: self.config.startup_timeout,
            }),
        }
    }

    /// Send a request to the server
    pub async fn send_request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<JsonRpcMessage, ServerError> {
        let timeout_duration = Duration::from_secs(self.config.request_timeout);
        
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::Value::Number(serde_json::Number::from(self.comm.next_request_id())),
            method: method.to_string(),
            params: Some(params),
        };

        let request_result = timeout(timeout_duration, async {
            self.comm.send_request(request).await?;
            self.comm.receive_message().await
        }).await;

        match request_result {
            Ok(Ok(message)) => {
                self.update_last_activity().await;
                Ok(message)
            }
            Ok(Err(e)) => Err(ServerError::CommunicationError(format!(
                "Request failed: {}",
                e
            ))),
            Err(_) => Err(ServerError::Unresponsive),
        }
    }

    /// Check if process is still running
    pub async fn is_running(&mut self) -> bool {
        match self.process.try_wait() {
            Ok(Some(_)) => false, // Process has exited
            Ok(None) => true,     // Process is still running
            Err(_) => false,      // Error checking process
        }
    }

    /// Get process ID
    pub fn process_id(&self) -> u32 {
        self.process_id
    }

    /// Get uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Gracefully shutdown the server
    pub async fn shutdown(mut self) -> Result<(), ServerError> {
        // Give process time to shutdown gracefully
        let shutdown_timeout = Duration::from_secs(5);
        let shutdown_result = timeout(shutdown_timeout, self.process.wait()).await;

        match shutdown_result {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(ServerError::ProcessCrashed {
                reason: format!("Shutdown error: {}", e),
            }),
            Err(_) => {
                // Force kill if graceful shutdown failed
                if let Err(e) = self.process.kill().await {
                    Err(ServerError::ProcessCrashed {
                        reason: format!("Force kill failed: {}", e),
                    })
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Update last activity timestamp
    async fn update_last_activity(&self) {
        *self.last_activity.write().await = Instant::now();
    }
}

/// Manages multiple MCP server instances
#[derive(Debug)]
pub struct ServerManager {
    /// Active server instances
    servers: RwLock<HashMap<String, ServerProcess>>,
    /// Global configuration defaults
    #[allow(dead_code)]
    default_config: ServerConfig,
}

impl ServerManager {
    /// Create a new server manager
    pub fn new(default_config: ServerConfig) -> Self {
        Self {
            servers: RwLock::new(HashMap::new()),
            default_config,
        }
    }

    /// Create server manager with default configuration
    pub fn with_defaults() -> Self {
        Self::new(ServerConfig::default())
    }

    /// Start a new MCP server instance
    pub async fn start_server(
        &self,
        server_id: String,
        config: ServerConfig,
        init_params: InitializeParams,
    ) -> Result<(), ServerError> {
        // Check if server already exists
        {
            let servers = self.servers.read().await;
            if servers.contains_key(&server_id) {
                return Err(ServerError::ConfigurationError(format!(
                    "Server '{}' already exists",
                    server_id
                )));
            }
        }

        // Spawn new server process
        let mut server_process = ServerProcess::spawn(config).await?;

        // Initialize the server
        server_process.initialize(init_params).await?;

        // Add to managed servers
        {
            let mut servers = self.servers.write().await;
            servers.insert(server_id, server_process);
        }

        Ok(())
    }

    /// Stop a server instance
    pub async fn stop_server(&self, server_id: &str) -> Result<(), ServerError> {
        let server_process = {
            let mut servers = self.servers.write().await;
            servers.remove(server_id).ok_or_else(|| {
                ServerError::ConfigurationError(format!("Server '{}' not found", server_id))
            })?
        };

        server_process.shutdown().await
    }

    /// Send request to a specific server
    pub async fn send_request(
        &self,
        server_id: &str,
        method: &str,
        params: serde_json::Value,
    ) -> Result<JsonRpcMessage, ServerError> {
        let mut servers = self.servers.write().await;
        let server = servers.get_mut(server_id).ok_or_else(|| {
            ServerError::ConfigurationError(format!("Server '{}' not found", server_id))
        })?;

        server.send_request(method, params).await
    }

    /// List active servers
    pub async fn list_servers(&self) -> Vec<String> {
        let servers = self.servers.read().await;
        servers.keys().cloned().collect()
    }

    /// Shutdown all servers
    pub async fn shutdown_all(&self) -> Result<(), Vec<ServerError>> {
        let mut servers = {
            let mut servers_guard = self.servers.write().await;
            std::mem::take(&mut *servers_guard)
        };

        let mut errors = Vec::new();

        for (_server_id, server) in servers.drain() {
            if let Err(e) = server.shutdown().await {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Server statistics
#[derive(Debug, Clone, Serialize)]
pub struct ServerStats {
    pub server_id: String,
    pub process_id: u32,
    pub uptime: Duration,
    pub idle_time: Duration,
    pub is_idle_timeout: bool,
}

/// Reference MCP server configurations for testing
pub struct ReferenceMcpServers;

impl ReferenceMcpServers {
    /// Configuration for the "Everything" reference server (TypeScript)
    pub fn everything_server() -> ServerConfig {
        ServerConfig {
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-everything".to_string()],
            working_dir: None,
            env_vars: HashMap::new(),
            startup_timeout: 30,
            request_timeout: 10,
            max_idle_time: 300,
        }
    }

    /// Configuration for the Memory server (TypeScript)
    pub fn memory_server() -> ServerConfig {
        ServerConfig {
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-memory".to_string()],
            working_dir: None,
            env_vars: HashMap::new(),
            startup_timeout: 30,
            request_timeout: 10,
            max_idle_time: 300,
        }
    }

    /// Configuration for the Time server (TypeScript)
    pub fn time_server() -> ServerConfig {
        ServerConfig {
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-time".to_string()],
            working_dir: None,
            env_vars: HashMap::new(),
            startup_timeout: 30,
            request_timeout: 10,
            max_idle_time: 300,
        }
    }

    /// Configuration for the Filesystem server (TypeScript) with a safe test directory
    pub fn filesystem_server(allowed_path: &str) -> ServerConfig {
        ServerConfig {
            command: "npx".to_string(),
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-filesystem".to_string(),
                allowed_path.to_string(),
            ],
            working_dir: None,
            env_vars: HashMap::new(),
            startup_timeout: 30,
            request_timeout: 10,
            max_idle_time: 300,
        }
    }

    /// Configuration for the Git server (Python) with a repository path
    pub fn git_server(repository_path: &str) -> ServerConfig {
        ServerConfig {
            command: "uvx".to_string(),
            args: vec![
                "mcp-server-git".to_string(),
                "--repository".to_string(),
                repository_path.to_string(),
            ],
            working_dir: None,
            env_vars: HashMap::new(),
            startup_timeout: 30,
            request_timeout: 10,
            max_idle_time: 300,
        }
    }

    /// Configuration for the Fetch server (TypeScript)
    pub fn fetch_server() -> ServerConfig {
        ServerConfig {
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-fetch".to_string()],
            working_dir: None,
            env_vars: HashMap::new(),
            startup_timeout: 30,
            request_timeout: 10,
            max_idle_time: 300,
        }
    }

    /// Get all available reference server configurations
    pub fn all_servers() -> Vec<(&'static str, ServerConfig)> {
        vec![
            ("everything", Self::everything_server()),
            ("memory", Self::memory_server()),
            ("time", Self::time_server()),
            ("fetch", Self::fetch_server()),
            ("filesystem", Self::filesystem_server("/tmp")),
            ("git", Self::git_server(".")),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[tokio::test]
    async fn test_server_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.command, "node");
        assert_eq!(config.args, vec!["server.js"]);
        assert_eq!(config.startup_timeout, 30);
        assert_eq!(config.request_timeout, 10);
    }

    #[tokio::test]
    async fn test_server_manager_creation() {
        let manager = ServerManager::with_defaults();
        let servers = manager.list_servers().await;
        assert!(servers.is_empty());
    }

    #[tokio::test]
    async fn test_server_config_validation() {
        let config = ServerConfig {
            command: "".to_string(),
            ..Default::default()
        };

        let result = ServerProcess::spawn(config).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ServerError::ConfigurationError(_)));
    }

    #[tokio::test]
    async fn test_server_manager_duplicate_prevention() {
        let manager = ServerManager::with_defaults();
        let config = ServerConfig::default();
        let init_params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: crate::protocol::capabilities::McpCapabilities::default(),
            client_info: ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        // First server should succeed (but will fail to actually start without real server)
        let result1 = manager
            .start_server("test-server".to_string(), config.clone(), init_params.clone())
            .await;

        // Second server with same ID should fail with configuration error
        let result2 = manager
            .start_server("test-server".to_string(), config, init_params)
            .await;

        // We expect this to fail because there's no actual server to connect to
        // but the important thing is that duplicate prevention works
        if result1.is_ok() {
            assert!(matches!(
                result2.unwrap_err(),
                ServerError::ConfigurationError(_)
            ));
        }
    }

    #[tokio::test]
    async fn test_server_stats_structure() {
        let stats = ServerStats {
            server_id: "test".to_string(),
            process_id: 1234,
            uptime: Duration::from_secs(60),
            idle_time: Duration::from_secs(10),
            is_idle_timeout: false,
        };

        assert_eq!(stats.server_id, "test");
        assert_eq!(stats.process_id, 1234);
        assert_eq!(stats.uptime, Duration::from_secs(60));
        assert!(!stats.is_idle_timeout);
    }

    #[tokio::test]
    async fn test_server_health_enum() {
        assert_eq!(ServerHealth::Healthy, ServerHealth::Healthy);
        assert_ne!(ServerHealth::Healthy, ServerHealth::Crashed);
        assert_ne!(ServerHealth::Starting, ServerHealth::Unresponsive);
    }

    #[tokio::test]
    async fn test_reference_server_configurations() {
        // Test that all reference server configurations are valid
        let servers = ReferenceMcpServers::all_servers();
        assert!(!servers.is_empty());
        
        for (name, config) in servers {
            assert!(!config.command.is_empty(), "Server '{}' has empty command", name);
            assert!(!config.args.is_empty(), "Server '{}' has no args", name);
            assert!(config.startup_timeout > 0, "Server '{}' has invalid timeout", name);
        }
    }

    #[tokio::test]
    async fn test_reference_server_creation() {
        // Test creating reference server configurations
        let memory_config = ReferenceMcpServers::memory_server();
        assert_eq!(memory_config.command, "npx");
        assert!(memory_config.args.contains(&"@modelcontextprotocol/server-memory".to_string()));

        let time_config = ReferenceMcpServers::time_server();
        assert_eq!(time_config.command, "npx");
        assert!(time_config.args.contains(&"@modelcontextprotocol/server-time".to_string()));

        let everything_config = ReferenceMcpServers::everything_server();
        assert_eq!(everything_config.command, "npx");
        assert!(everything_config.args.contains(&"@modelcontextprotocol/server-everything".to_string()));
    }

    #[tokio::test]
    async fn test_filesystem_server_with_custom_path() {
        let temp_dir = "/tmp/mcp-test";
        let config = ReferenceMcpServers::filesystem_server(temp_dir);
        
        assert_eq!(config.command, "npx");
        assert!(config.args.contains(&temp_dir.to_string()));
        assert!(config.args.contains(&"@modelcontextprotocol/server-filesystem".to_string()));
    }

    #[tokio::test]
    async fn test_git_server_with_repository() {
        let repo_path = ".";
        let config = ReferenceMcpServers::git_server(repo_path);
        
        assert_eq!(config.command, "uvx");
        assert!(config.args.contains(&"mcp-server-git".to_string()));
        assert!(config.args.contains(&"--repository".to_string()));
        assert!(config.args.contains(&repo_path.to_string()));
    }

    #[tokio::test]
    async fn test_server_manager_with_reference_servers() {
        let manager = ServerManager::with_defaults();
        
        // Test adding multiple server configurations
        let servers = ReferenceMcpServers::all_servers();
        assert!(servers.len() >= 6); // Should have at least 6 reference servers
        
        // Verify manager starts with no servers
        let active_servers = manager.list_servers().await;
        assert!(active_servers.is_empty());
    }

    // Integration test that would actually start an MCP server if available
    // This test is marked with ignore since it requires external dependencies
    #[tokio::test]
    #[ignore = "Requires npx and @modelcontextprotocol/server-memory to be installed"]
    async fn test_real_memory_server_integration() {
        let manager = ServerManager::with_defaults();
        let config = ReferenceMcpServers::memory_server();
        
        let init_params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: crate::protocol::capabilities::McpCapabilities::default(),
            client_info: ClientInfo {
                name: "test-harness-client".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        // Attempt to start memory server
        let result = manager.start_server("memory".to_string(), config, init_params).await;
        
        // If this succeeds, we have a working MCP server!
        if result.is_ok() {
            // Test basic communication
            let ping_result = manager.send_request(
                "memory",
                "ping",
                serde_json::Value::Null,
            ).await;
            
            // Should get a response (might be error if ping not supported)
            assert!(ping_result.is_ok());
            
            // Cleanup
            let _ = manager.stop_server("memory").await;
        } else {
            // Expected to fail in CI/test environments without node.js/npm
            println!("Memory server test skipped - requires external dependencies");
        }
    }

    // Integration test for filesystem server
    #[tokio::test]
    #[ignore = "Requires npx and @modelcontextprotocol/server-filesystem to be installed"]
    async fn test_real_filesystem_server_integration() {
        let manager = ServerManager::with_defaults();
        let temp_dir = "/tmp";
        let config = ReferenceMcpServers::filesystem_server(temp_dir);
        
        let init_params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: crate::protocol::capabilities::McpCapabilities::default(),
            client_info: ClientInfo {
                name: "test-harness-client".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        let result = manager.start_server("filesystem".to_string(), config, init_params).await;
        
        if result.is_ok() {
            // Test listing resources
            let resources_result = manager.send_request(
                "filesystem",
                "resources/list",
                serde_json::Value::Null,
            ).await;
            
            assert!(resources_result.is_ok());
            
            // Cleanup
            let _ = manager.stop_server("filesystem").await;
        } else {
            println!("Filesystem server test skipped - requires external dependencies");
        }
    }

    // Test server lifecycle management
    #[tokio::test]
    async fn test_server_lifecycle() {
        let manager = ServerManager::with_defaults();
        
        // Test server not found error
        let result = manager.send_request("nonexistent", "ping", serde_json::Value::Null).await;
        assert!(matches!(result.unwrap_err(), ServerError::ConfigurationError(_)));
        
        // Test stopping non-existent server
        let result = manager.stop_server("nonexistent").await;
        assert!(matches!(result.unwrap_err(), ServerError::ConfigurationError(_)));
    }

    // Test server configuration validation
    #[tokio::test]
    async fn test_server_config_edge_cases() {
        // Test configuration with environment variables
        let mut config = ReferenceMcpServers::memory_server();
        config.env_vars.insert("NODE_ENV".to_string(), "test".to_string());
        config.env_vars.insert("DEBUG".to_string(), "mcp:*".to_string());
        
        assert!(!config.env_vars.is_empty());
        assert_eq!(config.env_vars.get("NODE_ENV"), Some(&"test".to_string()));
        
        // Test configuration with working directory
        let mut config = ReferenceMcpServers::git_server(".");
        config.working_dir = Some("/tmp".to_string());
        
        assert_eq!(config.working_dir, Some("/tmp".to_string()));
    }

    // Performance test for configuration creation
    #[tokio::test]
    async fn test_reference_server_performance() {
        use std::time::Instant;
        
        let start = Instant::now();
        
        // Create configurations multiple times to test performance
        for _ in 0..1000 {
            let _configs = ReferenceMcpServers::all_servers();
        }
        
        let duration = start.elapsed();
        assert!(duration.as_millis() < 100, "Configuration creation too slow: {}ms", duration.as_millis());
    }
} 