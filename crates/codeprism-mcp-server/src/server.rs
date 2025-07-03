//! Core MCP server implementation using rust-sdk

use crate::Config;
use rmcp::{
    handler::server::router::tool::ToolRouter, model::*, service::RequestContext, tool,
    tool_handler, tool_router, transport::stdio, Error as McpError, RoleServer, ServerHandler,
    ServiceExt,
};
use tracing::{debug, info};

/// The main CodePrism MCP Server implementation
#[derive(Clone)]
pub struct CodePrismMcpServer {
    /// Server configuration
    config: Config,
    /// Combined tool router for handling MCP tool calls
    tool_router: ToolRouter<CodePrismMcpServer>,
}

#[tool_router]
impl CodePrismMcpServer {
    /// Create a new MCP server instance
    pub async fn new(config: Config) -> std::result::Result<Self, crate::Error> {
        info!("Initializing CodePrism MCP Server");

        // Validate configuration
        config.validate()?;

        debug!("Server configuration validated successfully");

        Ok(Self {
            config,
            tool_router: Self::tool_router(),
        })
    }

    /// Simple ping tool for testing MCP functionality
    #[tool(description = "Simple ping tool that responds with pong")]
    fn ping(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Ping tool called");
        Ok(CallToolResult::success(vec![Content::text("pong")]))
    }

    /// Version tool that returns server version information
    #[tool(description = "Get server version and configuration information")]
    fn version(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Version tool called");

        let version_info = serde_json::json!({
            "server_name": self.config.server.name,
            "server_version": self.config.server.version,
            "mcp_protocol_version": crate::MCP_VERSION,
            "tools_enabled": {
                "core": self.config.tools.enable_core,
                "search": self.config.tools.enable_search,
                "analysis": self.config.tools.enable_analysis,
                "workflow": self.config.tools.enable_workflow
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            version_info.to_string(),
        )]))
    }

    /// System information tool that returns system details
    #[tool(description = "Get system information including OS, memory, and environment")]
    fn system_info(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("System info tool called");

        let _current_time = chrono::Utc::now();
        let system_info = serde_json::json!({
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "family": std::env::consts::FAMILY,
            "rust_version": env!("CARGO_PKG_VERSION"),
            "server_config": {
                "name": self.config.server.name,
                "version": self.config.server.version,
                "max_concurrent_tools": self.config.server.max_concurrent_tools,
                "request_timeout_secs": self.config.server.request_timeout_secs
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            system_info.to_string(),
        )]))
    }

    /// Health check tool that verifies server status
    #[tool(description = "Perform health check on server components")]
    fn health_check(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Health check tool called");

        let health_status = serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "components": {
                "server": "operational",
                "tools": "available",
                "config": "valid"
            },
            "uptime_seconds": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });

        Ok(CallToolResult::success(vec![Content::text(
            health_status.to_string(),
        )]))
    }

    // Core Tools - Symbol Navigation (FUTURE: Full implementation in issues #168-171)

    /// Navigate to a specific symbol in the codebase
    #[tool(description = "Navigate to a symbol definition in the codebase")]
    fn navigate_to_symbol(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Navigate to symbol tool called");

        let response = serde_json::json!({
            "status": "not_implemented",
            "message": "Symbol navigation not yet implemented",
            "suggestion": "This will find and navigate to symbol definitions"
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    /// List all symbols in the codebase
    #[tool(description = "List all symbols available in the codebase")]
    fn list_symbols(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("List symbols tool called");

        let response = serde_json::json!({
            "status": "not_implemented",
            "message": "Symbol listing not yet implemented",
            "example_symbols": [
                {"name": "main", "type": "function", "file": "src/main.rs", "line": 10},
                {"name": "Config", "type": "struct", "file": "src/config.rs", "line": 15},
                {"name": "UserService", "type": "struct", "file": "src/service.rs", "line": 25}
            ]
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    /// Get detailed information about a specific symbol
    #[tool(description = "Get detailed information about a specific symbol")]
    fn get_symbol_info(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Get symbol info tool called");

        let response = serde_json::json!({
            "status": "not_implemented",
            "message": "Symbol information retrieval not yet implemented",
            "example_info": {
                "name": "example_symbol",
                "type": "function",
                "documentation": "Placeholder documentation",
                "file_location": "src/example.rs:42"
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    // Core Tools - Repository Analysis (FUTURE: Full implementation in issues #168-171)

    /// Get repository information and statistics
    #[tool(
        description = "Get comprehensive repository information including structure and statistics"
    )]
    fn get_repository_info(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Get repository info tool called");

        let response = serde_json::json!({
            "status": "not_implemented",
            "message": "Repository analysis not yet implemented",
            "example_info": {
                "name": "codeprism",
                "language": "Rust",
                "total_files": 150,
                "lines_of_code": 25000,
                "structure": {
                    "crates": 8,
                    "modules": 45,
                    "tests": 120
                }
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    /// Analyze project dependencies
    #[tool(description = "Analyze project dependencies and their relationships")]
    fn analyze_dependencies(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Analyze dependencies tool called");

        let response = serde_json::json!({
            "status": "not_implemented",
            "message": "Dependency analysis not yet implemented",
            "example_analysis": {
                "total_dependencies": 45,
                "direct_dependencies": 12,
                "example_deps": [
                    {"name": "serde", "version": "1.0.210", "type": "direct"},
                    {"name": "tokio", "version": "1.35.0", "type": "direct"},
                    {"name": "rmcp", "version": "0.1.0", "type": "direct"}
                ]
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    /// Run the MCP server with stdio transport
    pub async fn run(self) -> std::result::Result<(), crate::Error> {
        info!(
            "Starting MCP server '{}' version {}",
            self.config.server.name, self.config.server.version
        );

        info!("Enabled tools:");
        info!("  Core tools: {}", self.config.tools.enable_core);
        info!("  Search tools: {}", self.config.tools.enable_search);
        info!("  Analysis tools: {}", self.config.tools.enable_analysis);
        info!("  Workflow tools: {}", self.config.tools.enable_workflow);

        info!("Starting MCP server with stdio transport");

        // Start the MCP server with stdio transport
        let service = self
            .serve(stdio())
            .await
            .map_err(|e| crate::Error::server_init(format!("Failed to start MCP server: {}", e)))?;

        info!("MCP server is ready to accept connections");

        // Wait for the server to complete
        service
            .waiting()
            .await
            .map_err(|e| crate::Error::server_init(format!("Server error: {}", e)))?;

        info!("MCP server shut down successfully");
        Ok(())
    }

    /// Get server configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
}

#[tool_handler]
impl ServerHandler for CodePrismMcpServer {
    /// Provide server information and capabilities
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: self.config.server.name.clone(),
                version: self.config.server.version.clone(),
            },
            instructions: Some(
                "CodePrism MCP Server provides code analysis capabilities. \
                 Use the available tools to analyze code structure, search for patterns, \
                 and perform various code intelligence operations."
                    .to_string(),
            ),
        }
    }

    /// Initialize the server
    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<InitializeResult, McpError> {
        info!("MCP server initialized successfully");
        Ok(self.get_info())
    }

    // Note: list_tools and call_tool are automatically generated by #[tool_handler] macro

    /// List resources (not implemented for now)
    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![],
            next_cursor: None,
        })
    }

    /// Read resource (not implemented for now)
    async fn read_resource(
        &self,
        _request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ReadResourceResult, McpError> {
        Err(McpError::invalid_params(
            "Resource reading not implemented",
            None,
        ))
    }

    /// List prompts (not implemented for now)
    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            prompts: vec![],
            next_cursor: None,
        })
    }

    /// Get prompt (not implemented for now)
    async fn get_prompt(
        &self,
        _request: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<GetPromptResult, McpError> {
        Err(McpError::invalid_params("Prompts not implemented", None))
    }

    /// List resource templates (not implemented for now)
    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            resource_templates: vec![],
            next_cursor: None,
        })
    }
}
