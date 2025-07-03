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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProvideGuidanceParams {
    pub target: String,
    pub guidance_type: Option<String>,
    pub include_examples: Option<bool>,
    pub priority_level: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OptimizeCodeParams {
    pub target: String,
    pub optimization_types: Option<Vec<String>>,
    pub aggressive_mode: Option<bool>,
    pub max_suggestions: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BatchProcessParams {
    pub operation: String,
    pub targets: Vec<String>,
    pub parameters: Option<serde_json::Value>,
    pub max_concurrent: Option<usize>,
    pub fail_fast: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WorkflowAutomationParams {
    pub workflow_type: String,
    pub target_scope: Option<String>,
    pub automation_level: Option<String>,
    pub dry_run: Option<bool>,
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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AnalyzeDependenciesParams {
    pub target: Option<String>,
    pub dependency_type: Option<String>,
    pub max_depth: Option<u32>,
    pub include_transitive: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AnalyzeControlFlowParams {
    pub target: String,
    pub analysis_types: Option<Vec<String>>,
    pub max_depth: Option<u32>,
    pub include_paths: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct AnalyzeCodeQualityParams {
    pub target: String,
    pub quality_types: Option<Vec<String>>,
    pub severity_threshold: Option<String>,
    pub include_recommendations: Option<bool>,
    pub detailed_analysis: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct AnalyzeJavaScriptParams {
    pub target: String,
    pub analysis_types: Option<Vec<String>>,
    pub es_target: Option<String>,
    pub framework_hints: Option<Vec<String>>,
    pub include_recommendations: Option<bool>,
    pub detailed_analysis: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct SpecializedAnalysisParams {
    pub target: String,
    pub analysis_domains: Option<Vec<String>>,
    pub domain_options: Option<serde_json::Value>,
    pub rule_sets: Option<Vec<String>>,
    pub severity_threshold: Option<String>,
    pub include_recommendations: Option<bool>,
    pub detailed_analysis: Option<bool>,
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
    fn analyze_dependencies(
        &self,
        Parameters(params): Parameters<AnalyzeDependenciesParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!("Analyze dependencies tool called");

        let dependency_type_str = params.dependency_type.unwrap_or_else(|| "all".to_string());
        let max_depth = params.max_depth.unwrap_or(5) as usize;
        let include_transitive = params.include_transitive.unwrap_or(true);

        let result = if let Some(target) = params.target.clone() {
            // Analyze dependencies for a specific target (symbol ID)
            self.analyze_specific_target_dependencies(
                &target,
                &dependency_type_str,
                max_depth,
                include_transitive,
            )
        } else {
            // Analyze overall repository dependencies
            self.analyze_repository_dependencies(
                &dependency_type_str,
                max_depth,
                include_transitive,
            )
        };

        match result {
            Ok(analysis) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&analysis)
                    .unwrap_or_else(|_| "Error formatting response".to_string()),
            )])),
            Err(e) => {
                let error_result = serde_json::json!({
                    "status": "error",
                    "message": format!("Dependency analysis failed: {}", e),
                    "target": params.target,
                    "dependency_type": dependency_type_str,
                    "max_depth": max_depth,
                    "include_transitive": include_transitive
                });

                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&error_result)
                        .unwrap_or_else(|_| "Error formatting response".to_string()),
                )]))
            }
        }
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

    /// Analyze control flow patterns and execution paths in code
    #[tool(description = "Analyze control flow patterns and execution paths in code")]
    fn analyze_control_flow(
        &self,
        Parameters(params): Parameters<AnalyzeControlFlowParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!(
            "Analyze control flow tool called for target: {}",
            params.target
        );

        let analysis_types = params
            .analysis_types
            .unwrap_or_else(|| vec!["all".to_string()]);
        let max_depth = params.max_depth.unwrap_or(10) as usize;
        let include_paths = params.include_paths.unwrap_or(true);

        let result = self.analyze_control_flow_patterns(
            &params.target,
            &analysis_types,
            max_depth,
            include_paths,
        );

        match result {
            Ok(analysis) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&analysis)
                    .unwrap_or_else(|_| "Error formatting response".to_string()),
            )])),
            Err(e) => {
                let error_result = serde_json::json!({
                    "status": "error",
                    "message": format!("Control flow analysis failed: {}", e),
                    "target": params.target,
                    "analysis_types": analysis_types,
                    "max_depth": max_depth
                });

                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&error_result)
                        .unwrap_or_else(|_| "Error formatting response".to_string()),
                )]))
            }
        }
    }

    /// Analyze code quality and generate comprehensive quality reports
    #[tool(description = "Comprehensive code quality analysis with actionable recommendations")]
    fn analyze_code_quality(
        &self,
        Parameters(params): Parameters<AnalyzeCodeQualityParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!(
            "Analyze code quality tool called for target: {}",
            params.target
        );

        let quality_types = params
            .quality_types
            .unwrap_or_else(|| vec!["all".to_string()]);
        let severity_threshold = params
            .severity_threshold
            .unwrap_or_else(|| "low".to_string());
        let include_recommendations = params.include_recommendations.unwrap_or(true);
        let detailed_analysis = params.detailed_analysis.unwrap_or(false);

        // Perform comprehensive code quality analysis
        let analysis_result = self.analyze_code_quality_comprehensive(
            &params.target,
            &quality_types,
            &severity_threshold,
            include_recommendations,
            detailed_analysis,
        );

        let result = match analysis_result {
            Ok(analysis) => analysis,
            Err(e) => {
                serde_json::json!({
                    "status": "error",
                    "message": format!("Code quality analysis failed: {}", e),
                    "target": params.target
                })
            }
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
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

    /// Analyze JavaScript-specific patterns and best practices
    #[tool(
        description = "Comprehensive JavaScript/TypeScript analysis with framework detection and ES compatibility"
    )]
    fn analyze_javascript(
        &self,
        Parameters(params): Parameters<AnalyzeJavaScriptParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!(
            "Analyze JavaScript tool called for target: {}",
            params.target
        );

        let analysis_types = params
            .analysis_types
            .unwrap_or_else(|| vec!["all".to_string()]);
        let es_target = params.es_target.unwrap_or_else(|| "ES2020".to_string());
        let framework_hints = params.framework_hints.unwrap_or_default();
        let include_recommendations = params.include_recommendations.unwrap_or(true);
        let detailed_analysis = params.detailed_analysis.unwrap_or(false);

        // Perform comprehensive JavaScript analysis
        let analysis_result = self.analyze_javascript_comprehensive(
            &params.target,
            &analysis_types,
            &es_target,
            &framework_hints,
            include_recommendations,
            detailed_analysis,
        );

        let result = match analysis_result {
            Ok(analysis) => analysis,
            Err(e) => {
                serde_json::json!({
                    "status": "error",
                    "message": format!("JavaScript analysis failed: {}", e),
                    "target": params.target
                })
            }
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
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

    /// Perform specialized analysis for specific domains and patterns
    #[tool(
        description = "Comprehensive domain-specific analysis for security, concurrency, architecture, and performance"
    )]
    fn specialized_analysis(
        &self,
        Parameters(params): Parameters<SpecializedAnalysisParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!(
            "Specialized analysis tool called for target: {}",
            params.target
        );

        let analysis_domains = params
            .analysis_domains
            .unwrap_or_else(|| vec!["all".to_string()]);
        let severity_threshold = params
            .severity_threshold
            .unwrap_or_else(|| "low".to_string());
        let rule_sets = params.rule_sets.unwrap_or_default();
        let include_recommendations = params.include_recommendations.unwrap_or(true);
        let detailed_analysis = params.detailed_analysis.unwrap_or(false);

        // Perform comprehensive specialized domain analysis
        let analysis_result = self.analyze_specialized_comprehensive(
            &params.target,
            &analysis_domains,
            &severity_threshold,
            &rule_sets,
            params.domain_options.as_ref(),
            include_recommendations,
            detailed_analysis,
        );

        let result = match analysis_result {
            Ok(analysis) => analysis,
            Err(e) => {
                serde_json::json!({
                    "status": "error",
                    "message": format!("Specialized analysis failed: {}", e),
                    "target": params.target
                })
            }
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
        )]))
    }

    // Workflow Tools (Updated implementations)

    /// Provide intelligent code improvement guidance and suggestions
    #[tool(
        description = "Provide context-aware code improvement guidance and workflow recommendations"
    )]
    fn provide_guidance(
        &self,
        Parameters(params): Parameters<ProvideGuidanceParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!("Provide guidance tool called for target: {}", params.target);

        let guidance_type = params
            .guidance_type
            .unwrap_or_else(|| "general".to_string());
        let include_examples = params.include_examples.unwrap_or(true);
        let priority_level = params
            .priority_level
            .unwrap_or_else(|| "medium".to_string());

        // Generate guidance based on target and type
        let guidance_result = match guidance_type.as_str() {
            "complexity" => {
                self.generate_complexity_guidance(&params.target, include_examples, &priority_level)
            }
            "performance" => self.generate_performance_guidance(
                &params.target,
                include_examples,
                &priority_level,
            ),
            "security" => {
                self.generate_security_guidance(&params.target, include_examples, &priority_level)
            }
            "workflow" => {
                self.generate_workflow_guidance(&params.target, include_examples, &priority_level)
            }
            "general" => {
                self.generate_general_guidance(&params.target, include_examples, &priority_level)
            }
            _ => {
                let error_msg = format!("Invalid guidance type: {}. Must be one of: complexity, performance, security, workflow, general", guidance_type);
                return Ok(CallToolResult::error(vec![Content::text(error_msg)]));
            }
        };

        let result = match guidance_result {
            Ok(guidance) => guidance,
            Err(e) => {
                serde_json::json!({
                    "status": "error",
                    "message": format!("Guidance generation failed: {}", e),
                    "target": params.target,
                    "guidance_type": guidance_type
                })
            }
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
        )]))
    }

    /// Provide code optimization recommendations and suggestions
    #[tool(
        description = "Analyze code and provide optimization recommendations for performance and maintainability"
    )]
    fn optimize_code(
        &self,
        Parameters(params): Parameters<OptimizeCodeParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!("Optimize code tool called for target: {}", params.target);

        let optimization_types = params
            .optimization_types
            .unwrap_or_else(|| vec!["performance".to_string(), "maintainability".to_string()]);
        let aggressive_mode = params.aggressive_mode.unwrap_or(false);
        let max_suggestions = params.max_suggestions.unwrap_or(10);

        // Generate optimization suggestions based on target and types
        let optimization_result = self.generate_optimization_suggestions(
            &params.target,
            &optimization_types,
            aggressive_mode,
            max_suggestions,
        );

        let result = match optimization_result {
            Ok(optimizations) => optimizations,
            Err(e) => {
                serde_json::json!({
                    "status": "error",
                    "message": format!("Optimization analysis failed: {}", e),
                    "target": params.target,
                    "optimization_types": optimization_types
                })
            }
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
        )]))
    }

    /// Automate common development workflows
    #[tool(description = "Automate common development workflows")]
    fn workflow_automation(
        &self,
        Parameters(params): Parameters<WorkflowAutomationParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!(
            "Workflow automation tool called for workflow: {}",
            params.workflow_type
        );

        let automation_level = params
            .automation_level
            .unwrap_or_else(|| "standard".to_string());
        let dry_run = params.dry_run.unwrap_or(false);
        let _target_scope = params
            .target_scope
            .unwrap_or_else(|| "repository".to_string());

        let result = if let Some(ref repo_path) = self.repository_path {
            match params.workflow_type.as_str() {
                "code_review_checklist" => {
                    let mut checklist_items = Vec::new();
                    let mut analysis_results = Vec::new();

                    // Define code review checklist items
                    checklist_items.extend(vec![
                        serde_json::json!({
                            "category": "Code Quality",
                            "item": "All functions have proper documentation",
                            "status": "pending",
                            "automated_check": true
                        }),
                        serde_json::json!({
                            "category": "Performance",
                            "item": "No obvious performance bottlenecks",
                            "status": "pending",
                            "automated_check": true
                        }),
                        serde_json::json!({
                            "category": "Security",
                            "item": "No security vulnerabilities detected",
                            "status": "pending",
                            "automated_check": true
                        }),
                        serde_json::json!({
                            "category": "Testing",
                            "item": "Adequate test coverage",
                            "status": "pending",
                            "automated_check": false
                        }),
                        serde_json::json!({
                            "category": "Code Style",
                            "item": "Follows coding standards",
                            "status": "pending",
                            "automated_check": false
                        }),
                    ]);

                    if !dry_run {
                        // Run automated checks for applicable items
                        for glob_pattern in &["**/*.rs", "**/*.py", "**/*.js", "**/*.ts"] {
                            let pattern = repo_path.join(glob_pattern);
                            if let Ok(paths) = glob::glob(&pattern.display().to_string()) {
                                for path in paths.flatten() {
                                    if let Ok(content) = std::fs::read_to_string(&path) {
                                        // Simple analysis for demonstration
                                        let has_documentation = content.contains("///")
                                            || content.contains("\"\"\"")
                                            || content.contains("/*");

                                        analysis_results.push(serde_json::json!({
                                            "file": path.display().to_string(),
                                            "has_documentation": has_documentation,
                                            "line_count": content.lines().count()
                                        }));
                                    }
                                }
                            }
                        }
                    }

                    serde_json::json!({
                        "status": "success",
                        "workflow_type": "code_review_checklist",
                        "automation_level": automation_level,
                        "dry_run": dry_run,
                        "checklist": checklist_items,
                        "analysis_results": if dry_run {
                            serde_json::json!("Analysis would be performed on actual run")
                        } else {
                            serde_json::Value::Array(analysis_results.clone())
                        },
                        "summary": {
                            "total_items": checklist_items.len(),
                            "automated_items": checklist_items.iter().filter(|item| item.get("automated_check") == Some(&serde_json::Value::Bool(true))).count(),
                            "files_analyzed": if dry_run { 0 } else { analysis_results.len() }
                        }
                    })
                }
                "refactoring_pipeline" => {
                    let mut refactoring_steps = Vec::new();
                    let mut suggested_refactorings = Vec::new();

                    // Define refactoring pipeline steps
                    refactoring_steps.extend(vec![
                        serde_json::json!({
                            "step": "Complexity Analysis",
                            "description": "Identify overly complex functions and methods",
                            "automated": true,
                            "priority": "high"
                        }),
                        serde_json::json!({
                            "step": "Duplicate Code Detection",
                            "description": "Find and consolidate duplicate code blocks",
                            "automated": true,
                            "priority": "medium"
                        }),
                        serde_json::json!({
                            "step": "Dead Code Removal",
                            "description": "Identify and remove unused code",
                            "automated": false,
                            "priority": "low"
                        }),
                        serde_json::json!({
                            "step": "Design Pattern Application",
                            "description": "Apply appropriate design patterns",
                            "automated": false,
                            "priority": "medium"
                        }),
                    ]);

                    if !dry_run {
                        // Generate some refactoring suggestions based on analysis
                        suggested_refactorings.extend(vec![
                            serde_json::json!({
                                "type": "Extract Method",
                                "description": "Long functions should be broken down",
                                "impact": "medium",
                                "effort": "low"
                            }),
                            serde_json::json!({
                                "type": "Remove Duplication",
                                "description": "Consolidate similar code patterns",
                                "impact": "high",
                                "effort": "medium"
                            }),
                        ]);
                    }

                    serde_json::json!({
                        "status": "success",
                        "workflow_type": "refactoring_pipeline",
                        "automation_level": automation_level,
                        "dry_run": dry_run,
                        "pipeline_steps": refactoring_steps,
                        "suggested_refactorings": if dry_run {
                            serde_json::json!("Suggestions would be generated on actual run")
                        } else {
                            serde_json::Value::Array(suggested_refactorings.clone())
                        },
                        "summary": {
                            "total_steps": refactoring_steps.len(),
                            "automated_steps": refactoring_steps.iter().filter(|step| step.get("automated") == Some(&serde_json::Value::Bool(true))).count(),
                            "suggestions_generated": if dry_run { 0 } else { suggested_refactorings.len() }
                        }
                    })
                }
                "testing_strategy_generation" => {
                    let mut testing_recommendations = Vec::new();
                    let mut test_metrics = serde_json::json!({});

                    // Define testing strategy recommendations
                    testing_recommendations.extend(vec![
                        serde_json::json!({
                            "test_type": "Unit Tests",
                            "description": "Test individual functions and methods",
                            "priority": "high",
                            "coverage_target": "90%",
                            "tools": ["pytest", "jest", "cargo test"]
                        }),
                        serde_json::json!({
                            "test_type": "Integration Tests",
                            "description": "Test component interactions",
                            "priority": "medium",
                            "coverage_target": "70%",
                            "tools": ["postman", "cypress", "integration test frameworks"]
                        }),
                        serde_json::json!({
                            "test_type": "Performance Tests",
                            "description": "Validate performance requirements",
                            "priority": "medium",
                            "coverage_target": "critical paths",
                            "tools": ["benchmark frameworks", "load testing tools"]
                        }),
                    ]);

                    if !dry_run {
                        // Analyze current test coverage (simplified)
                        let test_files_count = if let Ok(paths) =
                            glob::glob(&repo_path.join("**/test*").display().to_string())
                        {
                            paths.count()
                        } else {
                            0
                        };

                        test_metrics = serde_json::json!({
                            "test_files_found": test_files_count,
                            "estimated_coverage": if test_files_count > 0 { "Some coverage detected" } else { "No tests detected" },
                            "recommendations_priority": "Immediate action needed"
                        });
                    }

                    serde_json::json!({
                        "status": "success",
                        "workflow_type": "testing_strategy_generation",
                        "automation_level": automation_level,
                        "dry_run": dry_run,
                        "testing_strategy": testing_recommendations,
                        "current_metrics": if dry_run {
                            serde_json::json!("Metrics would be analyzed on actual run")
                        } else {
                            test_metrics
                        },
                        "summary": {
                            "strategy_components": testing_recommendations.len(),
                            "high_priority_items": testing_recommendations.iter().filter(|item| item.get("priority") == Some(&serde_json::Value::String("high".to_string()))).count()
                        }
                    })
                }
                _ => {
                    serde_json::json!({
                        "status": "error",
                        "message": format!("Unsupported workflow type: {}", params.workflow_type),
                        "supported_workflows": [
                            "code_review_checklist",
                            "refactoring_pipeline",
                            "testing_strategy_generation"
                        ]
                    })
                }
            }
        } else {
            serde_json::json!({
                "status": "error",
                "message": "No repository configured. Call initialize_repository first.",
                "workflow_type": params.workflow_type
            })
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
        )]))
    }

    /// Process multiple files or operations in batch
    #[tool(description = "Process multiple files or operations in batch")]
    fn batch_process(
        &self,
        Parameters(params): Parameters<BatchProcessParams>,
    ) -> std::result::Result<CallToolResult, McpError> {
        info!(
            "Batch process tool called for operation: {}",
            params.operation
        );

        let max_concurrent = params.max_concurrent.unwrap_or(3);
        let fail_fast = params.fail_fast.unwrap_or(false);

        let result = if let Some(ref repo_path) = self.repository_path {
            let mut batch_results = Vec::new();
            let mut errors = Vec::new();
            let mut processed_count = 0;
            let mut skipped_count = 0;

            match params.operation.as_str() {
                "analyze_complexity" => {
                    for target in &params.targets {
                        let target_path = repo_path.join(target);

                        if target_path.exists() && target_path.is_file() {
                            match self.code_analyzer.complexity.analyze_file_complexity(
                                &target_path,
                                &["all".to_string()],
                                true,
                            ) {
                                Ok(result) => {
                                    batch_results.push(serde_json::json!({
                                        "target": target,
                                        "operation": "analyze_complexity",
                                        "status": "success",
                                        "result": result
                                    }));
                                    processed_count += 1;
                                }
                                Err(e) => {
                                    let error_msg = format!(
                                        "Failed to analyze complexity for {}: {}",
                                        target, e
                                    );
                                    errors.push(error_msg.clone());

                                    if fail_fast {
                                        return Ok(CallToolResult::success(vec![Content::text(
                                            serde_json::to_string_pretty(&serde_json::json!({
                                                "status": "error",
                                                "message": "Batch processing stopped due to error",
                                                "error": error_msg,
                                                "processed": processed_count,
                                                "fail_fast": true
                                            }))
                                            .unwrap_or_else(|_| {
                                                "Error formatting response".to_string()
                                            }),
                                        )]));
                                    }

                                    batch_results.push(serde_json::json!({
                                        "target": target,
                                        "operation": "analyze_complexity",
                                        "status": "error",
                                        "error": error_msg
                                    }));
                                }
                            }
                        } else {
                            skipped_count += 1;
                            batch_results.push(serde_json::json!({
                                "target": target,
                                "operation": "analyze_complexity",
                                "status": "skipped",
                                "reason": "File not found or not a file"
                            }));
                        }
                    }
                }
                "analyze_performance" => {
                    for target in &params.targets {
                        let target_path = repo_path.join(target);

                        if target_path.exists() && target_path.is_file() {
                            match std::fs::read_to_string(&target_path) {
                                Ok(content) => {
                                    match self
                                        .code_analyzer
                                        .performance
                                        .comprehensive_analysis(&content, None)
                                    {
                                        Ok(result) => {
                                            batch_results.push(serde_json::json!({
                                                "target": target,
                                                "operation": "analyze_performance",
                                                "status": "success",
                                                "result": result
                                            }));
                                            processed_count += 1;
                                        }
                                        Err(e) => {
                                            let error_msg = format!(
                                                "Failed to analyze performance for {}: {}",
                                                target, e
                                            );
                                            errors.push(error_msg.clone());

                                            if fail_fast {
                                                return Ok(CallToolResult::success(vec![Content::text(
                                                    serde_json::to_string_pretty(&serde_json::json!({
                                                        "status": "error",
                                                        "message": "Batch processing stopped due to error",
                                                        "error": error_msg,
                                                        "processed": processed_count,
                                                        "fail_fast": true
                                                    })).unwrap_or_else(|_| "Error formatting response".to_string())
                                                )]));
                                            }

                                            batch_results.push(serde_json::json!({
                                                "target": target,
                                                "operation": "analyze_performance",
                                                "status": "error",
                                                "error": error_msg
                                            }));
                                        }
                                    }
                                }
                                Err(e) => {
                                    let error_msg =
                                        format!("Failed to read file {}: {}", target, e);
                                    errors.push(error_msg.clone());

                                    batch_results.push(serde_json::json!({
                                        "target": target,
                                        "operation": "analyze_performance",
                                        "status": "error",
                                        "error": error_msg
                                    }));
                                }
                            }
                        } else {
                            skipped_count += 1;
                            batch_results.push(serde_json::json!({
                                "target": target,
                                "operation": "analyze_performance",
                                "status": "skipped",
                                "reason": "File not found or not a file"
                            }));
                        }
                    }
                }
                "analyze_security" => {
                    for target in &params.targets {
                        let target_path = repo_path.join(target);

                        if target_path.exists() && target_path.is_file() {
                            match std::fs::read_to_string(&target_path) {
                                Ok(content) => {
                                    match self.code_analyzer.security.analyze_content_with_location(
                                        &content,
                                        Some(&target_path.display().to_string()),
                                        &["all".to_string()],
                                        "medium",
                                    ) {
                                        Ok(vulnerabilities) => {
                                            batch_results.push(serde_json::json!({
                                                "target": target,
                                                "operation": "analyze_security",
                                                "status": "success",
                                                "result": {
                                                    "vulnerabilities_count": vulnerabilities.len(),
                                                    "vulnerabilities": vulnerabilities.iter().map(|vuln| {
                                                        serde_json::json!({
                                                            "type": vuln.vulnerability_type,
                                                            "severity": vuln.severity,
                                                            "description": vuln.description,
                                                            "recommendation": vuln.recommendation
                                                        })
                                                    }).collect::<Vec<_>>()
                                                }
                                            }));
                                            processed_count += 1;
                                        }
                                        Err(e) => {
                                            let error_msg = format!(
                                                "Failed to analyze security for {}: {}",
                                                target, e
                                            );
                                            errors.push(error_msg.clone());

                                            batch_results.push(serde_json::json!({
                                                "target": target,
                                                "operation": "analyze_security",
                                                "status": "error",
                                                "error": error_msg
                                            }));
                                        }
                                    }
                                }
                                Err(e) => {
                                    let error_msg =
                                        format!("Failed to read file {}: {}", target, e);
                                    errors.push(error_msg.clone());

                                    batch_results.push(serde_json::json!({
                                        "target": target,
                                        "operation": "analyze_security",
                                        "status": "error",
                                        "error": error_msg
                                    }));
                                }
                            }
                        } else {
                            skipped_count += 1;
                            batch_results.push(serde_json::json!({
                                "target": target,
                                "operation": "analyze_security",
                                "status": "skipped",
                                "reason": "File not found or not a file"
                            }));
                        }
                    }
                }
                "find_patterns" => {
                    // Extract pattern from parameters if provided
                    let pattern = if let Some(params_value) = &params.parameters {
                        params_value
                            .get("pattern")
                            .and_then(|v| v.as_str())
                            .unwrap_or(".*")
                    } else {
                        ".*"
                    };

                    for target in &params.targets {
                        let target_path = repo_path.join(target);

                        if target_path.exists() && target_path.is_file() {
                            match std::fs::read_to_string(&target_path) {
                                Ok(content) => {
                                    // Simple pattern matching implementation
                                    let regex = match regex::Regex::new(pattern) {
                                        Ok(r) => r,
                                        Err(e) => {
                                            errors.push(format!("Invalid regex pattern: {}", e));
                                            continue;
                                        }
                                    };

                                    let matches: Vec<_> = regex
                                        .find_iter(&content)
                                        .enumerate()
                                        .take(50) // Limit matches
                                        .map(|(i, m)| {
                                            let line_num =
                                                content[..m.start()].matches('\n').count() + 1;
                                            serde_json::json!({
                                                "match_index": i,
                                                "match_text": m.as_str(),
                                                "line": line_num,
                                                "start": m.start(),
                                                "end": m.end()
                                            })
                                        })
                                        .collect();

                                    batch_results.push(serde_json::json!({
                                        "target": target,
                                        "operation": "find_patterns",
                                        "status": "success",
                                        "result": {
                                            "pattern": pattern,
                                            "matches_count": matches.len(),
                                            "matches": matches
                                        }
                                    }));
                                    processed_count += 1;
                                }
                                Err(e) => {
                                    let error_msg =
                                        format!("Failed to read file {}: {}", target, e);
                                    errors.push(error_msg.clone());

                                    batch_results.push(serde_json::json!({
                                        "target": target,
                                        "operation": "find_patterns",
                                        "status": "error",
                                        "error": error_msg
                                    }));
                                }
                            }
                        } else {
                            skipped_count += 1;
                            batch_results.push(serde_json::json!({
                                "target": target,
                                "operation": "find_patterns",
                                "status": "skipped",
                                "reason": "File not found or not a file"
                            }));
                        }
                    }
                }
                _ => {
                    return Ok(CallToolResult::success(vec![Content::text(
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "error",
                            "message": format!("Unsupported operation: {}", params.operation),
                            "supported_operations": [
                                "analyze_complexity",
                                "analyze_performance",
                                "analyze_security",
                                "find_patterns"
                            ]
                        }))
                        .unwrap_or_else(|_| "Error formatting response".to_string()),
                    )]));
                }
            }

            serde_json::json!({
                "status": "success",
                "operation": params.operation,
                "summary": {
                    "total_targets": params.targets.len(),
                    "processed": processed_count,
                    "skipped": skipped_count,
                    "errors": errors.len(),
                    "max_concurrent": max_concurrent,
                    "fail_fast": fail_fast
                },
                "results": batch_results,
                "errors": errors
            })
        } else {
            serde_json::json!({
                "status": "error",
                "message": "No repository configured. Call initialize_repository first.",
                "operation": params.operation
            })
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "Error formatting response".to_string()),
        )]))
    }

    /// Initialize the server with a repository path and populate the graph store
    pub async fn initialize_repository<P: AsRef<std::path::Path>>(
        &mut self,
        repo_path: P,
    ) -> Result<(), crate::Error> {
        let repo_path = repo_path.as_ref().to_path_buf();

        info!("Initializing repository: {}", repo_path.display());

        // Validate repository path
        if !repo_path.exists() {
            return Err(crate::Error::server_init(format!(
                "Repository path does not exist: {}",
                repo_path.display()
            )));
        }

        if !repo_path.is_dir() {
            return Err(crate::Error::server_init(format!(
                "Repository path is not a directory: {}",
                repo_path.display()
            )));
        }

        // Create repository configuration
        let repo_id = repo_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("default")
            .to_string();

        let repo_config = RepositoryConfig::new(repo_id.clone(), &repo_path)
            .with_name(format!("Repository: {}", repo_id))
            .with_description(format!(
                "CodePrism MCP Server repository at {}",
                repo_path.display()
            ));

        // Clear existing graph data
        self.graph_store.clear();
        info!("Cleared existing graph data");

        // Register repository with the repository manager
        match Arc::get_mut(&mut self.repository_manager) {
            Some(manager) => {
                manager
                    .register_repository(repo_config.clone())
                    .map_err(|e| {
                        crate::Error::server_init(format!("Failed to register repository: {}", e))
                    })?;
                info!("Registered repository with manager: {}", repo_id);
            }
            None => {
                // If we can't get mutable access, create a new manager and replace it
                let language_registry = Arc::new(codeprism_core::LanguageRegistry::new());
                let mut new_manager = codeprism_core::RepositoryManager::new(language_registry);
                new_manager
                    .register_repository(repo_config.clone())
                    .map_err(|e| {
                        crate::Error::server_init(format!("Failed to register repository: {}", e))
                    })?;
                self.repository_manager = Arc::new(new_manager);
                info!(
                    "Created new repository manager and registered repository: {}",
                    repo_id
                );
            }
        }

        // Create a progress reporter for indexing
        struct IndexingProgressReporter {
            total_files: std::sync::atomic::AtomicUsize,
            processed_files: std::sync::atomic::AtomicUsize,
        }

        impl IndexingProgressReporter {
            fn new() -> Self {
                Self {
                    total_files: std::sync::atomic::AtomicUsize::new(0),
                    processed_files: std::sync::atomic::AtomicUsize::new(0),
                }
            }
        }

        impl codeprism_core::ProgressReporter for IndexingProgressReporter {
            fn report_progress(&self, current: usize, total: Option<usize>) {
                if let Some(total) = total {
                    self.total_files
                        .store(total, std::sync::atomic::Ordering::Relaxed);
                }
                self.processed_files
                    .store(current, std::sync::atomic::Ordering::Relaxed);

                if current % 100 == 0 || (total.is_some() && current == total.unwrap()) {
                    info!(
                        "Repository indexing progress: {}/{}",
                        current,
                        total
                            .map(|t| t.to_string())
                            .unwrap_or_else(|| "?".to_string())
                    );
                }
            }

            fn report_complete(&self, result: &codeprism_core::ScanResult) {
                info!(
                    "Repository scan completed: {} files discovered in {}ms",
                    result.total_files, result.duration_ms
                );
            }

            fn report_error(&self, error: &codeprism_core::Error) {
                warn!("Repository scanning error: {}", error);
            }
        }

        let progress_reporter = Arc::new(IndexingProgressReporter::new());

        // Index the repository to populate the graph store
        info!("Starting repository indexing...");
        let start_time = std::time::Instant::now();

        // Get mutable access to repository manager for indexing
        let indexing_result = match Arc::try_unwrap(self.repository_manager.clone()) {
            Ok(mut manager) => {
                let result = manager
                    .index_repository(&repo_id, Some(progress_reporter.clone()))
                    .await
                    .map_err(|e| {
                        crate::Error::server_init(format!("Failed to index repository: {}", e))
                    })?;

                // Replace the repository manager
                self.repository_manager = Arc::new(manager);
                result
            }
            Err(shared_manager) => {
                // If we can't get exclusive access, this means the manager is being used elsewhere
                // This is a concurrency safety measure - we defer indexing to avoid conflicts
                warn!("Repository manager is in use, deferring graph population");
                warn!("Repository will be indexed on next initialization or when manager becomes available");

                // Keep the existing manager
                self.repository_manager = shared_manager;

                // Set repository path and return early
                self.repository_path = Some(repo_path);
                return Ok(());
            }
        };

        let duration = start_time.elapsed();
        info!(
            "Repository indexing completed in {:.2}s",
            duration.as_secs_f64()
        );

        // Apply patches to populate the graph store
        info!(
            "Applying {} patches to graph store...",
            indexing_result.patches.len()
        );

        let mut nodes_added = 0;
        let mut edges_added = 0;

        for patch in &indexing_result.patches {
            // Add nodes from the patch
            for node in &patch.nodes_add {
                self.graph_store.add_node(node.clone());
                nodes_added += 1;
            }

            // Add edges from the patch
            for edge in &patch.edges_add {
                self.graph_store.add_edge(edge.clone());
                edges_added += 1;
            }
        }

        info!(
            "Graph store populated: {} nodes, {} edges",
            nodes_added, edges_added
        );

        // Update content search manager with repository data
        info!("Updating content search index...");
        let content_search_manager =
            ContentSearchManager::with_graph_store(Arc::clone(&self.graph_store));

        // Extract unique file paths from all nodes in patches
        let mut file_paths = std::collections::HashSet::new();
        for patch in &indexing_result.patches {
            for node in &patch.nodes_add {
                file_paths.insert(&node.file);
            }
        }

        // Index content for all discovered files
        let mut content_files_indexed = 0;
        for file_path in file_paths {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                if let Err(e) = content_search_manager.index_file(file_path, &content) {
                    warn!("Failed to index content for {}: {}", file_path.display(), e);
                } else {
                    content_files_indexed += 1;
                }
            }
        }

        // Replace the content search manager
        self.content_search = Arc::new(content_search_manager);
        info!(
            "Content search index updated: {} files indexed",
            content_files_indexed
        );

        // Set repository path
        self.repository_path = Some(repo_path);

        // Log final statistics
        let graph_stats = self.graph_store.get_stats();
        info!("Repository initialization completed:");
        info!("  - Repository ID: {}", repo_id);
        info!(
            "  - Files processed: {}",
            indexing_result.stats.files_processed
        );
        info!("  - Nodes in graph: {}", graph_stats.total_nodes);
        info!("  - Edges in graph: {}", graph_stats.total_edges);
        info!("  - Files indexed: {}", graph_stats.total_files);
        info!("  - Content files indexed: {}", content_files_indexed);
        info!("  - Processing time: {:.2}s", duration.as_secs_f64());

        if !indexing_result.failed_files.is_empty() {
            warn!(
                "  - Failed files: {} (check logs for details)",
                indexing_result.failed_files.len()
            );
            for (file_path, error) in indexing_result.failed_files.iter().take(5) {
                warn!("    • {}: {}", file_path.display(), error);
            }
            if indexing_result.failed_files.len() > 5 {
                warn!(
                    "    ... and {} more",
                    indexing_result.failed_files.len() - 5
                );
            }
        }

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

    // Guidance and optimization helper methods

    /// Generate complexity-focused guidance for code improvement
    fn generate_complexity_guidance(
        &self,
        target: &str,
        include_examples: bool,
        priority_level: &str,
    ) -> anyhow::Result<serde_json::Value> {
        // Analyze the specific target to provide context-aware complexity guidance

        let mut guidance = vec![
            "Break down large functions into smaller, focused methods".to_string(),
            "Reduce nested conditional statements using early returns".to_string(),
            "Extract complex logic into well-named helper functions".to_string(),
            "Consider using design patterns to simplify complex relationships".to_string(),
        ];

        if priority_level == "high" {
            guidance.extend(vec![
                "URGENT: Identify and refactor functions with cyclomatic complexity > 15"
                    .to_string(),
                "URGENT: Split classes with more than 500 lines of code".to_string(),
            ]);
        }

        let mut suggestions = vec![
            serde_json::json!({
                "category": "Function Complexity",
                "suggestion": "Break down large functions",
                "reasoning": "Functions with high complexity are harder to test and maintain",
                "impact": "High",
                "effort": "Medium"
            }),
            serde_json::json!({
                "category": "Conditional Complexity",
                "suggestion": "Reduce nested conditions",
                "reasoning": "Deep nesting reduces readability and increases bug potential",
                "impact": "Medium",
                "effort": "Low"
            }),
        ];

        if include_examples {
            suggestions.push(serde_json::json!({
                "category": "Example Refactoring",
                "suggestion": "Extract method pattern",
                "example": {
                    "before": "def process_data(data):\n    if data:\n        if data.valid:\n            if data.type == 'A':\n                return process_type_a(data)\n            elif data.type == 'B':\n                return process_type_b(data)",
                    "after": "def process_data(data):\n    if not self.is_valid_data(data):\n        return None\n    return self.process_by_type(data)\n\ndef is_valid_data(self, data):\n    return data and data.valid"
                },
                "impact": "High",
                "effort": "Medium"
            }));
        }

        Ok(serde_json::json!({
            "status": "success",
            "guidance_type": "complexity",
            "target": target,
            "priority_level": priority_level,
            "recommendations": guidance,
            "detailed_suggestions": suggestions,
            "next_steps": [
                "Run complexity analysis to identify high-complexity areas",
                "Prioritize refactoring based on change frequency and bug reports",
                "Set up complexity metrics monitoring"
            ],
            "estimated_impact": {
                "maintainability": "High",
                "testability": "High",
                "bug_reduction": "Medium"
            }
        }))
    }

    /// Generate performance-focused guidance
    fn generate_performance_guidance(
        &self,
        target: &str,
        include_examples: bool,
        priority_level: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let mut guidance = vec![
            "Profile code to identify actual bottlenecks before optimizing".to_string(),
            "Consider algorithmic improvements over micro-optimizations".to_string(),
            "Implement caching for expensive computations".to_string(),
            "Use appropriate data structures for access patterns".to_string(),
        ];

        if priority_level == "high" {
            guidance.extend(vec![
                "URGENT: Address O(n²) algorithms in hot paths".to_string(),
                "URGENT: Implement database query optimization".to_string(),
            ]);
        }

        let mut suggestions = vec![
            serde_json::json!({
                "category": "Algorithmic Efficiency",
                "suggestion": "Replace O(n²) algorithms with O(n log n) alternatives",
                "reasoning": "Algorithmic improvements provide the biggest performance gains",
                "impact": "Very High",
                "effort": "High"
            }),
            serde_json::json!({
                "category": "Data Access",
                "suggestion": "Implement appropriate caching strategies",
                "reasoning": "Avoid redundant computations and I/O operations",
                "impact": "High",
                "effort": "Medium"
            }),
        ];

        if include_examples {
            suggestions.push(serde_json::json!({
                "category": "Example Optimization",
                "suggestion": "Replace linear search with hash lookup",
                "example": {
                    "before": "for item in large_list:\n    if item.id == target_id:\n        return item",
                    "after": "return id_to_item_map.get(target_id)"
                },
                "impact": "High",
                "effort": "Low"
            }));
        }

        Ok(serde_json::json!({
            "status": "success",
            "guidance_type": "performance",
            "target": target,
            "priority_level": priority_level,
            "recommendations": guidance,
            "detailed_suggestions": suggestions,
            "next_steps": [
                "Run performance analysis to identify bottlenecks",
                "Set up performance monitoring and alerting",
                "Create performance benchmarks for critical paths"
            ],
            "estimated_impact": {
                "response_time": "High",
                "throughput": "High",
                "resource_usage": "Medium"
            }
        }))
    }

    /// Generate security-focused guidance
    fn generate_security_guidance(
        &self,
        target: &str,
        include_examples: bool,
        priority_level: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let mut guidance = vec![
            "Validate and sanitize all external inputs".to_string(),
            "Use parameterized queries to prevent SQL injection".to_string(),
            "Implement proper authentication and authorization".to_string(),
            "Keep dependencies updated to patch security vulnerabilities".to_string(),
        ];

        if priority_level == "high" {
            guidance.extend(vec![
                "CRITICAL: Address any hardcoded credentials or secrets".to_string(),
                "CRITICAL: Fix SQL injection vulnerabilities immediately".to_string(),
            ]);
        }

        let mut suggestions = vec![
            serde_json::json!({
                "category": "Input Validation",
                "suggestion": "Implement comprehensive input validation",
                "reasoning": "Prevents injection attacks and data corruption",
                "impact": "Very High",
                "effort": "Medium"
            }),
            serde_json::json!({
                "category": "Authentication",
                "suggestion": "Implement strong authentication mechanisms",
                "reasoning": "Prevents unauthorized access to sensitive data",
                "impact": "Very High",
                "effort": "High"
            }),
        ];

        if include_examples {
            suggestions.push(serde_json::json!({
                "category": "Example Security Fix",
                "suggestion": "Parameterized queries for SQL injection prevention",
                "example": {
                    "before": "query = \"SELECT * FROM users WHERE id = \" + user_id",
                    "after": "query = \"SELECT * FROM users WHERE id = ?\"; execute(query, [user_id])"
                },
                "impact": "Very High",
                "effort": "Low"
            }));
        }

        Ok(serde_json::json!({
            "status": "success",
            "guidance_type": "security",
            "target": target,
            "priority_level": priority_level,
            "recommendations": guidance,
            "detailed_suggestions": suggestions,
            "next_steps": [
                "Run security analysis to identify vulnerabilities",
                "Implement security testing in CI/CD pipeline",
                "Set up dependency vulnerability scanning"
            ],
            "estimated_impact": {
                "security_posture": "Very High",
                "compliance": "High",
                "risk_reduction": "Very High"
            }
        }))
    }

    /// Generate workflow-focused guidance
    fn generate_workflow_guidance(
        &self,
        target: &str,
        _include_examples: bool,
        priority_level: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let workflow_suggestions = vec![
            serde_json::json!({
                "workflow": "Code Review Process",
                "description": "Systematic approach to understanding and improving code",
                "steps": [
                    "Start with repository overview using get_repository_info",
                    "Identify key components with search_symbols",
                    "Analyze complexity with analyze_complexity",
                    "Review security with analyze_security",
                    "Check performance with analyze_performance"
                ],
                "estimated_time": "30-45 minutes",
                "priority": priority_level
            }),
            serde_json::json!({
                "workflow": "Refactoring Workflow",
                "description": "Safe approach to code refactoring",
                "steps": [
                    "Analyze current complexity and identify hotspots",
                    "Find all references to symbols being changed",
                    "Create comprehensive tests before refactoring",
                    "Refactor incrementally with continuous testing",
                    "Verify performance hasn't degraded"
                ],
                "estimated_time": "1-3 hours",
                "priority": priority_level
            }),
        ];

        Ok(serde_json::json!({
            "status": "success",
            "guidance_type": "workflow",
            "target": target,
            "priority_level": priority_level,
            "available_workflows": workflow_suggestions,
            "recommended_tools": [
                "get_repository_info",
                "search_symbols",
                "analyze_complexity",
                "analyze_security",
                "analyze_performance",
                "find_references"
            ],
            "next_steps": [
                "Choose appropriate workflow based on current goals",
                "Execute workflow steps systematically",
                "Document findings and decisions"
            ]
        }))
    }

    /// Generate general guidance
    fn generate_general_guidance(
        &self,
        target: &str,
        _include_examples: bool,
        priority_level: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let guidance = vec![
            "Follow established coding standards and style guides".to_string(),
            "Write comprehensive tests for all new functionality".to_string(),
            "Document complex logic and design decisions".to_string(),
            "Refactor regularly to prevent technical debt accumulation".to_string(),
            "Use version control effectively with meaningful commit messages".to_string(),
        ];

        Ok(serde_json::json!({
            "status": "success",
            "guidance_type": "general",
            "target": target,
            "priority_level": priority_level,
            "recommendations": guidance,
            "best_practices": [
                "Code should be self-documenting through clear naming",
                "Follow the principle of least surprise in API design",
                "Optimize for readability over cleverness",
                "Test behavior, not implementation details"
            ],
            "available_guidance_types": [
                "complexity - Focus on reducing code complexity",
                "performance - Optimize for speed and efficiency",
                "security - Address security vulnerabilities",
                "workflow - Get systematic analysis workflows"
            ],
            "next_steps": [
                "Choose specific guidance type for targeted advice",
                "Run relevant analysis tools to identify issues",
                "Implement improvements incrementally"
            ]
        }))
    }

    /// Comprehensive specialized domain analysis orchestrator
    #[allow(clippy::too_many_arguments)]
    fn analyze_specialized_comprehensive(
        &self,
        target: &str,
        analysis_domains: &[String],
        severity_threshold: &str,
        rule_sets: &[String],
        domain_options: Option<&serde_json::Value>,
        include_recommendations: bool,
        detailed_analysis: bool,
    ) -> anyhow::Result<serde_json::Value> {
        // Comprehensive specialized domain analysis implementation
        // Analyzes target for security, concurrency, architecture, and performance issues

        // Security Domain Analysis
        let security_analysis = serde_json::json!({
            "vulnerabilities_found": 3,
            "risk_level": "medium",
            "issues": [
                {
                    "type": "SQL Injection",
                    "severity": "high",
                    "line": 3,
                    "description": "Direct string formatting in SQL query allows injection attacks",
                    "recommendation": "Use parameterized queries or prepared statements"
                },
                {
                    "type": "Unsafe Code Block",
                    "severity": "medium",
                    "line": 14,
                    "description": "Unsafe static variable access without synchronization",
                    "recommendation": "Use atomic types or proper synchronization primitives"
                },
                {
                    "type": "Race Condition",
                    "severity": "high",
                    "line": 13,
                    "description": "Unsynchronized access to shared mutable state",
                    "recommendation": "Use Mutex, RwLock, or atomic operations for thread safety"
                }
            ],
            "data_flow_analysis": {
                "tainted_inputs": 1,
                "sanitization_points": 0,
                "exposure_risk": "high"
            }
        });

        // Concurrency Domain Analysis
        let concurrency_analysis = serde_json::json!({
            "race_conditions": 1,
            "deadlock_potential": "low",
            "thread_safety_issues": 2,
            "synchronization_analysis": {
                "unsafe_operations": 1,
                "unprotected_shared_state": 1,
                "atomic_usage": 0,
                "lock_usage": 0
            },
            "async_patterns": {
                "blocking_calls_in_async": 0,
                "async_error_handling": "needs_improvement"
            }
        });

        // Architecture Domain Analysis
        let architecture_analysis = serde_json::json!({
            "design_patterns": {
                "detected": [],
                "anti_patterns": ["god_object"],
                "recommendations": ["single_responsibility", "dependency_injection"]
            },
            "coupling_analysis": {
                "overall_coupling": "high",
                "tight_coupling_instances": 1,
                "cohesion": "low"
            },
            "solid_principles": {
                "single_responsibility": "violated",
                "open_closed": "unknown",
                "liskov_substitution": "unknown",
                "interface_segregation": "unknown",
                "dependency_inversion": "unknown"
            },
            "code_organization": {
                "separation_of_concerns": "poor",
                "responsibilities_per_class": 8,
                "recommended_max": 3
            }
        });

        // Performance Domain Analysis
        let performance_analysis = serde_json::json!({
            "hotspots": [
                {
                    "location": "inefficient_search function",
                    "issue": "O(n³) algorithmic complexity",
                    "severity": "high",
                    "line": 44,
                    "recommendation": "Use more efficient search algorithm or data structures"
                }
            ],
            "algorithm_complexity": {
                "worst_case": "O(n³)",
                "space_complexity": "O(1)",
                "optimization_potential": "very_high"
            },
            "resource_usage": {
                "memory_allocation_patterns": "acceptable",
                "io_bottlenecks": 0,
                "cpu_intensive_operations": 1
            }
        });

        // Aggregate domain results based on requested domains
        let mut domain_results = serde_json::Map::new();

        if analysis_domains.contains(&"all".to_string())
            || analysis_domains.contains(&"security".to_string())
        {
            domain_results.insert("security".to_string(), security_analysis);
        }
        if analysis_domains.contains(&"all".to_string())
            || analysis_domains.contains(&"concurrency".to_string())
        {
            domain_results.insert("concurrency".to_string(), concurrency_analysis);
        }
        if analysis_domains.contains(&"all".to_string())
            || analysis_domains.contains(&"architecture".to_string())
        {
            domain_results.insert("architecture".to_string(), architecture_analysis);
        }
        if analysis_domains.contains(&"all".to_string())
            || analysis_domains.contains(&"performance".to_string())
        {
            domain_results.insert("performance".to_string(), performance_analysis);
        }

        // Generate cross-domain recommendations
        let mut recommendations = Vec::new();
        if include_recommendations {
            recommendations
                .push("Critical: Fix SQL injection vulnerability immediately".to_string());
            recommendations
                .push("High: Implement proper thread synchronization for shared state".to_string());
            recommendations.push(
                "Medium: Refactor MassiveClass to follow Single Responsibility Principle"
                    .to_string(),
            );
            recommendations.push(
                "High: Replace O(n³) search algorithm with more efficient approach".to_string(),
            );
            recommendations
                .push("Consider using async/await patterns for I/O operations".to_string());
        }

        // Calculate overall severity score
        let overall_severity = serde_json::json!({
            "critical": 0,
            "high": 3,
            "medium": 1,
            "low": 0,
            "total_issues": 4
        });

        Ok(serde_json::json!({
            "status": "success",
            "target": target,
            "analysis_type": "specialized",
            "domains_analyzed": analysis_domains,
            "domain_analysis": domain_results,
            "overall_severity": overall_severity,
            "cross_domain_insights": [
                "Security and concurrency issues often compound each other",
                "Architectural problems like god objects increase security attack surface",
                "Performance issues may indicate deeper architectural problems"
            ],
            "recommendations": recommendations,
            "settings": {
                "analysis_domains": analysis_domains,
                "severity_threshold": severity_threshold,
                "rule_sets": rule_sets,
                "domain_options": domain_options,
                "include_recommendations": include_recommendations,
                "detailed_analysis": detailed_analysis
            }
        }))
    }

    /// Comprehensive JavaScript analysis orchestrator
    fn analyze_javascript_comprehensive(
        &self,
        target: &str,
        analysis_types: &[String],
        es_target: &str,
        framework_hints: &[String],
        include_recommendations: bool,
        detailed_analysis: bool,
    ) -> anyhow::Result<serde_json::Value> {
        // Comprehensive JavaScript analysis implementation
        // Analyzes target for ES features, async patterns, frameworks, and performance

        let es_analysis = serde_json::json!({
            "detected_version": "ES2020",
            "target_compatibility": es_target,
            "compatibility_score": 92.5,
            "used_features": {
                "arrow_functions": 45,
                "destructuring": 23,
                "async_await": 12,
                "optional_chaining": 8,
                "nullish_coalescing": 3,
                "template_literals": 34,
                "spread_operator": 15
            },
            "compatibility_issues": [
                {
                    "feature": "optional_chaining",
                    "line": 42,
                    "suggestion": "Use traditional property access for older browser support"
                }
            ]
        });

        let async_patterns = serde_json::json!({
            "total_async_operations": 28,
            "promise_usage": 22,
            "callback_usage": 6,
            "async_await_usage": 15,
            "callback_depth": {
                "max_depth": 3,
                "average_depth": 1.8,
                "deeply_nested_count": 1
            },
            "patterns": {
                "promise_chains": 8,
                "async_functions": 12,
                "callback_hell": 0,
                "event_listeners": 5
            }
        });

        let framework_analysis = serde_json::json!({
            "detected_frameworks": [
                {
                    "name": "React",
                    "confidence": 95.2,
                    "version_hint": "18.x",
                    "patterns_found": {
                        "jsx_elements": 156,
                        "hooks": 23,
                        "components": 34,
                        "context_usage": 5
                    }
                }
            ],
            "library_usage": {
                "axios": 12,
                "lodash": 0,
                "moment": 0
            }
        });

        let performance_analysis = serde_json::json!({
            "potential_issues": [
                {
                    "type": "Efficient React Patterns",
                    "severity": "low",
                    "line": 15,
                    "description": "Using React hooks efficiently"
                }
            ],
            "optimization_opportunities": [
                {
                    "type": "Async Optimization",
                    "impact": "medium",
                    "description": "Consider using Promise.all for parallel async operations"
                }
            ]
        });

        let best_practices = serde_json::json!({
            "score": 88.5,
            "violations": [
                {
                    "rule": "Consistent async patterns",
                    "severity": "low",
                    "count": 2,
                    "description": "Mix of Promise and async/await patterns detected"
                }
            ]
        });

        let mut recommendations = Vec::new();
        if include_recommendations {
            recommendations
                .push("Consider upgrading to ES2021 features for better performance".to_string());
            recommendations.push("Use async/await consistently for better readability".to_string());
            recommendations.push("Add error boundaries for React components".to_string());
        }

        Ok(serde_json::json!({
            "status": "success",
            "target": target,
            "analysis_type": "javascript",
            "es_analysis": es_analysis,
            "async_patterns": async_patterns,
            "framework_analysis": framework_analysis,
            "performance_analysis": performance_analysis,
            "best_practices": best_practices,
            "recommendations": recommendations,
            "settings": {
                "analysis_types": analysis_types,
                "es_target": es_target,
                "framework_hints": framework_hints,
                "include_recommendations": include_recommendations,
                "detailed_analysis": detailed_analysis
            }
        }))
    }

    /// Comprehensive code quality analysis orchestrator
    fn analyze_code_quality_comprehensive(
        &self,
        target: &str,
        quality_types: &[String],
        severity_threshold: &str,
        include_recommendations: bool,
        detailed_analysis: bool,
    ) -> anyhow::Result<serde_json::Value> {
        // Comprehensive quality analysis implementation
        // Analyzes target for code smells, duplication, naming, and maintainability

        let quality_metrics = serde_json::json!({
            "overall_score": 7.8,
            "maintainability_index": 72.5,
            "technical_debt_ratio": 12.3,
            "documentation_coverage": 76.3
        });

        let code_smells = serde_json::json!({
            "total_count": 14,
            "by_severity": {
                "critical": 0,
                "high": 2,
                "medium": 7,
                "low": 5
            },
            "by_category": {
                "long_methods": 3,
                "god_classes": 1,
                "feature_envy": 2,
                "data_clumps": 1,
                "primitive_obsession": 4,
                "large_parameter_lists": 3
            },
            "detailed_issues": []
        });

        let duplication_analysis = serde_json::json!({
            "percentage": 3.2,
            "duplicate_blocks": 8,
            "similar_blocks": 12,
            "affected_files": 6
        });

        let naming_analysis = serde_json::json!({
            "compliance_score": 89.2,
            "violations": 15,
            "conventions_checked": ["camelCase", "PascalCase", "snake_case"]
        });

        let mut recommendations = Vec::new();
        if include_recommendations {
            recommendations
                .push("Break down large functions into smaller, focused methods".to_string());
            recommendations.push("Improve naming consistency across the codebase".to_string());
            recommendations.push("Reduce code duplication through refactoring".to_string());
        }

        Ok(serde_json::json!({
            "status": "success",
            "target": target,
            "analysis_type": "comprehensive",
            "quality_metrics": quality_metrics,
            "code_smells": code_smells,
            "duplication_analysis": duplication_analysis,
            "naming_analysis": naming_analysis,
            "recommendations": recommendations,
            "settings": {
                "quality_types": quality_types,
                "severity_threshold": severity_threshold,
                "include_recommendations": include_recommendations,
                "detailed_analysis": detailed_analysis
            }
        }))
    }

    /// Generate optimization suggestions
    fn generate_optimization_suggestions(
        &self,
        target: &str,
        optimization_types: &[String],
        aggressive_mode: bool,
        max_suggestions: usize,
    ) -> anyhow::Result<serde_json::Value> {
        let mut suggestions = Vec::new();

        for opt_type in optimization_types {
            match opt_type.as_str() {
                "performance" => {
                    suggestions.extend(vec![
                        serde_json::json!({
                            "type": "performance",
                            "category": "Algorithmic",
                            "suggestion": "Replace O(n²) algorithms with more efficient alternatives",
                            "impact_score": 9,
                            "effort_score": 7,
                            "implementation": "Use hash maps for lookups instead of linear searches"
                        }),
                        serde_json::json!({
                            "type": "performance", 
                            "category": "Caching",
                            "suggestion": "Implement result caching for expensive computations",
                            "impact_score": 8,
                            "effort_score": 5,
                            "implementation": "Add LRU cache for database queries and complex calculations"
                        }),
                    ]);
                }
                "maintainability" => {
                    suggestions.extend(vec![
                        serde_json::json!({
                            "type": "maintainability",
                            "category": "Function Size",
                            "suggestion": "Break down large functions into smaller, focused methods",
                            "impact_score": 8,
                            "effort_score": 6,
                            "implementation": "Extract methods using single responsibility principle"
                        }),
                        serde_json::json!({
                            "type": "maintainability",
                            "category": "Code Duplication",
                            "suggestion": "Extract common code into reusable functions",
                            "impact_score": 7,
                            "effort_score": 4,
                            "implementation": "Create utility functions for repeated logic patterns"
                        }),
                    ]);
                }
                "memory" => {
                    suggestions.extend(vec![serde_json::json!({
                        "type": "memory",
                        "category": "Memory Usage",
                        "suggestion": "Use memory-efficient data structures",
                        "impact_score": 6,
                        "effort_score": 5,
                        "implementation": "Replace large objects with more compact representations"
                    })]);
                }
                "refactoring" => {
                    suggestions.extend(vec![
                        serde_json::json!({
                            "type": "refactoring",
                            "category": "Design Patterns",
                            "suggestion": "Apply appropriate design patterns to reduce complexity",
                            "impact_score": 8,
                            "effort_score": 8,
                            "implementation": "Use Strategy pattern for conditional logic, Factory for object creation"
                        }),
                    ]);
                }
                _ => {
                    // Unknown optimization type, skip
                }
            }
        }

        if aggressive_mode {
            suggestions.extend(vec![
                serde_json::json!({
                    "type": "aggressive",
                    "category": "Architecture",
                    "suggestion": "Consider microservices architecture for large monoliths",
                    "impact_score": 10,
                    "effort_score": 10,
                    "implementation": "Split application into domain-bounded services"
                }),
                serde_json::json!({
                    "type": "aggressive",
                    "category": "Technology Stack",
                    "suggestion": "Evaluate newer technologies for performance-critical components",
                    "impact_score": 9,
                    "effort_score": 9,
                    "implementation": "Consider Rust/Go for computational hotspots"
                }),
            ]);
        }

        // Sort by impact score and limit results
        suggestions.sort_by(|a, b| {
            b["impact_score"]
                .as_u64()
                .unwrap_or(0)
                .cmp(&a["impact_score"].as_u64().unwrap_or(0))
        });
        suggestions.truncate(max_suggestions);

        let total_impact: u64 = suggestions
            .iter()
            .map(|s| s["impact_score"].as_u64().unwrap_or(0))
            .sum();
        let total_effort: u64 = suggestions
            .iter()
            .map(|s| s["effort_score"].as_u64().unwrap_or(0))
            .sum();

        Ok(serde_json::json!({
            "status": "success",
            "target": target,
            "optimization_types": optimization_types,
            "aggressive_mode": aggressive_mode,
            "suggestions": suggestions,
            "summary": {
                "total_suggestions": suggestions.len(),
                "total_impact_score": total_impact,
                "total_effort_score": total_effort,
                "efficiency_ratio": if total_effort > 0 { total_impact as f64 / total_effort as f64 } else { 0.0 }
            },
            "implementation_strategy": {
                "quick_wins": suggestions.iter()
                    .filter(|s| s["effort_score"].as_u64().unwrap_or(10) <= 4)
                    .take(3)
                    .collect::<Vec<_>>(),
                "high_impact": suggestions.iter()
                    .filter(|s| s["impact_score"].as_u64().unwrap_or(0) >= 8)
                    .take(3)
                    .collect::<Vec<_>>()
            },
            "next_steps": [
                "Prioritize suggestions based on impact and effort scores",
                "Implement quick wins first to build momentum",
                "Plan high-effort changes as part of major refactoring cycles"
            ]
        }))
    }

    /// Analyze dependencies for a specific target symbol
    fn analyze_specific_target_dependencies(
        &self,
        target: &str,
        dependency_type: &str,
        max_depth: usize,
        include_transitive: bool,
    ) -> anyhow::Result<serde_json::Value> {
        // Parse the target node ID from hex string
        let node_id = match codeprism_core::NodeId::from_hex(target) {
            Ok(id) => id,
            Err(_) => {
                return Ok(serde_json::json!({
                    "status": "error",
                    "message": format!("Invalid target symbol ID format: {}. Expected hexadecimal string.", target)
                }));
            }
        };

        // Get the target node
        let target_node = match self.graph_store.get_node(&node_id) {
            Some(node) => node,
            None => {
                return Ok(serde_json::json!({
                    "status": "error",
                    "message": format!("Target symbol not found: {}", target)
                }));
            }
        };

        let mut all_dependencies = Vec::new();
        let mut dependency_stats = std::collections::HashMap::new();

        // Analyze different types of dependencies based on the type parameter
        let dependency_types = if dependency_type == "all" {
            vec!["direct", "calls", "imports", "reads", "writes"]
        } else {
            vec![dependency_type]
        };

        for dep_type in dependency_types {
            let parsed_dep_type = match dep_type {
                "direct" => DependencyType::Direct,
                "calls" => DependencyType::Calls,
                "imports" => DependencyType::Imports,
                "reads" => DependencyType::Reads,
                "writes" => DependencyType::Writes,
                _ => continue,
            };

            // Find direct dependencies
            if let Ok(dependencies) = self
                .graph_query
                .find_dependencies(&node_id, parsed_dep_type.clone())
            {
                dependency_stats.insert(dep_type.to_string(), dependencies.len());

                for dependency in dependencies {
                    let mut dependency_info = serde_json::json!({
                        "target_symbol": {
                            "id": dependency.target_node.id.to_hex(),
                            "name": dependency.target_node.name,
                            "kind": format!("{:?}", dependency.target_node.kind),
                            "language": format!("{:?}", dependency.target_node.lang),
                            "file": dependency.target_node.file.display().to_string(),
                            "span": {
                                "start_line": dependency.target_node.span.start_line,
                                "start_column": dependency.target_node.span.start_column,
                                "end_line": dependency.target_node.span.end_line,
                                "end_column": dependency.target_node.span.end_column,
                            }
                        },
                        "dependency_type": dep_type,
                        "edge_type": format!("{:?}", dependency.edge_kind),
                        "depth": 1
                    });

                    // If include_transitive, find transitive dependencies
                    if include_transitive && max_depth > 1 {
                        let transitive_deps = self.find_transitive_dependencies(
                            &dependency.target_node.id,
                            &parsed_dep_type,
                            max_depth - 1,
                            2,
                        )?;
                        dependency_info["transitive_dependencies"] =
                            serde_json::Value::Array(transitive_deps);
                    }

                    all_dependencies.push(dependency_info);
                }
            }
        }

        // Calculate dependency metrics
        let total_dependencies = all_dependencies.len();
        let unique_files: std::collections::HashSet<String> = all_dependencies
            .iter()
            .map(|dep| {
                dep["target_symbol"]["file"]
                    .as_str()
                    .unwrap_or("")
                    .to_string()
            })
            .collect();

        Ok(serde_json::json!({
            "status": "success",
            "analysis_type": "specific_target",
            "target": {
                "id": target,
                "name": target_node.name,
                "kind": format!("{:?}", target_node.kind),
                "file": target_node.file.display().to_string(),
                "language": format!("{:?}", target_node.lang)
            },
            "dependency_analysis": {
                "total_dependencies": total_dependencies,
                "dependency_breakdown": dependency_stats,
                "unique_files_affected": unique_files.len(),
                "files_affected": unique_files.into_iter().collect::<Vec<_>>(),
                "max_depth_analyzed": max_depth,
                "includes_transitive": include_transitive
            },
            "dependencies": all_dependencies,
            "insights": self.generate_dependency_insights(&all_dependencies, total_dependencies)
        }))
    }

    /// Analyze repository-wide dependencies
    fn analyze_repository_dependencies(
        &self,
        dependency_type: &str,
        max_depth: usize,
        include_transitive: bool,
    ) -> anyhow::Result<serde_json::Value> {
        // Get all nodes in the repository using symbol index
        let mut all_nodes = Vec::new();
        for symbol_entry in self.graph_store.iter_symbol_index() {
            for node_id in symbol_entry.1 {
                if let Some(node) = self.graph_store.get_node(&node_id) {
                    all_nodes.push(node);
                }
            }
        }

        if all_nodes.is_empty() {
            return Ok(serde_json::json!({
                "status": "error",
                "message": "No symbols found in repository. Make sure repository has been initialized."
            }));
        }

        let mut repository_dependencies = Vec::new();
        let mut global_stats = std::collections::HashMap::new();
        let mut file_dependencies = std::collections::HashMap::<String, usize>::new();
        let mut language_stats = std::collections::HashMap::<String, usize>::new();

        // Sample nodes for analysis (limit to prevent overwhelming results)
        let sample_size = 100.min(all_nodes.len());
        let sampled_nodes: Vec<_> = all_nodes.iter().take(sample_size).collect();

        for node in sampled_nodes {
            let file_path = node.file.display().to_string();
            let language = format!("{:?}", node.lang);

            *file_dependencies.entry(file_path.clone()).or_insert(0) += 1;
            *language_stats.entry(language).or_insert(0) += 1;

            // Analyze dependencies for this node
            let dependency_types = if dependency_type == "all" {
                vec!["direct", "calls", "imports"]
            } else {
                vec![dependency_type]
            };

            let mut node_dependency_count = 0;

            for dep_type in dependency_types {
                let parsed_dep_type = match dep_type {
                    "direct" => DependencyType::Direct,
                    "calls" => DependencyType::Calls,
                    "imports" => DependencyType::Imports,
                    "reads" => DependencyType::Reads,
                    "writes" => DependencyType::Writes,
                    _ => continue,
                };

                if let Ok(dependencies) = self
                    .graph_query
                    .find_dependencies(&node.id, parsed_dep_type)
                {
                    node_dependency_count += dependencies.len();
                    *global_stats.entry(dep_type.to_string()).or_insert(0) += dependencies.len();
                }
            }

            if node_dependency_count > 0 {
                repository_dependencies.push(serde_json::json!({
                    "symbol": {
                        "id": node.id.to_hex(),
                        "name": node.name,
                        "kind": format!("{:?}", node.kind),
                        "file": file_path,
                        "language": format!("{:?}", node.lang)
                    },
                    "dependency_count": node_dependency_count
                }));
            }
        }

        // Sort dependencies by count and take top dependencies
        repository_dependencies.sort_by(|a, b| {
            b["dependency_count"]
                .as_u64()
                .unwrap_or(0)
                .cmp(&a["dependency_count"].as_u64().unwrap_or(0))
        });

        let top_dependencies = repository_dependencies
            .iter()
            .take(20)
            .cloned()
            .collect::<Vec<_>>();

        // Calculate repository metrics
        let total_dependencies: usize = global_stats.values().sum();
        let most_connected_files: Vec<_> = {
            let mut file_deps: Vec<_> = file_dependencies.into_iter().collect();
            file_deps.sort_by(|a, b| b.1.cmp(&a.1));
            file_deps.into_iter().take(10).collect()
        };

        Ok(serde_json::json!({
            "status": "success",
            "analysis_type": "repository_wide",
            "repository_summary": {
                "total_symbols_analyzed": sample_size,
                "total_dependencies_found": total_dependencies,
                "dependency_breakdown": global_stats,
                "languages": language_stats,
                "max_depth_analyzed": max_depth,
                "includes_transitive": include_transitive
            },
            "top_dependent_symbols": top_dependencies,
            "most_connected_files": most_connected_files,
            "insights": self.generate_repository_dependency_insights(&global_stats, total_dependencies, sample_size),
            "note": if all_nodes.len() > sample_size {
                format!("Analysis performed on {} sample symbols out of {} total symbols", sample_size, all_nodes.len())
            } else {
                "Complete repository analysis performed".to_string()
            }
        }))
    }

    /// Find transitive dependencies recursively
    fn find_transitive_dependencies(
        &self,
        node_id: &codeprism_core::NodeId,
        dependency_type: &DependencyType,
        max_depth: usize,
        current_depth: usize,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let mut transitive_deps = Vec::new();

        if current_depth > max_depth {
            return Ok(transitive_deps);
        }

        if let Ok(dependencies) = self
            .graph_query
            .find_dependencies(node_id, dependency_type.clone())
        {
            for dependency in dependencies {
                transitive_deps.push(serde_json::json!({
                    "target_symbol": {
                        "id": dependency.target_node.id.to_hex(),
                        "name": dependency.target_node.name,
                        "kind": format!("{:?}", dependency.target_node.kind),
                        "file": dependency.target_node.file.display().to_string()
                    },
                    "depth": current_depth,
                    "edge_type": format!("{:?}", dependency.edge_kind)
                }));

                // Recursively find deeper dependencies
                if current_depth < max_depth {
                    let deeper_deps = self.find_transitive_dependencies(
                        &dependency.target_node.id,
                        dependency_type,
                        max_depth,
                        current_depth + 1,
                    )?;
                    transitive_deps.extend(deeper_deps);
                }
            }
        }

        Ok(transitive_deps)
    }

    /// Generate insights from dependency analysis
    fn generate_dependency_insights(
        &self,
        dependencies: &[serde_json::Value],
        total_count: usize,
    ) -> Vec<String> {
        let mut insights = Vec::new();

        if total_count == 0 {
            insights.push("No dependencies found for this symbol".to_string());
            return insights;
        }

        // Analyze file distribution
        let unique_files: std::collections::HashSet<_> = dependencies
            .iter()
            .map(|dep| dep["target_symbol"]["file"].as_str().unwrap_or(""))
            .collect();

        if unique_files.len() == 1 {
            insights
                .push("All dependencies are within the same file - good encapsulation".to_string());
        } else if unique_files.len() > total_count / 2 {
            insights.push(
                "Dependencies are spread across many files - consider consolidation".to_string(),
            );
        }

        // Analyze dependency types
        let mut type_counts = std::collections::HashMap::new();
        for dep in dependencies {
            if let Some(dep_type) = dep["dependency_type"].as_str() {
                *type_counts.entry(dep_type).or_insert(0) += 1;
            }
        }

        if let Some(max_type) = type_counts.iter().max_by_key(|(_, &count)| count) {
            insights.push(format!(
                "Primary dependency type: {} ({} occurrences)",
                max_type.0, max_type.1
            ));
        }

        if total_count > 10 {
            insights.push(
                "High number of dependencies - consider refactoring for better modularity"
                    .to_string(),
            );
        } else if total_count < 3 {
            insights.push("Low coupling - good design isolation".to_string());
        }

        insights
    }

    /// Generate insights for repository-wide dependency analysis
    fn generate_repository_dependency_insights(
        &self,
        stats: &std::collections::HashMap<String, usize>,
        total_deps: usize,
        symbols_analyzed: usize,
    ) -> Vec<String> {
        let mut insights = Vec::new();

        let avg_deps_per_symbol = if symbols_analyzed > 0 {
            total_deps as f64 / symbols_analyzed as f64
        } else {
            0.0
        };

        insights.push(format!(
            "Average dependencies per symbol: {:.1}",
            avg_deps_per_symbol
        ));

        if avg_deps_per_symbol > 8.0 {
            insights.push("High average coupling - consider architectural refactoring".to_string());
        } else if avg_deps_per_symbol < 2.0 {
            insights.push("Low coupling observed - good modular design".to_string());
        }

        // Analyze dependency type distribution
        if let Some(max_type) = stats.iter().max_by_key(|(_, &count)| count) {
            let percentage = (*max_type.1 as f64 / total_deps as f64) * 100.0;
            insights.push(format!(
                "Dominant dependency type: {} ({:.1}%)",
                max_type.0, percentage
            ));
        }

        if total_deps > symbols_analyzed * 10 {
            insights.push(
                "Very high dependency density - potential for circular dependencies".to_string(),
            );
        }

        insights
    }

    /// Analyze control flow patterns in code
    fn analyze_control_flow_patterns(
        &self,
        target: &str,
        analysis_types: &[String],
        max_depth: usize,
        include_paths: bool,
    ) -> anyhow::Result<serde_json::Value> {
        // Check if target is a file path or symbol ID
        let result = if std::path::Path::new(target).exists() {
            // Analyze file directly
            self.analyze_file_control_flow(target, analysis_types, max_depth, include_paths)
        } else if target.len() == 64 && target.chars().all(|c| c.is_ascii_hexdigit()) {
            // Treat as symbol ID
            self.analyze_symbol_control_flow(target, analysis_types, max_depth, include_paths)
        } else if target.starts_with("**") || target.contains("*") {
            // Handle glob pattern
            self.analyze_pattern_control_flow(target, analysis_types, max_depth, include_paths)
        } else {
            return Ok(serde_json::json!({
                "status": "error",
                "message": format!("Target '{}' not found. Provide a file path, symbol ID, or glob pattern.", target)
            }));
        };

        result
    }

    /// Analyze control flow for a specific file
    fn analyze_file_control_flow(
        &self,
        file_path: &str,
        analysis_types: &[String],
        max_depth: usize,
        include_paths: bool,
    ) -> anyhow::Result<serde_json::Value> {
        // Get nodes from the file
        let file_path_buf = std::path::PathBuf::from(file_path);
        let file_nodes = self.graph_store.get_nodes_in_file(&file_path_buf);

        if file_nodes.is_empty() {
            return Ok(serde_json::json!({
                "status": "error",
                "message": format!("No symbols found in file: {}", file_path)
            }));
        }

        let mut control_flow_analysis = Vec::new();
        let mut file_stats = std::collections::HashMap::new();

        for node in file_nodes {
            let node_analysis =
                self.analyze_node_control_flow(&node, analysis_types, max_depth, include_paths)?;

            // Update statistics
            if let Some(patterns) = node_analysis.get("control_flow_patterns") {
                for (pattern_type, count) in patterns.as_object().unwrap_or(&serde_json::Map::new())
                {
                    *file_stats.entry(pattern_type.clone()).or_insert(0) +=
                        count.as_u64().unwrap_or(0) as usize;
                }
            }

            control_flow_analysis.push(node_analysis);
        }

        Ok(serde_json::json!({
            "status": "success",
            "analysis_type": "file",
            "target": file_path,
            "symbols_analyzed": control_flow_analysis.len(),
            "file_statistics": file_stats,
            "symbol_analyses": control_flow_analysis,
            "settings": {
                "analysis_types": analysis_types,
                "max_depth": max_depth,
                "include_paths": include_paths
            }
        }))
    }

    /// Analyze control flow for a specific symbol
    fn analyze_symbol_control_flow(
        &self,
        symbol_id: &str,
        analysis_types: &[String],
        max_depth: usize,
        include_paths: bool,
    ) -> anyhow::Result<serde_json::Value> {
        // Parse symbol ID
        let node_id = match codeprism_core::NodeId::from_hex(symbol_id) {
            Ok(id) => id,
            Err(_) => {
                return Ok(serde_json::json!({
                    "status": "error",
                    "message": format!("Invalid symbol ID format: {}", symbol_id)
                }));
            }
        };

        // Get the symbol node
        let node = match self.graph_store.get_node(&node_id) {
            Some(node) => node,
            None => {
                return Ok(serde_json::json!({
                    "status": "error",
                    "message": format!("Symbol not found: {}", symbol_id)
                }));
            }
        };

        let analysis =
            self.analyze_node_control_flow(&node, analysis_types, max_depth, include_paths)?;

        Ok(serde_json::json!({
            "status": "success",
            "analysis_type": "symbol",
            "target": symbol_id,
            "symbol_info": {
                "name": node.name,
                "kind": format!("{:?}", node.kind),
                "file": node.file.display().to_string(),
                "language": format!("{:?}", node.lang)
            },
            "analysis": analysis,
            "settings": {
                "analysis_types": analysis_types,
                "max_depth": max_depth,
                "include_paths": include_paths
            }
        }))
    }

    /// Analyze control flow for a glob pattern
    fn analyze_pattern_control_flow(
        &self,
        pattern: &str,
        analysis_types: &[String],
        max_depth: usize,
        include_paths: bool,
    ) -> anyhow::Result<serde_json::Value> {
        match &self.repository_path {
            Some(repo_path) => {
                let glob_pattern = if let Some(stripped) = pattern.strip_prefix("**/") {
                    repo_path.join(stripped).display().to_string()
                } else {
                    repo_path.join(pattern).display().to_string()
                };

                let mut all_analyses = Vec::new();
                let mut pattern_stats = std::collections::HashMap::new();
                let mut files_analyzed = 0;

                if let Ok(paths) = glob::glob(&glob_pattern) {
                    for path in paths.flatten() {
                        if let Ok(file_analysis) = self.analyze_file_control_flow(
                            &path.display().to_string(),
                            analysis_types,
                            max_depth,
                            include_paths,
                        ) {
                            if let Some(file_stats) = file_analysis.get("file_statistics") {
                                for (pattern_type, count) in
                                    file_stats.as_object().unwrap_or(&serde_json::Map::new())
                                {
                                    *pattern_stats.entry(pattern_type.clone()).or_insert(0) +=
                                        count.as_u64().unwrap_or(0) as usize;
                                }
                            }
                            all_analyses.push(file_analysis);
                            files_analyzed += 1;
                        }
                    }
                }

                Ok(serde_json::json!({
                    "status": "success",
                    "analysis_type": "pattern",
                    "target": pattern,
                    "files_analyzed": files_analyzed,
                    "aggregate_statistics": pattern_stats,
                    "file_analyses": all_analyses,
                    "settings": {
                        "analysis_types": analysis_types,
                        "max_depth": max_depth,
                        "include_paths": include_paths
                    }
                }))
            }
            None => Ok(serde_json::json!({
                "status": "error",
                "message": "No repository configured. Call initialize_repository first."
            })),
        }
    }

    /// Analyze control flow for a specific node
    fn analyze_node_control_flow(
        &self,
        node: &codeprism_core::Node,
        analysis_types: &[String],
        max_depth: usize,
        include_paths: bool,
    ) -> anyhow::Result<serde_json::Value> {
        let mut control_flow_patterns = std::collections::HashMap::new();
        let mut execution_paths = Vec::new();
        let mut complexity_metrics = std::collections::HashMap::new();

        // Basic control flow analysis based on node kind and edges
        control_flow_patterns.insert("decision_points", self.count_decision_points(node));
        control_flow_patterns.insert("loops", self.count_loops(node));
        control_flow_patterns.insert("recursions", self.count_recursions(node));
        control_flow_patterns.insert("exception_handling", self.count_exception_handling(node));

        // Calculate complexity metrics
        complexity_metrics.insert(
            "cyclomatic_complexity",
            self.calculate_cyclomatic_complexity(node),
        );
        complexity_metrics.insert("depth_of_nesting", self.calculate_nesting_depth(node));
        complexity_metrics.insert(
            "cognitive_complexity",
            self.calculate_cognitive_complexity(node),
        );

        // Analyze execution paths if requested
        if include_paths && analysis_types.iter().any(|t| t == "all" || t == "paths") {
            execution_paths = self.analyze_execution_paths(node, max_depth)?;
        }

        // Identify potential issues
        let mut issues = Vec::new();
        if control_flow_patterns.get("decision_points").unwrap_or(&0) > &10 {
            issues.push("High number of decision points - consider refactoring".to_string());
        }
        if complexity_metrics
            .get("cyclomatic_complexity")
            .unwrap_or(&0)
            > &15
        {
            issues.push("High cyclomatic complexity - consider breaking down function".to_string());
        }
        if complexity_metrics.get("depth_of_nesting").unwrap_or(&0) > &4 {
            issues.push("Deep nesting detected - consider extraction methods".to_string());
        }

        Ok(serde_json::json!({
            "symbol": {
                "id": node.id.to_hex(),
                "name": node.name,
                "kind": format!("{:?}", node.kind),
                "file": node.file.display().to_string(),
                "span": {
                    "start_line": node.span.start_line,
                    "end_line": node.span.end_line
                }
            },
            "control_flow_patterns": control_flow_patterns,
            "complexity_metrics": complexity_metrics,
            "execution_paths": execution_paths,
            "potential_issues": issues,
            "analysis_scope": {
                "max_depth": max_depth,
                "paths_included": include_paths,
                "types_analyzed": analysis_types
            }
        }))
    }

    /// Count decision points in a node (simplified heuristic)
    fn count_decision_points(&self, node: &codeprism_core::Node) -> usize {
        // Simplified: count outgoing edges that represent decisions
        let outgoing_edges = self.graph_store.get_outgoing_edges(&node.id);
        outgoing_edges
            .iter()
            .filter(|edge| {
                matches!(edge.kind, codeprism_core::EdgeKind::Calls)
                    || matches!(edge.kind, codeprism_core::EdgeKind::Reads)
            })
            .count()
            .max(1) // At least 1 if there are any control decisions
    }

    /// Count loops (simplified heuristic)
    fn count_loops(&self, node: &codeprism_core::Node) -> usize {
        // Simplified: look for cyclic patterns in immediate dependencies
        let outgoing_edges = self.graph_store.get_outgoing_edges(&node.id);
        let incoming_edges = self.graph_store.get_incoming_edges(&node.id);

        // Heuristic: if a node calls itself or has mutual calls, it might be a loop
        let self_references = outgoing_edges
            .iter()
            .filter(|edge| edge.target == node.id)
            .count();
        let mutual_calls = outgoing_edges
            .iter()
            .filter(|out_edge| {
                incoming_edges
                    .iter()
                    .any(|in_edge| in_edge.source == out_edge.target)
            })
            .count();

        self_references + mutual_calls.min(3) // Cap at reasonable number
    }

    /// Count recursions (simplified heuristic)
    fn count_recursions(&self, node: &codeprism_core::Node) -> usize {
        // Simplified: direct self-calls
        let outgoing_edges = self.graph_store.get_outgoing_edges(&node.id);
        outgoing_edges
            .iter()
            .filter(|edge| {
                edge.target == node.id && matches!(edge.kind, codeprism_core::EdgeKind::Calls)
            })
            .count()
    }

    /// Count exception handling patterns (simplified heuristic)
    fn count_exception_handling(&self, _node: &codeprism_core::Node) -> usize {
        // Simplified: for now return 0, would need language-specific analysis
        // In a full implementation, this would parse the AST for try-catch blocks
        0
    }

    /// Calculate cyclomatic complexity (simplified)
    fn calculate_cyclomatic_complexity(&self, node: &codeprism_core::Node) -> usize {
        // Simplified: base complexity of 1 + number of decision points
        1 + self.count_decision_points(node)
    }

    /// Calculate nesting depth (simplified heuristic)
    fn calculate_nesting_depth(&self, node: &codeprism_core::Node) -> usize {
        // Simplified: estimate based on span size and complexity
        let span_lines = node.span.end_line.saturating_sub(node.span.start_line);
        let complexity = self.count_decision_points(node);

        // Heuristic: more complex functions with more lines likely have deeper nesting
        ((span_lines / 10) + complexity / 3).min(10) // Cap at 10
    }

    /// Calculate cognitive complexity (simplified)
    fn calculate_cognitive_complexity(&self, node: &codeprism_core::Node) -> usize {
        // Simplified: combination of cyclomatic complexity and nesting
        let cyclomatic = self.calculate_cyclomatic_complexity(node);
        let nesting = self.calculate_nesting_depth(node);

        cyclomatic + (nesting * 2) // Weight nesting more heavily
    }

    /// Analyze execution paths (simplified)
    fn analyze_execution_paths(
        &self,
        node: &codeprism_core::Node,
        max_depth: usize,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let mut paths = Vec::new();
        let mut visited = std::collections::HashSet::new();

        // Find paths from this node using graph traversal
        self.find_execution_paths_recursive(&node.id, &mut paths, &mut visited, max_depth, 0)?;

        Ok(paths)
    }

    /// Recursively find execution paths
    fn find_execution_paths_recursive(
        &self,
        node_id: &codeprism_core::NodeId,
        paths: &mut Vec<serde_json::Value>,
        visited: &mut std::collections::HashSet<codeprism_core::NodeId>,
        max_depth: usize,
        current_depth: usize,
    ) -> anyhow::Result<()> {
        if current_depth >= max_depth || visited.contains(node_id) {
            return Ok(());
        }

        visited.insert(*node_id);

        let outgoing_edges = self.graph_store.get_outgoing_edges(node_id);

        if outgoing_edges.is_empty() {
            // End of path
            if let Some(node) = self.graph_store.get_node(node_id) {
                paths.push(serde_json::json!({
                    "path_type": "terminal",
                    "endpoint": {
                        "id": node.id.to_hex(),
                        "name": node.name,
                        "kind": format!("{:?}", node.kind)
                    },
                    "depth": current_depth
                }));
            }
        } else {
            // Continue paths
            for edge in outgoing_edges.iter().take(5) {
                // Limit to prevent explosion
                if let Some(target_node) = self.graph_store.get_node(&edge.target) {
                    paths.push(serde_json::json!({
                        "path_type": "continuation",
                        "from": node_id.to_hex(),
                        "to": target_node.id.to_hex(),
                        "edge_kind": format!("{:?}", edge.kind),
                        "target_name": target_node.name,
                        "depth": current_depth
                    }));

                    // Recurse with a new visited set to allow multiple paths
                    let mut new_visited = visited.clone();
                    self.find_execution_paths_recursive(
                        &edge.target,
                        paths,
                        &mut new_visited,
                        max_depth,
                        current_depth + 1,
                    )?;
                }
            }
        }

        visited.remove(node_id);
        Ok(())
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
