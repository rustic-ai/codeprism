//! Core MCP server implementation using rust-sdk

use crate::{Config, Result};
use rmcp::{
    model::*, service::RequestContext, transport::stdio, Error as McpError, RoleServer,
    ServerHandler, ServiceExt,
};
use tracing::{debug, info};

/// The main CodePrism MCP Server implementation
#[derive(Clone)]
pub struct CodePrismMcpServer {
    /// Server configuration
    config: Config,
}

impl CodePrismMcpServer {
    /// Create a new MCP server instance
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing CodePrism MCP Server");

        // Validate configuration
        config.validate()?;

        debug!("Server configuration validated successfully");

        Ok(Self { config })
    }

    /// Run the MCP server with stdio transport
    pub async fn run(self) -> Result<()> {
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

    /// List available tools (empty for now, will be implemented in subsequent tasks)
    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListToolsResult, McpError> {
        Ok(ListToolsResult {
            tools: vec![],
            next_cursor: None,
        })
    }

    /// Handle tool calls (not implemented yet)
    async fn call_tool(
        &self,
        _request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<CallToolResult, McpError> {
        Err(McpError::invalid_params("No tools implemented yet", None))
    }

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
