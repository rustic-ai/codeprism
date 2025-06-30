//! Universal MCP server management for any MCP implementation
//! 
//! Provides unified interface for managing MCP servers regardless of transport mechanism.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, warn, error};

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
    WebSocket(WebSocketClient),
}

#[derive(Debug)]
pub struct StdioClient {
    process: Option<Child>,
}

#[derive(Debug)]
pub struct HttpClient {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Debug)]
pub struct WebSocketClient {
    // WebSocket implementation would go here
    url: String,
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
        info!("Starting MCP server with transport: {}", self.config.transport);
        
        match self.config.transport.as_str() {
            "stdio" => self.start_stdio_server().await,
            "http" => self.start_http_client().await,
            "websocket" => self.start_websocket_client().await,
            _ => Err(anyhow!("Unsupported transport type: {}", self.config.transport)),
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
                    process.kill().context("Failed to kill server process")?;
                    process.wait().context("Failed to wait for process termination")?;
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
        let client = self.client.as_mut()
            .ok_or_else(|| anyhow!("Server not started"))?;
        
        match client {
            McpClient::Stdio(stdio_client) => {
                stdio_client.send_request(method, params).await
            }
            McpClient::Http(http_client) => {
                http_client.send_request(method, params).await
            }
            McpClient::WebSocket(ws_client) => {
                ws_client.send_request(method, params).await
            }
        }
    }
    
    /// Check if server is healthy
    pub async fn health_check(&mut self) -> Result<bool> {
        debug!("Performing health check");
        
        match self.send_request("ping", None).await {
            Ok(_) => Ok(true),
            Err(_) => {
                // Try initialize as fallback
                match self.send_request("initialize", Some(serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "mcp-test-harness",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }))).await {
                    Ok(_) => Ok(true),
                    Err(e) => {
                        warn!("Health check failed: {}", e);
                        Ok(false)
                    }
                }
            }
        }
    }
    
    async fn start_stdio_server(&mut self) -> Result<()> {
        let command = self.config.command.as_ref()
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
        
        let process = cmd.spawn()
            .with_context(|| format!("Failed to start server command: {}", command))?;
        
        self.process = Some(process);
        self.client = Some(McpClient::Stdio(StdioClient {
            process: None, // Would hold reference to stdin/stdout pipes
        }));
        
        // Wait for server startup
        if let Some(delay) = self.config.startup_delay {
            debug!("Waiting {}s for server startup", delay);
            tokio::time::sleep(Duration::from_secs(delay)).await;
        }
        
        Ok(())
    }
    
    async fn start_http_client(&mut self) -> Result<()> {
        let url = self.config.url.as_ref()
            .ok_or_else(|| anyhow!("URL required for HTTP transport"))?;
        
        debug!("Connecting to HTTP server: {}", url);
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(self.config.connection_timeout.unwrap_or(10)))
            .build()
            .context("Failed to create HTTP client")?;
        
        self.client = Some(McpClient::Http(HttpClient {
            client,
            base_url: url.clone(),
        }));
        
        Ok(())
    }
    
    async fn start_websocket_client(&mut self) -> Result<()> {
        let url = self.config.url.as_ref()
            .ok_or_else(|| anyhow!("URL required for WebSocket transport"))?;
        
        debug!("Connecting to WebSocket server: {}", url);
        
        // WebSocket implementation would go here
        self.client = Some(McpClient::WebSocket(WebSocketClient {
            url: url.clone(),
        }));
        
        Ok(())
    }
}

impl StdioClient {
    async fn send_request(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        // Create JSON-RPC request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": uuid::Uuid::new_v4().to_string(),
            "method": method,
            "params": params.unwrap_or(serde_json::Value::Null)
        });
        
        debug!("Sending request: {}", serde_json::to_string(&request)?);
        
        // For stdio, we would write to stdin and read from stdout
        // This is a simplified version - real implementation would handle the stdio pipes
        
        // Simulate a basic response for now
        Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "id": request["id"],
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "resources": {},
                    "tools": {},
                    "prompts": {}
                },
                "serverInfo": {
                    "name": "test-server",
                    "version": "1.0.0"
                }
            }
        }))
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
        
        debug!("Sending HTTP request to {}: {}", self.base_url, serde_json::to_string(&request)?);
        
        let response = self.client
            .post(&self.base_url)
            .json(&request)
            .send()
            .await
            .context("Failed to send HTTP request")?;
        
        if !response.status().is_success() {
            return Err(anyhow!("HTTP request failed with status: {}", response.status()));
        }
        
        let response_body: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse HTTP response")?;
        
        debug!("Received HTTP response: {}", serde_json::to_string(&response_body)?);
        
        if let Some(error) = response_body.get("error") {
            return Err(anyhow!("MCP error: {}", error));
        }
        
        response_body.get("result")
            .cloned()
            .ok_or_else(|| anyhow!("No result in response"))
    }
}

impl WebSocketClient {
    async fn send_request(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        // WebSocket implementation would go here
        debug!("WebSocket request to {}: {} with params: {:?}", self.url, method, params);
        
        // For now, return a mock response
        Ok(serde_json::json!({
            "status": "ok",
            "method": method,
            "params": params
        }))
    }
}

/// Discover MCP servers on specified port range
pub async fn discover_mcp_servers(port_range: &str, timeout_secs: u64) -> Result<Vec<DiscoveredServer>> {
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
    let probe_result = timeout(
        Duration::from_secs(timeout_secs),
        client.get(&url).send()
    ).await;
    
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
        _ => Err(anyhow!("No MCP server found on {}:{}", host, port))
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
        return Err(anyhow!("Invalid port range format. Use 'start-end' (e.g., '3000-3010')"));
    }
    
    let start: u16 = parts[0].parse()
        .with_context(|| format!("Invalid start port: {}", parts[0]))?;
    let end: u16 = parts[1].parse()
        .with_context(|| format!("Invalid end port: {}", parts[1]))?;
    
    if start > end {
        return Err(anyhow!("Start port ({}) must be less than end port ({})", start, end));
    }
    
    Ok((start, end))
}

impl Drop for McpServer {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            if let Err(e) = process.kill() {
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
}
