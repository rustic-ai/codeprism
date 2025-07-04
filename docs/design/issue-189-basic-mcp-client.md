# Basic MCP Client Implementation Design Document

## Problem Statement

Implement a basic MCP client wrapper using the official rmcp SDK that can connect to MCP servers and perform basic operations. This client will serve as the foundation for the test harness execution engine.

## Proposed Solution

### High-Level Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   McpClient     │───▶│   rmcp::Service │───▶│   MCP Server    │
│  (Test Harness) │    │  (Official SDK) │    │  (Under Test)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
        │                       │                       │
        ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Connection    │    │   Transport     │    │   Process       │
│   Management    │    │ (stdio/HTTP)    │    │  Management     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Core Components

1. **McpClient**: Main wrapper around rmcp SDK
2. **ConnectionManager**: Handles server lifecycle and connections
3. **ProtocolHandler**: Wraps rmcp operations with error handling
4. **ServerProcess**: Manages external server processes

## API Design

### McpClient Interface

```rust
use rmcp::{ServiceExt, model::*};
use tokio::process::Command;
use std::path::PathBuf;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<PathBuf>,
    pub transport: Transport,
    pub startup_timeout: Duration,
    pub shutdown_timeout: Duration,
}

#[derive(Debug, Clone)]
pub enum Transport {
    Stdio,
    Http { url: String },
    Sse { url: String },
}

pub struct McpClient {
    service: Box<dyn rmcp::Service>,
    server_process: Option<ServerProcess>,
    config: ServerConfig,
    connection_state: ConnectionState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

impl McpClient {
    /// Create a new MCP client with server configuration
    pub async fn new(config: ServerConfig) -> Result<Self>;
    
    /// Connect to the MCP server
    pub async fn connect(&mut self) -> Result<()>;
    
    /// Disconnect from the MCP server
    pub async fn disconnect(&mut self) -> Result<()>;
    
    /// Get current connection state
    pub fn connection_state(&self) -> ConnectionState;
    
    /// Get server capabilities and information
    pub fn server_info(&self) -> Option<&ServerInfo>;
    
    /// List all available tools
    pub async fn list_tools(&self) -> Result<Vec<Tool>>;
    
    /// Call a tool with parameters
    pub async fn call_tool(&self, name: &str, arguments: Option<serde_json::Value>) -> Result<CallToolResult>;
    
    /// List all available resources
    pub async fn list_resources(&self) -> Result<Vec<Resource>>;
    
    /// Read a resource by URI
    pub async fn read_resource(&self, uri: &str) -> Result<ReadResourceResult>;
    
    /// List all available prompts
    pub async fn list_prompts(&self) -> Result<Vec<Prompt>>;
    
    /// Get a prompt with arguments
    pub async fn get_prompt(&self, name: &str, arguments: Option<serde_json::Value>) -> Result<GetPromptResult>;
    
    /// Check if server is healthy/responsive
    pub async fn health_check(&self) -> Result<bool>;
}
```

### Error Handling Strategy

```rust
use crate::error::{Error, Result};

impl McpClient {
    /// Call tool with comprehensive error handling and retries
    pub async fn call_tool_safe(&self, name: &str, arguments: Option<serde_json::Value>) -> Result<CallToolResult> {
        if self.connection_state != ConnectionState::Connected {
            return Err(Error::connection("Client not connected to server"));
        }
        
        match self.service.call_tool(CallToolRequestParam {
            name: name.to_string(),
            arguments,
        }).await {
            Ok(result) => {
                if result.is_error.unwrap_or(false) {
                    Err(Error::execution(format!("Tool '{}' returned error", name)))
                } else {
                    Ok(result)
                }
            }
            Err(rmcp::Error::Transport(e)) => {
                Err(Error::connection(format!("Transport error: {}", e)))
            }
            Err(rmcp::Error::Protocol(e)) => {
                Err(Error::validation(format!("Protocol error: {}", e)))
            }
            Err(e) => Err(Error::mcp(e)),
        }
    }
}
```

### Connection Management

```rust
use tokio::process::{Child, Command};
use tokio::time::{timeout, Duration};

pub struct ServerProcess {
    child: Option<Child>,
    config: ServerConfig,
}

impl ServerProcess {
    pub async fn start(config: &ServerConfig) -> Result<Self> {
        let mut cmd = Command::new(&config.command);
        
        for arg in &config.args {
            cmd.arg(arg);
        }
        
        for (key, value) in &config.env {
            cmd.env(key, value);
        }
        
        if let Some(working_dir) = &config.working_dir {
            cmd.current_dir(working_dir);
        }
        
        // Configure for stdio transport
        cmd.stdin(std::process::Stdio::piped())
           .stdout(std::process::Stdio::piped())
           .stderr(std::process::Stdio::piped());
        
        let child = cmd.spawn()?;
        
        Ok(Self {
            child: Some(child),
            config: config.clone(),
        })
    }
    
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            // Try graceful shutdown first
            let _ = child.kill().await;
            
            // Wait for process to exit with timeout
            match timeout(self.config.shutdown_timeout, child.wait()).await {
                Ok(Ok(status)) => {
                    tracing::info!("Server process exited with status: {:?}", status);
                    Ok(())
                }
                Ok(Err(e)) => Err(Error::connection(format!("Error waiting for process: {}", e))),
                Err(_) => {
                    tracing::warn!("Server process did not exit gracefully, forcing termination");
                    let _ = child.kill().await;
                    Err(Error::connection("Server process shutdown timeout"))
                }
            }
        } else {
            Ok(())
        }
    }
}
```

## Implementation Plan

### Step 1: Basic Client Structure
1. Create `McpClient` struct with rmcp SDK integration
2. Implement connection state management
3. Add basic error handling and logging

### Step 2: Server Process Management
1. Create `ServerProcess` for managing external servers
2. Implement startup and shutdown logic with timeouts
3. Add process monitoring and health checks

### Step 3: Protocol Operations
1. Implement tool operations (list, call)
2. Add resource operations (list, read)
3. Implement prompt operations (list, get)

### Step 4: Transport Support
1. Start with stdio transport (most common)
2. Add comprehensive error handling for transport issues
3. Plan for HTTP/SSE transport support in future

### Step 5: Connection Management
1. Implement robust connection lifecycle
2. Add reconnection logic for transient failures
3. Create connection health monitoring

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_client_creation() {
        let config = ServerConfig {
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            env: HashMap::new(),
            working_dir: None,
            transport: Transport::Stdio,
            startup_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(5),
        };
        
        let client = McpClient::new(config).await;
        assert!(client.is_ok());
    }
    
    #[tokio::test]
    async fn test_connection_state_management() {
        let mut client = create_test_client().await;
        
        assert_eq!(client.connection_state(), ConnectionState::Disconnected);
        
        // Test connection (will fail with echo, but state should change)
        let _ = client.connect().await;
        assert_ne!(client.connection_state(), ConnectionState::Disconnected);
    }
    
    #[tokio::test]
    async fn test_server_process_lifecycle() {
        let config = create_test_config();
        let mut process = ServerProcess::start(&config).await.unwrap();
        
        // Process should be running
        assert!(process.child.is_some());
        
        // Should stop gracefully
        let result = process.stop().await;
        assert!(result.is_ok());
        assert!(process.child.is_none());
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_with_mock_mcp_server() {
    // Create a mock MCP server for testing
    let mock_server = create_mock_server().await;
    
    let config = ServerConfig {
        command: mock_server.command(),
        args: mock_server.args(),
        env: HashMap::new(),
        working_dir: None,
        transport: Transport::Stdio,
        startup_timeout: Duration::from_secs(10),
        shutdown_timeout: Duration::from_secs(5),
    };
    
    let mut client = McpClient::new(config).await.unwrap();
    
    // Test full lifecycle
    client.connect().await.unwrap();
    assert_eq!(client.connection_state(), ConnectionState::Connected);
    
    // Test basic operations
    let tools = client.list_tools().await.unwrap();
    assert!(!tools.is_empty());
    
    let result = client.call_tool("echo", Some(json!({"message": "test"}))).await.unwrap();
    assert!(!result.is_error.unwrap_or(true));
    
    client.disconnect().await.unwrap();
    assert_eq!(client.connection_state(), ConnectionState::Disconnected);
}
```

## Success Criteria

1. ✅ Client can successfully connect to MCP servers via stdio transport
2. ✅ All basic MCP operations work (tools, resources, prompts)
3. ✅ Robust error handling for all failure scenarios
4. ✅ Process lifecycle management with proper cleanup
5. ✅ Connection state tracking and health monitoring
6. ✅ Comprehensive test suite with 90%+ coverage
7. ✅ Integration with existing error system
8. ✅ Clear API for test harness consumption

## Performance Requirements

- Connection establishment: <5 seconds for stdio transport
- Tool calls: <1 second response time for simple operations
- Memory usage: <50MB for client overhead
- Process cleanup: Complete within shutdown timeout

## Security Considerations

- Process isolation for server execution
- Input validation for all MCP operations
- Secure handling of server credentials/tokens
- Resource limits for server processes

## Alternative Approaches Considered

1. **Direct JSON-RPC implementation**: Rejected in favor of official SDK
2. **Synchronous client**: Rejected for performance reasons
3. **Single-connection model**: Chosen for simplicity, can extend later

## References

- docs/Building_MCP_Clients_with_Rust_SDK.md
- external_repos/rust-sdk/examples/clients/
- MCP specification: specification/2025-06-18/
- Issue #189: https://github.com/rustic-ai/codeprism/issues/189 