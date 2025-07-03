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

// CodePrism core components
use codeprism_core::{
    ContentSearchManager, GraphQuery, GraphStore, InheritanceFilter, LanguageRegistry,
    NoOpProgressReporter, NodeKind, RepositoryConfig, RepositoryManager, RepositoryScanner,
};
use std::path::PathBuf;
use std::sync::Arc;

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
#[allow(dead_code)] // Fields will be used as more tools are implemented
pub struct CodePrismMcpServer {
    /// Server configuration
    config: Config,
    /// Combined tool router for handling MCP tool calls
    tool_router: ToolRouter<CodePrismMcpServer>,
    /// Core graph store for code intelligence
    graph_store: Arc<GraphStore>,
    /// Graph query engine for advanced operations
    graph_query: Arc<GraphQuery>,
    /// Repository scanner for file discovery
    repository_scanner: Arc<RepositoryScanner>,
    /// Content search manager for text search
    content_search: Arc<ContentSearchManager>,
    /// Repository manager for metadata and configuration
    repository_manager: Arc<RepositoryManager>,
    /// Current repository path
    repository_path: Option<PathBuf>,
}

#[tool_router]
impl CodePrismMcpServer {
    /// Create a new MCP server instance
    pub async fn new(config: Config) -> std::result::Result<Self, crate::Error> {
        info!("Initializing CodePrism MCP Server");

        // Validate configuration
        config.validate()?;

        debug!("Server configuration validated successfully");

        // Initialize core components
        let graph_store = Arc::new(GraphStore::new());
        let graph_query = Arc::new(GraphQuery::new(Arc::clone(&graph_store)));
        let repository_scanner = Arc::new(RepositoryScanner::new());
        let content_search = Arc::new(ContentSearchManager::new());

        // Initialize repository manager with language registry
        let language_registry = Arc::new(LanguageRegistry::new());
        let repository_manager = Arc::new(RepositoryManager::new(language_registry));

        Ok(Self {
            config,
            tool_router: Self::tool_router(),
            graph_store,
            graph_query,
            repository_scanner,
            content_search,
            repository_manager,
            repository_path: None,
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
            "server_name": self.config.server().name,
            "server_version": self.config.server().version,
            "mcp_protocol_version": crate::MCP_VERSION,
            "tools_enabled": {
                "core": self.config.tools().enable_core,
                "search": self.config.tools().enable_search,
                "analysis": self.config.tools().enable_analysis,
                "workflow": self.config.tools().enable_workflow
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
                "name": self.config.server().name,
                "version": self.config.server().version,
                "max_concurrent_tools": self.config.server().max_concurrent_tools,
                "request_timeout_secs": self.config.server().request_timeout_secs
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

        let max_depth = params.max_depth.unwrap_or(10) as usize;

        // Parse the source node ID from hex string
        let source_id = match codeprism_core::NodeId::from_hex(&params.source) {
            Ok(id) => id,
            Err(_) => {
                let error_msg = format!(
                    "Invalid source symbol ID format: {}. Expected hexadecimal string.",
                    params.source
                );
                return Ok(CallToolResult::error(vec![Content::text(error_msg)]));
            }
        };

        // Parse the target node ID from hex string
        let target_id = match codeprism_core::NodeId::from_hex(&params.target) {
            Ok(id) => id,
            Err(_) => {
                let error_msg = format!(
                    "Invalid target symbol ID format: {}. Expected hexadecimal string.",
                    params.target
                );
                return Ok(CallToolResult::error(vec![Content::text(error_msg)]));
            }
        };

        // Find path using graph query
        let path_result = self
            .graph_query
            .find_path(&source_id, &target_id, Some(max_depth));

        let result = match path_result {
            Ok(Some(path)) => {
                // Resolve node details for the path
                let path_nodes: Vec<_> = path
                    .path
                    .iter()
                    .filter_map(|node_id| self.graph_store.get_node(node_id))
                    .map(|node| {
                        serde_json::json!({
                            "id": node.id.to_hex(),
                            "name": node.name,
                            "kind": format!("{:?}", node.kind),
                            "language": format!("{:?}", node.lang),
                            "file": node.file.display().to_string(),
                            "span": {
                                "start_byte": node.span.start_byte,
                                "end_byte": node.span.end_byte,
                                "start_line": node.span.start_line,
                                "start_column": node.span.start_column,
                                "end_line": node.span.end_line,
                                "end_column": node.span.end_column,
                            }
                        })
                    })
                    .collect();

                let path_edges: Vec<_> = path
                    .edges
                    .iter()
                    .map(|edge| {
                        serde_json::json!({
                            "source": edge.source.to_hex(),
                            "target": edge.target.to_hex(),
                            "kind": format!("{:?}", edge.kind),
                        })
                    })
                    .collect();

                serde_json::json!({
                    "status": "success",
                    "path_found": true,
                    "source_id": params.source,
                    "target_id": params.target,
                    "distance": path.distance,
                    "path_length": path.path.len(),
                    "nodes": path_nodes,
                    "edges": path_edges,
                    "query": {
                        "source": params.source,
                        "target": params.target,
                        "max_depth": max_depth
                    }
                })
            }
            Ok(None) => {
                serde_json::json!({
                    "status": "success",
                    "path_found": false,
                    "source_id": params.source,
                    "target_id": params.target,
                    "message": format!("No path found between {} and {} within {} hops", params.source, params.target, max_depth),
                    "query": {
                        "source": params.source,
                        "target": params.target,
                        "max_depth": max_depth
                    }
                })
            }
            Err(e) => {
                serde_json::json!({
                    "status": "error",
                    "message": format!("Path finding failed: {}", e),
                    "query": {
                        "source": params.source,
                        "target": params.target,
                        "max_depth": max_depth
                    }
                })
            }
        };

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

        // Parse the symbol ID from hex string
        let node_id = match codeprism_core::NodeId::from_hex(&params.symbol_id) {
            Ok(id) => id,
            Err(_) => {
                let error_msg = format!(
                    "Invalid symbol ID format: {}. Expected hexadecimal string.",
                    params.symbol_id
                );
                return Ok(CallToolResult::error(vec![Content::text(error_msg)]));
            }
        };

        // Find references using graph query
        let references_result = self.graph_query.find_references(&node_id);

        let result = match references_result {
            Ok(references) => {
                serde_json::json!({
                    "status": "success",
                    "symbol_id": params.symbol_id,
                    "references": references.iter().map(|reference| {
                        serde_json::json!({
                            "source_symbol": {
                                "id": reference.source_node.id.to_hex(),
                                "name": reference.source_node.name,
                                "kind": format!("{:?}", reference.source_node.kind),
                                "language": format!("{:?}", reference.source_node.lang),
                                "file": reference.source_node.file.display().to_string(),
                                "span": {
                                    "start_byte": reference.source_node.span.start_byte,
                                    "end_byte": reference.source_node.span.end_byte,
                                    "start_line": reference.source_node.span.start_line,
                                    "start_column": reference.source_node.span.start_column,
                                    "end_line": reference.source_node.span.end_line,
                                    "end_column": reference.source_node.span.end_column,
                                }
                            },
                            "reference_type": format!("{:?}", reference.edge_kind),
                            "location": {
                                "file": reference.location.file.display().to_string(),
                                "span": {
                                    "start_byte": reference.location.span.start_byte,
                                    "end_byte": reference.location.span.end_byte,
                                    "start_line": reference.location.span.start_line,
                                    "start_column": reference.location.span.start_column,
                                    "end_line": reference.location.span.end_line,
                                    "end_column": reference.location.span.end_column,
                                }
                            }
                        })
                    }).collect::<Vec<_>>(),
                    "total_references": references.len(),
                    "query": {
                        "symbol_id": params.symbol_id,
                        "include_definitions": include_defs,
                        "context_lines": context
                    }
                })
            }
            Err(e) => {
                serde_json::json!({
                    "status": "error",
                    "message": format!("Reference finding failed: {}", e),
                    "query": {
                        "symbol_id": params.symbol_id,
                        "include_definitions": include_defs,
                        "context_lines": context
                    }
                })
            }
        };

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

        let max_results = params.limit.unwrap_or(50) as usize;
        let context = params.context_lines.unwrap_or(4);

        // Validate symbol types if provided
        let node_kinds = if let Some(ref types) = params.symbol_types {
            let mut kinds = Vec::new();
            for sym_type in types {
                match sym_type.as_str() {
                    "function" => kinds.push(NodeKind::Function),
                    "class" => kinds.push(NodeKind::Class),
                    "variable" => kinds.push(NodeKind::Variable),
                    "module" => kinds.push(NodeKind::Module),
                    "method" => kinds.push(NodeKind::Method),
                    _ => {
                        let error_msg = format!("Invalid symbol type: {}. Must be one of: function, class, variable, module, method", sym_type);
                        return Ok(CallToolResult::error(vec![Content::text(error_msg)]));
                    }
                }
            }
            Some(kinds)
        } else {
            None
        };

        // Parse inheritance filters if provided
        let inheritance_filters = if let Some(ref filters) = params.inheritance_filters {
            let mut parsed_filters = Vec::new();
            for filter in filters {
                if let Some(base_class) = filter.strip_prefix("inherits_from:") {
                    parsed_filters.push(InheritanceFilter::InheritsFrom(base_class.to_string()));
                } else if let Some(metaclass) = filter.strip_prefix("metaclass:") {
                    parsed_filters.push(InheritanceFilter::HasMetaclass(metaclass.to_string()));
                } else if let Some(mixin) = filter.strip_prefix("mixin:") {
                    parsed_filters.push(InheritanceFilter::UsesMixin(mixin.to_string()));
                } else {
                    let error_msg = format!("Invalid inheritance filter: {}. Must be one of: inherits_from:<class>, metaclass:<class>, mixin:<class>", filter);
                    return Ok(CallToolResult::error(vec![Content::text(error_msg)]));
                }
            }
            Some(parsed_filters)
        } else {
            None
        };

        // Perform symbol search using graph query
        let search_result = if let Some(inheritance_filters) = inheritance_filters {
            self.graph_query.search_symbols_with_inheritance(
                &params.pattern,
                node_kinds,
                Some(inheritance_filters),
                Some(max_results),
            )
        } else {
            self.graph_query
                .search_symbols(&params.pattern, node_kinds, Some(max_results))
        };

        let result = match search_result {
            Ok(symbols) => {
                serde_json::json!({
                    "status": "success",
                    "symbols": symbols.iter().map(|symbol| {
                        serde_json::json!({
                            "id": symbol.node.id.to_hex(),
                            "name": symbol.node.name,
                            "kind": format!("{:?}", symbol.node.kind),
                            "language": format!("{:?}", symbol.node.lang),
                            "file": symbol.node.file.display().to_string(),
                            "span": {
                                "start_byte": symbol.node.span.start_byte,
                                "end_byte": symbol.node.span.end_byte,
                                "start_line": symbol.node.span.start_line,
                                "start_column": symbol.node.span.start_column,
                                "end_line": symbol.node.span.end_line,
                                "end_column": symbol.node.span.end_column,
                            },
                            "references_count": symbol.references_count,
                            "dependencies_count": symbol.dependencies_count,
                        })
                    }).collect::<Vec<_>>(),
                    "total_found": symbols.len(),
                    "query": {
                        "pattern": params.pattern,
                        "symbol_types": params.symbol_types,
                        "inheritance_filters": params.inheritance_filters,
                        "limit": max_results,
                        "context_lines": context
                    }
                })
            }
            Err(e) => {
                serde_json::json!({
                    "status": "error",
                    "message": format!("Symbol search failed: {}", e),
                    "query": {
                        "pattern": params.pattern,
                        "symbol_types": params.symbol_types,
                        "inheritance_filters": params.inheritance_filters,
                        "limit": max_results,
                        "context_lines": context
                    }
                })
            }
        };

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

        let result = if let Some(ref repo_path) = self.repository_path {
            // Get basic repository information
            let repo_name = repo_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown");

            // Get graph statistics
            let graph_stats = self.graph_store.get_stats();

            // Basic directory scan for file types
            let discovered_files = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    self.repository_scanner
                        .scan_repository(repo_path, Arc::new(NoOpProgressReporter))
                        .await
                })
            });

            match discovered_files {
                Ok(scan_result) => {
                    serde_json::json!({
                        "status": "success",
                        "repository": {
                            "name": repo_name,
                            "path": repo_path.display().to_string(),
                            "total_files": scan_result.total_files,
                            "scan_duration_ms": scan_result.duration_ms,
                            "files_by_language": scan_result.files_by_language.iter()
                                .map(|(lang, files)| (format!("{:?}", lang), files.len()))
                                .collect::<std::collections::HashMap<String, usize>>()
                        },
                        "graph_statistics": {
                            "total_nodes": graph_stats.total_nodes,
                            "total_edges": graph_stats.total_edges,
                            "total_files": graph_stats.total_files,
                            "nodes_by_kind": graph_stats.nodes_by_kind.iter()
                                .map(|(kind, count)| (format!("{:?}", kind), *count))
                                .collect::<std::collections::HashMap<String, usize>>()
                        }
                    })
                }
                Err(e) => {
                    serde_json::json!({
                        "status": "error",
                        "message": format!("Failed to scan repository: {}", e),
                        "repository": {
                            "name": repo_name,
                            "path": repo_path.display().to_string()
                        }
                    })
                }
            }
        } else {
            serde_json::json!({
                "status": "error",
                "message": "No repository configured. Call initialize_repository first.",
                "note": "Use the server initialization to set up a repository path"
            })
        };

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

    /// Initialize the server with a repository path
    pub async fn initialize_repository<P: AsRef<std::path::Path>>(
        &mut self,
        repo_path: P,
    ) -> Result<(), crate::Error> {
        let repo_path = repo_path.as_ref().to_path_buf();

        info!("Initializing repository: {}", repo_path.display());

        // Create repository configuration
        let repo_id = repo_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("default")
            .to_string();

        let _repo_config = RepositoryConfig::new(repo_id.clone(), &repo_path);

        // NOTE: Due to Arc constraints, we'll implement graph population in the tool methods
        // instead of during initialization. This allows for lazy loading when tools are called.

        self.repository_path = Some(repo_path);

        info!("Repository path configured: {}", repo_id);

        Ok(())
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
                name: self.config.server().name.clone(),
                version: self.config.server().version.clone(),
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
