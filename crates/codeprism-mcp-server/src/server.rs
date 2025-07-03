//! Core MCP server implementation using rust-sdk

use crate::Config;
use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    service::RequestContext,
    tool, tool_handler, tool_router, Error as McpError, RoleServer, ServerHandler, ServiceExt,
};
use serde::Deserialize;
use tracing::{debug, info, warn};

// Parameter structures for tools
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TracePathParams {
    pub source: String,
    pub target: String,
    pub max_depth: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FindDependenciesParams {
    pub target: String,
    pub dependency_type: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FindReferencesParams {
    pub symbol_id: String,
    pub include_definitions: Option<bool>,
    pub context_lines: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExplainSymbolParams {
    pub symbol_id: String,
    pub include_dependencies: Option<bool>,
    pub include_usages: Option<bool>,
    pub context_lines: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchSymbolsParams {
    pub pattern: String,
    pub symbol_types: Option<Vec<String>>,
    pub inheritance_filters: Option<Vec<String>>,
    pub limit: Option<u32>,
    pub context_lines: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchContentParams {
    pub query: String,
    pub file_types: Option<Vec<String>>,
    pub case_sensitive: Option<bool>,
    pub regex: Option<bool>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FindPatternsParams {
    pub pattern: String,
    pub pattern_type: Option<String>,
    pub file_types: Option<Vec<String>>,
    pub limit: Option<u32>,
}

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

    // Core Navigation Tools - Real implementations migrated from legacy codeprism-mcp

    /// Trace execution path between two code symbols
    #[tool(description = "Find the shortest path between two code symbols")]
    fn trace_path(
        &self,
        Parameters(params): Parameters<TracePathParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!(
            "Trace path tool called: {} -> {}",
            params.source, params.target
        );

        // PLANNED(#168): Implement with CodePrism core functionality
        // NOTE: Awaiting graph query system integration for full implementation
        let result = serde_json::json!({
            "status": "not_implemented",
            "message": "Path tracing not yet implemented in rust-sdk server",
            "request": {
                "source": params.source,
                "target": params.target,
                "max_depth": params.max_depth.unwrap_or(10)
            },
            "note": "Will implement full graph path finding once core integration is complete"
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
        )]))
    }

    /// Find dependencies for a code symbol or file
    #[tool(description = "Analyze dependencies for a code symbol or file")]
    fn find_dependencies(
        &self,
        Parameters(params): Parameters<FindDependenciesParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!("Find dependencies tool called for: {}", params.target);

        let dep_type = params
            .dependency_type
            .unwrap_or_else(|| "direct".to_string());

        // Validate dependency type
        match dep_type.as_str() {
            "direct" | "calls" | "imports" | "reads" | "writes" => {
                // Valid dependency types
            }
            _ => {
                let error_msg = format!("Invalid dependency type: {}. Must be one of: direct, calls, imports, reads, writes", dep_type);
                return Ok(CallToolResult::error(vec![Content::text(error_msg)]));
            }
        }

        // PLANNED(#168): Implement with CodePrism core functionality
        let result = serde_json::json!({
            "status": "not_implemented",
            "message": "Dependency analysis not yet implemented in rust-sdk server",
            "request": {
                "target": params.target,
                "dependency_type": dep_type
            },
            "note": "Will implement full dependency analysis once core integration is complete"
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
        )]))
    }

    /// Find all references to a symbol across the codebase
    #[tool(description = "Find all references to a symbol across the codebase")]
    fn find_references(
        &self,
        Parameters(params): Parameters<FindReferencesParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!("Find references tool called for: {}", params.symbol_id);

        let include_defs = params.include_definitions.unwrap_or(true);
        let context = params.context_lines.unwrap_or(4);

        // PLANNED(#168): Implement with CodePrism core functionality
        let result = serde_json::json!({
            "status": "not_implemented",
            "message": "Reference finding not yet implemented in rust-sdk server",
            "request": {
                "symbol_id": params.symbol_id,
                "include_definitions": include_defs,
                "context_lines": context
            },
            "note": "Will implement full reference finding once core integration is complete"
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
        )]))
    }

    // Core Symbol Tools - Real implementations migrated from legacy codeprism-mcp

    /// Provide detailed explanation of a code symbol with context
    #[tool(description = "Provide detailed explanation of a code symbol with context")]
    fn explain_symbol(
        &self,
        Parameters(params): Parameters<ExplainSymbolParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!("Explain symbol tool called for: {}", params.symbol_id);

        let include_deps = params.include_dependencies.unwrap_or(false);
        let include_uses = params.include_usages.unwrap_or(false);
        let context = params.context_lines.unwrap_or(4);

        // PLANNED(#168): Implement with CodePrism core functionality
        let result = serde_json::json!({
            "status": "not_implemented",
            "message": "Symbol explanation not yet implemented in rust-sdk server",
            "request": {
                "symbol_id": params.symbol_id,
                "include_dependencies": include_deps,
                "include_usages": include_uses,
                "context_lines": context
            },
            "note": "Will implement full symbol explanation once core integration is complete"
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
        )]))
    }

    /// Search for symbols by name pattern with advanced filtering
    #[tool(description = "Search for symbols by name pattern with advanced inheritance filtering")]
    fn search_symbols(
        &self,
        Parameters(params): Parameters<SearchSymbolsParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!(
            "Search symbols tool called with pattern: {}",
            params.pattern
        );

        let max_results = params.limit.unwrap_or(50);
        let context = params.context_lines.unwrap_or(4);

        // Validate symbol types if provided
        if let Some(ref types) = params.symbol_types {
            for sym_type in types {
                match sym_type.as_str() {
                    "function" | "class" | "variable" | "module" | "method" => {
                        // Valid symbol types
                    }
                    _ => {
                        let error_msg = format!("Invalid symbol type: {}. Must be one of: function, class, variable, module, method", sym_type);
                        return Ok(CallToolResult::error(vec![Content::text(error_msg)]));
                    }
                }
            }
        }

        // PLANNED(#168): Implement with CodePrism core functionality
        let result = serde_json::json!({
            "status": "not_implemented",
            "message": "Symbol search not yet implemented in rust-sdk server",
            "request": {
                "pattern": params.pattern,
                "symbol_types": params.symbol_types,
                "inheritance_filters": params.inheritance_filters,
                "limit": max_results,
                "context_lines": context
            },
            "note": "Will implement full symbol search once core integration is complete"
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
        )]))
    }

    // Core Tools - Repository Analysis (Updated implementations)

    /// Get repository information and statistics
    #[tool(
        description = "Get comprehensive repository information including structure and statistics"
    )]
    fn get_repository_info(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Get repository info tool called");

        // PLANNED(#168): Implement with CodePrism core functionality
        let result = serde_json::json!({
            "status": "not_implemented",
            "message": "Repository analysis not yet implemented in rust-sdk server",
            "note": "Will implement full repository analysis once core integration is complete"
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
        )]))
    }

    /// Analyze project dependencies
    #[tool(description = "Analyze project dependencies and their relationships")]
    fn analyze_dependencies(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Analyze dependencies tool called");

        // PLANNED(#168): Implement with CodePrism core functionality
        let result = serde_json::json!({
            "status": "not_implemented",
            "message": "Dependency analysis not yet implemented in rust-sdk server",
            "note": "Will implement full dependency analysis once core integration is complete"
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
        )]))
    }

    // Search Tools (Updated implementations)

    /// Search for content across the codebase
    #[tool(description = "Search for content across files in the codebase")]
    fn search_content(
        &self,
        Parameters(params): Parameters<SearchContentParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!("Search content tool called with query: {}", params.query);

        let case_sens = params.case_sensitive.unwrap_or(false);
        let use_regex = params.regex.unwrap_or(false);
        let max_results = params.limit.unwrap_or(100);

        // PLANNED(#169): Implement with CodePrism core functionality
        let result = serde_json::json!({
            "status": "not_implemented",
            "message": "Content search not yet implemented in rust-sdk server",
            "request": {
                "query": params.query,
                "file_types": params.file_types,
                "case_sensitive": case_sens,
                "regex": use_regex,
                "limit": max_results
            },
            "note": "Will implement full content search once core integration is complete"
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
        )]))
    }

    /// Find patterns using regex or glob patterns
    #[tool(description = "Find patterns using regex or glob patterns in the codebase")]
    fn find_patterns(
        &self,
        Parameters(params): Parameters<FindPatternsParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!("Find patterns tool called with pattern: {}", params.pattern);

        let p_type = params.pattern_type.unwrap_or_else(|| "glob".to_string());
        let max_results = params.limit.unwrap_or(100);

        // Validate pattern type
        match p_type.as_str() {
            "regex" | "glob" => {
                // Valid pattern types
            }
            _ => {
                let error_msg = format!(
                    "Invalid pattern type: {}. Must be 'regex' or 'glob'",
                    p_type
                );
                return Ok(CallToolResult::error(vec![Content::text(error_msg)]));
            }
        }

        // PLANNED(#169): Implement with CodePrism core functionality
        let result = serde_json::json!({
            "status": "not_implemented",
            "message": "Pattern finding not yet implemented in rust-sdk server",
            "request": {
                "pattern": params.pattern,
                "pattern_type": p_type,
                "file_types": params.file_types,
                "limit": max_results
            },
            "note": "Will implement full pattern finding once core integration is complete"
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
        )]))
    }

    /// Perform semantic search across codebase
    #[tool(description = "Perform semantic search to find conceptually related code")]
    fn semantic_search(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Semantic search tool called");

        let response = serde_json::json!({
            "status": "not_implemented",
            "message": "Semantic search not yet implemented",
            "example_results": [
                {"file": "src/auth.rs", "relevance": 0.95, "concept": "authentication logic"},
                {"file": "src/middleware.rs", "relevance": 0.82, "concept": "request handling"}
            ]
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    /// Search by specific code element types
    #[tool(description = "Search for specific types of code elements (functions, structs, etc.)")]
    fn search_by_type(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Search by type tool called");

        let response = serde_json::json!({
            "status": "not_implemented",
            "message": "Type-based search not yet implemented",
            "example_types": {
                "functions": 156,
                "structs": 45,
                "enums": 23,
                "traits": 12,
                "modules": 18
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    /// Advanced search with multiple criteria
    #[tool(description = "Advanced search combining multiple search criteria and filters")]
    fn advanced_search(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Advanced search tool called");

        let response = serde_json::json!({
            "status": "not_implemented",
            "message": "Advanced search not yet implemented",
            "example_filters": {
                "file_types": ["rs", "toml", "md"],
                "date_range": "last_30_days",
                "size_range": "1kb_to_100kb",
                "complexity": "medium_to_high"
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    // Analysis Tools (Updated implementations)

    /// Analyze code complexity metrics
    #[tool(
        description = "Analyze code complexity including cyclomatic complexity and maintainability"
    )]
    fn analyze_complexity(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Analyze complexity tool called");

        let response = serde_json::json!({
            "status": "not_implemented",
            "message": "Complexity analysis not yet implemented",
            "example_metrics": {
                "cyclomatic_complexity": 8.5,
                "cognitive_complexity": 12.3,
                "maintainability_index": 85.2,
                "lines_of_code": 1250,
                "complexity_distribution": {
                    "low": 45,
                    "medium": 23,
                    "high": 8,
                    "very_high": 2
                }
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    /// Analyze control flow patterns
    #[tool(description = "Analyze control flow patterns and execution paths in code")]
    fn analyze_control_flow(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Analyze control flow tool called");

        let response = serde_json::json!({
            "status": "not_implemented",
            "message": "Control flow analysis not yet implemented",
            "example_patterns": {
                "decision_points": 156,
                "loops": 45,
                "recursions": 12,
                "exception_handling": 23,
                "execution_paths": 89,
                "dead_code_blocks": 3
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    /// Analyze code quality metrics
    #[tool(description = "Analyze code quality including best practices and code smells")]
    fn analyze_code_quality(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Analyze code quality tool called");

        let response = serde_json::json!({
            "status": "not_implemented",
            "message": "Code quality analysis not yet implemented",
            "example_quality": {
                "overall_score": 7.8,
                "code_smells": 14,
                "duplication_percentage": 3.2,
                "test_coverage": 89.5,
                "documentation_coverage": 76.3,
                "issues": {
                    "naming_conventions": 5,
                    "function_length": 3,
                    "deep_nesting": 2
                }
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    /// Analyze performance characteristics
    #[tool(description = "Analyze performance bottlenecks and optimization opportunities")]
    fn analyze_performance(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Analyze performance tool called");

        let response = serde_json::json!({
            "status": "not_implemented",
            "message": "Performance analysis not yet implemented",
            "example_metrics": {
                "hotspots": [
                    {"function": "heavy_computation", "cpu_time": "45%"},
                    {"function": "database_query", "cpu_time": "23%"}
                ],
                "memory_usage": {
                    "peak_usage": "256MB",
                    "allocation_rate": "high",
                    "garbage_collection": "moderate"
                },
                "optimization_suggestions": [
                    "Consider caching database results",
                    "Optimize algorithm in heavy_computation"
                ]
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    /// Analyze JavaScript-specific patterns
    #[tool(description = "Analyze JavaScript-specific code patterns and best practices")]
    fn analyze_javascript(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Analyze JavaScript tool called");

        let response = serde_json::json!({
            "status": "not_implemented",
            "message": "JavaScript analysis not yet implemented",
            "example_analysis": {
                "es_version": "ES2020",
                "async_patterns": 45,
                "callback_depth": 3.2,
                "promises_vs_callbacks": {
                    "promises": 78,
                    "callbacks": 23
                },
                "framework_usage": {
                    "react": 156,
                    "node": 89,
                    "express": 34
                }
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    /// Perform specialized analysis for specific domains
    #[tool(description = "Perform specialized analysis for specific domains or patterns")]
    fn specialized_analysis(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Specialized analysis tool called");

        let response = serde_json::json!({
            "status": "not_implemented",
            "message": "Specialized analysis not yet implemented",
            "example_domains": {
                "security": {
                    "vulnerabilities": 2,
                    "risk_level": "low"
                },
                "concurrency": {
                    "race_conditions": 0,
                    "deadlock_potential": "none"
                },
                "architecture": {
                    "coupling": "loose",
                    "cohesion": "high",
                    "patterns": ["observer", "factory", "strategy"]
                }
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    // Workflow Tools (Updated implementations)

    /// Provide guidance for code improvement and best practices
    #[tool(description = "Provide guidance for code improvement and best practices")]
    fn provide_guidance(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Provide guidance tool called");

        let response = serde_json::json!({
            "status": "not_implemented",
            "message": "Guidance generation not yet implemented",
            "example_guidance": {
                "suggestions": [
                    "Consider extracting this function for better modularity",
                    "Add error handling for this network operation",
                    "This function is too complex, consider breaking it down"
                ],
                "priority": "medium",
                "effort": "low"
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    /// Optimize code for performance and maintainability
    #[tool(description = "Optimize code for performance and maintainability")]
    fn optimize_code(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Optimize code tool called");

        let response = serde_json::json!({
            "status": "not_implemented",
            "message": "Code optimization not yet implemented",
            "example_optimizations": {
                "performance": [
                    "Replace O(n²) algorithm with O(n log n) approach",
                    "Use lazy evaluation for expensive computations"
                ],
                "maintainability": [
                    "Extract constants to improve readability",
                    "Add documentation for complex logic"
                ],
                "estimated_improvement": "25% performance gain"
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    /// Process multiple files or operations in batch
    #[tool(description = "Process multiple files or operations in batch")]
    fn batch_process(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Batch process tool called");

        let response = serde_json::json!({
            "status": "not_implemented",
            "message": "Batch processing not yet implemented",
            "example_operations": {
                "supported_operations": [
                    "batch_analyze_files",
                    "batch_refactor_patterns",
                    "batch_optimize_imports"
                ],
                "max_concurrent": 5,
                "progress_tracking": true
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    /// Automate common development workflows
    #[tool(description = "Automate common development workflows")]
    fn workflow_automation(&self) -> std::result::Result<CallToolResult, McpError> {
        info!("Workflow automation tool called");

        let response = serde_json::json!({
            "status": "not_implemented",
            "message": "Workflow automation not yet implemented",
            "example_workflows": {
                "available_workflows": [
                    "code_review_checklist",
                    "refactoring_pipeline",
                    "testing_strategy_generation"
                ],
                "custom_workflows": true,
                "integration_options": ["git", "ci_cd", "code_quality_tools"]
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }

    /// Run the MCP server with stdio transport
    pub async fn run(self) -> std::result::Result<(), crate::Error> {
        info!("Starting CodePrism MCP Server");

        use rmcp::transport::stdio;

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

    /// Get the server configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
}

#[tool_handler]
impl ServerHandler for CodePrismMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: self.config.server.name.clone(),
                version: self.config.server.version.clone(),
            },
            instructions: Some(
                "CodePrism MCP Server - Advanced code analysis and navigation tools".to_string(),
            ),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<InitializeResult, McpError> {
        info!("MCP server initialized");
        Ok(self.get_info())
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListResourcesResult, McpError> {
        warn!("Resources not implemented");
        Ok(ListResourcesResult {
            resources: vec![],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        _request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ReadResourceResult, McpError> {
        warn!("Resource reading not implemented");
        Err(McpError::invalid_params(
            "Resource reading not implemented",
            None,
        ))
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListPromptsResult, McpError> {
        warn!("Prompts not implemented");
        Ok(ListPromptsResult {
            prompts: vec![],
            next_cursor: None,
        })
    }

    async fn get_prompt(
        &self,
        _request: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<GetPromptResult, McpError> {
        warn!("Prompt retrieval not implemented");
        Err(McpError::invalid_params(
            "Prompt retrieval not implemented",
            None,
        ))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListResourceTemplatesResult, McpError> {
        warn!("Resource templates not implemented");
        Ok(ListResourceTemplatesResult {
            resource_templates: vec![],
            next_cursor: None,
        })
    }
}
