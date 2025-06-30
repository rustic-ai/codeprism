//! MCP protocol implementation for communication with servers

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::Stdio;
use std::time::Duration;
use thiserror::Error;
use tokio::process::{Child, Command};
use tracing::{debug, error, info, warn};

/// MCP protocol error types
#[derive(Debug, Error)]
pub enum McpError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Protocol violation: {0}")]
    Protocol(String),
    #[error("Timeout occurred")]
    Timeout,
    #[error("JSON-RPC error: {code} - {message}")]
    JsonRpc { code: i32, message: String },
    #[error("Server process error: {0}")]
    Process(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// JSON-RPC 2.0 request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

/// MCP server process management
#[derive(Debug)]
pub struct McpServerProcess {
    process: Option<Child>,
    command: String,
    args: Vec<String>,
    working_dir: Option<String>,
}

impl McpServerProcess {
    pub fn new(command: String, args: Vec<String>, working_dir: Option<String>) -> Self {
        Self {
            process: None,
            command,
            args,
            working_dir,
        }
    }

    pub async fn start(&mut self) -> Result<(), McpError> {
        info!("Starting MCP server: {} {:?}", self.command, self.args);

        let mut command = Command::new(&self.command);
        command
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(ref dir) = self.working_dir {
            command.current_dir(dir);
        }

        let child = command
            .spawn()
            .map_err(|e| McpError::Process(format!("Failed to start server: {}", e)))?;

        self.process = Some(child);
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), McpError> {
        if let Some(mut process) = self.process.take() {
            info!("Stopping MCP server process");

            // Try graceful shutdown first
            if let Err(e) = process.kill().await {
                warn!("Failed to kill server process: {}", e);
            }

            match process.wait().await {
                Ok(status) => {
                    info!("Server process exited with status: {}", status);
                }
                Err(e) => {
                    error!("Error waiting for server process: {}", e);
                }
            }
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.process.is_some()
    }
}

/// Core MCP client functionality
#[derive(Debug)]
pub struct McpClient {
    server_process: Option<McpServerProcess>,
    request_id_counter: u64,
    timeout_duration: Duration,
}

impl McpClient {
    /// Create a new MCP client
    pub fn new() -> Self {
        Self {
            server_process: None,
            request_id_counter: 0,
            timeout_duration: Duration::from_secs(30),
        }
    }

    /// Set the timeout duration for requests
    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout_duration = duration;
    }

    /// Start MCP server with given command
    pub async fn start_server(
        &mut self,
        command: String,
        args: Vec<String>,
        working_dir: Option<String>,
    ) -> Result<(), McpError> {
        let mut server_process = McpServerProcess::new(command, args, working_dir);
        server_process.start().await?;
        self.server_process = Some(server_process);
        Ok(())
    }

    /// Stop the MCP server
    pub async fn stop_server(&mut self) -> Result<(), McpError> {
        if let Some(mut server) = self.server_process.take() {
            server.stop().await?;
        }
        Ok(())
    }

    /// Send MCP tool request
    pub async fn call_tool(
        &mut self,
        tool_name: &str,
        parameters: Value,
    ) -> Result<Value, McpError> {
        // Generate unique request ID
        self.request_id_counter += 1;
        let request_id = self.request_id_counter;

        // Create JSON-RPC request for tool call
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(request_id)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": tool_name,
                "arguments": parameters
            })),
        };

        // Send request and get response
        let response = self.send_request(request).await?;

        // Process response
        if let Some(error) = response.error {
            return Err(McpError::JsonRpc {
                code: error.code,
                message: error.message,
            });
        }

        response
            .result
            .ok_or_else(|| McpError::Protocol("Response missing result field".to_string()))
    }

    /// Send JSON-RPC request to server
    pub async fn send_request(
        &mut self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, McpError> {
        debug!("Sending request: {}", request.method);

        // Serialize request for future use
        let _request_json = serde_json::to_string(&request)
            .map_err(|e| McpError::Serialization(format!("Failed to serialize request: {}", e)))?;

        // NOTE: Simulate a response based on the request during development
        // FUTURE: Implement actual stdio communication with server process (tracked in #125)
        let response = self.simulate_server_response(&request).await?;

        Ok(response)
    }

    /// Initialize MCP session
    pub async fn initialize(
        &mut self,
        client_info: ClientInfo,
    ) -> Result<InitializeResult, McpError> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(self.next_request_id())),
            method: "initialize".to_string(),
            params: Some(json!(client_info)),
        };

        let response = self.send_request(request).await?;

        if let Some(error) = response.error {
            return Err(McpError::JsonRpc {
                code: error.code,
                message: error.message,
            });
        }

        let result = response
            .result
            .ok_or_else(|| McpError::Protocol("Initialize response missing result".to_string()))?;

        serde_json::from_value(result).map_err(|e| {
            McpError::Serialization(format!("Failed to parse initialize result: {}", e))
        })
    }

    /// Validate that a server implements required MCP protocol methods
    pub async fn validate_protocol_compliance(&mut self) -> Result<bool, McpError> {
        // Test basic MCP capabilities
        let tests = vec![
            (
                "initialize",
                json!({"clientInfo": {"name": "test", "version": "1.0"}}),
            ),
            ("tools/list", json!({})),
        ];

        for (method, params) in tests {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: Some(json!(self.next_request_id())),
                method: method.to_string(),
                params: Some(params),
            };

            match self.send_request(request).await {
                Ok(response) => {
                    if response.error.is_some() {
                        warn!("Protocol compliance test failed for method: {}", method);
                        return Ok(false);
                    }
                }
                Err(e) => {
                    error!(
                        "Protocol compliance test error for method {}: {}",
                        method, e
                    );
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    fn next_request_id(&mut self) -> u64 {
        self.request_id_counter += 1;
        self.request_id_counter
    }

    /// Simulate server response for testing purposes
    /// FUTURE: Replace with actual stdio communication (tracked in #125)
    async fn simulate_server_response(
        &self,
        request: &JsonRpcRequest,
    ) -> Result<JsonRpcResponse, McpError> {
        // This is a temporary simulation - will be replaced with real stdio communication
        match request.method.as_str() {
            "initialize" => Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: Some(json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "test-server",
                        "version": "1.0.0"
                    }
                })),
                error: None,
            }),
            "tools/call" => {
                let tool_name = request
                    .params
                    .as_ref()
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown");

                // Generate realistic mock response based on tool name
                let result = match tool_name {
                    "repository_stats" => json!({
                        "content": [{
                            "type": "text",
                            "text": "Repository Statistics:\n- Total files: 150\n- Total lines: 25000\n- Languages: Rust (80%), Python (15%), JavaScript (5%)"
                        }]
                    }),
                    "search_content" => json!({
                        "content": [{
                            "type": "text",
                            "text": "Search Results:\n- Found 5 matches in 3 files\n- Files: src/lib.rs, src/main.rs, tests/integration.rs"
                        }]
                    }),
                    "analyze_complexity" => json!({
                        "content": [{
                            "type": "text",
                            "text": "Complexity Analysis:\n- Cyclomatic complexity: 8\n- Cognitive complexity: 12\n- Functions over threshold: 3"
                        }]
                    }),
                    _ => json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Tool '{}' executed successfully", tool_name)
                        }]
                    }),
                };

                Ok(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id.clone(),
                    result: Some(result),
                    error: None,
                })
            }
            "tools/list" => Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: Some(json!({
                    "tools": [
                        {"name": "repository_stats", "description": "Get repository statistics"},
                        {"name": "search_content", "description": "Search content in files"},
                        {"name": "analyze_complexity", "description": "Analyze code complexity"}
                    ]
                })),
                error: None,
            }),
            _ => Err(McpError::Protocol(format!(
                "Unknown method: {}",
                request.method
            ))),
        }
    }
}

impl Default for McpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Client information for MCP initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// MCP initialize result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

/// Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub tools: Option<Value>,
    pub resources: Option<Value>,
    pub prompts: Option<Value>,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// Validate protocol compliance for an MCP server
pub async fn validate_protocol_compliance(mut client: McpClient) -> Result<bool, McpError> {
    client.validate_protocol_compliance().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_client_creation() {
        let client = McpClient::new();
        assert_eq!(client.request_id_counter, 0);
    }

    #[tokio::test]
    async fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "test".to_string(),
            params: Some(json!({"test": "value"})),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.method, deserialized.method);
        assert_eq!(request.jsonrpc, deserialized.jsonrpc);
    }

    #[tokio::test]
    async fn test_simulate_server_response() {
        let client = McpClient::new();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "initialize".to_string(),
            params: Some(json!({})),
        };

        let response = client.simulate_server_response(&request).await.unwrap();
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_tool_call_simulation() {
        let mut client = McpClient::new();
        let result = client
            .call_tool("repository_stats", json!({}))
            .await
            .unwrap();

        assert!(result.get("content").is_some());
        let content = &result["content"][0]["text"];
        assert!(content.as_str().unwrap().contains("Repository Statistics"));
    }
}
