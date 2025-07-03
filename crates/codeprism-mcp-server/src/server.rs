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
use codeprism_analysis::CodeAnalyzer;
use codeprism_core::graph::DependencyType;
use codeprism_core::{
    ContentSearchManager, GraphQuery, GraphStore, InheritanceFilter, LanguageRegistry,
    NoOpProgressReporter, NodeKind, RepositoryConfig, RepositoryManager, RepositoryScanner,
    SearchQueryBuilder,
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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchByTypeParams {
    pub symbol_types: Vec<String>,
    pub include_inherited: Option<bool>,
    pub file_patterns: Option<Vec<String>>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SemanticSearchParams {
    pub concept: String,
    pub context: Option<String>,
    pub relevance_threshold: Option<f32>,
    pub include_similar: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AdvancedSearchParams {
    pub query: String,
    pub file_types: Option<Vec<String>>,
    pub symbol_types: Option<Vec<String>>,
    pub date_range: Option<String>,
    pub size_range: Option<String>,
    pub complexity_filter: Option<String>,
    pub exclude_patterns: Option<Vec<String>>,
    pub include_tests: Option<bool>,
    pub include_dependencies: Option<bool>,
    pub limit: Option<usize>,
}

// Analysis tool parameter types

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AnalyzeComplexityParams {
    pub target: String,
    pub metrics: Option<Vec<String>>,
    pub threshold_warnings: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AnalyzePerformanceParams {
    pub target: String,
    pub analysis_types: Option<Vec<String>>,
    pub complexity_threshold: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AnalyzeSecurityParams {
    pub target: String,
    pub vulnerability_types: Option<Vec<String>>,
    pub severity_threshold: Option<String>,
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
    /// Code analyzer for complexity, performance, and security analysis
    code_analyzer: Arc<CodeAnalyzer>,
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

        // Initialize code analyzer
        let code_analyzer = Arc::new(CodeAnalyzer::new());

        Ok(Self {
            config,
            tool_router: Self::tool_router(),
            graph_store,
            graph_query,
            repository_scanner,
            content_search,
            repository_manager,
            repository_path: None,
            code_analyzer,
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

        let dep_type_str = params
            .dependency_type
            .unwrap_or_else(|| "direct".to_string());

        // Parse dependency type
        let dependency_type = match dep_type_str.as_str() {
            "direct" => DependencyType::Direct,
            "calls" => DependencyType::Calls,
            "imports" => DependencyType::Imports,
            "reads" => DependencyType::Reads,
            "writes" => DependencyType::Writes,
            _ => {
                let error_msg = format!("Invalid dependency type: {}. Must be one of: direct, calls, imports, reads, writes", dep_type_str);
                return Ok(CallToolResult::error(vec![Content::text(error_msg)]));
            }
        };

        // Parse the target node ID from hex string
        let node_id = match codeprism_core::NodeId::from_hex(&params.target) {
            Ok(id) => id,
            Err(_) => {
                let error_msg = format!(
                    "Invalid target symbol ID format: {}. Expected hexadecimal string.",
                    params.target
                );
                return Ok(CallToolResult::error(vec![Content::text(error_msg)]));
            }
        };

        // Find dependencies using graph query
        let dependencies_result = self
            .graph_query
            .find_dependencies(&node_id, dependency_type.clone());

        let result = match dependencies_result {
            Ok(dependencies) => {
                serde_json::json!({
                    "status": "success",
                    "target_symbol_id": params.target,
                    "dependency_type": dep_type_str,
                    "dependencies": dependencies.iter().map(|dependency| {
                        serde_json::json!({
                            "target_symbol": {
                                "id": dependency.target_node.id.to_hex(),
                                "name": dependency.target_node.name,
                                "kind": format!("{:?}", dependency.target_node.kind),
                                "language": format!("{:?}", dependency.target_node.lang),
                                "file": dependency.target_node.file.display().to_string(),
                                "span": {
                                    "start_byte": dependency.target_node.span.start_byte,
                                    "end_byte": dependency.target_node.span.end_byte,
                                    "start_line": dependency.target_node.span.start_line,
                                    "start_column": dependency.target_node.span.start_column,
                                    "end_line": dependency.target_node.span.end_line,
                                    "end_column": dependency.target_node.span.end_column,
                                }
                            },
                            "edge_type": format!("{:?}", dependency.edge_kind),
                            "dependency_classification": format!("{:?}", dependency.dependency_type),
                        })
                    }).collect::<Vec<_>>(),
                    "total_dependencies": dependencies.len(),
                    "query": {
                        "target": params.target,
                        "dependency_type": dep_type_str
                    }
                })
            }
            Err(e) => {
                serde_json::json!({
                    "status": "error",
                    "message": format!("Dependency finding failed: {}", e),
                    "query": {
                        "target": params.target,
                        "dependency_type": dep_type_str
                    }
                })
            }
        };

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

        // Get the symbol node
        let symbol_node = match self.graph_store.get_node(&node_id) {
            Some(node) => node,
            None => {
                let error_msg = format!("Symbol with ID {} not found in graph", params.symbol_id);
                return Ok(CallToolResult::error(vec![Content::text(error_msg)]));
            }
        };

        // Get basic symbol information
        let mut explanation = serde_json::json!({
            "status": "success",
            "symbol": {
                "id": symbol_node.id.to_hex(),
                "name": symbol_node.name,
                "kind": format!("{:?}", symbol_node.kind),
                "language": format!("{:?}", symbol_node.lang),
                "file": symbol_node.file.display().to_string(),
                "span": {
                    "start_byte": symbol_node.span.start_byte,
                    "end_byte": symbol_node.span.end_byte,
                    "start_line": symbol_node.span.start_line,
                    "start_column": symbol_node.span.start_column,
                    "end_line": symbol_node.span.end_line,
                    "end_column": symbol_node.span.end_column,
                }
            }
        });

        // Get inheritance information for classes
        if symbol_node.kind == NodeKind::Class {
            match self.graph_query.get_inheritance_info(&node_id) {
                Ok(inheritance_info) => {
                    explanation["inheritance"] = serde_json::json!({
                        "base_classes": inheritance_info.base_classes.iter().map(|base| {
                            serde_json::json!({
                                "name": base.class_name,
                                "relationship": base.relationship_type,
                                "file": base.file.display().to_string()
                            })
                        }).collect::<Vec<_>>(),
                        "subclasses": inheritance_info.subclasses.iter().map(|sub| {
                            serde_json::json!({
                                "name": sub.class_name,
                                "relationship": sub.relationship_type,
                                "file": sub.file.display().to_string()
                            })
                        }).collect::<Vec<_>>(),
                        "method_resolution_order": inheritance_info.method_resolution_order,
                        "is_metaclass": inheritance_info.is_metaclass
                    });
                }
                Err(_) => {
                    explanation["inheritance"] = serde_json::json!({
                        "note": "Inheritance information not available"
                    });
                }
            }
        }

        // Include dependencies if requested
        if include_deps {
            match self
                .graph_query
                .find_dependencies(&node_id, DependencyType::Direct)
            {
                Ok(dependencies) => {
                    explanation["dependencies"] = serde_json::json!({
                        "count": dependencies.len(),
                        "items": dependencies.iter().take(10).map(|dep| {
                            serde_json::json!({
                                "name": dep.target_node.name,
                                "kind": format!("{:?}", dep.target_node.kind),
                                "file": dep.target_node.file.display().to_string(),
                                "relationship": format!("{:?}", dep.edge_kind)
                            })
                        }).collect::<Vec<_>>(),
                        "truncated": dependencies.len() > 10
                    });
                }
                Err(_) => {
                    explanation["dependencies"] = serde_json::json!({
                        "note": "Dependencies information not available"
                    });
                }
            }
        }

        // Include usages/references if requested
        if include_uses {
            match self.graph_query.find_references(&node_id) {
                Ok(references) => {
                    explanation["usages"] = serde_json::json!({
                        "count": references.len(),
                        "items": references.iter().take(10).map(|reference| {
                            serde_json::json!({
                                "source_name": reference.source_node.name,
                                "source_kind": format!("{:?}", reference.source_node.kind),
                                "file": reference.source_node.file.display().to_string(),
                                "relationship": format!("{:?}", reference.edge_kind),
                                "location": {
                                    "line": reference.location.span.start_line,
                                    "column": reference.location.span.start_column
                                }
                            })
                        }).collect::<Vec<_>>(),
                        "truncated": references.len() > 10
                    });
                }
                Err(_) => {
                    explanation["usages"] = serde_json::json!({
                        "note": "Usage information not available"
                    });
                }
            }
        }

        // Add query information
        explanation["query"] = serde_json::json!({
            "symbol_id": params.symbol_id,
            "include_dependencies": include_deps,
            "include_usages": include_uses,
            "context_lines": context
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&explanation)
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
        let max_results = params.limit.unwrap_or(100) as usize;

        // Check if repository is configured
        let _repo_path = match &self.repository_path {
            Some(path) => path.clone(),
            None => {
                let error_msg = "No repository configured. Call initialize_repository first.";
                return Ok(CallToolResult::error(vec![Content::text(
                    error_msg.to_string(),
                )]));
            }
        };

        // Build search query
        let mut query_builder = SearchQueryBuilder::new(&params.query).max_results(max_results);

        if case_sens {
            query_builder = query_builder.case_sensitive();
        }

        if use_regex {
            query_builder = query_builder.use_regex();
        }

        // Add file type filters if provided
        if let Some(ref file_types) = params.file_types {
            let file_patterns = file_types.iter().map(|ext| format!("*.{}", ext)).collect();
            query_builder = query_builder.include_files(file_patterns);
        }

        let search_query = query_builder.build();

        // Perform search using content search manager
        let search_result = self.content_search.search(&search_query);

        let result = match search_result {
            Ok(search_results) => {
                serde_json::json!({
                    "status": "success",
                    "query_text": params.query,
                    "results": search_results.iter().map(|result| {
                        serde_json::json!({
                            "file": result.chunk.file_path.display().to_string(),
                            "content_type": format!("{:?}", result.chunk.content_type),
                            "relevance_score": result.score,
                            "matches": result.matches.iter().map(|match_item| {
                                serde_json::json!({
                                    "matched_text": match_item.text,
                                    "line_number": match_item.line_number,
                                    "column_number": match_item.column_number,
                                    "position": match_item.position,
                                    "context_before": match_item.context_before,
                                    "context_after": match_item.context_after
                                })
                            }).collect::<Vec<_>>(),
                            "chunk_content": if result.chunk.content.len() > 500 {
                                format!("{}...", &result.chunk.content[..500])
                            } else {
                                result.chunk.content.clone()
                            }
                        })
                    }).collect::<Vec<_>>(),
                    "total_results": search_results.len(),
                    "search_settings": {
                        "case_sensitive": case_sens,
                        "regex": use_regex,
                        "file_types": params.file_types,
                        "max_results": max_results
                    }
                })
            }
            Err(e) => {
                serde_json::json!({
                    "status": "error",
                    "message": format!("Content search failed: {}", e),
                    "query": {
                        "query": params.query,
                        "file_types": params.file_types,
                        "case_sensitive": case_sens,
                        "regex": use_regex,
                        "limit": max_results
                    }
                })
            }
        };

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
        let max_results = params.limit.unwrap_or(100) as usize;

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

        let result = match p_type.as_str() {
            "regex" => {
                // Use regex search for content patterns
                match self
                    .content_search
                    .regex_search(&params.pattern, Some(max_results))
                {
                    Ok(search_results) => {
                        let mut pattern_matches = Vec::new();

                        for search_result in search_results {
                            let file_path =
                                search_result.chunk.file_path.to_string_lossy().to_string();

                            // Check file type filters if specified
                            if let Some(ref file_types) = params.file_types {
                                let extension = search_result
                                    .chunk
                                    .file_path
                                    .extension()
                                    .and_then(|ext| ext.to_str())
                                    .unwrap_or("");

                                if !file_types.iter().any(|ft| {
                                    ft.trim_start_matches('*').trim_start_matches('.') == extension
                                        || ft == "*"
                                        || ft == "all"
                                }) {
                                    continue;
                                }
                            }

                            for search_match in search_result.matches {
                                pattern_matches.push(serde_json::json!({
                                    "file_path": file_path,
                                    "match_text": search_match.text,
                                    "line_number": search_match.line_number,
                                    "column_number": search_match.column_number,
                                    "position": search_match.position,
                                    "context_before": search_match.context_before,
                                    "context_after": search_match.context_after,
                                    "score": search_result.score
                                }));
                            }
                        }

                        serde_json::json!({
                            "status": "success",
                            "pattern_type": "regex",
                            "pattern": params.pattern,
                            "matches_found": pattern_matches.len(),
                            "matches": pattern_matches,
                            "file_types": params.file_types,
                            "limit": max_results
                        })
                    }
                    Err(e) => {
                        serde_json::json!({
                            "status": "error",
                            "message": format!("Regex pattern search failed: {}", e),
                            "pattern": params.pattern,
                            "pattern_type": "regex"
                        })
                    }
                }
            }
            "glob" => {
                // Use file pattern matching for glob patterns
                match self.content_search.find_files(&params.pattern) {
                    Ok(file_paths) => {
                        let mut filtered_files: Vec<_> =
                            file_paths.into_iter().take(max_results).collect();

                        // Apply file type filters if specified
                        if let Some(ref file_types) = params.file_types {
                            filtered_files.retain(|path| {
                                let extension =
                                    path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

                                file_types.iter().any(|ft| {
                                    ft.trim_start_matches('*').trim_start_matches('.') == extension
                                        || ft == "*"
                                        || ft == "all"
                                })
                            });
                        }

                        let file_matches: Vec<_> = filtered_files
                            .iter()
                            .map(|path| {
                                serde_json::json!({
                                    "file_path": path.to_string_lossy(),
                                    "file_name": path.file_name()
                                        .and_then(|name| name.to_str())
                                        .unwrap_or(""),
                                    "extension": path.extension()
                                        .and_then(|ext| ext.to_str())
                                        .unwrap_or(""),
                                    "directory": path.parent()
                                        .map(|p| p.to_string_lossy().to_string())
                                        .unwrap_or_else(|| ".".to_string())
                                })
                            })
                            .collect();

                        serde_json::json!({
                            "status": "success",
                            "pattern_type": "glob",
                            "pattern": params.pattern,
                            "files_found": file_matches.len(),
                            "files": file_matches,
                            "file_types": params.file_types,
                            "limit": max_results
                        })
                    }
                    Err(e) => {
                        serde_json::json!({
                            "status": "error",
                            "message": format!("Glob pattern search failed: {}", e),
                            "pattern": params.pattern,
                            "pattern_type": "glob"
                        })
                    }
                }
            }
            _ => unreachable!("Pattern type already validated"),
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
        )]))
    }

    /// Perform semantic search across codebase
    #[tool(description = "Perform semantic search to find conceptually related code")]
    fn semantic_search(
        &self,
        Parameters(params): Parameters<SemanticSearchParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!(
            "Semantic search tool called for concept: {}",
            params.concept
        );

        let max_results = params.limit.unwrap_or(20);
        let relevance_threshold = params.relevance_threshold.unwrap_or(0.3);
        let include_similar = params.include_similar.unwrap_or(true);

        // Build a comprehensive search using multiple strategies
        let mut semantic_results = Vec::new();
        let mut seen_files = std::collections::HashSet::new();

        // Strategy 1: Direct keyword search with variations
        let keywords = self.extract_semantic_keywords(&params.concept);

        for keyword in &keywords {
            // Search in content
            if let Ok(content_results) = self.content_search.search(
                &SearchQueryBuilder::new(keyword)
                    .max_results(max_results / keywords.len().max(1))
                    .build(),
            ) {
                for content_result in content_results {
                    let file_path = content_result.chunk.file_path.to_string_lossy().to_string();

                    if seen_files.contains(&file_path) {
                        continue;
                    }
                    seen_files.insert(file_path.clone());

                    // Calculate semantic relevance based on context match
                    let relevance = self.calculate_semantic_relevance(
                        &params.concept,
                        &content_result.chunk.content,
                        params.context.as_deref(),
                    );

                    if relevance >= relevance_threshold {
                        semantic_results.push(serde_json::json!({
                            "type": "content_match",
                            "file": file_path,
                            "relevance": relevance,
                            "keyword": keyword,
                            "matches": content_result.matches.iter().map(|m| {
                                serde_json::json!({
                                    "text": m.text,
                                    "line": m.line_number,
                                    "column": m.column_number,
                                    "context_before": m.context_before,
                                    "context_after": m.context_after
                                })
                            }).collect::<Vec<_>>(),
                            "chunk_type": content_result.chunk.content_type,
                            "score": content_result.score
                        }));
                    }
                }
            }

            // Search in symbols
            if let Ok(symbol_results) = self.graph_query.search_symbols(
                keyword,
                None,
                Some(max_results / keywords.len().max(1)),
            ) {
                for symbol_result in symbol_results {
                    let file_path = symbol_result.node.file.to_string_lossy().to_string();

                    // Calculate semantic relevance for symbols
                    let relevance = self.calculate_symbol_semantic_relevance(
                        &params.concept,
                        &symbol_result.node,
                        params.context.as_deref(),
                    );

                    if relevance >= relevance_threshold {
                        semantic_results.push(serde_json::json!({
                            "type": "symbol_match",
                            "file": file_path,
                            "relevance": relevance,
                            "keyword": keyword,
                            "symbol": {
                                "id": symbol_result.node.id.to_hex(),
                                "name": symbol_result.node.name,
                                "kind": format!("{:?}", symbol_result.node.kind).to_lowercase(),
                                "line": symbol_result.node.span.start_line,
                                "column": symbol_result.node.span.start_column,
                                "metadata": symbol_result.node.metadata
                            },
                            "references_count": symbol_result.references_count,
                            "dependencies_count": symbol_result.dependencies_count
                        }));
                    }
                }
            }
        }

        // Strategy 2: Look for related symbols by naming patterns (if include_similar is true)
        if include_similar {
            let concept_variations = self.generate_concept_variations(&params.concept);

            for variation in concept_variations {
                if let Ok(similar_symbols) =
                    self.graph_query.search_symbols(&variation, None, Some(5))
                {
                    for symbol_result in similar_symbols {
                        let file_path = symbol_result.node.file.to_string_lossy().to_string();

                        if !seen_files.contains(&file_path) {
                            let relevance = self.calculate_symbol_semantic_relevance(
                                &params.concept,
                                &symbol_result.node,
                                params.context.as_deref(),
                            ) * 0.8; // Slightly lower relevance for variations

                            if relevance >= relevance_threshold {
                                semantic_results.push(serde_json::json!({
                                    "type": "similar_symbol",
                                    "file": file_path,
                                    "relevance": relevance,
                                    "variation": variation,
                                    "symbol": {
                                        "id": symbol_result.node.id.to_hex(),
                                        "name": symbol_result.node.name,
                                        "kind": format!("{:?}", symbol_result.node.kind).to_lowercase(),
                                        "line": symbol_result.node.span.start_line,
                                        "column": symbol_result.node.span.start_column
                                    }
                                }));
                            }
                        }
                    }
                }
            }
        }

        // Sort by relevance and limit results
        semantic_results.sort_by(|a, b| {
            let relevance_a = a["relevance"].as_f64().unwrap_or(0.0);
            let relevance_b = b["relevance"].as_f64().unwrap_or(0.0);
            relevance_b
                .partial_cmp(&relevance_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        semantic_results.truncate(max_results);

        let result = serde_json::json!({
            "status": "success",
            "concept": params.concept,
            "context": params.context,
            "results_found": semantic_results.len(),
            "results": semantic_results,
            "search_strategy": {
                "keywords_used": keywords,
                "relevance_threshold": relevance_threshold,
                "include_similar": include_similar,
                "max_results": max_results
            },
            "notes": [
                "Semantic search combines keyword matching with contextual analysis",
                "Relevance scores are calculated based on concept match and context",
                "Similar symbol variations are included when include_similar=true"
            ]
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
        )]))
    }

    /// Advanced search with multiple criteria
    #[tool(description = "Advanced search combining multiple search criteria and filters")]
    fn advanced_search(
        &self,
        Parameters(params): Parameters<AdvancedSearchParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!("Advanced search tool called with query: {}", params.query);

        let max_results = params.limit.unwrap_or(50);
        let include_tests = params.include_tests.unwrap_or(false);
        let include_dependencies = params.include_dependencies.unwrap_or(false);

        // Build comprehensive search strategy
        let mut search_results = Vec::new();
        let mut processed_files = std::collections::HashSet::new();

        // Strategy 1: Content-based search with file type filtering
        let mut content_query_builder =
            SearchQueryBuilder::new(&params.query).max_results(max_results);

        // Apply file type filters
        if let Some(ref file_types) = params.file_types {
            let file_patterns: Vec<String> = file_types
                .iter()
                .map(|ext| format!("*.{}", ext.trim_start_matches('*').trim_start_matches('.')))
                .collect();
            content_query_builder = content_query_builder.include_files(file_patterns);
        }

        // Apply exclude patterns
        if let Some(ref exclude_patterns) = params.exclude_patterns {
            content_query_builder = content_query_builder.exclude_files(exclude_patterns.clone());
        }

        // Perform content search
        if let Ok(content_results) = self.content_search.search(&content_query_builder.build()) {
            for content_result in content_results {
                let file_path = content_result.chunk.file_path.to_string_lossy().to_string();

                // Filter out test files if not included
                if !include_tests && self.is_test_file(&file_path) {
                    continue;
                }

                // Filter out dependency files if not included
                if !include_dependencies && self.is_dependency_file(&file_path) {
                    continue;
                }

                // Apply size range filter if specified
                if let Some(ref size_range) = params.size_range {
                    if let Ok(metadata) = std::fs::metadata(&content_result.chunk.file_path) {
                        if !self.matches_size_range(metadata.len(), size_range) {
                            continue;
                        }
                    }
                }

                processed_files.insert(file_path.clone());

                search_results.push(serde_json::json!({
                    "type": "content_match",
                    "file": file_path,
                    "score": content_result.score,
                    "matches": content_result.matches.iter().map(|m| {
                        serde_json::json!({
                            "text": m.text,
                            "line": m.line_number,
                            "column": m.column_number,
                            "context_before": m.context_before,
                            "context_after": m.context_after
                        })
                    }).collect::<Vec<_>>(),
                    "content_type": content_result.chunk.content_type,
                    "file_size": std::fs::metadata(&content_result.chunk.file_path)
                        .map(|m| m.len())
                        .unwrap_or(0)
                }));
            }
        }

        // Strategy 2: Symbol-based search
        let symbol_types = if let Some(ref types) = params.symbol_types {
            let mut node_kinds = Vec::new();
            for sym_type in types {
                match sym_type.as_str() {
                    "function" | "functions" => node_kinds.push(NodeKind::Function),
                    "class" | "classes" => node_kinds.push(NodeKind::Class),
                    "method" | "methods" => node_kinds.push(NodeKind::Method),
                    "variable" | "variables" => node_kinds.push(NodeKind::Variable),
                    "module" | "modules" => node_kinds.push(NodeKind::Module),
                    _ => {}
                }
            }
            Some(node_kinds)
        } else {
            None
        };

        if let Ok(symbol_results) =
            self.graph_query
                .search_symbols(&params.query, symbol_types, Some(max_results))
        {
            for symbol_result in symbol_results {
                let file_path = symbol_result.node.file.to_string_lossy().to_string();

                // Skip if already processed or filtered
                if processed_files.contains(&file_path) {
                    continue;
                }

                // Apply same filters as content search
                if !include_tests && self.is_test_file(&file_path) {
                    continue;
                }

                if !include_dependencies && self.is_dependency_file(&file_path) {
                    continue;
                }

                // Apply complexity filter if specified
                if let Some(ref complexity_filter) = params.complexity_filter {
                    let complexity_score = self.estimate_symbol_complexity(&symbol_result.node);
                    if !self.matches_complexity_filter(complexity_score, complexity_filter) {
                        continue;
                    }
                }

                search_results.push(serde_json::json!({
                    "type": "symbol_match",
                    "file": file_path,
                    "symbol": {
                        "id": symbol_result.node.id.to_hex(),
                        "name": symbol_result.node.name,
                        "kind": format!("{:?}", symbol_result.node.kind).to_lowercase(),
                        "line": symbol_result.node.span.start_line,
                        "column": symbol_result.node.span.start_column,
                        "metadata": symbol_result.node.metadata
                    },
                    "references_count": symbol_result.references_count,
                    "dependencies_count": symbol_result.dependencies_count,
                    "complexity_estimate": self.estimate_symbol_complexity(&symbol_result.node)
                }));
            }
        }

        // Apply date range filter if specified (file modification time)
        if let Some(ref date_range) = params.date_range {
            search_results.retain(|result| {
                if let Some(file_path) = result["file"].as_str() {
                    if let Ok(metadata) = std::fs::metadata(file_path) {
                        if let Ok(modified) = metadata.modified() {
                            return self.matches_date_range(modified, date_range);
                        }
                    }
                }
                false
            });
        }

        // Sort results by relevance/score
        search_results.sort_by(|a, b| {
            let score_a = a["score"].as_f64().unwrap_or(0.0);
            let score_b = b["score"].as_f64().unwrap_or(0.0);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit final results
        search_results.truncate(max_results);

        let result = serde_json::json!({
            "status": "success",
            "query": params.query,
            "results_found": search_results.len(),
            "results": search_results,
            "filters_applied": {
                "file_types": params.file_types,
                "symbol_types": params.symbol_types,
                "date_range": params.date_range,
                "size_range": params.size_range,
                "complexity_filter": params.complexity_filter,
                "exclude_patterns": params.exclude_patterns,
                "include_tests": include_tests,
                "include_dependencies": include_dependencies,
                "limit": max_results
            },
            "search_strategy": [
                "Content-based search with text matching",
                "Symbol-based search with type filtering",
                "File metadata filtering (size, date, type)",
                "Complexity analysis integration",
                "Test and dependency file filtering"
            ]
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
        )]))
    }

    // Analysis Tools (Updated implementations)

    /// Analyze code complexity metrics
    #[tool(
        description = "Analyze code complexity including cyclomatic complexity and maintainability"
    )]
    fn analyze_complexity(
        &self,
        Parameters(params): Parameters<AnalyzeComplexityParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!(
            "Analyze complexity tool called for target: {}",
            params.target
        );

        let metrics = params.metrics.unwrap_or_else(|| vec!["all".to_string()]);
        let threshold_warnings = params.threshold_warnings.unwrap_or(true);

        // Check if target is a file path
        let result = if std::path::Path::new(&params.target).exists() {
            // Analyze file directly
            match self.code_analyzer.complexity.analyze_file_complexity(
                std::path::Path::new(&params.target),
                &metrics,
                threshold_warnings,
            ) {
                Ok(analysis) => {
                    serde_json::json!({
                        "status": "success",
                        "target_type": "file",
                        "target": params.target,
                        "analysis": analysis,
                        "settings": {
                            "metrics": metrics,
                            "threshold_warnings": threshold_warnings
                        }
                    })
                }
                Err(e) => {
                    serde_json::json!({
                        "status": "error",
                        "message": format!("Failed to analyze file complexity: {}", e),
                        "target": params.target
                    })
                }
            }
        } else if params.target.starts_with("**") || params.target.contains("*") {
            // Handle glob pattern
            match &self.repository_path {
                Some(repo_path) => {
                    let pattern = if params.target.starts_with("**/") {
                        // Convert **/*.ext to repo_path/**/*.ext
                        repo_path.join(&params.target[3..]).display().to_string()
                    } else {
                        repo_path.join(&params.target).display().to_string()
                    };

                    // Find matching files using glob
                    let mut all_results = Vec::new();
                    if let Ok(paths) = glob::glob(&pattern) {
                        for path in paths.flatten() {
                            if let Ok(analysis) = self
                                .code_analyzer
                                .complexity
                                .analyze_file_complexity(&path, &metrics, threshold_warnings)
                            {
                                all_results.push(analysis);
                            }
                        }
                    }

                    if all_results.is_empty() {
                        serde_json::json!({
                            "status": "success",
                            "target_type": "pattern",
                            "target": params.target,
                            "message": "No files found matching pattern",
                            "files_analyzed": 0
                        })
                    } else {
                        serde_json::json!({
                            "status": "success",
                            "target_type": "pattern",
                            "target": params.target,
                            "files_analyzed": all_results.len(),
                            "results": all_results,
                            "settings": {
                                "metrics": metrics,
                                "threshold_warnings": threshold_warnings
                            }
                        })
                    }
                }
                None => {
                    serde_json::json!({
                        "status": "error",
                        "message": "No repository configured. Call initialize_repository first.",
                        "target": params.target
                    })
                }
            }
        } else {
            // Check if target might be a symbol ID, but for now return an error
            serde_json::json!({
                "status": "error",
                "message": format!("Target '{}' not found. Provide a valid file path or glob pattern.", params.target),
                "target": params.target,
                "hint": "Use a file path like 'src/main.rs' or a pattern like '**/*.rs'"
            })
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
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
    fn analyze_performance(
        &self,
        Parameters(params): Parameters<AnalyzePerformanceParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!(
            "Analyze performance tool called for target: {}",
            params.target
        );

        let analysis_types = params
            .analysis_types
            .unwrap_or_else(|| vec!["all".to_string()]);
        let complexity_threshold = params
            .complexity_threshold
            .unwrap_or_else(|| "medium".to_string());

        // Check if target is a file path
        let result = if std::path::Path::new(&params.target).exists() {
            // Analyze file directly
            let file_content = match std::fs::read_to_string(&params.target) {
                Ok(content) => content,
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to read file '{}': {}",
                        params.target, e
                    ))]));
                }
            };

            match self.code_analyzer.performance.analyze_content(
                &file_content,
                &analysis_types,
                &complexity_threshold,
            ) {
                Ok(issues) => {
                    let recommendations = self
                        .code_analyzer
                        .performance
                        .get_performance_recommendations(&issues);

                    serde_json::json!({
                        "status": "success",
                        "target_type": "file",
                        "target": params.target,
                        "performance_analysis": {
                            "issues_found": issues.len(),
                            "issues": issues.iter().map(|issue| {
                                serde_json::json!({
                                    "type": issue.issue_type,
                                    "severity": issue.severity,
                                    "description": issue.description,
                                    "location": issue.location,
                                    "recommendation": issue.recommendation,
                                    "complexity_estimate": issue.complexity_estimate,
                                    "impact_score": issue.impact_score,
                                    "optimization_effort": issue.optimization_effort
                                })
                            }).collect::<Vec<_>>(),
                            "recommendations": recommendations,
                            "overall_grade": self.calculate_performance_grade(&issues)
                        },
                        "settings": {
                            "analysis_types": analysis_types,
                            "complexity_threshold": complexity_threshold
                        }
                    })
                }
                Err(e) => {
                    serde_json::json!({
                        "status": "error",
                        "message": format!("Failed to analyze performance: {}", e),
                        "target": params.target
                    })
                }
            }
        } else if params.target.starts_with("**") || params.target.contains("*") {
            // Handle glob pattern
            match &self.repository_path {
                Some(repo_path) => {
                    let pattern = if params.target.starts_with("**/") {
                        repo_path.join(&params.target[3..]).display().to_string()
                    } else {
                        repo_path.join(&params.target).display().to_string()
                    };

                    let mut all_issues = Vec::new();
                    let mut files_analyzed = 0;

                    if let Ok(paths) = glob::glob(&pattern) {
                        for path in paths.flatten() {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok(issues) = self.code_analyzer.performance.analyze_content(
                                    &content,
                                    &analysis_types,
                                    &complexity_threshold,
                                ) {
                                    all_issues.extend(issues);
                                    files_analyzed += 1;
                                }
                            }
                        }
                    }

                    let recommendations = self
                        .code_analyzer
                        .performance
                        .get_performance_recommendations(&all_issues);

                    serde_json::json!({
                        "status": "success",
                        "target_type": "pattern",
                        "target": params.target,
                        "files_analyzed": files_analyzed,
                        "performance_analysis": {
                            "total_issues": all_issues.len(),
                            "issues": all_issues.iter().map(|issue| {
                                serde_json::json!({
                                    "type": issue.issue_type,
                                    "severity": issue.severity,
                                    "description": issue.description,
                                    "location": issue.location,
                                    "recommendation": issue.recommendation,
                                    "complexity_estimate": issue.complexity_estimate,
                                    "impact_score": issue.impact_score,
                                    "optimization_effort": issue.optimization_effort
                                })
                            }).collect::<Vec<_>>(),
                            "recommendations": recommendations,
                            "overall_grade": self.calculate_performance_grade(&all_issues)
                        },
                        "settings": {
                            "analysis_types": analysis_types,
                            "complexity_threshold": complexity_threshold
                        }
                    })
                }
                None => {
                    serde_json::json!({
                        "status": "error",
                        "message": "No repository configured. Call initialize_repository first.",
                        "target": params.target
                    })
                }
            }
        } else {
            serde_json::json!({
                "status": "error",
                "message": format!("Target '{}' not found. Provide a valid file path or glob pattern.", params.target),
                "target": params.target,
                "hint": "Use a file path like 'src/main.rs' or a pattern like '**/*.rs'"
            })
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
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

    /// Analyze security vulnerabilities
    #[tool(description = "Analyze security vulnerabilities and potential threats")]
    fn analyze_security(
        &self,
        Parameters(params): Parameters<AnalyzeSecurityParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!("Analyze security tool called for target: {}", params.target);

        let vulnerability_types = params
            .vulnerability_types
            .unwrap_or_else(|| vec!["all".to_string()]);
        let severity_threshold = params
            .severity_threshold
            .unwrap_or_else(|| "low".to_string());

        // Check if target is a file path
        let result = if std::path::Path::new(&params.target).exists() {
            // Analyze file directly
            let file_content = match std::fs::read_to_string(&params.target) {
                Ok(content) => content,
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to read file '{}': {}",
                        params.target, e
                    ))]));
                }
            };

            match self.code_analyzer.security.analyze_content_with_location(
                &file_content,
                Some(&params.target),
                &vulnerability_types,
                &severity_threshold,
            ) {
                Ok(vulnerabilities) => {
                    let recommendations = self
                        .code_analyzer
                        .security
                        .get_security_recommendations(&vulnerabilities);

                    let security_report = self
                        .code_analyzer
                        .security
                        .generate_security_report(&vulnerabilities);

                    serde_json::json!({
                        "status": "success",
                        "target_type": "file",
                        "target": params.target,
                        "security_analysis": {
                            "vulnerabilities_found": vulnerabilities.len(),
                            "vulnerabilities": vulnerabilities.iter().map(|vuln| {
                                serde_json::json!({
                                    "type": vuln.vulnerability_type,
                                    "severity": vuln.severity,
                                    "description": vuln.description,
                                    "location": vuln.location,
                                    "recommendation": vuln.recommendation,
                                    "cvss_score": vuln.cvss_score,
                                    "owasp_category": vuln.owasp_category,
                                    "confidence": vuln.confidence,
                                    "line_number": vuln.line_number
                                })
                            }).collect::<Vec<_>>(),
                            "recommendations": recommendations,
                            "security_report": security_report
                        },
                        "settings": {
                            "vulnerability_types": vulnerability_types,
                            "severity_threshold": severity_threshold
                        }
                    })
                }
                Err(e) => {
                    serde_json::json!({
                        "status": "error",
                        "message": format!("Failed to analyze security: {}", e),
                        "target": params.target
                    })
                }
            }
        } else if params.target.starts_with("**") || params.target.contains("*") {
            // Handle glob pattern
            match &self.repository_path {
                Some(repo_path) => {
                    let pattern = if params.target.starts_with("**/") {
                        repo_path.join(&params.target[3..]).display().to_string()
                    } else {
                        repo_path.join(&params.target).display().to_string()
                    };

                    let mut all_vulnerabilities = Vec::new();
                    let mut files_analyzed = 0;

                    if let Ok(paths) = glob::glob(&pattern) {
                        for path in paths.flatten() {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok(vulnerabilities) =
                                    self.code_analyzer.security.analyze_content_with_location(
                                        &content,
                                        Some(&path.display().to_string()),
                                        &vulnerability_types,
                                        &severity_threshold,
                                    )
                                {
                                    all_vulnerabilities.extend(vulnerabilities);
                                    files_analyzed += 1;
                                }
                            }
                        }
                    }

                    let recommendations = self
                        .code_analyzer
                        .security
                        .get_security_recommendations(&all_vulnerabilities);

                    let security_report = self
                        .code_analyzer
                        .security
                        .generate_security_report(&all_vulnerabilities);

                    serde_json::json!({
                        "status": "success",
                        "target_type": "pattern",
                        "target": params.target,
                        "files_analyzed": files_analyzed,
                        "security_analysis": {
                            "total_vulnerabilities": all_vulnerabilities.len(),
                            "vulnerabilities": all_vulnerabilities.iter().map(|vuln| {
                                serde_json::json!({
                                    "type": vuln.vulnerability_type,
                                    "severity": vuln.severity,
                                    "description": vuln.description,
                                    "location": vuln.location,
                                    "recommendation": vuln.recommendation,
                                    "cvss_score": vuln.cvss_score,
                                    "owasp_category": vuln.owasp_category,
                                    "confidence": vuln.confidence,
                                    "file_path": vuln.file_path,
                                    "line_number": vuln.line_number
                                })
                            }).collect::<Vec<_>>(),
                            "recommendations": recommendations,
                            "security_report": security_report
                        },
                        "settings": {
                            "vulnerability_types": vulnerability_types,
                            "severity_threshold": severity_threshold
                        }
                    })
                }
                None => {
                    serde_json::json!({
                        "status": "error",
                        "message": "No repository configured. Call initialize_repository first.",
                        "target": params.target
                    })
                }
            }
        } else {
            serde_json::json!({
                "status": "error",
                "message": format!("Target '{}' not found. Provide a valid file path or glob pattern.", params.target),
                "target": params.target,
                "hint": "Use a file path like 'src/main.rs' or a pattern like '**/*.rs'"
            })
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
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

    /// Calculate performance grade based on issues found
    fn calculate_performance_grade(
        &self,
        issues: &[codeprism_analysis::performance::PerformanceIssue],
    ) -> String {
        if issues.is_empty() {
            return "A".to_string();
        }

        let critical_count = issues.iter().filter(|i| i.severity == "critical").count();
        let high_count = issues.iter().filter(|i| i.severity == "high").count();
        let medium_count = issues.iter().filter(|i| i.severity == "medium").count();

        match (critical_count, high_count, medium_count) {
            (0, 0, 0..=2) => "A",
            (0, 0, 3..=5) => "B",
            (0, 0, _) => "C", // 6 or more medium issues
            (0, 1..=2, _) => "C",
            (0, _, _) => "D", // 3 or more high issues
            (_, _, _) => "F", // any critical issues
        }
        .to_string()
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

    /// Extract semantic keywords from a concept for search
    fn extract_semantic_keywords(&self, concept: &str) -> Vec<String> {
        let mut keywords = Vec::new();

        // Add the original concept
        keywords.push(concept.to_lowercase());

        // Split by common separators and add individual words
        let words: Vec<&str> = concept
            .split(&[' ', '_', '-', '.', '/', '\\'][..])
            .filter(|w| !w.is_empty() && w.len() > 2)
            .collect();

        for word in words {
            keywords.push(word.to_lowercase());
        }

        // Add programming-related variations
        let programming_keywords = [
            "function",
            "method",
            "class",
            "interface",
            "service",
            "manager",
            "handler",
            "controller",
            "repository",
            "model",
            "view",
            "component",
            "module",
            "package",
            "config",
            "configuration",
            "settings",
            "utils",
            "utilities",
            "helpers",
        ];

        for keyword in &programming_keywords {
            if concept.to_lowercase().contains(keyword) {
                keywords.push(keyword.to_string());
            }
        }

        // Remove duplicates and return
        keywords.sort();
        keywords.dedup();
        keywords
    }

    /// Calculate semantic relevance between concept and content
    fn calculate_semantic_relevance(
        &self,
        concept: &str,
        content: &str,
        context: Option<&str>,
    ) -> f32 {
        let mut relevance = 0.0;
        let concept_lower = concept.to_lowercase();
        let content_lower = content.to_lowercase();

        // Direct match gets highest score
        if content_lower.contains(&concept_lower) {
            relevance += 0.8;
        }

        // Word-by-word matching
        let concept_words: Vec<&str> = concept_lower.split_whitespace().collect();
        let mut matched_words = 0;

        for word in &concept_words {
            if content_lower.contains(word) {
                matched_words += 1;
            }
        }

        if !concept_words.is_empty() {
            relevance += (matched_words as f32 / concept_words.len() as f32) * 0.5;
        }

        // Context matching bonus
        if let Some(ctx) = context {
            let context_lower = ctx.to_lowercase();
            if content_lower.contains(&context_lower) {
                relevance += 0.3;
            }
        }

        // Length and complexity factors
        if content.len() > 50 && content.len() < 500 {
            relevance += 0.1; // Prefer moderate-length content
        }

        relevance.min(1.0f32)
    }

    /// Calculate semantic relevance for symbols
    fn calculate_symbol_semantic_relevance(
        &self,
        concept: &str,
        node: &codeprism_core::Node,
        context: Option<&str>,
    ) -> f32 {
        let mut relevance: f32 = 0.0;
        let concept_lower = concept.to_lowercase();
        let name_lower = node.name.to_lowercase();

        // Direct name match
        if name_lower.contains(&concept_lower) {
            relevance += 0.7;
        }

        // Symbol type relevance
        match node.kind {
            NodeKind::Function | NodeKind::Method => {
                if concept_lower.contains("function") || concept_lower.contains("method") {
                    relevance += 0.3;
                }
            }
            NodeKind::Class => {
                if concept_lower.contains("class") || concept_lower.contains("type") {
                    relevance += 0.3;
                }
            }
            NodeKind::Variable => {
                if concept_lower.contains("variable") || concept_lower.contains("data") {
                    relevance += 0.2;
                }
            }
            _ => {}
        }

        // Context matching
        if let Some(ctx) = context {
            let context_lower = ctx.to_lowercase();
            if name_lower.contains(&context_lower) {
                relevance += 0.2;
            }

            // Check file path context
            let file_path = node.file.to_string_lossy().to_lowercase();
            if file_path.contains(&context_lower) {
                relevance += 0.2;
            }
        }

        // Metadata matching
        if let serde_json::Value::Object(metadata_obj) = &node.metadata {
            for (key, value) in metadata_obj {
                let metadata_text = format!("{}: {}", key, value).to_lowercase();
                if metadata_text.contains(&concept_lower) {
                    relevance += 0.1;
                }
            }
        }

        relevance.min(1.0f32)
    }

    /// Generate concept variations for broader search
    fn generate_concept_variations(&self, concept: &str) -> Vec<String> {
        let mut variations = Vec::new();
        let concept_lower = concept.to_lowercase();

        // Add camelCase and snake_case variations
        let words: Vec<&str> = concept.split_whitespace().collect();
        if words.len() > 1 {
            // camelCase
            let mut camel_case = words[0].to_lowercase();
            for word in &words[1..] {
                let mut chars = word.chars();
                if let Some(first) = chars.next() {
                    camel_case.push(first.to_uppercase().next().unwrap());
                    camel_case.push_str(chars.as_str().to_lowercase().as_str());
                }
            }
            variations.push(camel_case);

            // snake_case
            variations.push(words.join("_").to_lowercase());

            // PascalCase
            let pascal_case = words
                .iter()
                .map(|word| {
                    let mut chars = word.chars();
                    if let Some(first) = chars.next() {
                        first
                            .to_uppercase()
                            .chain(chars.as_str().to_lowercase().chars())
                            .collect()
                    } else {
                        String::new()
                    }
                })
                .collect::<Vec<String>>()
                .join("");
            variations.push(pascal_case);
        }

        // Add common programming suffixes/prefixes
        let suffixes = [
            "er",
            "or",
            "ed",
            "ing",
            "s",
            "es",
            "Manager",
            "Service",
            "Handler",
            "Controller",
        ];
        let prefixes = ["get", "set", "is", "has", "can", "should", "will"];

        for suffix in &suffixes {
            variations.push(format!("{}{}", concept_lower, suffix.to_lowercase()));
        }

        for prefix in &prefixes {
            variations.push(format!("{}{}", prefix, concept));
        }

        // Add regex patterns for flexible matching
        if concept_lower.len() > 3 {
            variations.push(format!(".*{}.*", concept_lower));
        }

        variations
    }

    /// Check if a file is a test file based on path patterns
    fn is_test_file(&self, file_path: &str) -> bool {
        let path_lower = file_path.to_lowercase();
        path_lower.contains("/test/")
            || path_lower.contains("/tests/")
            || path_lower.contains("\\test\\")
            || path_lower.contains("\\tests\\")
            || path_lower.ends_with("_test.rs")
            || path_lower.ends_with("_test.py")
            || path_lower.ends_with("_test.js")
            || path_lower.ends_with("_test.ts")
            || path_lower.ends_with(".test.js")
            || path_lower.ends_with(".test.ts")
            || path_lower.ends_with(".spec.js")
            || path_lower.ends_with(".spec.ts")
            || path_lower.contains("test_")
            || path_lower.contains("spec_")
    }

    /// Check if a file is a dependency file (node_modules, vendor, etc.)
    fn is_dependency_file(&self, file_path: &str) -> bool {
        let path_lower = file_path.to_lowercase();
        path_lower.contains("/node_modules/")
            || path_lower.contains("\\node_modules\\")
            || path_lower.contains("/vendor/")
            || path_lower.contains("\\vendor\\")
            || path_lower.contains("/target/")
            || path_lower.contains("\\target\\")
            || path_lower.contains("/.cargo/")
            || path_lower.contains("\\.cargo\\")
            || path_lower.contains("/build/")
            || path_lower.contains("\\build\\")
            || path_lower.contains("/dist/")
            || path_lower.contains("\\dist\\")
            || path_lower.contains("/coverage/")
            || path_lower.contains("\\coverage\\")
    }

    /// Check if file size matches the specified range
    fn matches_size_range(&self, file_size: u64, size_range: &str) -> bool {
        match size_range {
            "small" => file_size < 10_000,                        // < 10KB
            "medium" => (10_000..100_000).contains(&file_size),   // 10KB - 100KB
            "large" => (100_000..1_000_000).contains(&file_size), // 100KB - 1MB
            "very_large" => file_size >= 1_000_000,               // > 1MB
            range if range.contains("kb") => {
                if let Ok(kb) = range.trim_end_matches("kb").parse::<u64>() {
                    file_size <= kb * 1_024
                } else {
                    true
                }
            }
            range if range.contains("mb") => {
                if let Ok(mb) = range.trim_end_matches("mb").parse::<u64>() {
                    file_size <= mb * 1_024 * 1_024
                } else {
                    true
                }
            }
            range if range.contains("-") => {
                // Handle range like "1kb-100kb"
                let parts: Vec<&str> = range.split('-').collect();
                if parts.len() == 2 {
                    let min_size = Self::parse_size(parts[0]).unwrap_or(0);
                    let max_size = Self::parse_size(parts[1]).unwrap_or(u64::MAX);
                    file_size >= min_size && file_size <= max_size
                } else {
                    true
                }
            }
            _ => true, // If range format is not recognized, include all files
        }
    }

    /// Parse size string to bytes
    fn parse_size(size_str: &str) -> Option<u64> {
        let size_str = size_str.trim().to_lowercase();
        if size_str.ends_with("kb") {
            size_str
                .trim_end_matches("kb")
                .parse::<u64>()
                .ok()
                .map(|s| s * 1_024)
        } else if size_str.ends_with("mb") {
            size_str
                .trim_end_matches("mb")
                .parse::<u64>()
                .ok()
                .map(|s| s * 1_024 * 1_024)
        } else if size_str.ends_with("gb") {
            size_str
                .trim_end_matches("gb")
                .parse::<u64>()
                .ok()
                .map(|s| s * 1_024 * 1_024 * 1_024)
        } else {
            size_str.parse::<u64>().ok()
        }
    }

    /// Estimate complexity of a symbol based on its properties
    fn estimate_symbol_complexity(&self, node: &codeprism_core::Node) -> u32 {
        let mut complexity = 1; // Base complexity

        // Factor in symbol type
        complexity += match node.kind {
            NodeKind::Function | NodeKind::Method => 3,
            NodeKind::Class => 5,
            NodeKind::Module => 2,
            NodeKind::Variable => 1,
            _ => 0,
        };

        // Factor in span size (rough measure of code length)
        let span_lines = node.span.end_line.saturating_sub(node.span.start_line);
        complexity += match span_lines {
            0..=10 => 1,
            11..=50 => 3,
            51..=100 => 5,
            101..=200 => 8,
            _ => 10,
        };

        // Factor in metadata complexity indicators
        if let serde_json::Value::Object(metadata) = &node.metadata {
            // Check for complexity indicators in metadata
            for (key, value) in metadata {
                if key.contains("complexity") || key.contains("cyclomatic") {
                    if let Some(complex_value) = value.as_u64() {
                        complexity += complex_value as u32;
                    }
                }
            }
        }

        complexity
    }

    /// Check if complexity matches the specified filter
    fn matches_complexity_filter(&self, complexity_score: u32, complexity_filter: &str) -> bool {
        match complexity_filter {
            "low" => complexity_score <= 5,
            "medium" => complexity_score > 5 && complexity_score <= 15,
            "high" => complexity_score > 15 && complexity_score <= 30,
            "very_high" => complexity_score > 30,
            filter if filter.starts_with("<=") => {
                if let Ok(threshold) = filter[2..].parse::<u32>() {
                    complexity_score <= threshold
                } else {
                    true
                }
            }
            filter if filter.starts_with(">=") => {
                if let Ok(threshold) = filter[2..].parse::<u32>() {
                    complexity_score >= threshold
                } else {
                    true
                }
            }
            filter if filter.starts_with('<') => {
                if let Ok(threshold) = filter[1..].parse::<u32>() {
                    complexity_score < threshold
                } else {
                    true
                }
            }
            filter if filter.starts_with('>') => {
                if let Ok(threshold) = filter[1..].parse::<u32>() {
                    complexity_score > threshold
                } else {
                    true
                }
            }
            _ => true, // Unknown filter, include all
        }
    }

    /// Check if file modification time matches the specified date range
    fn matches_date_range(&self, modified_time: std::time::SystemTime, date_range: &str) -> bool {
        let now = std::time::SystemTime::now();

        match date_range {
            "today" => {
                if let Ok(duration) = now.duration_since(modified_time) {
                    duration.as_secs() < 24 * 60 * 60 // Less than 1 day
                } else {
                    false
                }
            }
            "week" | "last_week" => {
                if let Ok(duration) = now.duration_since(modified_time) {
                    duration.as_secs() < 7 * 24 * 60 * 60 // Less than 7 days
                } else {
                    false
                }
            }
            "month" | "last_month" => {
                if let Ok(duration) = now.duration_since(modified_time) {
                    duration.as_secs() < 30 * 24 * 60 * 60 // Less than 30 days
                } else {
                    false
                }
            }
            "year" | "last_year" => {
                if let Ok(duration) = now.duration_since(modified_time) {
                    duration.as_secs() < 365 * 24 * 60 * 60 // Less than 365 days
                } else {
                    false
                }
            }
            range if range.ends_with("d") => {
                if let Ok(days) = range.trim_end_matches('d').parse::<u64>() {
                    if let Ok(duration) = now.duration_since(modified_time) {
                        duration.as_secs() < days * 24 * 60 * 60
                    } else {
                        false
                    }
                } else {
                    true
                }
            }
            range if range.ends_with("h") => {
                if let Ok(hours) = range.trim_end_matches('h').parse::<u64>() {
                    if let Ok(duration) = now.duration_since(modified_time) {
                        duration.as_secs() < hours * 60 * 60
                    } else {
                        false
                    }
                } else {
                    true
                }
            }
            _ => true, // Unknown format, include all
        }
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
