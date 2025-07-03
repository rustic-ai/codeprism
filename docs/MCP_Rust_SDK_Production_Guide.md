# Production-Ready MCP Server Development with Rust SDK

A comprehensive guide to building modular, scalable Model Context Protocol (MCP) servers using the official Rust SDK (`rmcp`), with emphasis on multiple tool routers and production deployment patterns.

## Table of Contents

1. [Overview](#overview)
2. [Architecture Patterns](#architecture-patterns)
3. [Tool Router System](#tool-router-system)
4. [Multiple Tool Routers](#multiple-tool-routers)
5. [Production Server Structure](#production-server-structure)
6. [Transport Options](#transport-options)
7. [Authentication & Security](#authentication--security)
8. [Error Handling](#error-handling)
9. [Testing Strategies](#testing-strategies)
10. [Performance Optimization](#performance-optimization)
11. [Deployment Considerations](#deployment-considerations)

## Overview

The Rust MCP SDK (`rmcp`) provides a powerful foundation for building MCP servers that expose tools, resources, and prompts to AI assistants. The SDK follows these key principles from the source code analysis:

- **Type Safety**: Leverages Rust's type system for compile-time guarantees
- **Async-First**: Built on `tokio` for high-performance async I/O
- **Modular Design**: Tool routers enable composable server architecture
- **Transport Agnostic**: Supports STDIO, HTTP, SSE, and custom transports

### Core SDK Components

Based on [`external_repos/rust-sdk/crates/rmcp/src/lib.rs`](external_repos/rust-sdk/crates/rmcp/src/lib.rs):

```rust
// Key imports for MCP server development
use rmcp::{
    Error as McpError,
    ServerHandler,
    ServiceExt,
    model::*,
    tool, tool_router, tool_handler,
    handler::server::{
        router::tool::ToolRouter,
        tool::{Parameters, ToolCallContext}
    },
    transport::{stdio, sse_server::SseServer},
    service::{RequestContext, RoleServer},
};
```

## Architecture Patterns

### 1. Single Service Pattern

For simple servers with cohesive functionality, as shown in [`external_repos/rust-sdk/examples/servers/src/common/counter.rs`](external_repos/rust-sdk/examples/servers/src/common/counter.rs):

```rust
#[derive(Clone)]
pub struct Counter {
    counter: Arc<Mutex<i32>>,
    tool_router: ToolRouter<Counter>,
}

#[tool_router]
impl Counter {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Increment the counter by 1")]
    async fn increment(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }
}

#[tool_handler]
impl ServerHandler for Counter {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
            instructions: Some("Counter service with increment/decrement tools".to_string()),
            ..Default::default()
        }
    }
}
```

### 2. Modular Service Pattern

For complex servers with multiple domains, based on [`external_repos/rust-sdk/examples/servers/src/common/generic_service.rs`](external_repos/rust-sdk/examples/servers/src/common/generic_service.rs):

```rust
pub trait DataService: Send + Sync + 'static {
    fn get_data(&self) -> String;
    fn set_data(&mut self, data: String);
}

#[derive(Debug, Clone)]
pub struct GenericService<DS: DataService> {
    data_service: Arc<DS>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl<DS: DataService> GenericService<DS> {
    pub fn new(data_service: DS) -> Self {
        Self {
            data_service: Arc::new(data_service),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "get memory from service")]
    pub async fn get_data(&self) -> String {
        self.data_service.get_data()
    }
}
```

## Tool Router System

### Understanding Tool Routers

From [`external_repos/rust-sdk/crates/rmcp/src/handler/server/router/tool.rs`](external_repos/rust-sdk/crates/rmcp/src/handler/server/router/tool.rs), tool routers manage tool registration and dispatch:

```rust
#[derive(Debug)]
pub struct ToolRouter<S> {
    pub map: std::collections::HashMap<Cow<'static, str>, ToolRoute<S>>,
    pub transparent_when_not_found: bool,
}

impl<S> ToolRouter<S>
where
    S: Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            map: std::collections::HashMap::new(),
            transparent_when_not_found: false,
        }
    }

    pub fn with_route<R, A>(mut self, route: R) -> Self
    where
        R: IntoToolRoute<S, A>,
    {
        self.add_route(route.into_tool_route());
        self
    }

    pub fn merge(&mut self, other: ToolRouter<S>) {
        for item in other.map.into_values() {
            self.add_route(item);
        }
    }
}
```

### Tool Macro Usage

The `#[tool]` macro from [`external_repos/rust-sdk/examples/servers/src/common/calculator.rs`](external_repos/rust-sdk/examples/servers/src/common/calculator.rs) demonstrates parameter handling:

```rust
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SumRequest {
    #[schemars(description = "the left hand side number")]
    pub a: i32,
    #[schemars(description = "the right hand side number")]
    pub b: i32,
}

#[tool_router]
impl Calculator {
    #[tool(description = "Calculate the sum of two numbers")]
    fn sum(&self, Parameters(SumRequest { a, b }): Parameters<SumRequest>) -> String {
        (a + b).to_string()
    }

    #[tool(description = "Calculate the difference of two numbers")]
    fn sub(&self, Parameters(SubRequest { a, b }): Parameters<SubRequest>) -> Json<i32> {
        Json(a - b)
    }
}
```

## Multiple Tool Routers

### Understanding the Macro System

From `external_repos/rust-sdk/crates/rmcp-macros/src/lib.rs`, the `#[tool_router]` macro generates a function that returns a `ToolRouter<Self>` instance. Key insights:

- **Configurable Names**: Use `router = function_name` to specify custom router function names
- **Visibility Control**: Use `vis = pub` to make router functions public for composition
- **Composition Support**: Router functions can be combined using the `+` operator
- **Field Storage Pattern**: Store the combined router in a struct field

#### `vis` Parameter Details

The `vis` parameter accepts any valid Rust visibility modifier and has the data type `Option<syn::Visibility>`:

```rust
// From external_repos/rust-sdk/crates/rmcp-macros/src/tool_router.rs
#[derive(FromMeta)]
#[darling(default)]
pub struct ToolRouterAttribute {
    pub router: Ident,
    pub vis: Option<Visibility>,  // syn::Visibility from syn crate
}
```

**Supported visibility modifiers:**
- `vis = pub` - Public visibility (most common for composition)
- `vis = pub(crate)` - Crate-local visibility
- `vis = pub(super)` - Parent module visibility  
- `vis = pub(self)` - Current module visibility
- `vis = pub(in path::to::module)` - Restricted visibility to specific path
- No `vis` parameter - Private visibility (default)

**Examples of different visibility levels:**

```rust
// Public router function - can be used across crate boundaries
#[tool_router(router = public_auth_router, vis = pub)]
impl AuthService { /* ... */ }

// Crate-local router function - only within same crate
#[tool_router(router = crate_auth_router, vis = pub(crate))]
impl AuthService { /* ... */ }

// Module-local router function - only within parent module
#[tool_router(router = super_auth_router, vis = pub(super))]
impl AuthService { /* ... */ }

// Private router function - only within same impl block/module
#[tool_router(router = private_auth_router)]  // No vis = private
impl AuthService { /* ... */ }
```

For multi-router composition, you typically want `vis = pub` to allow router functions to be called from the main server module.

### Advanced Router Composition Pattern

The macro system enables sophisticated composition patterns for production servers:

```rust
use rmcp::{
    ServerHandler, Error as McpError,
    handler::server::router::tool::ToolRouter,
    model::{ServerInfo, ServerCapabilities, CallToolResult, CallToolRequestParam},
    service::{RequestContext, RoleServer},
    tool, tool_router, tool_handler,
};
use std::sync::Arc;
use tokio::sync::RwLock;

// Individual service modules with named, public router functions
mod auth_service {
    use super::*;
    
    #[derive(Clone)]
    pub struct AuthService {
        jwt_secret: String,
        user_store: Arc<dyn UserStore>,
    }

    // Generate a public router function named `auth_tool_router`
    #[tool_router(router = auth_tool_router, vis = pub)]
    impl AuthService {
        pub async fn new(config: &AuthConfig) -> Result<Self, McpError> {
            Ok(Self {
                jwt_secret: config.jwt_secret.clone(),
                user_store: Arc::new(DatabaseUserStore::new(&config.database_url).await?),
            })
        }

        #[tool(description = "Authenticate user and return JWT token")]
        async fn login(
            &self,
            Parameters(LoginRequest { username, password }): Parameters<LoginRequest>,
        ) -> Result<CallToolResult, McpError> {
            // Implementation...
            Ok(CallToolResult::success(vec![Content::text("Authentication successful")]))
        }

        #[tool(description = "Validate JWT token")]
        async fn validate_token(
            &self,
            Parameters(ValidateTokenRequest { token }): Parameters<ValidateTokenRequest>,
        ) -> Result<CallToolResult, McpError> {
            // Implementation...
            Ok(CallToolResult::success(vec![Content::text("Token valid")]))
        }
    }
}

mod file_service {
    use super::*;
    
    #[derive(Clone)]
    pub struct FileService {
        root_path: String,
        max_file_size: u64,
    }

    // Generate a public router function named `file_tool_router`
    #[tool_router(router = file_tool_router, vis = pub)]
    impl FileService {
        pub async fn new(config: &StorageConfig) -> Result<Self, McpError> {
            Ok(Self {
                root_path: config.root_path.clone(),
                max_file_size: config.max_file_size,
            })
        }

        #[tool(description = "Read file contents")]
        async fn read_file(
            &self,
            Parameters(ReadFileRequest { path }): Parameters<ReadFileRequest>,
        ) -> Result<CallToolResult, McpError> {
            // Implementation...
            Ok(CallToolResult::success(vec![Content::text("File contents")]))
        }

        #[tool(description = "Write file contents")]
        async fn write_file(
            &self,
            Parameters(WriteFileRequest { path, content }): Parameters<WriteFileRequest>,
        ) -> Result<CallToolResult, McpError> {
            // Implementation...
            Ok(CallToolResult::success(vec![Content::text("File written")]))
        }
    }
}

mod database_service {
    use super::*;
    
    #[derive(Clone)]
    pub struct DatabaseService {
        connection_pool: Arc<sqlx::Pool<sqlx::Postgres>>,
    }

    // Generate a public router function named `database_tool_router`
    #[tool_router(router = database_tool_router, vis = pub)]
    impl DatabaseService {
        pub async fn new(config: &DatabaseConfig) -> Result<Self, McpError> {
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(config.max_connections)
                .connect(&config.connection_string)
                .await?;
                
            Ok(Self {
                connection_pool: Arc::new(pool),
            })
        }

        #[tool(description = "Execute SQL query")]
        async fn query(
            &self,
            Parameters(QueryRequest { sql, params }): Parameters<QueryRequest>,
        ) -> Result<CallToolResult, McpError> {
            // Implementation...
            Ok(CallToolResult::success(vec![Content::text("Query results")]))
        }

        #[tool(description = "Get table schema")]
        async fn describe_table(
            &self,
            Parameters(DescribeTableRequest { table_name }): Parameters<DescribeTableRequest>,
        ) -> Result<CallToolResult, McpError> {
            // Implementation...
            Ok(CallToolResult::success(vec![Content::text("Table schema")]))
        }
    }
}

// Main production server that composes multiple tool routers
#[derive(Clone)]
pub struct ProductionMcpServer {
    // Individual services
    auth_service: Arc<auth_service::AuthService>,
    file_service: Arc<file_service::FileService>,
    database_service: Arc<database_service::DatabaseService>,
    
    // Combined tool router - stores the result of router composition
    tool_router: ToolRouter<Self>,
    
    // Shared state
    config: Arc<ServerConfig>,
    metrics: Arc<RwLock<ServerMetrics>>,
}

impl ProductionMcpServer {
    pub async fn new(config: ServerConfig) -> Result<Self, McpError> {
        // Initialize individual services
        let auth_service = Arc::new(auth_service::AuthService::new(&config.auth).await?);
        let file_service = Arc::new(file_service::FileService::new(&config.storage).await?);
        let database_service = Arc::new(database_service::DatabaseService::new(&config.database).await?);
        
        // Compose tool routers using the + operator
        // This leverages the generated public router functions
        let combined_router = auth_service::auth_tool_router() 
            + file_service::file_tool_router() 
            + database_service::database_tool_router();
        
        Ok(Self {
            auth_service,
            file_service,
            database_service,
            tool_router: combined_router,
            config: Arc::new(config),
            metrics: Arc::new(RwLock::new(ServerMetrics::default())),
        })
    }

    fn requires_auth(&self, tool_name: &str) -> bool {
        // Define which tools require authentication
        matches!(tool_name, "query" | "write_file" | "describe_table")
    }
}

// Use the #[tool_handler] macro to generate call_tool and list_tools implementations
#[tool_handler]
impl ServerHandler for ProductionMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "Production MCP server with authentication, file management, \
                 and database operations capabilities.".to_string()
            ),
        }
    }

    // The #[tool_handler] macro generates these methods:
    // - async fn call_tool(&self, request: CallToolRequestParam, context: RequestContext<RoleServer>) -> Result<CallToolResult, McpError>
    // - async fn list_tools(&self, request: Option<PaginatedRequestParam>, context: RequestContext<RoleServer>) -> Result<ListToolsResult, McpError>
    
    // But we can override them to add custom logic like authentication
    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.tool_calls += 1;
            metrics.last_activity = std::time::SystemTime::now();
        }

        // Validate authentication for protected tools
        if self.requires_auth(&request.name) {
            self.auth_service.validate_request(&context).await?;
        }

        // Delegate to the combined router (generated by #[tool_handler])
        let tool_context = ToolCallContext::new(self, request, context);
        self.tool_router.call(tool_context).await
    }
}

### Alternative Pattern: Conditional Router Composition

For more dynamic scenarios, you can compose routers conditionally:

```rust
impl ProductionMcpServer {
    pub async fn new_with_features(config: ServerConfig, features: &[&str]) -> Result<Self, McpError> {
        let mut combined_router = ToolRouter::new();
        
        // Always include auth
        combined_router += auth_service::auth_tool_router();
        
        // Conditionally add other services
        if features.contains(&"files") {
            let file_service = Arc::new(file_service::FileService::new(&config.storage).await?);
            combined_router += file_service::file_tool_router();
        }
        
        if features.contains(&"database") {
            let database_service = Arc::new(database_service::DatabaseService::new(&config.database).await?);
            combined_router += database_service::database_tool_router();
        }
        
        // ... rest of initialization
        Ok(Self {
            tool_router: combined_router,
            // ...
        })
    }
}
```

### Generated Code Understanding

From the macro source, `#[tool_handler]` generates:

```rust
// This is what #[tool_handler] generates behind the scenes
impl ServerHandler for ProductionMcpServer {
    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::Error> {
        let tcc = ToolCallContext::new(self, request, context);
        self.tool_router.call(tcc).await  // Uses the field specified or defaults to `self.tool_router`
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, rmcp::Error> {
        let items = self.tool_router.list_all();
        Ok(ListToolsResult::with_all_items(items))
    }
}
```

### Custom Router Field Access

You can customize which router field the macro uses:

```rust
#[derive(Clone)]
pub struct CustomServer {
    primary_router: ToolRouter<Self>,
    secondary_router: ToolRouter<Self>,
}

// Use custom router expression
#[tool_handler(router = self.primary_router)]
impl ServerHandler for CustomServer {
    // Generated methods will use `self.primary_router` instead of default `self.tool_router`
}
```

### Key Benefits of This Approach

1. **Modularity**: Each service module is self-contained with its own tools
2. **Composition**: Router functions can be combined flexibly using `+` operator
3. **Generated Code**: `#[tool_handler]` automatically handles the boilerplate
4. **Type Safety**: All tool routing is type-safe at compile time
5. **Extensibility**: Easy to add new services without modifying existing code
6. **Testing**: Individual service routers can be tested in isolation

This pattern leverages the full power of the rmcp macro system for building scalable, production-ready MCP servers with multiple tool routers.

### Required Parameter Types

The examples above reference several parameter types that need to be defined:

```rust
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

// Authentication parameter types
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LoginRequest {
    #[schemars(description = "Username for authentication")]
    pub username: String,
    #[schemars(description = "Password for authentication")]
    pub password: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateTokenRequest {
    #[schemars(description = "JWT token to validate")]
    pub token: String,
}

// File service parameter types
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadFileRequest {
    #[schemars(description = "Path to the file to read")]
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WriteFileRequest {
    #[schemars(description = "Path to the file to write")]
    pub path: String,
    #[schemars(description = "Content to write to the file")]
    pub content: String,
}

// Database service parameter types
#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryRequest {
    #[schemars(description = "SQL query to execute")]
    pub sql: String,
    #[schemars(description = "Parameters for the SQL query")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DescribeTableRequest {
    #[schemars(description = "Name of the table to describe")]
    pub table_name: String,
}

// Configuration types
pub struct AuthConfig {
    pub jwt_secret: String,
    pub database_url: String,
}

pub struct StorageConfig {
    pub root_path: String,
    pub max_file_size: u64,
}

pub struct DatabaseConfig {
    pub connection_string: String,
    pub max_connections: u32,
}

pub struct ServerConfig {
    pub auth: AuthConfig,
    pub storage: StorageConfig,
    pub database: DatabaseConfig,
}

// Trait definitions
pub trait UserStore: Send + Sync {
    async fn authenticate(&self, username: &str, password: &str) -> Result<User, String>;
    async fn validate_request(&self, context: &RequestContext<RoleServer>) -> Result<(), McpError>;
}

pub struct User {
    pub id: String,
    pub username: String,
}

pub struct DatabaseUserStore;

impl DatabaseUserStore {
    pub async fn new(_database_url: &str) -> Result<Self, McpError> {
        Ok(Self)
    }
}

impl UserStore for DatabaseUserStore {
    async fn authenticate(&self, _username: &str, _password: &str) -> Result<User, String> {
        // Implementation would go here
        Ok(User {
            id: "user123".to_string(),
            username: "testuser".to_string(),
        })
    }
    
    async fn validate_request(&self, _context: &RequestContext<RoleServer>) -> Result<(), McpError> {
        // Implementation would go here
        Ok(())
    }
}

#[derive(Default)]
pub struct ServerMetrics {
    pub tool_calls: u64,
    pub last_activity: std::time::SystemTime,
}
```

## Production Server Structure

### Complete Production Example

Based on the SDK patterns, here's a complete production server structure:

```rust
// main.rs
use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::env;

mod server;
mod config;
mod services;
mod utils;

use server::ProductionMcpServer;
use config::ServerConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = ServerConfig::from_env()
        .or_else(|_| ServerConfig::from_file("config.toml"))
        .unwrap_or_default();

    tracing::info!("Starting production MCP server");

    // Create server instance
    let server = ProductionMcpServer::new(config).await?;

    // Start server with appropriate transport
    let transport = match env::var("MCP_TRANSPORT").as_deref() {
        Ok("stdio") => stdio().into(),
        Ok("sse") => {
            let sse_config = SseServerConfig::default();
            SseServer::new(sse_config).await?.into()
        }
        _ => stdio().into(),
    };

    let service = server.serve(transport).await.inspect_err(|e| {
        tracing::error!("Server startup error: {:?}", e);
    })?;

    // Handle graceful shutdown
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        tracing::info!("Received shutdown signal");
    };

    tokio::select! {
        _ = service.waiting() => {
            tracing::info!("Server stopped");
        }
        _ = shutdown_signal => {
            tracing::info!("Shutting down server");
            service.cancel().await?;
        }
    }

    Ok(())
}
```

### Configuration Management

```rust
// config.rs
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub auth: AuthConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub analytics: AnalyticsConfig,
    pub server: ServerSettings,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub token_expiry: u64,
    pub database_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerSettings {
    pub max_concurrent_tools: usize,
    pub tool_timeout_seconds: u64,
    pub rate_limit_per_minute: u32,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        // Load from environment variables
        config::Config::builder()
            .add_source(config::Environment::with_prefix("MCP"))
            .build()?
            .try_deserialize()
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(config::File::with_name(path.as_ref().to_str().unwrap()))
            .build()?
            .try_deserialize()
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            auth: AuthConfig {
                jwt_secret: "default-secret-change-in-production".to_string(),
                token_expiry: 3600,
                database_url: "sqlite://./auth.db".to_string(),
            },
            database: DatabaseConfig::default(),
            storage: StorageConfig::default(),
            analytics: AnalyticsConfig::default(),
            server: ServerSettings {
                max_concurrent_tools: 10,
                tool_timeout_seconds: 30,
                rate_limit_per_minute: 100,
            },
        }
    }
}
```

## Transport Options

### STDIO Transport

For command-line integration, as shown in [`external_repos/rust-sdk/examples/servers/src/counter_stdio.rs`](external_repos/rust-sdk/examples/servers/src/counter_stdio.rs):

```rust
use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> Result<()> {
    let service = Counter::new().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
```

### SSE Transport

For web integration, based on [`external_repos/rust-sdk/examples/servers/src/counter_sse.rs`](external_repos/rust-sdk/examples/servers/src/counter_sse.rs):

```rust
use rmcp::{ServiceExt, transport::sse_server::{SseServer, SseServerConfig}};

#[tokio::main]
async fn main() -> Result<()> {
    let config = SseServerConfig {
        bind_address: "127.0.0.1:8000".parse()?,
        sse_endpoint: "/sse".to_string(),
        ..Default::default()
    };

    let sse_server = SseServer::new(config).await?;
    let service = Counter::new().serve(sse_server).await?;
    
    println!("Server running on http://127.0.0.1:8000/sse");
    service.waiting().await?;
    Ok(())
}
```

### HTTP Streaming Transport

For high-performance HTTP, based on [`external_repos/rust-sdk/examples/servers/src/counter_streamhttp.rs`](external_repos/rust-sdk/examples/servers/src/counter_streamhttp.rs):

```rust
use rmcp::{ServiceExt, transport::streamable_http_server};

#[tokio::main]
async fn main() -> Result<()> {
    let app = streamable_http_server::create_app(Counter::new()).await?;
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("Server running on http://127.0.0.1:3000");
    
    axum::serve(listener, app).await?;
    Ok(())
}
```

## Authentication & Security

### OAuth2 Integration

Based on the complex auth example in [`external_repos/rust-sdk/examples/servers/src/complex_auth_sse.rs`](external_repos/rust-sdk/examples/servers/src/complex_auth_sse.rs):

```rust
use rmcp::transport::{
    auth::{AuthorizationMetadata, ClientRegistrationRequest},
    sse_server::SseServerConfig,
};

#[derive(Clone, Debug)]
struct McpOAuthStore {
    clients: Arc<RwLock<HashMap<String, OAuthClientConfig>>>,
    auth_sessions: Arc<RwLock<HashMap<String, AuthSession>>>,
    access_tokens: Arc<RwLock<HashMap<String, McpAccessToken>>>,
}

impl McpOAuthStore {
    async fn validate_token(&self, token: &str) -> Option<McpAccessToken> {
        self.access_tokens.read().await.get(token).cloned()
    }

    async fn create_mcp_token(&self, session_id: &str) -> Result<McpAccessToken, String> {
        // Implementation details...
    }
}

// Auth middleware for SSE connections
async fn validate_token_middleware(
    State(token_store): State<Arc<McpOAuthStore>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    // Extract and validate token from Authorization header
    let auth_header = request.headers().get("Authorization");
    // Validation logic...
}
```

### Rate Limiting

```rust
use governor::{Quota, RateLimiter};
use std::time::Duration;

#[derive(Clone)]
pub struct RateLimitedServer {
    inner: ProductionMcpServer,
    rate_limiter: Arc<RateLimiter<String, dashmap::DashMap<String, governor::InMemoryState>, governor::clock::DefaultClock>>,
}

impl RateLimitedServer {
    pub fn new(inner: ProductionMcpServer, rate_limit: u32) -> Self {
        let quota = Quota::per_minute(rate_limit.try_into().unwrap());
        Self {
            inner,
            rate_limiter: Arc::new(RateLimiter::dashmap(quota)),
        }
    }

    async fn check_rate_limit(&self, client_id: &str) -> Result<(), McpError> {
        if self.rate_limiter.check_key(client_id).is_err() {
            return Err(McpError::invalid_params("Rate limit exceeded", None));
        }
        Ok(())
    }
}
```

## Error Handling

### Comprehensive Error Types

```rust
use rmcp::Error as McpError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("File system error: {0}")]
    FileSystemError(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Tool timeout after {seconds} seconds")]
    ToolTimeout { seconds: u64 },
}

impl From<ServerError> for McpError {
    fn from(err: ServerError) -> Self {
        match err {
            ServerError::AuthenticationFailed(msg) => {
                McpError::invalid_params(format!("Auth failed: {}", msg), None)
            }
            ServerError::RateLimitExceeded => {
                McpError::invalid_params("Rate limit exceeded", None)
            }
            ServerError::ToolTimeout { seconds } => {
                McpError::internal_error(format!("Tool timeout after {} seconds", seconds), None)
            }
            _ => McpError::internal_error(err.to_string(), None),
        }
    }
}
```

### Tool Timeout Implementation

```rust
use tokio::time::{timeout, Duration};

#[tool_router]
impl ProductionMcpServer {
    #[tool(description = "Tool with timeout protection")]
    async fn protected_operation(&self) -> Result<CallToolResult, McpError> {
        let operation = async {
            // Long-running operation
            tokio::time::sleep(Duration::from_secs(10)).await;
            "Operation completed"
        };

        let timeout_duration = Duration::from_secs(self.config.server.tool_timeout_seconds);
        
        match timeout(timeout_duration, operation).await {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
            Err(_) => Err(ServerError::ToolTimeout { 
                seconds: self.config.server.tool_timeout_seconds 
            }.into()),
        }
    }
}
```

## Testing Strategies

### Unit Testing Tool Functions

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::{CallToolRequestParam, JsonObject};
    use tokio_test;

    #[tokio::test]
    async fn test_counter_increment() {
        let counter = Counter::new();
        let result = counter.increment().await.unwrap();
        
        assert!(!result.is_error);
        assert_eq!(result.content.len(), 1);
        
        if let Content::Text(text_content) = &result.content[0] {
            assert_eq!(text_content.text, "1");
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_tool_router_merge() {
        let auth_service = AuthService::new(&AuthConfig {
            jwt_secret: "test".to_string(),
            database_url: "sqlite::memory:".to_string(),
        }).await.unwrap();
        
        let mut combined_router = ToolRouter::new();
        combined_router.merge(auth_service.tool_router().clone());
        
        assert!(combined_router.has_route("login"));
        assert!(combined_router.has_route("validate_token"));
    }
}
```

### Testing Individual Service Routers

One of the key benefits of the modular approach is the ability to test individual service routers in isolation:

```rust
#[cfg(test)]
mod auth_service_tests {
    use super::auth_service::*;
    use rmcp::{
        handler::server::tool::ToolCallContext,
        model::{CallToolRequestParam, Meta},
        service::{RequestContext, RoleServer, Peer},
    };
    use std::sync::Arc;
    use tokio_util::sync::CancellationToken;

    #[tokio::test]
    async fn test_auth_router_login_tool() {
        // Test the individual auth router in isolation
        let router = auth_tool_router();
        let tools = router.list_all();
        
        // Verify expected tools are present
        assert!(tools.iter().any(|tool| tool.name == "login"));
        assert!(tools.iter().any(|tool| tool.name == "validate_token"));
        
        // Test tool execution
        let service = AuthService::new(&AuthConfig {
            jwt_secret: "test-secret".to_string(),
            database_url: "sqlite::memory:".to_string(),
        }).await.unwrap();
        
        let request = CallToolRequestParam {
            name: "login".into(),
            arguments: Some(serde_json::json!({
                "username": "testuser",
                "password": "testpass"
            }).as_object().unwrap().clone()),
        };
        
        let context = create_test_context();
        let tool_context = ToolCallContext::new(&service, request, context);
        
        let result = router.call(tool_context).await.unwrap();
        assert!(!result.is_error);
    }

    #[tokio::test]
    async fn test_file_router_tools() {
        // Test the file service router independently
        let router = file_service::file_tool_router();
        let tools = router.list_all();
        
        assert!(tools.iter().any(|tool| tool.name == "read_file"));
        assert!(tools.iter().any(|tool| tool.name == "write_file"));
    }

    #[tokio::test]
    async fn test_database_router_tools() {
        // Test the database service router independently
        let router = database_service::database_tool_router();
        let tools = router.list_all();
        
        assert!(tools.iter().any(|tool| tool.name == "query"));
        assert!(tools.iter().any(|tool| tool.name == "describe_table"));
    }

    #[tokio::test]
    async fn test_router_composition() {
        // Test that routers can be composed correctly
        let auth_router = auth_service::auth_tool_router();
        let file_router = file_service::file_tool_router();
        let db_router = database_service::database_tool_router();
        
        let combined = auth_router + file_router + db_router;
        let all_tools = combined.list_all();
        
        // Verify all tools are present in combined router
        assert!(all_tools.iter().any(|tool| tool.name == "login"));
        assert!(all_tools.iter().any(|tool| tool.name == "read_file"));
        assert!(all_tools.iter().any(|tool| tool.name == "query"));
        
        // Verify no duplicate tools
        let tool_names: Vec<&str> = all_tools.iter().map(|tool| tool.name.as_str()).collect();
        let unique_names: std::collections::HashSet<&str> = tool_names.iter().cloned().collect();
        assert_eq!(tool_names.len(), unique_names.len(), "Duplicate tool names found");
    }

    fn create_test_context() -> RequestContext<RoleServer> {
        // Helper function to create test context
        RequestContext {
            id: "test-id".into(),
            meta: Meta::default(),
            ct: CancellationToken::new(),
            peer: Peer::new(), // This would need proper initialization in real tests
            extensions: rmcp::model::Extensions::new(),
        }
    }
}
```

### Integration Testing Multi-Router Servers

Test the complete server with all routers integrated:

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use rmcp::{ServiceExt, transport::stdio};

    #[tokio::test]
    async fn test_production_server_initialization() {
        let config = ServerConfig {
            auth: AuthConfig {
                jwt_secret: "test-secret".to_string(),
                database_url: "sqlite::memory:".to_string(),
            },
            storage: StorageConfig {
                root_path: "/tmp/test".to_string(),
                max_file_size: 1024 * 1024,
            },
            database: DatabaseConfig {
                connection_string: "sqlite::memory:".to_string(),
                max_connections: 10,
            },
        };

        let server = ProductionMcpServer::new(config).await.unwrap();
        
        let server_info = server.get_info();
        assert!(server_info.capabilities.tools.is_some());
        
        // Test that all expected tools are available
        let context = create_test_context();
        let tools_result = server.list_tools(None, context).await.unwrap();
        
        let tool_names: Vec<&str> = tools_result.tools.iter()
            .map(|tool| tool.name.as_str())
            .collect();
            
        assert!(tool_names.contains(&"login"));
        assert!(tool_names.contains(&"validate_token"));
        assert!(tool_names.contains(&"read_file"));
        assert!(tool_names.contains(&"write_file"));
        assert!(tool_names.contains(&"query"));
        assert!(tool_names.contains(&"describe_table"));
    }

    #[tokio::test]
    async fn test_conditional_router_composition() {
        let config = create_test_config();
        
        // Test with only file features enabled
        let server = ProductionMcpServer::new_with_features(config.clone(), &["files"]).await.unwrap();
        let context = create_test_context();
        let tools = server.list_tools(None, context).await.unwrap();
        
        let tool_names: Vec<&str> = tools.tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"login")); // Always included
        assert!(tool_names.contains(&"read_file"));
        assert!(!tool_names.contains(&"query")); // Database not enabled
        
        // Test with both features enabled
        let server = ProductionMcpServer::new_with_features(config, &["files", "database"]).await.unwrap();
        let context = create_test_context();
        let tools = server.list_tools(None, context).await.unwrap();
        
        let tool_names: Vec<&str> = tools.tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"read_file"));
        assert!(tool_names.contains(&"query"));
    }

    fn create_test_config() -> ServerConfig {
        ServerConfig {
            auth: AuthConfig {
                jwt_secret: "test-secret".to_string(),
                database_url: "sqlite::memory:".to_string(),
            },
            storage: StorageConfig {
                root_path: "/tmp/test".to_string(),
                max_file_size: 1024 * 1024,
            },
            database: DatabaseConfig {
                connection_string: "sqlite::memory:".to_string(),
                max_connections: 10,
            },
        }
    }
}
```

### Mock Testing for External Dependencies

For testing services with external dependencies:

```rust
#[cfg(test)]
mod mock_tests {
    use super::*;
    use mockall::{automock, predicate::*};

    #[automock]
    trait MockUserStore: Send + Sync {
        async fn authenticate(&self, username: &str, password: &str) -> Result<User, String>;
        async fn validate_request(&self, context: &RequestContext<RoleServer>) -> Result<(), McpError>;
    }

    #[tokio::test]
    async fn test_auth_service_with_mock() {
        let mut mock_store = MockMockUserStore::new();
        
        mock_store
            .expect_authenticate()
            .with(eq("testuser"), eq("password123"))
            .times(1)
            .returning(|username, _| {
                Ok(User {
                    id: "user123".to_string(),
                    username: username.to_string(),
                })
            });

        // Test would use the mock store to verify behavior
        // This pattern allows testing without actual database connections
    }
}
```

This testing approach provides:

1. **Isolation**: Test individual routers without dependencies
2. **Integration**: Verify router composition works correctly
3. **Mocking**: Test with controlled external dependencies
4. **Coverage**: Ensure all tools are properly registered and accessible
5. **Validation**: Verify conditional composition logic works as expected

## Performance Optimization

### Concurrent Tool Execution

```rust
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct OptimizedMcpServer {
    inner: ProductionMcpServer,
    semaphore: Arc<Semaphore>,
}

impl OptimizedMcpServer {
    pub fn new(inner: ProductionMcpServer, max_concurrent: usize) -> Self {
        Self {
            inner,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }
}

#[tool_handler]
impl ServerHandler for OptimizedMcpServer {
    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        // Acquire semaphore permit for concurrency control
        let _permit = self.semaphore.acquire().await.unwrap();
        
        // Delegate to inner server
        self.inner.call_tool(request, context).await
    }
}
```

### Connection Pooling

```rust
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct DatabaseService {
    pool: Pool<Postgres>,
    tool_router: ToolRouter<Self>,
}

impl DatabaseService {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .connect(database_url)
            .await?;

        Ok(Self {
            pool,
            tool_router: Self::tool_router(),
        })
    }
}
```

## Deployment Considerations

### Docker Configuration

```dockerfile
# Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

COPY --from=builder /app/target/release/production-mcp-server /usr/local/bin/

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD echo '{"jsonrpc":"2.0","id":1,"method":"ping"}' | production-mcp-server || exit 1

EXPOSE 8000
CMD ["production-mcp-server"]
```

### Kubernetes Deployment

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mcp-server
spec:
  replicas: 3
  selector:
    matchLabels:
      app: mcp-server
  template:
    metadata:
      labels:
        app: mcp-server
    spec:
      containers:
      - name: mcp-server
        image: mcp-server:latest
        ports:
        - containerPort: 8000
        env:
        - name: MCP_TRANSPORT
          value: "sse"
        - name: MCP_AUTH_JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: mcp-secrets
              key: jwt-secret
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 30
          periodSeconds: 10
```

This comprehensive guide covers building production-ready MCP servers with the Rust SDK's powerful macro system for multiple tool router composition, based on real patterns from the official SDK source code and examples.