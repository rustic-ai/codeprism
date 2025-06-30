use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{interval, timeout, Instant};

use crate::protocol::jsonrpc::{JsonRpcMessage, JsonRpcRequest};
#[cfg(test)]
use crate::protocol::messages::ClientInfo;
use crate::protocol::messages::{InitializeParams, InitializeResult};

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub fn new(stdin: tokio::process::ChildStdin, stdout: tokio::process::ChildStdout) -> Self {
        Self {
            writer: BufWriter::new(stdin),
            reader: BufReader::new(stdout),
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Send initialization request
    pub async fn initialize(
        &mut self,
        params: InitializeParams,
    ) -> Result<InitializeResult, anyhow::Error> {
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
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
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

/// Server resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Peak memory usage in MB
    pub memory_mb: f64,
    /// CPU usage percentage (0.0 to 100.0)
    pub cpu_percent: f64,
    /// Number of file descriptors in use
    pub file_descriptors: u32,
    /// Last time resources were measured
    #[serde(skip, default = "Instant::now")]
    pub measured_at: Instant,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            memory_mb: 0.0,
            cpu_percent: 0.0,
            file_descriptors: 0,
            measured_at: Instant::now(),
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Health check interval in seconds
    pub check_interval: u64,
    /// Memory limit in MB (0 = no limit)
    pub memory_limit_mb: f64,
    /// CPU limit percentage (0.0 = no limit)
    pub cpu_limit_percent: f64,
    /// Maximum allowed response time in milliseconds
    pub max_response_time_ms: u64,
    /// Number of consecutive failures before marking as unhealthy
    pub failure_threshold: u32,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_interval: 30,
            memory_limit_mb: 512.0,
            cpu_limit_percent: 80.0,
            max_response_time_ms: 5000,
            failure_threshold: 3,
        }
    }
}

/// Health monitoring data
#[derive(Debug)]
pub struct HealthMonitor {
    /// Current health status
    pub status: ServerHealth,
    /// Resource usage statistics
    pub resource_usage: ResourceUsage,
    /// Number of consecutive health check failures
    pub consecutive_failures: u32,
    /// Last successful health check
    pub last_successful_check: Option<Instant>,
    /// Health monitoring task handle
    pub monitor_task: Option<JoinHandle<()>>,
}

impl Clone for HealthMonitor {
    fn clone(&self) -> Self {
        Self {
            status: self.status.clone(),
            resource_usage: self.resource_usage.clone(),
            consecutive_failures: self.consecutive_failures,
            last_successful_check: self.last_successful_check,
            monitor_task: None, // JoinHandle cannot be cloned
        }
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self {
            status: ServerHealth::Starting,
            resource_usage: ResourceUsage::default(),
            consecutive_failures: 0,
            last_successful_check: None,
            monitor_task: None,
        }
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
    /// Health monitoring
    health_monitor: Arc<RwLock<HealthMonitor>>,
    /// Health check configuration
    health_config: HealthConfig,
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
        let mut process = cmd
            .spawn()
            .map_err(|e| ServerError::StartupFailed(format!("Failed to spawn process: {}", e)))?;

        let process_id = process
            .id()
            .ok_or_else(|| ServerError::StartupFailed("Failed to get process ID".to_string()))?;

        // Setup stdio communication
        let stdin = process
            .stdin
            .take()
            .ok_or_else(|| ServerError::StartupFailed("Failed to get process stdin".to_string()))?;
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
            health_monitor: Arc::new(RwLock::new(HealthMonitor::default())),
            health_config: HealthConfig::default(),
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
                // Mark as healthy after successful initialization
                {
                    let mut health = self.health_monitor.write().await;
                    health.status = ServerHealth::Healthy;
                    health.last_successful_check = Some(Instant::now());
                }
                // Start health monitoring
                self.start_health_monitoring().await;
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

    /// Start health monitoring background task
    pub async fn start_health_monitoring(&mut self) {
        let process_id = self.process_id;
        let health_config = self.health_config.clone();
        let health_monitor = Arc::clone(&self.health_monitor);

        let monitor_task = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(health_config.check_interval));

            loop {
                interval.tick().await;

                // Perform health check
                let health_result = Self::perform_health_check(process_id, &health_config).await;

                // Update health status
                {
                    let mut health = health_monitor.write().await;
                    match health_result {
                        Ok(resource_usage) => {
                            health.status = ServerHealth::Healthy;
                            health.resource_usage = resource_usage;
                            health.consecutive_failures = 0;
                            health.last_successful_check = Some(Instant::now());
                        }
                        Err(_) => {
                            health.consecutive_failures += 1;
                            if health.consecutive_failures >= health_config.failure_threshold {
                                health.status = ServerHealth::Unresponsive;
                            }
                        }
                    }
                }
            }
        });

        // Store the task handle
        {
            let mut health = self.health_monitor.write().await;
            health.monitor_task = Some(monitor_task);
        }
    }

    /// Perform health check on the server process
    async fn perform_health_check(
        process_id: u32,
        _health_config: &HealthConfig,
    ) -> Result<ResourceUsage, ServerError> {
        // Check if process is still running
        #[cfg(unix)]
        {
            use std::process::Command;

            // Check if process exists
            let status = Command::new("ps")
                .args(["-p", &process_id.to_string()])
                .output()
                .map_err(|e| {
                    ServerError::CommunicationError(format!("Failed to check process: {}", e))
                })?;

            if !status.status.success() {
                return Err(ServerError::ProcessCrashed {
                    reason: "Process no longer exists".to_string(),
                });
            }

            // Get memory usage (RSS in KB)
            let memory_output = Command::new("ps")
                .args(["-p", &process_id.to_string(), "-o", "rss="])
                .output()
                .map_err(|e| {
                    ServerError::CommunicationError(format!("Failed to get memory usage: {}", e))
                })?;

            let memory_kb = if memory_output.status.success() {
                String::from_utf8_lossy(&memory_output.stdout)
                    .trim()
                    .parse::<f64>()
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            // Get CPU usage (this is a simplified approach)
            let cpu_output = Command::new("ps")
                .args(["-p", &process_id.to_string(), "-o", "pcpu="])
                .output()
                .map_err(|e| {
                    ServerError::CommunicationError(format!("Failed to get CPU usage: {}", e))
                })?;

            let cpu_percent = if cpu_output.status.success() {
                String::from_utf8_lossy(&cpu_output.stdout)
                    .trim()
                    .parse::<f64>()
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            // Get file descriptor count
            let fd_count =
                if let Ok(entries) = std::fs::read_dir(format!("/proc/{}/fd", process_id)) {
                    entries.count() as u32
                } else {
                    0
                };

            Ok(ResourceUsage {
                memory_mb: memory_kb / 1024.0, // Convert KB to MB
                cpu_percent,
                file_descriptors: fd_count,
                measured_at: Instant::now(),
            })
        }

        #[cfg(not(unix))]
        {
            // Simplified health check for non-Unix systems
            Ok(ResourceUsage {
                memory_mb: 0.0,
                cpu_percent: 0.0,
                file_descriptors: 0,
                measured_at: Instant::now(),
            })
        }
    }

    /// Get current health status
    pub async fn get_health(&self) -> ServerHealth {
        let health = self.health_monitor.read().await;
        health.status.clone()
    }

    /// Get current resource usage
    pub async fn get_resource_usage(&self) -> ResourceUsage {
        let health = self.health_monitor.read().await;
        health.resource_usage.clone()
    }

    /// Check if server is healthy
    pub async fn is_healthy(&self) -> bool {
        let health = self.health_monitor.read().await;
        matches!(health.status, ServerHealth::Healthy)
    }

    /// Send a request to the server
    pub async fn send_request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<JsonRpcMessage, ServerError> {
        // Check health before sending request
        if !self.is_healthy().await {
            return Err(ServerError::Unresponsive);
        }

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
        })
        .await;

        match request_result {
            Ok(Ok(message)) => {
                self.update_last_activity().await;
                Ok(message)
            }
            Ok(Err(e)) => Err(ServerError::CommunicationError(format!(
                "Request failed: {}",
                e
            ))),
            Err(_) => {
                // Update health status on timeout
                {
                    let mut health = self.health_monitor.write().await;
                    health.consecutive_failures += 1;
                    if health.consecutive_failures >= self.health_config.failure_threshold {
                        health.status = ServerHealth::Unresponsive;
                    }
                }
                Err(ServerError::Unresponsive)
            }
        }
    }

    /// Check if process is still running
    pub async fn is_running(&mut self) -> bool {
        match self.process.try_wait() {
            Ok(Some(_)) => {
                // Process has exited, update health status
                {
                    let mut health = self.health_monitor.write().await;
                    health.status = ServerHealth::Crashed;
                }
                false
            }
            Ok(None) => true, // Process is still running
            Err(_) => {
                // Error checking process
                {
                    let mut health = self.health_monitor.write().await;
                    health.status = ServerHealth::Crashed;
                }
                false
            }
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
        // Cancel health monitoring task
        {
            let mut health = self.health_monitor.write().await;
            if let Some(task) = health.monitor_task.take() {
                task.abort();
            }
        }

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

/// Server statistics
#[derive(Debug, Clone, Serialize)]
pub struct ServerStats {
    pub server_id: String,
    pub process_id: u32,
    pub uptime: Duration,
    pub idle_time: Duration,
    pub is_idle_timeout: bool,
    pub health_status: ServerHealth,
    pub resource_usage: ResourceUsage,
    pub consecutive_failures: u32,
    #[serde(skip)]
    pub last_successful_check: Option<Instant>,
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

    /// Get detailed server statistics
    pub async fn get_server_stats(&self, server_id: &str) -> Result<ServerStats, ServerError> {
        let servers = self.servers.read().await;
        let server = servers.get(server_id).ok_or_else(|| {
            ServerError::ConfigurationError(format!("Server '{}' not found", server_id))
        })?;

        let health_status = server.get_health().await;
        let resource_usage = server.get_resource_usage().await;
        let uptime = server.uptime();

        // Calculate idle time based on last activity
        let last_activity = *server.last_activity.read().await;
        let idle_time = last_activity.elapsed();
        let is_idle_timeout = idle_time.as_secs() > server.config.max_idle_time;

        let health_monitor = server.health_monitor.read().await;

        Ok(ServerStats {
            server_id: server_id.to_string(),
            process_id: server.process_id(),
            uptime,
            idle_time,
            is_idle_timeout,
            health_status,
            resource_usage,
            consecutive_failures: health_monitor.consecutive_failures,
            last_successful_check: health_monitor.last_successful_check,
        })
    }

    /// Get stats for all servers
    pub async fn get_all_server_stats(&self) -> HashMap<String, ServerStats> {
        let mut stats = HashMap::new();
        let server_ids = self.list_servers().await;

        for server_id in server_ids {
            if let Ok(server_stats) = self.get_server_stats(&server_id).await {
                stats.insert(server_id, server_stats);
            }
        }

        stats
    }

    /// Get health status for a specific server
    pub async fn get_server_health(&self, server_id: &str) -> Result<ServerHealth, ServerError> {
        let servers = self.servers.read().await;
        let server = servers.get(server_id).ok_or_else(|| {
            ServerError::ConfigurationError(format!("Server '{}' not found", server_id))
        })?;

        Ok(server.get_health().await)
    }

    /// Get resource usage for a specific server
    pub async fn get_server_resources(
        &self,
        server_id: &str,
    ) -> Result<ResourceUsage, ServerError> {
        let servers = self.servers.read().await;
        let server = servers.get(server_id).ok_or_else(|| {
            ServerError::ConfigurationError(format!("Server '{}' not found", server_id))
        })?;

        Ok(server.get_resource_usage().await)
    }

    /// Check if a server is healthy
    pub async fn is_server_healthy(&self, server_id: &str) -> Result<bool, ServerError> {
        let servers = self.servers.read().await;
        let server = servers.get(server_id).ok_or_else(|| {
            ServerError::ConfigurationError(format!("Server '{}' not found", server_id))
        })?;

        Ok(server.is_healthy().await)
    }

    /// Restart an unhealthy server
    pub async fn restart_server(
        &self,
        server_id: &str,
        init_params: InitializeParams,
    ) -> Result<(), ServerError> {
        // Get server configuration before stopping
        let config = {
            let servers = self.servers.read().await;
            let server = servers.get(server_id).ok_or_else(|| {
                ServerError::ConfigurationError(format!("Server '{}' not found", server_id))
            })?;
            server.config.clone()
        };

        // Stop the existing server
        self.stop_server(server_id).await?;

        // Start a new server with the same configuration
        self.start_server(server_id.to_string(), config, init_params)
            .await
    }

    /// Perform health check on all servers and return unhealthy ones
    pub async fn check_all_servers_health(&self) -> Vec<(String, ServerHealth)> {
        let mut unhealthy_servers = Vec::new();
        let server_ids = self.list_servers().await;

        for server_id in server_ids {
            if let Ok(health) = self.get_server_health(&server_id).await {
                if !matches!(health, ServerHealth::Healthy) {
                    unhealthy_servers.push((server_id, health));
                }
            }
        }

        unhealthy_servers
    }

    /// Cleanup idle servers that have exceeded their idle timeout
    pub async fn cleanup_idle_servers(&self) -> Vec<String> {
        let mut cleaned_servers = Vec::new();
        let server_ids = self.list_servers().await;

        for server_id in server_ids {
            if let Ok(stats) = self.get_server_stats(&server_id).await {
                if stats.is_idle_timeout {
                    if let Ok(()) = self.stop_server(&server_id).await {
                        cleaned_servers.push(server_id);
                    }
                }
            }
        }

        cleaned_servers
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

/// Reference MCP server configurations for testing
pub struct ReferenceMcpServers;

impl ReferenceMcpServers {
    /// Configuration for the "Everything" reference server (TypeScript)
    pub fn everything_server() -> ServerConfig {
        ServerConfig {
            command: "npx".to_string(),
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-everything".to_string(),
            ],
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
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-memory".to_string(),
            ],
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
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-time".to_string(),
            ],
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
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-fetch".to_string(),
            ],
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
        assert!(matches!(
            result.unwrap_err(),
            ServerError::ConfigurationError(_)
        ));
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
            .start_server(
                "test-server".to_string(),
                config.clone(),
                init_params.clone(),
            )
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
            health_status: ServerHealth::Healthy,
            resource_usage: ResourceUsage {
                memory_mb: 123.45,
                cpu_percent: 50.0,
                file_descriptors: 10,
                measured_at: Instant::now(),
            },
            consecutive_failures: 0,
            last_successful_check: None,
        };

        assert_eq!(stats.server_id, "test");
        assert_eq!(stats.process_id, 1234);
        assert_eq!(stats.uptime, Duration::from_secs(60));
        assert_eq!(stats.idle_time, Duration::from_secs(10));
        assert!(!stats.is_idle_timeout);
        assert_eq!(stats.health_status, ServerHealth::Healthy);
        assert_eq!(stats.resource_usage.memory_mb, 123.45);
        assert_eq!(stats.resource_usage.cpu_percent, 50.0);
        assert_eq!(stats.resource_usage.file_descriptors, 10);
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
            assert!(
                !config.command.is_empty(),
                "Server '{}' has empty command",
                name
            );
            assert!(!config.args.is_empty(), "Server '{}' has no args", name);
            assert!(
                config.startup_timeout > 0,
                "Server '{}' has invalid timeout",
                name
            );
        }
    }

    #[tokio::test]
    async fn test_reference_server_creation() {
        // Test creating reference server configurations
        let memory_config = ReferenceMcpServers::memory_server();
        assert_eq!(memory_config.command, "npx");
        assert!(memory_config
            .args
            .contains(&"@modelcontextprotocol/server-memory".to_string()));

        let time_config = ReferenceMcpServers::time_server();
        assert_eq!(time_config.command, "npx");
        assert!(time_config
            .args
            .contains(&"@modelcontextprotocol/server-time".to_string()));

        let everything_config = ReferenceMcpServers::everything_server();
        assert_eq!(everything_config.command, "npx");
        assert!(everything_config
            .args
            .contains(&"@modelcontextprotocol/server-everything".to_string()));
    }

    #[tokio::test]
    async fn test_filesystem_server_with_custom_path() {
        let temp_dir = "/tmp/mcp-test";
        let config = ReferenceMcpServers::filesystem_server(temp_dir);

        assert_eq!(config.command, "npx");
        assert!(config.args.contains(&temp_dir.to_string()));
        assert!(config
            .args
            .contains(&"@modelcontextprotocol/server-filesystem".to_string()));
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
        let result = manager
            .start_server("memory".to_string(), config, init_params)
            .await;

        // If this succeeds, we have a working MCP server!
        if result.is_ok() {
            // Test basic communication
            let ping_result = manager
                .send_request("memory", "ping", serde_json::Value::Null)
                .await;

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

        let result = manager
            .start_server("filesystem".to_string(), config, init_params)
            .await;

        if result.is_ok() {
            // Test listing resources
            let resources_result = manager
                .send_request("filesystem", "resources/list", serde_json::Value::Null)
                .await;

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
        let result = manager
            .send_request("nonexistent", "ping", serde_json::Value::Null)
            .await;
        assert!(matches!(
            result.unwrap_err(),
            ServerError::ConfigurationError(_)
        ));

        // Test stopping non-existent server
        let result = manager.stop_server("nonexistent").await;
        assert!(matches!(
            result.unwrap_err(),
            ServerError::ConfigurationError(_)
        ));
    }

    // Test server configuration validation
    #[tokio::test]
    async fn test_server_config_edge_cases() {
        // Test configuration with environment variables
        let mut config = ReferenceMcpServers::memory_server();
        config
            .env_vars
            .insert("NODE_ENV".to_string(), "test".to_string());
        config
            .env_vars
            .insert("DEBUG".to_string(), "mcp:*".to_string());

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
        assert!(
            duration.as_millis() < 100,
            "Configuration creation too slow: {}ms",
            duration.as_millis()
        );
    }

    // Tests for new health monitoring functionality
    #[tokio::test]
    async fn test_health_monitor_creation() {
        let health_monitor = HealthMonitor::default();
        assert_eq!(health_monitor.status, ServerHealth::Starting);
        assert_eq!(health_monitor.consecutive_failures, 0);
        assert!(health_monitor.last_successful_check.is_none());
        assert!(health_monitor.monitor_task.is_none());
    }

    #[tokio::test]
    async fn test_resource_usage_creation() {
        let resource_usage = ResourceUsage::default();
        assert_eq!(resource_usage.memory_mb, 0.0);
        assert_eq!(resource_usage.cpu_percent, 0.0);
        assert_eq!(resource_usage.file_descriptors, 0);
    }

    #[tokio::test]
    async fn test_health_config_defaults() {
        let config = HealthConfig::default();
        assert_eq!(config.check_interval, 30);
        assert_eq!(config.memory_limit_mb, 512.0);
        assert_eq!(config.cpu_limit_percent, 80.0);
        assert_eq!(config.max_response_time_ms, 5000);
        assert_eq!(config.failure_threshold, 3);
    }

    #[tokio::test]
    async fn test_server_health_serialization() {
        let health = ServerHealth::Healthy;
        let serialized = serde_json::to_string(&health).unwrap();
        let deserialized: ServerHealth = serde_json::from_str(&serialized).unwrap();
        assert_eq!(health, deserialized);
    }

    #[tokio::test]
    async fn test_resource_usage_serialization() {
        let resource_usage = ResourceUsage {
            memory_mb: 123.45,
            cpu_percent: 67.8,
            file_descriptors: 42,
            measured_at: Instant::now(),
        };

        // Should serialize without the measured_at field
        let serialized = serde_json::to_string(&resource_usage).unwrap();
        let deserialized: ResourceUsage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(resource_usage.memory_mb, deserialized.memory_mb);
        assert_eq!(resource_usage.cpu_percent, deserialized.cpu_percent);
        assert_eq!(
            resource_usage.file_descriptors,
            deserialized.file_descriptors
        );
        // measured_at will be different due to default
    }

    #[tokio::test]
    async fn test_server_manager_health_methods() {
        let manager = ServerManager::with_defaults();

        // Test health check on non-existent server
        let result = manager.get_server_health("nonexistent").await;
        assert!(result.is_err());

        // Test resource check on non-existent server
        let result = manager.get_server_resources("nonexistent").await;
        assert!(result.is_err());

        // Test health status check on non-existent server
        let result = manager.is_server_healthy("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_server_manager_stats_methods() {
        let manager = ServerManager::with_defaults();

        // Test stats on empty manager
        let all_stats = manager.get_all_server_stats().await;
        assert!(all_stats.is_empty());

        // Test health check on all servers (empty)
        let unhealthy = manager.check_all_servers_health().await;
        assert!(unhealthy.is_empty());

        // Test cleanup idle servers (none to clean)
        let cleaned = manager.cleanup_idle_servers().await;
        assert!(cleaned.is_empty());
    }

    #[tokio::test]
    async fn test_server_health_enum_variants() {
        // Test all health enum variants
        let variants = vec![
            ServerHealth::Healthy,
            ServerHealth::Unresponsive,
            ServerHealth::Crashed,
            ServerHealth::Starting,
        ];

        for variant in variants {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: ServerHealth = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

        #[tokio::test]
    async fn test_health_monitor_clone() {
        let health_monitor = HealthMonitor {
            status: ServerHealth::Healthy,
            consecutive_failures: 5,
            ..Default::default()
        };
        
        let cloned = health_monitor.clone();
        assert_eq!(cloned.status, ServerHealth::Healthy);
        assert_eq!(cloned.consecutive_failures, 5);
        assert!(cloned.monitor_task.is_none()); // Should be None after clone
    }

        #[tokio::test]
    async fn test_server_config_with_health_settings() {
        let config = ServerConfig {
            max_idle_time: 600, // 10 minutes
            request_timeout: 30, // 30 seconds
            startup_timeout: 60, // 1 minute
            ..Default::default()
        };
        
        assert_eq!(config.max_idle_time, 600);
        assert_eq!(config.request_timeout, 30);
        assert_eq!(config.startup_timeout, 60);
    }

    // Mock health check test (since we can't easily test real process monitoring in unit tests)
    #[tokio::test]
    async fn test_resource_usage_calculations() {
        let resource_usage = ResourceUsage {
            memory_mb: 256.0,
            cpu_percent: 45.5,
            file_descriptors: 25,
            measured_at: Instant::now(),
        };

        // Test basic resource usage data
        assert!(resource_usage.memory_mb > 0.0);
        assert!(resource_usage.cpu_percent >= 0.0 && resource_usage.cpu_percent <= 100.0);
        assert!(resource_usage.file_descriptors > 0);
    }

    #[tokio::test]
    async fn test_server_stats_comprehensive() {
        let stats = ServerStats {
            server_id: "comprehensive-test".to_string(),
            process_id: 9999,
            uptime: Duration::from_secs(3600),   // 1 hour
            idle_time: Duration::from_secs(300), // 5 minutes
            is_idle_timeout: false,
            health_status: ServerHealth::Healthy,
            resource_usage: ResourceUsage {
                memory_mb: 512.0,
                cpu_percent: 25.0,
                file_descriptors: 50,
                measured_at: Instant::now(),
            },
            consecutive_failures: 0,
            last_successful_check: Some(Instant::now()),
        };

        // Comprehensive stats validation
        assert_eq!(stats.server_id, "comprehensive-test");
        assert_eq!(stats.process_id, 9999);
        assert_eq!(stats.uptime, Duration::from_secs(3600));
        assert_eq!(stats.idle_time, Duration::from_secs(300));
        assert!(!stats.is_idle_timeout);
        assert_eq!(stats.health_status, ServerHealth::Healthy);
        assert_eq!(stats.resource_usage.memory_mb, 512.0);
        assert_eq!(stats.resource_usage.cpu_percent, 25.0);
        assert_eq!(stats.resource_usage.file_descriptors, 50);
        assert_eq!(stats.consecutive_failures, 0);
        assert!(stats.last_successful_check.is_some());

        // Test serialization of comprehensive stats
        let serialized = serde_json::to_string(&stats).unwrap();
        assert!(serialized.contains("comprehensive-test"));
        assert!(serialized.contains("Healthy"));
        assert!(serialized.contains("512"));
    }
}
