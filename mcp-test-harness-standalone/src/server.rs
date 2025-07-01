//! Universal MCP server management for any MCP implementation
//!
//! Provides unified interface for managing MCP servers regardless of transport mechanism.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use crate::config::ServerConfig;

/// Universal MCP server manager
#[derive(Debug)]
pub struct McpServer {
    config: ServerConfig,
    process: Option<Child>,
    client: Option<McpClient>,
}

/// MCP client for different transport types
#[derive(Debug)]
pub enum McpClient {
    Stdio(StdioClient),
    Http(HttpClient),
}

/// JSON-RPC request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// MCP initialization parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: serde_json::Value,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// MCP stdio client with real JSON-RPC communication
#[derive(Debug)]
pub struct StdioClient {
    #[allow(dead_code)]
    process: Option<Child>,
    stdin: Option<Arc<Mutex<ChildStdin>>>,
    stdout: Option<Arc<Mutex<BufReader<ChildStdout>>>>,
    request_id_counter: Arc<AtomicU64>,
    pending_requests: Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<JsonRpcResponse>>>>,
}

#[derive(Debug)]
pub struct HttpClient {
    client: reqwest::Client,
    base_url: String,
}

/// Discovered MCP server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredServer {
    pub host: String,
    pub port: u16,
    pub server_type: String,
    pub capabilities: Option<serde_json::Value>,
    pub version: Option<String>,
}

impl McpServer {
    /// Create new MCP server manager
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            process: None,
            client: None,
        }
    }

    /// Start the MCP server
    pub async fn start(&mut self) -> Result<()> {
        info!(
            "Starting MCP server with transport: {}",
            self.config.transport
        );

        match self.config.transport.as_str() {
            "stdio" => self.start_stdio_server().await,
            "http" => self.start_http_client().await,
            _ => Err(anyhow!(
                "Unsupported transport type: {}",
                self.config.transport
            )),
        }
    }

    /// Stop the MCP server
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping MCP server");

        if let Some(mut process) = self.process.take() {
            match process.try_wait() {
                Ok(Some(status)) => {
                    debug!("Process already exited with status: {}", status);
                }
                Ok(None) => {
                    debug!("Terminating running process");
                    process
                        .kill()
                        .await
                        .context("Failed to kill server process")?;
                    process
                        .wait()
                        .await
                        .context("Failed to wait for process termination")?;
                }
                Err(e) => {
                    warn!("Error checking process status: {}", e);
                }
            }
        }

        self.client = None;
        Ok(())
    }

    /// Send request to MCP server
    pub async fn send_request(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| anyhow!("Server not started"))?;

        match client {
            McpClient::Stdio(stdio_client) => stdio_client.send_request(method, params).await,
            McpClient::Http(http_client) => http_client.send_request(method, params).await,
        }
    }

    /// Initialize MCP session
    pub async fn initialize(&mut self) -> Result<serde_json::Value> {
        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: serde_json::json!({}),
            client_info: ClientInfo {
                name: "mcp-test-harness".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        self.send_request("initialize", Some(serde_json::to_value(params)?))
            .await
    }

    /// List available tools from the MCP server
    #[allow(dead_code)]
    pub async fn list_tools(&mut self) -> Result<serde_json::Value> {
        self.send_request("tools/list", None).await
    }

    /// List available prompts from the MCP server
    #[allow(dead_code)]
    pub async fn list_prompts(&mut self) -> Result<serde_json::Value> {
        self.send_request("prompts/list", None).await
    }

    /// List available resources from the MCP server
    #[allow(dead_code)]
    pub async fn list_resources(&mut self) -> Result<serde_json::Value> {
        self.send_request("resources/list", None).await
    }

    /// Call a specific tool with parameters
    pub async fn call_tool(
        &mut self,
        tool_name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let params = serde_json::json!({
            "name": tool_name,
            "arguments": arguments.unwrap_or(serde_json::json!({}))
        });

        self.send_request("tools/call", Some(params)).await
    }

    /// Get a specific prompt with arguments
    #[allow(dead_code)]
    pub async fn get_prompt(
        &mut self,
        prompt_name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let params = serde_json::json!({
            "name": prompt_name,
            "arguments": arguments.unwrap_or(serde_json::json!({}))
        });

        self.send_request("prompts/get", Some(params)).await
    }

    /// Read a specific resource
    #[allow(dead_code)]
    pub async fn read_resource(&mut self, uri: &str) -> Result<serde_json::Value> {
        let params = serde_json::json!({
            "uri": uri
        });

        self.send_request("resources/read", Some(params)).await
    }

    /// Check if server is healthy
    pub async fn health_check(&mut self) -> Result<bool> {
        debug!("Performing health check via initialize");

        match self.initialize().await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Health check failed: {}", e);
                Ok(false)
            }
        }
    }

    async fn start_stdio_server(&mut self) -> Result<()> {
        let command = self
            .config
            .command
            .as_ref()
            .ok_or_else(|| anyhow!("Command required for stdio transport"))?;

        debug!("Starting stdio server: {}", command);

        let mut cmd = Command::new(command);

        if let Some(args) = &self.config.args {
            cmd.args(args);
        }

        if let Some(working_dir) = &self.config.working_dir {
            cmd.current_dir(working_dir);
        }

        if let Some(env_vars) = &self.config.env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut process = cmd
            .spawn()
            .with_context(|| format!("Failed to start server command: {}", command))?;

        let stdin = process
            .stdin
            .take()
            .ok_or_else(|| anyhow!("Failed to get stdin pipe"))?;
        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to get stdout pipe"))?;

        let mut stdio_client = StdioClient::new(stdin, stdout);

        // Start the response handler
        stdio_client.start_response_handler().await?;

        self.process = Some(process);
        self.client = Some(McpClient::Stdio(stdio_client));

        // Wait for server startup
        if let Some(delay) = self.config.startup_delay {
            debug!("Waiting {}s for server startup", delay);
            tokio::time::sleep(Duration::from_secs(delay)).await;
        }

        Ok(())
    }

    async fn start_http_client(&mut self) -> Result<()> {
        let url = self
            .config
            .url
            .as_ref()
            .ok_or_else(|| anyhow!("URL required for HTTP transport"))?;

        debug!("Connecting to HTTP server: {}", url);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(
                self.config.connection_timeout.unwrap_or(10),
            ))
            .build()
            .context("Failed to create HTTP client")?;

        self.client = Some(McpClient::Http(HttpClient {
            client,
            base_url: url.clone(),
        }));

        Ok(())
    }
}

impl StdioClient {
    /// Create new stdio client with pipes
    pub fn new(stdin: ChildStdin, stdout: ChildStdout) -> Self {
        Self {
            process: None,
            stdin: Some(Arc::new(Mutex::new(stdin))),
            stdout: Some(Arc::new(Mutex::new(BufReader::new(stdout)))),
            request_id_counter: Arc::new(AtomicU64::new(1)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start the response handler task
    pub async fn start_response_handler(&mut self) -> Result<()> {
        let stdout = self
            .stdout
            .clone()
            .ok_or_else(|| anyhow!("No stdout available"))?;
        let pending_requests = self.pending_requests.clone();

        tokio::spawn(async move {
            let mut reader = stdout.lock().await;
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        debug!("Stdout closed");
                        break;
                    }
                    Ok(_) => {
                        if let Err(e) = Self::handle_response(&line, &pending_requests).await {
                            error!("Failed to handle response: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to read from stdout: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Handle incoming response from server
    async fn handle_response(
        line: &str,
        pending_requests: &Arc<
            Mutex<HashMap<String, tokio::sync::oneshot::Sender<JsonRpcResponse>>>,
        >,
    ) -> Result<()> {
        let line = line.trim();
        if line.is_empty() {
            return Ok(());
        }

        debug!("Received response: {}", line);

        let response: JsonRpcResponse =
            serde_json::from_str(line).context("Failed to parse JSON-RPC response")?;

        let request_id = response
            .id
            .as_str()
            .or_else(|| response.id.as_u64().map(|_| "numeric_id"))
            .unwrap_or("unknown")
            .to_string();

        let mut pending = pending_requests.lock().await;
        if let Some(sender) = pending.remove(&request_id) {
            if let Err(_) = sender.send(response) {
                warn!("Failed to send response to waiting request");
            }
        } else {
            debug!("Received response for unknown request ID: {}", request_id);
        }

        Ok(())
    }

    /// Send JSON-RPC request and wait for response
    async fn send_request(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let request_id = self.request_id_counter.fetch_add(1, Ordering::SeqCst);
        let request_id_str = request_id.to_string();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::Value::String(request_id_str.clone()),
            method: method.to_string(),
            params,
        };

        let request_json =
            serde_json::to_string(&request).context("Failed to serialize request")?;

        debug!("Sending request: {}", request_json);

        // Create response channel
        let (tx, rx) = tokio::sync::oneshot::channel();
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(request_id_str, tx);
        }

        // Send request
        {
            let stdin = self
                .stdin
                .as_ref()
                .ok_or_else(|| anyhow!("No stdin available"))?;
            let mut stdin_guard = stdin.lock().await;
            stdin_guard
                .write_all(request_json.as_bytes())
                .await
                .context("Failed to write to stdin")?;
            stdin_guard
                .write_all(b"\n")
                .await
                .context("Failed to write newline")?;
            stdin_guard.flush().await.context("Failed to flush stdin")?;
        }

        // Wait for response with timeout
        let response = timeout(Duration::from_secs(30), rx)
            .await
            .context("Request timed out")?
            .context("Response channel closed")?;

        debug!("Received response: {:?}", response);

        if let Some(error) = response.error {
            return Err(anyhow!("JSON-RPC error {}: {}", error.code, error.message));
        }

        response
            .result
            .ok_or_else(|| anyhow!("No result in response"))
    }
}

impl HttpClient {
    async fn send_request(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": uuid::Uuid::new_v4().to_string(),
            "method": method,
            "params": params.unwrap_or(serde_json::Value::Null)
        });

        debug!(
            "Sending HTTP request to {}: {}",
            self.base_url,
            serde_json::to_string(&request)?
        );

        let response = self
            .client
            .post(&self.base_url)
            .json(&request)
            .send()
            .await
            .context("Failed to send HTTP request")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "HTTP request failed with status: {}",
                response.status()
            ));
        }

        let response_body: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse HTTP response")?;

        debug!(
            "Received HTTP response: {}",
            serde_json::to_string(&response_body)?
        );

        if let Some(error) = response_body.get("error") {
            return Err(anyhow!("MCP error: {}", error));
        }

        response_body
            .get("result")
            .cloned()
            .ok_or_else(|| anyhow!("No result in response"))
    }
}

/// Discover MCP servers on specified port range
pub async fn discover_mcp_servers(
    port_range: &str,
    timeout_secs: u64,
) -> Result<Vec<DiscoveredServer>> {
    info!("Discovering MCP servers on port range: {}", port_range);

    let (start_port, end_port) = parse_port_range(port_range)?;
    let mut discovered = Vec::new();

    for port in start_port..=end_port {
        if let Ok(server) = probe_mcp_server("localhost", port, timeout_secs).await {
            discovered.push(server);
        }
    }

    info!("Discovered {} MCP servers", discovered.len());
    Ok(discovered)
}

async fn probe_mcp_server(host: &str, port: u16, timeout_secs: u64) -> Result<DiscoveredServer> {
    let url = format!("http://{}:{}", host, port);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()?;

    // Try to probe the server
    let probe_result = timeout(Duration::from_secs(timeout_secs), client.get(&url).send()).await;

    match probe_result {
        Ok(Ok(response)) if response.status().is_success() => {
            // Try to determine if this is an MCP server
            let server_type = detect_server_type(&response).await;

            Ok(DiscoveredServer {
                host: host.to_string(),
                port,
                server_type,
                capabilities: None,
                version: None,
            })
        }
        _ => Err(anyhow!("No MCP server found on {}:{}", host, port)),
    }
}

async fn detect_server_type(response: &reqwest::Response) -> String {
    // Try to detect server type from headers or response
    if let Some(server_header) = response.headers().get("server") {
        if let Ok(server_str) = server_header.to_str() {
            if server_str.contains("mcp") || server_str.contains("MCP") {
                return "mcp".to_string();
            }
        }
    }

    "unknown".to_string()
}

fn parse_port_range(range: &str) -> Result<(u16, u16)> {
    let parts: Vec<&str> = range.split('-').collect();
    if parts.len() != 2 {
        return Err(anyhow!(
            "Invalid port range format. Use 'start-end' (e.g., '3000-3010')"
        ));
    }

    let start: u16 = parts[0]
        .parse()
        .with_context(|| format!("Invalid start port: {}", parts[0]))?;
    let end: u16 = parts[1]
        .parse()
        .with_context(|| format!("Invalid end port: {}", parts[1]))?;

    if start > end {
        return Err(anyhow!(
            "Start port ({}) must be less than end port ({})",
            start,
            end
        ));
    }

    Ok((start, end))
}

impl Drop for McpServer {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            // For tokio::process::Child, we need to use start_kill() for non-async context
            if let Err(e) = process.start_kill() {
                error!("Failed to kill server process during drop: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_range_parsing() {
        assert_eq!(parse_port_range("3000-3010").unwrap(), (3000, 3010));
        assert_eq!(parse_port_range("8080-8080").unwrap(), (8080, 8080));

        assert!(parse_port_range("invalid").is_err());
        assert!(parse_port_range("3010-3000").is_err());
        assert!(parse_port_range("abc-def").is_err());
    }

    #[tokio::test]
    async fn test_server_creation() {
        let config = ServerConfig {
            transport: "stdio".to_string(),
            command: Some("echo".to_string()),
            args: Some(vec!["hello".to_string()]),
            working_dir: None,
            env: None,
            url: None,
            connection_timeout: Some(5),
            startup_delay: Some(1),
        };

        let server = McpServer::new(config);
        assert!(matches!(server.client, None));
    }

    #[test]
    fn test_discovered_server_serialization() {
        let server = DiscoveredServer {
            host: "localhost".to_string(),
            port: 3000,
            server_type: "mcp".to_string(),
            capabilities: Some(serde_json::json!({"tools": {}})),
            version: Some("1.0.0".to_string()),
        };

        let json = serde_json::to_string(&server).unwrap();
        let _deserialized: DiscoveredServer = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::Value::String("1".to_string()),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({"test": "data"})),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"initialize\""));
    }
}
