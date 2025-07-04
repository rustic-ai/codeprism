# Building MCP Clients with the Official Rust SDK

## Overview

This document provides a comprehensive guide for building Model Context Protocol (MCP) clients using the official Rust SDK (`rmcp`). The MCP Rust SDK enables developers to create robust, production-ready clients that can communicate with any MCP-compliant server.

Based on analysis of the official MCP Rust SDK in the `external_repos/rust-sdk` directory and the MCP specifications in `specification/2025-06-18/`, this guide covers architecture, implementation patterns, and best practices.

## Table of Contents

1. [SDK Architecture](#sdk-architecture)
2. [Getting Started](#getting-started)
3. [Client Lifecycle Management](#client-lifecycle-management)
4. [Transport Layer](#transport-layer)
5. [Capability Negotiation](#capability-negotiation)
6. [Core Operations](#core-operations)
7. [Error Handling](#error-handling)
8. [Best Practices](#best-practices)
9. [Advanced Features](#advanced-features)
10. [Real-World Examples](#real-world-examples)

---

## SDK Architecture

### Core Components

The MCP Rust SDK (`rmcp`) is structured with clear separation of concerns:

```
rmcp/
├── service/          # Service layer for client/server operations
├── transport/        # Transport implementations (stdio, HTTP, SSE)
├── model/           # MCP protocol data structures
├── handler/         # Request/response handlers
└── macros/          # Procedural macros for tool definitions
```

### Key Crates

- **`rmcp`**: Main SDK with client and server functionality
- **`rmcp-macros`**: Procedural macros for simplified tool creation

### Feature Flags

```toml
[dependencies]
rmcp = { version = "0.1", features = [
    "client",                           # Enable client functionality
    "transport-child-process",          # stdio transport via child processes
    "transport-sse-client",            # Server-Sent Events client
    "transport-streamable-http-client", # HTTP streaming client
    "auth"                             # OAuth2 authentication support
] }
```

### Core Traits

- **`ServiceExt`**: Extension trait providing client/server creation methods
- **`Transport`**: Abstraction for different communication mechanisms
- **`ClientHandler`**: Interface for handling client-side protocol events

---

## Getting Started

### Project Setup

```toml
# Cargo.toml
[package]
name = "my-mcp-client"
version = "0.1.0"
edition = "2021"

[dependencies]
rmcp = { git = "https://github.com/modelcontextprotocol/rust-sdk", branch = "main", features = ["client", "transport-child-process"] }
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Basic Client Structure

```rust
use rmcp::{ServiceExt, transport::{TokioChildProcess, ConfigureCommandExt}};
use tokio::process::Command;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create and connect to MCP server
    let client = create_client().await?;
    
    // Perform operations
    run_client_operations(&client).await?;
    
    // Graceful shutdown
    client.cancel().await?;
    
    Ok(())
}

async fn create_client() -> Result<impl rmcp::service::Service> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("npx").configure(|cmd| {
            cmd.arg("-y").arg("@modelcontextprotocol/server-everything");
        }))?)
        .await?;
    
    Ok(client)
}
```

---

## Client Lifecycle Management

### MCP Client Lifecycle

The MCP protocol defines a strict lifecycle that all clients must follow:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Initialization │───▶│   Operation     │───▶│    Shutdown     │
│                 │    │                 │    │                 │
│ - Version nego. │    │ - Tool calls    │    │ - Cleanup       │
│ - Capability    │    │ - Resource read │    │ - Close conn.   │
│   negotiation   │    │ - Prompt fetch  │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Implementation Pattern

```rust
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParam, GetPromptRequestParam, ReadResourceRequestParam},
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;
use anyhow::Result;

pub struct McpClient {
    service: Box<dyn rmcp::service::Service>,
}

impl McpClient {
    /// Create and initialize a new MCP client
    pub async fn new(server_command: &str, args: Vec<&str>) -> Result<Self> {
        // 1. Create transport
        let mut cmd = Command::new(server_command);
        for arg in args {
            cmd.arg(arg);
        }
        
        let transport = TokioChildProcess::new(cmd)?;
        
        // 2. Create service and establish connection
        let service = ().serve(transport).await?;
        
        // 3. The SDK automatically handles initialization and capability negotiation
        
        tracing::info!("MCP client connected successfully");
        
        Ok(Self {
            service: Box::new(service),
        })
    }
    
    /// Get server information and capabilities
    pub fn peer_info(&self) -> &rmcp::model::ServerInfo {
        self.service.peer_info()
    }
    
    /// Gracefully shutdown the client
    pub async fn shutdown(self) -> Result<()> {
        self.service.cancel().await?;
        Ok(())
    }
}
```

---

## Transport Layer

### Available Transports

The SDK supports multiple transport mechanisms:

#### 1. Child Process (stdio)

Most common for local MCP servers:

```rust
use rmcp::transport::{TokioChildProcess, ConfigureCommandExt};
use tokio::process::Command;

// Python MCP server
let transport = TokioChildProcess::new(
    Command::new("python").configure(|cmd| {
        cmd.arg("my_mcp_server.py");
    })
)?;

// Node.js MCP server
let transport = TokioChildProcess::new(
    Command::new("npx").configure(|cmd| {
        cmd.arg("-y").arg("@modelcontextprotocol/server-git");
    })
)?;

// Custom binary MCP server
let transport = TokioChildProcess::new(
    Command::new("./my-mcp-server").configure(|cmd| {
        cmd.arg("--config").arg("config.json");
    })
)?;
```

#### 2. Server-Sent Events (SSE)

For HTTP-based MCP servers:

```rust
use rmcp::transport::SseClientTransport;

let transport = SseClientTransport::new("https://my-mcp-server.com/mcp/sse")?;
```

#### 3. Streamable HTTP

For HTTP streaming:

```rust
use rmcp::transport::StreamableHttpClientTransport;

let transport = StreamableHttpClientTransport::new("https://my-mcp-server.com/mcp")?;
```

### Transport Configuration

```rust
use rmcp::transport::{TokioChildProcess, ConfigureCommandExt};
use tokio::process::Command;
use std::collections::HashMap;

async fn create_configured_transport() -> Result<TokioChildProcess> {
    let transport = TokioChildProcess::new(
        Command::new("python").configure(|cmd| {
            cmd.arg("server.py")
               .arg("--port").arg("8080")
               .arg("--log-level").arg("info")
               .env("MCP_SERVER_MODE", "production")
               .current_dir("/path/to/server");
        })
    )?;
    
    Ok(transport)
}
```

---

## Capability Negotiation

### Understanding Capabilities

MCP capabilities determine what features are available during the session:

```rust
use rmcp::model::{ServerInfo, ServerCapabilities};

// After connection, inspect server capabilities
let server_info = client.peer_info();
println!("Connected to: {} v{}", server_info.name, server_info.version);

// Check what the server supports
if server_info.capabilities.tools.is_some() {
    println!("Server supports tools");
}
if server_info.capabilities.resources.is_some() {
    println!("Server supports resources");
}
if server_info.capabilities.prompts.is_some() {
    println!("Server supports prompts");
}
```

### Capability-Aware Client

```rust
pub struct CapabilityAwareClient {
    service: Box<dyn rmcp::service::Service>,
    capabilities: ServerCapabilities,
}

impl CapabilityAwareClient {
    pub async fn new(transport: impl rmcp::transport::Transport + 'static) -> Result<Self> {
        let service = ().serve(transport).await?;
        let capabilities = service.peer_info().capabilities.clone();
        
        Ok(Self {
            service: Box::new(service),
            capabilities,
        })
    }
    
    pub async fn list_tools(&self) -> Result<Option<Vec<rmcp::model::Tool>>> {
        if self.capabilities.tools.is_some() {
            let tools = self.service.list_all_tools().await?;
            Ok(Some(tools))
        } else {
            Ok(None)
        }
    }
    
    pub async fn list_resources(&self) -> Result<Option<Vec<rmcp::model::Resource>>> {
        if self.capabilities.resources.is_some() {
            let resources = self.service.list_all_resources().await?;
            Ok(Some(resources))
        } else {
            Ok(None)
        }
    }
}

---

## Core Operations

### Tool Operations

Tools are executable functions provided by MCP servers:

```rust
use rmcp::model::{CallToolRequestParam, CallToolResult};
use serde_json::json;

impl McpClient {
    /// List all available tools
    pub async fn list_tools(&self) -> Result<Vec<rmcp::model::Tool>> {
        let tools = self.service.list_all_tools().await?;
        Ok(tools)
    }
    
    /// Call a tool with parameters
    pub async fn call_tool(&self, name: &str, arguments: Option<serde_json::Value>) -> Result<CallToolResult> {
        let result = self.service.call_tool(CallToolRequestParam {
            name: name.to_string(),
            arguments,
        }).await?;
        
        Ok(result)
    }
    
    /// Call tool with typed arguments
    pub async fn call_tool_typed<T: serde::Serialize>(&self, name: &str, args: &T) -> Result<CallToolResult> {
        let arguments = Some(serde_json::to_value(args)?);
        self.call_tool(name, arguments).await
    }
}

// Example usage
async fn use_tools(client: &McpClient) -> Result<()> {
    // List available tools
    let tools = client.list_tools().await?;
    for tool in &tools {
        println!("Tool: {} - {}", tool.name, tool.description);
    }
    
    // Call echo tool
    let result = client.call_tool("echo", Some(json!({
        "message": "Hello from MCP client!"
    }))).await?;
    
    println!("Tool result: {:?}", result);
    
    // Call file operation tool
    let result = client.call_tool("read_file", Some(json!({
        "path": "/path/to/file.txt"
    }))).await?;
    
    Ok(())
}
```

### Resource Operations

Resources provide access to data and content:

```rust
use rmcp::model::{ReadResourceRequestParam, ResourceResult};

impl McpClient {
    /// List all available resources
    pub async fn list_resources(&self) -> Result<Vec<rmcp::model::Resource>> {
        let resources = self.service.list_all_resources().await?;
        Ok(resources)
    }
    
    /// Read a specific resource
    pub async fn read_resource(&self, uri: &str) -> Result<ResourceResult> {
        let result = self.service.read_resource(ReadResourceRequestParam {
            uri: uri.to_string(),
        }).await?;
        
        Ok(result)
    }
    
    /// Read resource with URI template
    pub async fn read_resource_template(&self, template: &str, params: &serde_json::Value) -> Result<ResourceResult> {
        // Expand URI template with parameters
        let uri = expand_uri_template(template, params)?;
        self.read_resource(&uri).await
    }
}

// Helper function for URI template expansion
fn expand_uri_template(template: &str, params: &serde_json::Value) -> Result<String> {
    let mut uri = template.to_string();
    
    if let Some(obj) = params.as_object() {
        for (key, value) in obj {
            let placeholder = format!("{{{}}}", key);
            if let Some(str_value) = value.as_str() {
                uri = uri.replace(&placeholder, str_value);
            }
        }
    }
    
    Ok(uri)
}

// Example usage
async fn use_resources(client: &McpClient) -> Result<()> {
    // List available resources
    let resources = client.list_resources().await?;
    for resource in &resources {
        println!("Resource: {} ({})", resource.uri, resource.name);
    }
    
    // Read specific resource
    let content = client.read_resource("file://project/README.md").await?;
    println!("Resource content: {:?}", content);
    
    // Read resource using template
    let content = client.read_resource_template(
        "file://project/{filename}",
        &json!({"filename": "package.json"})
    ).await?;
    
    Ok(())
}
```

### Prompt Operations

Prompts provide templated messages and workflows:

```rust
use rmcp::model::{GetPromptRequestParam, PromptResult};

impl McpClient {
    /// List all available prompts
    pub async fn list_prompts(&self) -> Result<Vec<rmcp::model::Prompt>> {
        let prompts = self.service.list_all_prompts().await?;
        Ok(prompts)
    }
    
    /// Get a prompt with arguments
    pub async fn get_prompt(&self, name: &str, arguments: Option<serde_json::Value>) -> Result<PromptResult> {
        let result = self.service.get_prompt(GetPromptRequestParam {
            name: name.to_string(),
            arguments,
        }).await?;
        
        Ok(result)
    }
}

// Example usage
async fn use_prompts(client: &McpClient) -> Result<()> {
    // List available prompts
    let prompts = client.list_prompts().await?;
    for prompt in &prompts {
        println!("Prompt: {} - {}", prompt.name, prompt.description);
    }
    
    // Get prompt without arguments
    let prompt = client.get_prompt("simple_prompt", None).await?;
    println!("Prompt content: {:?}", prompt);
    
    // Get prompt with arguments
    let prompt = client.get_prompt("code_review_prompt", Some(json!({
        "language": "rust",
        "style": "detailed"
    }))).await?;
    
    Ok(())
}
```

---

## Error Handling

### MCP Error Types

The SDK provides comprehensive error handling:

```rust
use rmcp::{Error as McpError, service::ServiceError};
use anyhow::{Result, Context};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("MCP protocol error: {0}")]
    Protocol(#[from] McpError),
    
    #[error("Service error: {0}")]
    Service(#[from] ServiceError),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Timeout error: operation took too long")]
    Timeout,
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

impl McpClient {
    /// Call tool with comprehensive error handling
    pub async fn call_tool_safe(&self, name: &str, arguments: Option<serde_json::Value>) -> Result<CallToolResult, ClientError> {
        match self.service.call_tool(CallToolRequestParam {
            name: name.to_string(),
            arguments,
        }).await {
            Ok(result) => {
                // Check if the tool call resulted in an error
                if result.is_error.unwrap_or(false) {
                    return Err(ClientError::InvalidResponse(
                        format!("Tool '{}' returned an error", name)
                    ));
                }
                Ok(result)
            }
            Err(McpError::Transport(e)) => Err(ClientError::Connection(e.to_string())),
            Err(McpError::Protocol(e)) => Err(ClientError::Protocol(McpError::Protocol(e))),
            Err(e) => Err(ClientError::Protocol(e)),
        }
    }
    
    /// Retry mechanism for operations
    pub async fn call_tool_with_retry(&self, name: &str, arguments: Option<serde_json::Value>, max_retries: u32) -> Result<CallToolResult, ClientError> {
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            match self.call_tool_safe(name, arguments.clone()).await {
                Ok(result) => return Ok(result),
                Err(ClientError::Connection(_)) | Err(ClientError::Timeout) if attempt < max_retries => {
                    tracing::warn!("Attempt {} failed, retrying...", attempt + 1);
                    tokio::time::sleep(std::time::Duration::from_millis(1000 * (attempt + 1) as u64)).await;
                    last_error = Some(ClientError::Connection("Retry failed".to_string()));
                }
                Err(e) => return Err(e),
            }
        }
        
        Err(last_error.unwrap_or(ClientError::Connection("All retries failed".to_string())))
    }
}
```

### Timeout Handling

```rust
use tokio::time::{timeout, Duration};

impl McpClient {
    /// Call tool with timeout
    pub async fn call_tool_with_timeout(&self, name: &str, arguments: Option<serde_json::Value>, timeout_duration: Duration) -> Result<CallToolResult, ClientError> {
        match timeout(timeout_duration, self.call_tool_safe(name, arguments)).await {
            Ok(result) => result,
            Err(_) => Err(ClientError::Timeout),
        }
    }
}
```

---

## Best Practices

### 1. Resource Management

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ManagedMcpClient {
    inner: Arc<RwLock<McpClient>>,
    health_check_interval: Duration,
}

impl ManagedMcpClient {
    pub async fn new(transport: impl rmcp::transport::Transport + 'static) -> Result<Self> {
        let client = McpClient::new_with_transport(transport).await?;
        
        Ok(Self {
            inner: Arc::new(RwLock::new(client)),
            health_check_interval: Duration::from_secs(30),
        })
    }
    
    /// Start health monitoring
    pub async fn start_health_monitoring(&self) {
        let client = self.inner.clone();
        let interval = self.health_check_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            
            loop {
                interval.tick().await;
                
                let client = client.read().await;
                if let Err(e) = client.ping().await {
                    tracing::error!("Health check failed: {}", e);
                    // Implement reconnection logic here
                }
            }
        });
    }
}
```

### 2. Concurrent Operations

```rust
use futures::future::join_all;

impl McpClient {
    /// Execute multiple tool calls concurrently
    pub async fn call_tools_concurrent(&self, requests: Vec<(&str, Option<serde_json::Value>)>) -> Result<Vec<Result<CallToolResult, ClientError>>> {
        let futures = requests.into_iter().map(|(name, args)| {
            self.call_tool_safe(name, args)
        });
        
        let results = join_all(futures).await;
        Ok(results)
    }
    
    /// Batch resource reads
    pub async fn read_resources_batch(&self, uris: Vec<&str>) -> Result<Vec<Result<ResourceResult, ClientError>>> {
        let futures = uris.into_iter().map(|uri| {
            async move {
                self.read_resource(uri).await.map_err(|e| ClientError::Protocol(e))
            }
        });
        
        let results = join_all(futures).await;
        Ok(results)
    }
}
```

### 3. Configuration Management

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ClientConfig {
    pub server_command: String,
    pub server_args: Vec<String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub health_check_interval_seconds: u64,
    pub environment: std::collections::HashMap<String, String>,
}

impl ClientConfig {
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub async fn create_client(&self) -> Result<McpClient> {
        let mut cmd = tokio::process::Command::new(&self.server_command);
        
        for arg in &self.server_args {
            cmd.arg(arg);
        }
        
        for (key, value) in &self.environment {
            cmd.env(key, value);
        }
        
        let transport = rmcp::transport::TokioChildProcess::new(cmd)?;
        let client = McpClient::new_with_transport(transport).await?;
        
        Ok(client)
    }
}
```

---

## Advanced Features

### 1. Authentication Support

For HTTP-based transports, the SDK supports OAuth2 authentication:

```rust
use rmcp::auth::{OAuth2Config, TokenProvider};

pub struct AuthenticatedClient {
    client: McpClient,
    token_provider: Box<dyn TokenProvider>,
}

impl AuthenticatedClient {
    pub async fn new(server_url: &str, oauth_config: OAuth2Config) -> Result<Self> {
        let token_provider = oauth_config.create_token_provider().await?;
        
        let transport = rmcp::transport::SseClientTransport::new_with_auth(
            server_url,
            token_provider.clone()
        )?;
        
        let client = McpClient::new_with_transport(transport).await?;
        
        Ok(Self {
            client,
            token_provider,
        })
    }
    
    /// Refresh authentication token
    pub async fn refresh_token(&mut self) -> Result<()> {
        self.token_provider.refresh().await?;
        Ok(())
    }
}
```

### 2. Custom Transport Implementation

You can implement custom transports for specific communication needs:

```rust
use rmcp::transport::Transport;
use async_trait::async_trait;

pub struct CustomTransport {
    // Custom transport implementation
}

#[async_trait]
impl Transport for CustomTransport {
    async fn send(&mut self, message: rmcp::model::Message) -> Result<(), rmcp::Error> {
        // Custom send implementation
        todo!()
    }
    
    async fn receive(&mut self) -> Result<Option<rmcp::model::Message>, rmcp::Error> {
        // Custom receive implementation
        todo!()
    }
    
    async fn close(&mut self) -> Result<(), rmcp::Error> {
        // Custom cleanup implementation
        Ok(())
    }
}
```

### 3. Middleware and Interceptors

Add middleware for logging, metrics, or request modification:

```rust
pub struct LoggingClient {
    inner: McpClient,
    logger: tracing::Span,
}

impl LoggingClient {
    pub fn new(client: McpClient) -> Self {
        Self {
            inner: client,
            logger: tracing::info_span!("mcp_client"),
        }
    }
    
    pub async fn call_tool(&self, name: &str, arguments: Option<serde_json::Value>) -> Result<CallToolResult, ClientError> {
        let _guard = self.logger.enter();
        
        tracing::info!("Calling tool: {}", name);
        let start = std::time::Instant::now();
        
        let result = self.inner.call_tool_safe(name, arguments).await;
        
        let duration = start.elapsed();
        match &result {
            Ok(_) => tracing::info!("Tool call succeeded in {:?}", duration),
            Err(e) => tracing::error!("Tool call failed in {:?}: {}", duration, e),
        }
        
        result
    }
}
```

---

## Real-World Examples

### Example 1: File System Client

A client for interacting with file system MCP servers:

```rust
use rmcp::{ServiceExt, transport::TokioChildProcess};
use serde_json::json;
use anyhow::Result;

pub struct FileSystemClient {
    client: McpClient,
}

impl FileSystemClient {
    pub async fn new(server_path: &str, root_directory: &str) -> Result<Self> {
        let transport = TokioChildProcess::new(
            tokio::process::Command::new(server_path)
                .arg("--root")
                .arg(root_directory)
        )?;
        
        let client = McpClient::new_with_transport(transport).await?;
        
        Ok(Self { client })
    }
    
    pub async fn read_file(&self, path: &str) -> Result<String> {
        let result = self.client.call_tool("read_file", Some(json!({
            "path": path
        }))).await?;
        
        // Extract file content from result
        if let Some(content) = result.content.first() {
            if let rmcp::model::Content::Text { text } = content {
                return Ok(text.clone());
            }
        }
        
        Err(anyhow::anyhow!("Failed to read file content"))
    }
    
    pub async fn write_file(&self, path: &str, content: &str) -> Result<()> {
        let _result = self.client.call_tool("write_file", Some(json!({
            "path": path,
            "content": content
        }))).await?;
        
        Ok(())
    }
    
    pub async fn list_directory(&self, path: &str) -> Result<Vec<String>> {
        let result = self.client.call_tool("list_directory", Some(json!({
            "path": path
        }))).await?;
        
        // Parse directory listing from result
        // Implementation depends on server response format
        todo!("Parse directory listing")
    }
}

// Usage example
async fn file_operations_example() -> Result<()> {
    let fs_client = FileSystemClient::new("mcp-file-server", "/allowed/path").await?;
    
    // Read a file
    let content = fs_client.read_file("example.txt").await?;
    println!("File content: {}", content);
    
    // Write a file
    fs_client.write_file("output.txt", "Hello from MCP client!").await?;
    
    // List directory
    let files = fs_client.list_directory(".").await?;
    for file in files {
        println!("File: {}", file);
    }
    
    Ok(())
}
```

### Example 2: Git Repository Client

A client for interacting with Git repository MCP servers:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub modified_files: Vec<String>,
    pub staged_files: Vec<String>,
    pub untracked_files: Vec<String>,
}

pub struct GitClient {
    client: McpClient,
}

impl GitClient {
    pub async fn new(repo_path: &str) -> Result<Self> {
        let transport = TokioChildProcess::new(
            tokio::process::Command::new("uvx")
                .arg("mcp-server-git")
                .arg("--repo")
                .arg(repo_path)
        )?;
        
        let client = McpClient::new_with_transport(transport).await?;
        
        Ok(Self { client })
    }
    
    pub async fn get_status(&self) -> Result<GitStatus> {
        let result = self.client.call_tool("git_status", Some(json!({
            "repo_path": "."
        }))).await?;
        
        // Parse git status from result
        if let Some(content) = result.content.first() {
            if let rmcp::model::Content::Text { text } = content {
                let status: GitStatus = serde_json::from_str(text)?;
                return Ok(status);
            }
        }
        
        Err(anyhow::anyhow!("Failed to parse git status"))
    }
    
    pub async fn commit(&self, message: &str) -> Result<String> {
        let result = self.client.call_tool("git_commit", Some(json!({
            "message": message
        }))).await?;
        
        // Extract commit hash from result
        if let Some(content) = result.content.first() {
            if let rmcp::model::Content::Text { text } = content {
                return Ok(text.clone());
            }
        }
        
        Err(anyhow::anyhow!("Failed to get commit hash"))
    }
    
    pub async fn get_diff(&self, file_path: Option<&str>) -> Result<String> {
        let mut params = json!({});
        if let Some(path) = file_path {
            params["file_path"] = json!(path);
        }
        
        let result = self.client.call_tool("git_diff", Some(params)).await?;
        
        if let Some(content) = result.content.first() {
            if let rmcp::model::Content::Text { text } = content {
                return Ok(text.clone());
            }
        }
        
        Err(anyhow::anyhow!("Failed to get diff"))
    }
}
```

### Example 3: Multi-Server Client Manager

A manager for coordinating multiple MCP servers:

```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct MultiServerManager {
    clients: RwLock<HashMap<String, McpClient>>,
}

impl MultiServerManager {
    pub fn new() -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
        }
    }
    
    pub async fn add_server(&self, name: String, client: McpClient) {
        let mut clients = self.clients.write().await;
        clients.insert(name, client);
    }
    
    pub async fn call_tool_on_server(&self, server_name: &str, tool_name: &str, arguments: Option<serde_json::Value>) -> Result<CallToolResult, ClientError> {
        let clients = self.clients.read().await;
        
        if let Some(client) = clients.get(server_name) {
            client.call_tool_safe(tool_name, arguments).await
        } else {
            Err(ClientError::Connection(format!("Server '{}' not found", server_name)))
        }
    }
    
    pub async fn broadcast_tool_call(&self, tool_name: &str, arguments: Option<serde_json::Value>) -> Vec<(String, Result<CallToolResult, ClientError>)> {
        let clients = self.clients.read().await;
        let mut results = Vec::new();
        
        for (server_name, client) in clients.iter() {
            let result = client.call_tool_safe(tool_name, arguments.clone()).await;
            results.push((server_name.clone(), result));
        }
        
        results
    }
    
    pub async fn list_all_tools(&self) -> Result<HashMap<String, Vec<rmcp::model::Tool>>> {
        let clients = self.clients.read().await;
        let mut all_tools = HashMap::new();
        
        for (server_name, client) in clients.iter() {
            if let Ok(tools) = client.list_tools().await {
                all_tools.insert(server_name.clone(), tools);
            }
        }
        
        Ok(all_tools)
    }
}

// Usage example
async fn multi_server_example() -> Result<()> {
    let manager = MultiServerManager::new();
    
    // Add file system server
    let fs_client = FileSystemClient::new("mcp-file-server", "/project").await?.client;
    manager.add_server("filesystem".to_string(), fs_client).await;
    
    // Add git server
    let git_client = GitClient::new("/project").await?.client;
    manager.add_server("git".to_string(), git_client).await;
    
    // Use tools from different servers
    let file_content = manager.call_tool_on_server("filesystem", "read_file", Some(json!({
        "path": "README.md"
    }))).await?;
    
    let git_status = manager.call_tool_on_server("git", "git_status", Some(json!({
        "repo_path": "."
    }))).await?;
    
    // List all available tools
    let all_tools = manager.list_all_tools().await?;
    for (server, tools) in all_tools {
        println!("Server {}: {} tools", server, tools.len());
    }
    
    Ok(())
}
```

---

## Conclusion

The MCP Rust SDK provides a robust foundation for building MCP clients with excellent performance, type safety, and async support. Key takeaways:

### Key Benefits

1. **Type Safety**: Full Rust type system support with compile-time guarantees
2. **Async/Await**: Built on Tokio for high-performance async operations
3. **Transport Flexibility**: Multiple transport options for different deployment scenarios
4. **Error Handling**: Comprehensive error types and handling patterns
5. **Extensibility**: Plugin architecture for custom transports and middleware

### Best Practices Summary

1. **Always handle errors gracefully** with proper error types and retry logic
2. **Use capability negotiation** to build adaptive clients
3. **Implement health monitoring** for production deployments
4. **Structure code modularly** with clear separation of concerns
5. **Add comprehensive logging** for debugging and monitoring
6. **Handle timeouts appropriately** for network operations
7. **Use concurrent operations** when possible for better performance

### Next Steps

1. Explore the official SDK examples in `external_repos/rust-sdk/examples/`
2. Read the MCP specification in `specification/2025-06-18/`
3. Build prototypes with different transport mechanisms
4. Implement comprehensive error handling and monitoring
5. Consider contributing to the open-source MCP ecosystem

The MCP Rust SDK enables building sophisticated clients that can integrate seamlessly with the growing ecosystem of MCP servers, providing AI applications with rich contextual capabilities.
```
