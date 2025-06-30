//! Standard I/O transport for MCP communication

use super::{Transport, TransportError};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

/// Standard I/O transport implementation with full process management
#[derive(Debug)]
pub struct StdioTransport {
    /// The spawned MCP server process
    process: Option<Child>,
    /// Stdin pipe to the server process
    stdin: Option<ChildStdin>,
    /// Buffered reader for stdout from the server process
    stdout_reader: Option<Mutex<Lines<BufReader<ChildStdout>>>>,
    /// Connection state
    connected: bool,
    /// Server command for spawning
    command: Option<String>,
    /// Server arguments
    args: Vec<String>,
    /// Environment variables for the server
    env_vars: HashMap<String, String>,
    /// Working directory for the server process
    working_dir: Option<String>,
    /// Startup timeout in seconds
    startup_timeout: Duration,
    /// Shutdown timeout in seconds
    shutdown_timeout: Duration,
}

impl StdioTransport {
    /// Create a new stdio transport
    pub fn new() -> Self {
        Self {
            process: None,
            stdin: None,
            stdout_reader: None,
            connected: false,
            command: None,
            args: Vec::new(),
            env_vars: HashMap::new(),
            working_dir: None,
            startup_timeout: Duration::from_secs(30),
            shutdown_timeout: Duration::from_secs(10),
        }
    }

    /// Configure the server command and arguments
    pub fn with_server_config(
        mut self,
        command: String,
        args: Vec<String>,
        env_vars: HashMap<String, String>,
        working_dir: Option<String>,
    ) -> Self {
        self.command = Some(command);
        self.args = args;
        self.env_vars = env_vars;
        self.working_dir = working_dir;
        self
    }

    /// Set startup timeout
    pub fn with_startup_timeout(mut self, timeout: Duration) -> Self {
        self.startup_timeout = timeout;
        self
    }

    /// Set shutdown timeout
    pub fn with_shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout = timeout;
        self
    }

    /// Start the MCP server process
    pub async fn start_server(
        &mut self,
        command: &str,
        args: &[String],
    ) -> Result<(), TransportError> {
        if self.process.is_some() {
            return Err(TransportError::ConnectionFailed(
                "Server process already running - disconnect first".to_string(),
            ));
        }

        info!("Starting MCP server: {} {:?}", command, args);

        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        // Set environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        // Set working directory if specified
        if let Some(working_dir) = &self.working_dir {
            cmd.current_dir(working_dir);
        }

        let mut child = cmd.spawn().map_err(|e| {
            error!("Failed to spawn server process: {}", e);
            TransportError::ConnectionFailed(format!("Failed to spawn server process: {}", e))
        })?;

        // Extract stdin and stdout handles
        let stdin = child.stdin.take().ok_or_else(|| {
            error!("Failed to get stdin handle from child process");
            TransportError::ConnectionFailed("Failed to get stdin handle".to_string())
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            error!("Failed to get stdout handle from child process");
            TransportError::ConnectionFailed("Failed to get stdout handle".to_string())
        })?;

        // Create buffered reader for stdout
        let reader = BufReader::new(stdout);
        let lines = reader.lines();

        self.stdin = Some(stdin);
        self.stdout_reader = Some(Mutex::new(lines));
        self.process = Some(child);
        self.connected = true;

        info!("MCP server process started successfully");
        Ok(())
    }

    /// Check if the server process is still running
    async fn is_process_running(&mut self) -> bool {
        if let Some(ref mut process) = self.process {
            match process.try_wait() {
                Ok(Some(status)) => {
                    warn!("Server process exited with status: {:?}", status);
                    false
                }
                Ok(None) => true, // Still running
                Err(e) => {
                    error!("Error checking process status: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }

    /// Shutdown the server process gracefully
    async fn shutdown_process(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            info!("Shutting down MCP server process");

            // Close stdin to signal shutdown
            if let Some(stdin) = self.stdin.take() {
                drop(stdin);
            }

            // Wait for graceful shutdown with timeout
            let shutdown_result = timeout(self.shutdown_timeout, process.wait()).await;

            match shutdown_result {
                Ok(Ok(status)) => {
                    info!("Server process exited gracefully with status: {:?}", status);
                }
                Ok(Err(e)) => {
                    error!("Error waiting for process exit: {}", e);
                }
                Err(_) => {
                    warn!("Server process didn't exit within timeout, killing forcefully");
                    if let Err(e) = process.kill().await {
                        error!("Failed to kill server process: {}", e);
                    }
                }
            }
        }

        self.stdout_reader = None;
        self.connected = false;
        Ok(())
    }

    /// Send a JSON-RPC message to the server
    async fn send_message(&mut self, message: &Value) -> Result<(), TransportError> {
        if !self.connected {
            return Err(TransportError::ConnectionFailed(
                "Not connected".to_string(),
            ));
        }

        let message_str = serde_json::to_string(message).map_err(TransportError::Serialization)?;

        if let Some(ref mut stdin) = self.stdin {
            debug!("Sending message: {}", message_str);

            // Write message followed by newline for framing
            stdin.write_all(message_str.as_bytes()).await.map_err(|e| {
                error!("Failed to write message to server: {}", e);
                TransportError::Io(e)
            })?;

            stdin.write_all(b"\n").await.map_err(|e| {
                error!("Failed to write message terminator: {}", e);
                TransportError::Io(e)
            })?;

            stdin.flush().await.map_err(|e| {
                error!("Failed to flush message to server: {}", e);
                TransportError::Io(e)
            })?;

            debug!("Message sent successfully");
        } else {
            return Err(TransportError::ConnectionFailed(
                "Stdin not available".to_string(),
            ));
        }

        Ok(())
    }

    /// Receive a JSON-RPC message from the server
    async fn receive_message(&mut self) -> Result<Value, TransportError> {
        if !self.connected {
            return Err(TransportError::ConnectionFailed(
                "Not connected".to_string(),
            ));
        }

        if let Some(ref stdout_reader) = self.stdout_reader {
            let mut reader = stdout_reader.lock().await;

            // Read line-by-line for JSON-RPC message framing
            match reader.next_line().await {
                Ok(Some(line)) => {
                    debug!("Received raw line: {}", line);

                    // Parse JSON message
                    serde_json::from_str(&line).map_err(|e| {
                        error!("Failed to parse JSON message: {} - Line: {}", e, line);
                        TransportError::Serialization(e)
                    })
                }
                Ok(None) => {
                    warn!("Server closed stdout connection");
                    Err(TransportError::ConnectionFailed(
                        "Server closed connection".to_string(),
                    ))
                }
                Err(e) => {
                    error!("Failed to read from server stdout: {}", e);
                    Err(TransportError::Io(e))
                }
            }
        } else {
            Err(TransportError::ConnectionFailed(
                "Stdout reader not available".to_string(),
            ))
        }
    }
}

#[async_trait]
impl Transport for StdioTransport {
    /// Connect to the MCP server
    async fn connect(&mut self) -> Result<(), TransportError> {
        if self.connected {
            return Ok(());
        }

        let command = self.command.clone().ok_or_else(|| {
            TransportError::ConnectionFailed("No server command configured".to_string())
        })?;
        let args = self.args.clone();

        self.start_server(&command, &args).await?;

        // Verify the process is still running after startup
        if !self.is_process_running().await {
            return Err(TransportError::ConnectionFailed(
                "Server process exited immediately after startup".to_string(),
            ));
        }

        Ok(())
    }

    /// Send a message to the server
    async fn send(&mut self, message: Value) -> Result<(), TransportError> {
        // Check if process is still running before sending
        if !self.is_process_running().await {
            self.connected = false;
            return Err(TransportError::ConnectionFailed(
                "Server process has died".to_string(),
            ));
        }

        self.send_message(&message).await
    }

    /// Receive a message from the server
    async fn receive(&mut self) -> Result<Value, TransportError> {
        // Check if process is still running before receiving
        if !self.is_process_running().await {
            self.connected = false;
            return Err(TransportError::ConnectionFailed(
                "Server process has died".to_string(),
            ));
        }

        self.receive_message().await
    }

    /// Disconnect from the server
    async fn disconnect(&mut self) -> Result<(), TransportError> {
        self.shutdown_process()
            .await
            .map_err(|e| TransportError::ConnectionFailed(format!("Shutdown failed: {}", e)))?;
        Ok(())
    }

    /// Check if the transport is connected
    fn is_connected(&self) -> bool {
        self.connected
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        if self.connected {
            // Note: We can't call async shutdown in Drop, but the process will be killed
            // due to kill_on_drop(true) in the Command configuration
            warn!("StdioTransport dropped while connected - process will be forcefully terminated");
        }
    }
}
