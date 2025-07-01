//! Test MCP Server Implementations
//!
//! Simple MCP servers designed specifically for testing the MCP Test Harness.
//! These servers implement various MCP protocol features and edge cases.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Test server implementations for validating the MCP Test Harness
#[derive(Debug, Clone)]
pub enum TestServerType {
    Echo,
    Calculator,
    Filesystem,
    Error,
}

/// A test MCP server for integration testing
#[derive(Debug)]
pub struct TestMcpServer {
    server_type: TestServerType,
    process: Option<Child>,
    temp_dir: Option<TempDir>,
}

/// Echo server - reflects back requests for basic protocol testing
pub struct EchoServer;

/// Calculator server - implements basic math operations
pub struct CalculatorServer {
    memory: f64,
}

/// Filesystem server - provides file operations for resource testing
pub struct FilesystemServer {
    root_path: PathBuf,
}

/// Error server - deliberately generates errors for negative testing
pub struct ErrorServer {
    error_scenarios: HashMap<String, ErrorScenario>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorScenario {
    pub error_code: i32,
    pub error_message: String,
    pub trigger_method: Option<String>,
    pub trigger_param: Option<String>,
}

impl TestMcpServer {
    /// Create a new test MCP server
    pub fn new(server_type: TestServerType) -> Self {
        Self {
            server_type,
            process: None,
            temp_dir: None,
        }
    }

    /// Start the test server as a subprocess
    pub async fn start(&mut self) -> Result<()> {
        match self.server_type {
            TestServerType::Echo => self.start_echo_server().await,
            TestServerType::Calculator => self.start_calculator_server().await,
            TestServerType::Filesystem => self.start_filesystem_server().await,
            TestServerType::Error => self.start_error_server().await,
        }
    }

    /// Stop the test server
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            process.kill().await?;
            process.wait().await?;
        }
        Ok(())
    }

    /// Get the command and arguments to start this server
    pub fn get_command(&self) -> (String, Vec<String>) {
        match self.server_type {
            TestServerType::Echo => ("mcp-test-echo-server".to_string(), vec![]),
            TestServerType::Calculator => ("mcp-test-calculator-server".to_string(), vec![]),
            TestServerType::Filesystem => {
                let root = self
                    .temp_dir
                    .as_ref()
                    .map(|d| d.path().display().to_string())
                    .unwrap_or_else(|| "/tmp".to_string());
                (
                    "mcp-test-filesystem-server".to_string(),
                    vec!["--root".to_string(), root],
                )
            }
            TestServerType::Error => ("mcp-test-error-server".to_string(), vec![]),
        }
    }

    async fn start_echo_server(&mut self) -> Result<()> {
        info!("Starting Echo test server");

        // For testing, we'll run the echo server in-process
        let (tx, _rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let server = EchoServer;
            server.run_stdio(tx).await
        });

        // Simulate subprocess for compatibility
        let mut cmd = Command::new("sleep");
        cmd.arg("3600"); // Keep alive for testing
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let process = cmd.spawn()?;
        self.process = Some(process);

        Ok(())
    }

    async fn start_calculator_server(&mut self) -> Result<()> {
        info!("Starting Calculator test server");

        let mut cmd = Command::new("sleep");
        cmd.arg("3600"); // Keep alive for testing
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let process = cmd.spawn()?;
        self.process = Some(process);

        Ok(())
    }

    async fn start_filesystem_server(&mut self) -> Result<()> {
        info!("Starting Filesystem test server");

        // Create temp directory for filesystem server
        let temp_dir = TempDir::new()?;

        // Create test files
        std::fs::write(temp_dir.path().join("test.txt"), "Hello, World!")?;
        std::fs::write(temp_dir.path().join("data.json"), r#"{"key": "value"}"#)?;

        let mut cmd = Command::new("sleep");
        cmd.arg("3600"); // Keep alive for testing
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let process = cmd.spawn()?;
        self.process = Some(process);
        self.temp_dir = Some(temp_dir);

        Ok(())
    }

    async fn start_error_server(&mut self) -> Result<()> {
        info!("Starting Error test server");

        let mut cmd = Command::new("sleep");
        cmd.arg("3600"); // Keep alive for testing
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let process = cmd.spawn()?;
        self.process = Some(process);

        Ok(())
    }
}

impl EchoServer {
    /// Run the echo server with stdio transport
    pub async fn run_stdio(&self, _tx: mpsc::Sender<Value>) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if let Err(e) = self.handle_request(&line).await {
                        error!("Error handling request: {}", e);
                    }
                }
                Err(e) => {
                    error!("Error reading from stdin: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_request(&self, request_line: &str) -> Result<()> {
        let request_line = request_line.trim();
        if request_line.is_empty() {
            return Ok(());
        }

        debug!("Echo server received: {}", request_line);

        // Parse JSON-RPC request
        let request: Value = serde_json::from_str(request_line)?;

        let method = request
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown");

        let response = match method {
            "initialize" => {
                json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {}
                        },
                        "serverInfo": {
                            "name": "Echo Test Server",
                            "version": "1.0.0"
                        }
                    }
                })
            }
            "tools/list" => {
                json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "result": {
                        "tools": [
                            {
                                "name": "echo",
                                "description": "Echo back the input",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "message": {
                                            "type": "string"
                                        }
                                    },
                                    "required": ["message"]
                                }
                            }
                        ]
                    }
                })
            }
            "tools/call" => {
                let empty_params = json!({});
                let params = request.get("params").unwrap_or(&empty_params);
                let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let empty_args = json!({});
                let arguments = params.get("arguments").unwrap_or(&empty_args);

                if tool_name == "echo" {
                    let message = arguments
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("No message provided");

                    json!({
                        "jsonrpc": "2.0",
                        "id": request.get("id"),
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": format!("Echo: {}", message)
                                }
                            ]
                        }
                    })
                } else {
                    json!({
                        "jsonrpc": "2.0",
                        "id": request.get("id"),
                        "error": {
                            "code": -32601,
                            "message": format!("Unknown tool: {}", tool_name)
                        }
                    })
                }
            }
            _ => {
                json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: {}", method)
                    }
                })
            }
        };

        // Send response
        let response_str = serde_json::to_string(&response)?;
        println!("{}", response_str);

        Ok(())
    }
}

impl Default for CalculatorServer {
    fn default() -> Self {
        Self::new()
    }
}

impl CalculatorServer {
    pub fn new() -> Self {
        Self { memory: 0.0 }
    }

    pub async fn run_stdio(&mut self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    if let Err(e) = self.handle_request(&line).await {
                        error!("Calculator error: {}", e);
                    }
                }
                Err(e) => {
                    error!("Calculator read error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_request(&mut self, request_line: &str) -> Result<()> {
        let request_line = request_line.trim();
        if request_line.is_empty() {
            return Ok(());
        }

        let request: Value = serde_json::from_str(request_line)?;
        let method = request
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown");

        let response = match method {
            "initialize" => {
                json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {}
                        },
                        "serverInfo": {
                            "name": "Calculator Test Server",
                            "version": "1.0.0"
                        }
                    }
                })
            }
            "tools/list" => {
                json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "result": {
                        "tools": [
                            {
                                "name": "add",
                                "description": "Add two numbers",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "a": { "type": "number" },
                                        "b": { "type": "number" }
                                    },
                                    "required": ["a", "b"]
                                }
                            },
                            {
                                "name": "multiply",
                                "description": "Multiply two numbers",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "a": { "type": "number" },
                                        "b": { "type": "number" }
                                    },
                                    "required": ["a", "b"]
                                }
                            },
                            {
                                "name": "memory_store",
                                "description": "Store a value in memory",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "value": { "type": "number" }
                                    },
                                    "required": ["value"]
                                }
                            },
                            {
                                "name": "memory_recall",
                                "description": "Recall the stored memory value",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {},
                                    "additionalProperties": false
                                }
                            }
                        ]
                    }
                })
            }
            "tools/call" => self.handle_tool_call(&request).await?,
            _ => {
                json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: {}", method)
                    }
                })
            }
        };

        let response_str = serde_json::to_string(&response)?;
        println!("{}", response_str);

        Ok(())
    }

    async fn handle_tool_call(&mut self, request: &Value) -> Result<Value> {
        let empty_params = json!({});
        let params = request.get("params").unwrap_or(&empty_params);
        let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let empty_args = json!({});
        let arguments = params.get("arguments").unwrap_or(&empty_args);

        let result = match tool_name {
            "add" => {
                let a = arguments.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let b = arguments.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let sum = a + b;

                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("{} + {} = {}", a, b, sum)
                        }
                    ]
                })
            }
            "multiply" => {
                let a = arguments.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let b = arguments.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let product = a * b;

                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("{} * {} = {}", a, b, product)
                        }
                    ]
                })
            }
            "memory_store" => {
                let value = arguments
                    .get("value")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                self.memory = value;

                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Stored {} in memory", value)
                        }
                    ]
                })
            }
            "memory_recall" => {
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Memory contains: {}", self.memory)
                        }
                    ]
                })
            }
            _ => {
                return Ok(json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "error": {
                        "code": -32601,
                        "message": format!("Unknown tool: {}", tool_name)
                    }
                }));
            }
        };

        Ok(json!({
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": result
        }))
    }
}

impl FilesystemServer {
    pub fn new(root_path: PathBuf) -> Self {
        Self { root_path }
    }

    pub async fn run_stdio(&self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    if let Err(e) = self.handle_request(&line).await {
                        error!("Filesystem error: {}", e);
                    }
                }
                Err(e) => {
                    error!("Filesystem read error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_request(&self, request_line: &str) -> Result<()> {
        let request_line = request_line.trim();
        if request_line.is_empty() {
            return Ok(());
        }

        let request: Value = serde_json::from_str(request_line)?;
        let method = request
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown");

        let response = match method {
            "initialize" => {
                json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "resources": {}
                        },
                        "serverInfo": {
                            "name": "Filesystem Test Server",
                            "version": "1.0.0"
                        }
                    }
                })
            }
            "resources/list" => {
                let files = std::fs::read_dir(&self.root_path)?
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
                    .map(|entry| {
                        let path = entry.path();
                        let filename = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");

                        json!({
                            "uri": format!("file://{}", path.display()),
                            "name": filename,
                            "mimeType": "text/plain"
                        })
                    })
                    .collect::<Vec<_>>();

                json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "result": {
                        "resources": files
                    }
                })
            }
            "resources/read" => {
                let empty_params = json!({});
                let params = request.get("params").unwrap_or(&empty_params);
                let uri = params.get("uri").and_then(|u| u.as_str()).unwrap_or("");

                if let Some(file_path) = uri.strip_prefix("file://") {
                    let full_path = self.root_path.join(file_path);

                    if full_path.exists() && full_path.starts_with(&self.root_path) {
                        match std::fs::read_to_string(&full_path) {
                            Ok(content) => {
                                json!({
                                    "jsonrpc": "2.0",
                                    "id": request.get("id"),
                                    "result": {
                                        "contents": [
                                            {
                                                "uri": uri,
                                                "mimeType": "text/plain",
                                                "text": content
                                            }
                                        ]
                                    }
                                })
                            }
                            Err(e) => {
                                json!({
                                    "jsonrpc": "2.0",
                                    "id": request.get("id"),
                                    "error": {
                                        "code": -32603,
                                        "message": format!("Failed to read file: {}", e)
                                    }
                                })
                            }
                        }
                    } else {
                        json!({
                            "jsonrpc": "2.0",
                            "id": request.get("id"),
                            "error": {
                                "code": -32602,
                                "message": "File not found or access denied"
                            }
                        })
                    }
                } else {
                    json!({
                        "jsonrpc": "2.0",
                        "id": request.get("id"),
                        "error": {
                            "code": -32602,
                            "message": "Invalid URI format"
                        }
                    })
                }
            }
            _ => {
                json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: {}", method)
                    }
                })
            }
        };

        let response_str = serde_json::to_string(&response)?;
        println!("{}", response_str);

        Ok(())
    }
}

impl Default for ErrorServer {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorServer {
    pub fn new() -> Self {
        let mut error_scenarios = HashMap::new();

        // Pre-configured error scenarios
        error_scenarios.insert(
            "timeout".to_string(),
            ErrorScenario {
                error_code: -32603,
                error_message: "Request timed out".to_string(),
                trigger_method: Some("tools/call".to_string()),
                trigger_param: Some("timeout_test".to_string()),
            },
        );

        error_scenarios.insert(
            "invalid_params".to_string(),
            ErrorScenario {
                error_code: -32602,
                error_message: "Invalid parameters".to_string(),
                trigger_method: Some("tools/call".to_string()),
                trigger_param: Some("invalid_test".to_string()),
            },
        );

        error_scenarios.insert(
            "server_error".to_string(),
            ErrorScenario {
                error_code: -32603,
                error_message: "Internal server error".to_string(),
                trigger_method: Some("tools/call".to_string()),
                trigger_param: Some("error_test".to_string()),
            },
        );

        Self { error_scenarios }
    }

    pub async fn run_stdio(&self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    if let Err(e) = self.handle_request(&line).await {
                        error!("Error server error: {}", e);
                    }
                }
                Err(e) => {
                    error!("Error server read error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_request(&self, request_line: &str) -> Result<()> {
        let request_line = request_line.trim();
        if request_line.is_empty() {
            return Ok(());
        }

        let request: Value = serde_json::from_str(request_line)?;
        let method = request
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown");

        let response = match method {
            "initialize" => {
                json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {}
                        },
                        "serverInfo": {
                            "name": "Error Test Server",
                            "version": "1.0.0"
                        }
                    }
                })
            }
            "tools/list" => {
                json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "result": {
                        "tools": [
                            {
                                "name": "timeout_test",
                                "description": "Triggers a timeout error",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {}
                                }
                            },
                            {
                                "name": "invalid_test",
                                "description": "Triggers an invalid parameters error",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {}
                                }
                            },
                            {
                                "name": "error_test",
                                "description": "Triggers a server error",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {}
                                }
                            }
                        ]
                    }
                })
            }
            "tools/call" => {
                let empty_params = json!({});
                let params = request.get("params").unwrap_or(&empty_params);
                let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");

                if let Some(scenario) = self
                    .error_scenarios
                    .values()
                    .find(|s| s.trigger_param.as_ref() == Some(&tool_name.to_string()))
                {
                    json!({
                        "jsonrpc": "2.0",
                        "id": request.get("id"),
                        "error": {
                            "code": scenario.error_code,
                            "message": scenario.error_message
                        }
                    })
                } else {
                    json!({
                        "jsonrpc": "2.0",
                        "id": request.get("id"),
                        "error": {
                            "code": -32601,
                            "message": format!("Unknown tool: {}", tool_name)
                        }
                    })
                }
            }
            _ => {
                json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: {}", method)
                    }
                })
            }
        };

        let response_str = serde_json::to_string(&response)?;
        println!("{}", response_str);

        Ok(())
    }
}

impl Drop for TestMcpServer {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            if let Err(e) = process.start_kill() {
                warn!("Failed to kill test server process: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let echo_server = TestMcpServer::new(TestServerType::Echo);
        assert!(matches!(echo_server.server_type, TestServerType::Echo));

        let calc_server = TestMcpServer::new(TestServerType::Calculator);
        assert!(matches!(
            calc_server.server_type,
            TestServerType::Calculator
        ));
    }

    #[test]
    fn test_get_command() {
        let echo_server = TestMcpServer::new(TestServerType::Echo);
        let (cmd, args) = echo_server.get_command();
        assert_eq!(cmd, "mcp-test-echo-server");
        assert!(args.is_empty());

        let calc_server = TestMcpServer::new(TestServerType::Calculator);
        let (cmd, _) = calc_server.get_command();
        assert_eq!(cmd, "mcp-test-calculator-server");
    }

    #[tokio::test]
    async fn test_echo_server_request_handling() {
        let server = EchoServer;

        // Test initialize request
        let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        assert!(server.handle_request(init_request).await.is_ok());

        // Test tools/list request
        let list_request = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
        assert!(server.handle_request(list_request).await.is_ok());
    }

    #[tokio::test]
    async fn test_calculator_operations() {
        let mut calc = CalculatorServer::new();
        assert_eq!(calc.memory, 0.0);

        // Test addition
        let add_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "add",
                "arguments": {
                    "a": 5.0,
                    "b": 3.0
                }
            }
        });

        let result = calc.handle_tool_call(&add_request).await.unwrap();
        assert!(result.get("result").is_some());
    }

    #[test]
    fn test_error_scenarios() {
        let error_server = ErrorServer::new();
        assert!(error_server.error_scenarios.contains_key("timeout"));
        assert!(error_server.error_scenarios.contains_key("invalid_params"));
        assert!(error_server.error_scenarios.contains_key("server_error"));
    }
}
