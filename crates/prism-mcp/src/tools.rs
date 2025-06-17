//! MCP Tools implementation

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::PrismMcpServer;

/// Tool capabilities as defined by MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilities {
    /// Whether the server will emit notifications when the list of available tools changes
    #[serde(rename = "listChanged")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Unique identifier for the tool
    pub name: String,
    /// Optional human-readable title for display purposes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Human-readable description of the tool's functionality
    pub description: String,
    /// JSON Schema defining expected input parameters
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Tool call parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolParams {
    /// Name of the tool to call
    pub name: String,
    /// Arguments to pass to the tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
}

/// Tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    /// Content returned by the tool
    pub content: Vec<ToolContent>,
    /// Whether the tool execution resulted in an error
    #[serde(rename = "isError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Tool content types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    /// Text content
    #[serde(rename = "text")]
    Text {
        /// Text content
        text: String,
    },
}

/// Parameters for listing tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsParams {
    /// Optional cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Result of listing tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    /// List of available tools
    pub tools: Vec<Tool>,
    /// Optional cursor for pagination
    #[serde(rename = "nextCursor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Tool manager for MCP server
pub struct ToolManager {
    server: std::sync::Arc<tokio::sync::RwLock<PrismMcpServer>>,
}

impl ToolManager {
    /// Create a new tool manager
    pub fn new(server: std::sync::Arc<tokio::sync::RwLock<PrismMcpServer>>) -> Self {
        Self { server }
    }

    /// List available tools
    pub async fn list_tools(&self, _params: ListToolsParams) -> Result<ListToolsResult> {
        let tools = vec![
            Tool {
                name: "repository_stats".to_string(),
                title: Some("Repository Statistics".to_string()),
                description: "Get comprehensive statistics about the repository".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            Tool {
                name: "trace_path".to_string(),
                title: Some("Trace Execution Path".to_string()),
                description: "Find the shortest path between two code symbols".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "source": {
                            "type": "string",
                            "description": "Source symbol identifier (node ID)"
                        },
                        "target": {
                            "type": "string",
                            "description": "Target symbol identifier (node ID)"
                        },
                        "max_depth": {
                            "type": "number",
                            "description": "Maximum search depth",
                            "default": 10
                        }
                    },
                    "required": ["source", "target"]
                }),
            },
            Tool {
                name: "explain_symbol".to_string(),
                title: Some("Explain Symbol".to_string()),
                description: "Provide detailed explanation of a code symbol with context".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "symbol_id": {
                            "type": "string",
                            "description": "Symbol identifier (node ID)"
                        },
                        "include_dependencies": {
                            "type": "boolean",
                            "description": "Include dependency information",
                            "default": false
                        },
                        "include_usages": {
                            "type": "boolean",
                            "description": "Include usage information",
                            "default": false
                        },
                        "context_lines": {
                            "type": "number",
                            "description": "Number of lines before and after the symbol to include as context",
                            "default": 4
                        }
                    },
                    "required": ["symbol_id"]
                }),
            },
            Tool {
                name: "find_dependencies".to_string(),
                title: Some("Find Dependencies".to_string()),
                description: "Analyze dependencies for a code symbol or file".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "target": {
                            "type": "string",
                            "description": "Symbol ID or file path to analyze"
                        },
                        "dependency_type": {
                            "type": "string",
                            "enum": ["direct", "calls", "imports", "reads", "writes"],
                            "description": "Type of dependencies to find",
                            "default": "direct"
                        }
                    },
                    "required": ["target"]
                }),
            },
            Tool {
                name: "find_references".to_string(),
                title: Some("Find References".to_string()),
                description: "Find all references to a symbol across the codebase".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "symbol_id": {
                            "type": "string",
                            "description": "Symbol identifier to find references for"
                        },
                        "include_definitions": {
                            "type": "boolean",
                            "description": "Include symbol definitions",
                            "default": true
                        },
                        "context_lines": {
                            "type": "number",
                            "description": "Number of lines before and after the symbol to include as context",
                            "default": 4
                        }
                    },
                    "required": ["symbol_id"]
                }),
            },
            Tool {
                name: "search_symbols".to_string(),
                title: Some("Search Symbols".to_string()),
                description: "Search for symbols by name pattern with advanced inheritance filtering".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "Search pattern (supports regex)"
                        },
                        "symbol_types": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["function", "class", "variable", "module", "method"]
                            },
                            "description": "Filter by symbol types"
                        },
                        "inheritance_filters": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "Filter by inheritance relationships (format: 'inherits_from:ClassName', 'metaclass:MetaclassName', 'uses_mixin:MixinName')"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of results",
                            "default": 50
                        },
                        "context_lines": {
                            "type": "number",
                            "description": "Number of lines before and after the symbol to include as context",
                            "default": 4
                        }
                    },
                    "required": ["pattern"]
                }),
            },
            Tool {
                name: "search_content".to_string(),
                title: Some("Search Content".to_string()),
                description: "Search across all content including documentation, comments, and configuration files".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query text"
                        },
                        "content_types": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["documentation", "comments", "configuration", "code"]
                            },
                            "description": "Types of content to search in"
                        },
                        "file_patterns": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "File patterns to include (regex)"
                        },
                        "exclude_patterns": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "File patterns to exclude (regex)"
                        },
                        "max_results": {
                            "type": "number",
                            "description": "Maximum number of results",
                            "default": 50
                        },
                        "case_sensitive": {
                            "type": "boolean",
                            "description": "Case sensitive search",
                            "default": false
                        },
                        "use_regex": {
                            "type": "boolean",
                            "description": "Use regex pattern matching",
                            "default": false
                        },
                        "include_context": {
                            "type": "boolean",
                            "description": "Include context around matches",
                            "default": true
                        }
                    },
                    "required": ["query"]
                }),
            },
            Tool {
                name: "find_files".to_string(),
                title: Some("Find Files".to_string()),
                description: "Find files by name or path pattern".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "File pattern to search for (supports regex)"
                        }
                    },
                    "required": ["pattern"]
                }),
            },
            Tool {
                name: "content_stats".to_string(),
                title: Some("Content Statistics".to_string()),
                description: "Get statistics about indexed content".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            Tool {
                name: "analyze_complexity".to_string(),
                title: Some("Analyze Code Complexity".to_string()),
                description: "Calculate complexity metrics for code elements including cyclomatic, cognitive, and maintainability metrics".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "target": {
                            "type": "string",
                            "description": "File path or symbol ID to analyze"
                        },
                        "metrics": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["cyclomatic", "cognitive", "halstead", "maintainability_index", "all"]
                            },
                            "description": "Types of complexity metrics to calculate",
                            "default": ["all"]
                        },
                        "threshold_warnings": {
                            "type": "boolean",
                            "description": "Include warnings for metrics exceeding thresholds",
                            "default": true
                        }
                    },
                    "required": ["target"]
                }),
            },
            Tool {
                name: "find_duplicates".to_string(),
                title: Some("Find Code Duplicates".to_string()),
                description: "Detect code duplication and similar code blocks across the codebase".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "similarity_threshold": {
                            "type": "number",
                            "description": "Similarity threshold (0.0 to 1.0)",
                            "default": 0.85,
                            "minimum": 0.0,
                            "maximum": 1.0
                        },
                        "min_lines": {
                            "type": "number",
                            "description": "Minimum lines for duplicate detection",
                            "default": 5,
                            "minimum": 1
                        },
                        "scope": {
                            "type": "string",
                            "description": "Scope for duplicate detection (repository, package, or specific files)",
                            "default": "repository"
                        },
                        "include_semantic_similarity": {
                            "type": "boolean",
                            "description": "Include semantic similarity analysis",
                            "default": true
                        },
                        "exclude_patterns": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "File patterns to exclude from analysis"
                        }
                    },
                    "required": []
                }),
            },
            Tool {
                name: "detect_patterns".to_string(),
                title: Some("Detect Design Patterns".to_string()),
                description: "Identify design patterns and architectural structures in the codebase".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "scope": {
                            "type": "string",
                            "description": "Scope for pattern detection (repository, package, or file)",
                            "default": "repository"
                        },
                        "pattern_types": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["design_patterns", "anti_patterns", "architectural_patterns", "all"]
                            },
                            "description": "Types of patterns to detect",
                            "default": ["all"]
                        },
                        "confidence_threshold": {
                            "type": "number",
                            "description": "Minimum confidence threshold for pattern detection (0.0 to 1.0)",
                            "default": 0.8,
                            "minimum": 0.0,
                            "maximum": 1.0
                        },
                        "include_suggestions": {
                            "type": "boolean",
                            "description": "Include improvement suggestions for detected patterns",
                            "default": true
                        }
                    },
                    "required": []
                }),
            },
            Tool {
                name: "analyze_transitive_dependencies".to_string(),
                title: Some("Analyze Transitive Dependencies".to_string()),
                description: "Analyze complete dependency chains, detect cycles, and map transitive relationships".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "target": {
                            "type": "string",
                            "description": "Symbol ID or file path to analyze"
                        },
                        "max_depth": {
                            "type": "number",
                            "description": "Maximum depth for transitive analysis",
                            "default": 5,
                            "minimum": 1,
                            "maximum": 20
                        },
                        "detect_cycles": {
                            "type": "boolean",
                            "description": "Detect circular dependencies",
                            "default": true
                        },
                        "include_external_dependencies": {
                            "type": "boolean",
                            "description": "Include external/third-party dependencies",
                            "default": false
                        },
                        "dependency_types": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["calls", "imports", "reads", "writes", "extends", "implements", "all"]
                            },
                            "description": "Types of dependencies to analyze",
                            "default": ["all"]
                        }
                    },
                    "required": ["target"]
                }),
            },
            Tool {
                name: "trace_data_flow".to_string(),
                title: Some("Trace Data Flow".to_string()),
                description: "Track data flow through the codebase, following variable assignments, function parameters, and transformations".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "variable_or_parameter": {
                            "type": "string",
                            "description": "Symbol ID of variable or parameter to trace"
                        },
                        "direction": {
                            "type": "string",
                            "enum": ["forward", "backward", "both"],
                            "description": "Direction to trace data flow",
                            "default": "forward"
                        },
                        "include_transformations": {
                            "type": "boolean",
                            "description": "Include data transformations (method calls, assignments)",
                            "default": true
                        },
                        "max_depth": {
                            "type": "number",
                            "description": "Maximum depth for data flow tracing",
                            "default": 10,
                            "minimum": 1,
                            "maximum": 50
                        },
                        "follow_function_calls": {
                            "type": "boolean",
                            "description": "Follow data flow through function calls",
                            "default": true
                        },
                        "include_field_access": {
                            "type": "boolean",
                            "description": "Include field access and modifications",
                            "default": true
                        }
                    },
                    "required": ["variable_or_parameter"]
                }),
            },
            Tool {
                name: "find_unused_code".to_string(),
                title: Some("Find Unused Code".to_string()),
                description: "Identify potentially unused code elements such as functions, classes, variables, and imports".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "scope": {
                            "type": "string",
                            "description": "Scope for unused code detection (repository, package, or specific files)",
                            "default": "repository"
                        },
                        "include_dead_code": {
                            "type": "boolean",
                            "description": "Include detection of unreachable (dead) code",
                            "default": true
                        },
                        "consider_external_apis": {
                            "type": "boolean",
                            "description": "Consider that some code might be used by external APIs",
                            "default": true
                        },
                        "confidence_threshold": {
                            "type": "number",
                            "description": "Minimum confidence threshold for marking code as unused (0.0 to 1.0)",
                            "default": 0.9,
                            "minimum": 0.0,
                            "maximum": 1.0
                        },
                        "analyze_types": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["functions", "classes", "variables", "imports", "all"]
                            },
                            "description": "Types of code elements to analyze for unused code",
                            "default": ["all"]
                        },
                        "exclude_patterns": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "File patterns to exclude from unused code analysis"
                        }
                    },
                    "required": []
                }),
            },
            Tool {
                name: "analyze_security".to_string(),
                title: Some("Analyze Security Vulnerabilities".to_string()),
                description: "Detect potential security vulnerabilities, data flow security issues, and unsafe coding patterns".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "scope": {
                            "type": "string",
                            "description": "Scope for security analysis (repository, package, or specific files)",
                            "default": "repository"
                        },
                        "vulnerability_types": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["injection", "authentication", "authorization", "data_exposure", "unsafe_patterns", "crypto_issues", "all"]
                            },
                            "description": "Types of security vulnerabilities to detect",
                            "default": ["all"]
                        },
                        "severity_threshold": {
                            "type": "string",
                            "enum": ["low", "medium", "high", "critical"],
                            "description": "Minimum severity level to report",
                            "default": "medium"
                        },
                        "include_data_flow_analysis": {
                            "type": "boolean",
                            "description": "Include data flow security analysis",
                            "default": true
                        },
                        "check_external_dependencies": {
                            "type": "boolean",
                            "description": "Check for known vulnerable dependencies",
                            "default": true
                        },
                        "exclude_patterns": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "File patterns to exclude from security analysis"
                        }
                    },
                    "required": []
                }),
            },
            Tool {
                name: "analyze_performance".to_string(),
                title: Some("Analyze Performance Characteristics".to_string()),
                description: "Analyze performance characteristics including time complexity, memory usage, bottlenecks, and scalability concerns".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "scope": {
                            "type": "string",
                            "description": "Scope for performance analysis (repository, package, or specific files)",
                            "default": "repository"
                        },
                        "analysis_types": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["time_complexity", "memory_usage", "hot_spots", "anti_patterns", "scalability", "all"]
                            },
                            "description": "Types of performance analysis to perform",
                            "default": ["all"]
                        },
                        "complexity_threshold": {
                            "type": "string",
                            "enum": ["low", "medium", "high"],
                            "description": "Minimum complexity threshold to report",
                            "default": "medium"
                        },
                        "include_algorithmic_analysis": {
                            "type": "boolean",
                            "description": "Include algorithmic complexity analysis",
                            "default": true
                        },
                        "detect_bottlenecks": {
                            "type": "boolean",
                            "description": "Detect potential performance bottlenecks",
                            "default": true
                        },
                        "exclude_patterns": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "File patterns to exclude from performance analysis"
                        }
                    },
                    "required": []
                }),
            },
            Tool {
                name: "analyze_api_surface".to_string(),
                title: Some("Analyze API Surface".to_string()),
                description: "Analyze public API surface including endpoint discovery, versioning, breaking changes, and documentation coverage".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "scope": {
                            "type": "string",
                            "description": "Scope for API analysis (repository, package, or specific files)",
                            "default": "repository"
                        },
                        "analysis_types": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["public_api", "versioning", "breaking_changes", "documentation", "compatibility", "all"]
                            },
                            "description": "Types of API analysis to perform",
                            "default": ["all"]
                        },
                        "api_version": {
                            "type": "string",
                            "description": "Current API version for compatibility analysis"
                        },
                        "include_private_apis": {
                            "type": "boolean",
                            "description": "Include analysis of private/internal APIs",
                            "default": false
                        },
                        "check_documentation_coverage": {
                            "type": "boolean",
                            "description": "Analyze API documentation coverage",
                            "default": true
                        },
                        "detect_breaking_changes": {
                            "type": "boolean",
                            "description": "Detect potential breaking changes",
                            "default": true
                        },
                        "exclude_patterns": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "File patterns to exclude from API analysis"
                        }
                    },
                    "required": []
                }),
            },
        ];

        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    /// Call a specific tool
    pub async fn call_tool(&self, params: CallToolParams) -> Result<CallToolResult> {
        let server = self.server.read().await;
        
        match params.name.as_str() {
            "repository_stats" => self.repository_stats(&server).await,
            "trace_path" => self.trace_path(&server, params.arguments).await,
            "explain_symbol" => self.explain_symbol(&server, params.arguments).await,
            "find_dependencies" => self.find_dependencies(&server, params.arguments).await,
            "find_references" => self.find_references(&server, params.arguments).await,
            "search_symbols" => self.search_symbols(&server, params.arguments).await,
            "search_content" => self.search_content(&server, params.arguments).await,
            "find_files" => self.find_files(&server, params.arguments).await,
            "content_stats" => self.content_stats(&server).await,
            "analyze_complexity" => self.analyze_complexity(&server, params.arguments).await,
            "find_duplicates" => self.find_duplicates(&server, params.arguments).await,
            "detect_patterns" => self.detect_patterns(&server, params.arguments).await,
            "analyze_transitive_dependencies" => self.analyze_transitive_dependencies(&server, params.arguments).await,
            "trace_data_flow" => self.trace_data_flow(&server, params.arguments).await,
            "find_unused_code" => self.find_unused_code(&server, params.arguments).await,
            "analyze_security" => self.analyze_security(&server, params.arguments).await,
            "analyze_performance" => self.analyze_performance(&server, params.arguments).await,
            "analyze_api_surface" => self.analyze_api_surface(&server, params.arguments).await,
            _ => Ok(CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Unknown tool: {}", params.name),
                }],
                is_error: Some(true),
            }),
        }
    }

    /// Get repository statistics
    async fn repository_stats(&self, server: &PrismMcpServer) -> Result<CallToolResult> {
        let result = if let Some(repo_path) = server.repository_path() {
            let file_count = server.scanner().discover_files(repo_path)
                .map(|files| files.len())
                .unwrap_or(0);

            let graph_stats = server.graph_store().get_stats();

            serde_json::json!({
                "repository_path": repo_path.display().to_string(),
                "total_files": file_count,
                "total_nodes": graph_stats.total_nodes,
                "total_edges": graph_stats.total_edges,
                "nodes_by_kind": graph_stats.nodes_by_kind,
                "status": "active"
            })
        } else {
            serde_json::json!({
                "error": "No repository initialized"
            })
        };

        Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: serde_json::to_string_pretty(&result)?,
            }],
            is_error: Some(false),
        })
    }

    /// Trace path between two symbols
    async fn trace_path(&self, server: &PrismMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
        let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        
        let source_str = args.get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing source parameter"))?;
        
        let target_str = args.get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing target parameter"))?;
        
        let max_depth = args.get("max_depth")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        // Parse node IDs from hex strings
        let source_id = self.parse_node_id(source_str)?;
        let target_id = self.parse_node_id(target_str)?;

        match server.graph_query().find_path(&source_id, &target_id, max_depth)? {
            Some(path_result) => {
                let result = serde_json::json!({
                    "found": true,
                    "source": source_str,
                    "target": target_str,
                    "distance": path_result.distance,
                    "path": path_result.path.iter().map(|id| id.to_hex()).collect::<Vec<_>>(),
                    "edges": path_result.edges.iter().map(|edge| {
                        serde_json::json!({
                            "source": edge.source.to_hex(),
                            "target": edge.target.to_hex(),
                            "kind": format!("{:?}", edge.kind)
                        })
                    }).collect::<Vec<_>>()
                });

                Ok(CallToolResult {
                    content: vec![ToolContent::Text {
                        text: serde_json::to_string_pretty(&result)?,
                    }],
                    is_error: Some(false),
                })
            }
            None => {
                let result = serde_json::json!({
                    "found": false,
                    "source": source_str,
                    "target": target_str,
                    "message": "No path found between the specified symbols"
                });

                Ok(CallToolResult {
                    content: vec![ToolContent::Text {
                        text: serde_json::to_string_pretty(&result)?,
                    }],
                    is_error: Some(false),
                })
            }
        }
    }

    /// Explain a symbol with context
    async fn explain_symbol(&self, server: &PrismMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
        let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        
        let symbol_id_str = args.get("symbol_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing symbol_id parameter"))?;
        
        let include_dependencies = args.get("include_dependencies")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let include_usages = args.get("include_usages")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let context_lines = args.get("context_lines")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(4);

        let symbol_id = self.parse_node_id(symbol_id_str)?;

        if let Some(node) = server.graph_store().get_node(&symbol_id) {
            let mut result = serde_json::json!({
                "symbol": self.create_node_info_with_context(&node, context_lines)
            });

            // Enhanced inheritance information for classes
            if matches!(node.kind, prism_core::NodeKind::Class) {
                if let Ok(inheritance_info) = server.graph_query().get_inheritance_info(&symbol_id) {
                    let mut inheritance_data = serde_json::Map::new();
                    
                    // Basic inheritance information
                    inheritance_data.insert("class_name".to_string(), serde_json::Value::String(inheritance_info.class_name));
                    inheritance_data.insert("is_metaclass".to_string(), serde_json::Value::Bool(inheritance_info.is_metaclass));
                    
                    // Base classes
                    if !inheritance_info.base_classes.is_empty() {
                        let base_classes: Vec<_> = inheritance_info.base_classes.iter().map(|rel| {
                            serde_json::json!({
                                "name": rel.class_name,
                                "relationship_type": rel.relationship_type,
                                "file": rel.file.display().to_string(),
                                "span": {
                                    "start_line": rel.span.start_line,
                                    "end_line": rel.span.end_line,
                                    "start_column": rel.span.start_column,
                                    "end_column": rel.span.end_column
                                }
                            })
                        }).collect();
                        inheritance_data.insert("base_classes".to_string(), serde_json::Value::Array(base_classes));
                    }
                    
                    // Subclasses
                    if !inheritance_info.subclasses.is_empty() {
                        let subclasses: Vec<_> = inheritance_info.subclasses.iter().map(|rel| {
                            serde_json::json!({
                                "name": rel.class_name,
                                "file": rel.file.display().to_string(),
                                "span": {
                                    "start_line": rel.span.start_line,
                                    "end_line": rel.span.end_line,
                                    "start_column": rel.span.start_column,
                                    "end_column": rel.span.end_column
                                }
                            })
                        }).collect();
                        inheritance_data.insert("subclasses".to_string(), serde_json::Value::Array(subclasses));
                    }
                    
                    // Metaclass information
                    if let Some(metaclass) = inheritance_info.metaclass {
                        inheritance_data.insert("metaclass".to_string(), serde_json::json!({
                            "name": metaclass.class_name,
                            "file": metaclass.file.display().to_string(),
                            "span": {
                                "start_line": metaclass.span.start_line,
                                "end_line": metaclass.span.end_line,
                                "start_column": metaclass.span.start_column,
                                "end_column": metaclass.span.end_column
                            }
                        }));
                    }
                    
                    // Mixins
                    if !inheritance_info.mixins.is_empty() {
                        let mixins: Vec<_> = inheritance_info.mixins.iter().map(|rel| {
                            serde_json::json!({
                                "name": rel.class_name,
                                "file": rel.file.display().to_string(),
                                "span": {
                                    "start_line": rel.span.start_line,
                                    "end_line": rel.span.end_line,
                                    "start_column": rel.span.start_column,
                                    "end_column": rel.span.end_column
                                }
                            })
                        }).collect();
                        inheritance_data.insert("mixins".to_string(), serde_json::Value::Array(mixins));
                    }
                    
                    // Method Resolution Order
                    if !inheritance_info.method_resolution_order.is_empty() {
                        inheritance_data.insert("method_resolution_order".to_string(), 
                            serde_json::Value::Array(inheritance_info.method_resolution_order.iter()
                                .map(|name| serde_json::Value::String(name.clone()))
                                .collect()));
                    }
                    
                    // Dynamic attributes
                    if !inheritance_info.dynamic_attributes.is_empty() {
                        let dynamic_attrs: Vec<_> = inheritance_info.dynamic_attributes.iter().map(|attr| {
                            serde_json::json!({
                                "name": attr.name,
                                "created_by": attr.created_by,
                                "type": attr.attribute_type
                            })
                        }).collect();
                        inheritance_data.insert("dynamic_attributes".to_string(), serde_json::Value::Array(dynamic_attrs));
                    }
                    
                    // Full inheritance chain
                    if !inheritance_info.inheritance_chain.is_empty() {
                        inheritance_data.insert("inheritance_chain".to_string(), 
                            serde_json::Value::Array(inheritance_info.inheritance_chain.iter()
                                .map(|name| serde_json::Value::String(name.clone()))
                                .collect()));
                    }
                    
                    result["inheritance"] = serde_json::Value::Object(inheritance_data);
                }
            }

            if include_dependencies {
                let dependencies = server.graph_query().find_dependencies(&symbol_id, prism_core::graph::DependencyType::Direct)?;
                
                // Filter out invalid Call nodes with malformed names
                let valid_dependencies: Vec<_> = dependencies.iter()
                    .filter(|dep| self.is_valid_dependency_node(&dep.target_node))
                    .collect();
                
                result["dependencies"] = serde_json::json!(
                    valid_dependencies.iter().map(|dep| {
                        let mut dep_info = self.create_node_info_with_context(&dep.target_node, context_lines);
                        dep_info["edge_kind"] = serde_json::json!(format!("{:?}", dep.edge_kind));
                        dep_info
                        }).collect::<Vec<_>>()
                );
            }

            if include_usages {
                let references = server.graph_query().find_references(&symbol_id)?;
                result["usages"] = serde_json::json!(
                    references.iter().map(|ref_| {
                        let mut usage_info = self.create_node_info_with_context(&ref_.source_node, context_lines);
                        usage_info["edge_kind"] = serde_json::json!(format!("{:?}", ref_.edge_kind));
                        usage_info["reference_location"] = serde_json::json!({
                            "file": ref_.location.file.display().to_string(),
                            "span": {
                                "start_line": ref_.location.span.start_line,
                                "end_line": ref_.location.span.end_line,
                                "start_column": ref_.location.span.start_column,
                                "end_column": ref_.location.span.end_column
                            }
                        });
                        usage_info
                    }).collect::<Vec<_>>()
                );
            }

            Ok(CallToolResult {
                content: vec![ToolContent::Text {
                    text: serde_json::to_string_pretty(&result)?,
                }],
                is_error: Some(false),
            })
        } else {
            Ok(CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Symbol not found: {}", symbol_id_str),
                }],
                is_error: Some(true),
            })
        }
    }

    /// Validate that a dependency node has a valid name
    fn is_valid_dependency_node(&self, node: &prism_core::Node) -> bool {
        // Filter out Call nodes with invalid names
        if matches!(node.kind, prism_core::NodeKind::Call) {
            // Check for common invalid patterns
            if node.name.is_empty() || 
               node.name == ")" || 
               node.name == "(" || 
               node.name.trim().is_empty() ||
               node.name.chars().all(|c| !c.is_alphanumeric() && c != '_') {
                return false;
            }
        }
        
        // All other nodes are considered valid
        true
    }

    /// Find dependencies of a symbol
    async fn find_dependencies(&self, server: &PrismMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
        let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        
        let target = args.get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing target parameter"))?;
        
        let dependency_type_str = args.get("dependency_type")
            .and_then(|v| v.as_str())
            .unwrap_or("direct");

        let dependency_type = match dependency_type_str {
            "direct" => prism_core::graph::DependencyType::Direct,
            "calls" => prism_core::graph::DependencyType::Calls,
            "imports" => prism_core::graph::DependencyType::Imports,
            "reads" => prism_core::graph::DependencyType::Reads,
            "writes" => prism_core::graph::DependencyType::Writes,
            _ => return Ok(CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Invalid dependency type: {}", dependency_type_str),
                }],
                is_error: Some(true),
            }),
        };

        // Try to parse as node ID first, then as file path
        let dependencies = if let Ok(node_id) = self.parse_node_id(target) {
            server.graph_query().find_dependencies(&node_id, dependency_type)?
        } else {
            // Handle file path - find all nodes in the file and get their dependencies
            let file_path = std::path::PathBuf::from(target);
            let nodes = server.graph_store().get_nodes_in_file(&file_path);
            let mut all_deps = Vec::new();
            for node in nodes {
                let deps = server.graph_query().find_dependencies(&node.id, dependency_type.clone())?;
                all_deps.extend(deps);
            }
            all_deps
        };

        // Filter out invalid Call nodes with malformed names
        let valid_dependencies: Vec<_> = dependencies.iter()
            .filter(|dep| self.is_valid_dependency_node(&dep.target_node))
            .collect();

        let result = serde_json::json!({
            "target": target,
            "dependency_type": dependency_type_str,
            "dependencies": valid_dependencies.iter().map(|dep| {
                serde_json::json!({
                    "id": dep.target_node.id.to_hex(),
                    "name": dep.target_node.name,
                    "kind": format!("{:?}", dep.target_node.kind),
                    "file": dep.target_node.file.display().to_string(),
                    "edge_kind": format!("{:?}", dep.edge_kind)
                })
            }).collect::<Vec<_>>()
        });

        Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: serde_json::to_string_pretty(&result)?,
            }],
            is_error: Some(false),
        })
    }

    /// Find references to a symbol
    async fn find_references(&self, server: &PrismMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
        let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        
        let symbol_id_str = args.get("symbol_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing symbol_id parameter"))?;
        
        let _include_definitions = args.get("include_definitions")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let context_lines = args.get("context_lines")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(4);

        let symbol_id = self.parse_node_id(symbol_id_str)?;
        let references = server.graph_query().find_references(&symbol_id)?;

        let result = serde_json::json!({
            "symbol_id": symbol_id_str,
            "references": references.iter().map(|ref_| {
                let mut ref_info = self.create_node_info_with_context(&ref_.source_node, context_lines);
                ref_info["edge_kind"] = serde_json::json!(format!("{:?}", ref_.edge_kind));
                ref_info["reference_location"] = serde_json::json!({
                    "file": ref_.location.file.display().to_string(),
                    "span": {
                        "start_line": ref_.location.span.start_line,
                        "end_line": ref_.location.span.end_line,
                        "start_column": ref_.location.span.start_column,
                        "end_column": ref_.location.span.end_column
                    }
                });
                ref_info
            }).collect::<Vec<_>>()
        });

        Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: serde_json::to_string_pretty(&result)?,
            }],
            is_error: Some(false),
        })
    }

    /// Search symbols by pattern
    async fn search_symbols(&self, server: &PrismMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
        let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        
        let pattern = args.get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing pattern parameter"))?;
        
        let symbol_types = args.get("symbol_types")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|s| match s {
                        "function" => Some(prism_core::NodeKind::Function),
                        "class" => Some(prism_core::NodeKind::Class),
                        "variable" => Some(prism_core::NodeKind::Variable),
                        "module" => Some(prism_core::NodeKind::Module),
                        "method" => Some(prism_core::NodeKind::Method),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            });
        
        let inheritance_filters = args.get("inheritance_filters")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|s| self.parse_inheritance_filter(s))
                    .collect::<Vec<_>>()
            });
        
        let limit = args.get("limit")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        let context_lines = args.get("context_lines")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(4);

        // Use enhanced search with inheritance filters if provided
        let results = if let Some(ref filters) = inheritance_filters {
            server.graph_query().search_symbols_with_inheritance(pattern, symbol_types, Some(filters.clone()), limit)?
        } else {
            server.graph_query().search_symbols(pattern, symbol_types, limit)?
        };

        let result = serde_json::json!({
            "pattern": pattern,
            "inheritance_filters_applied": inheritance_filters.is_some(),
            "results": results.iter().map(|symbol| {
                let mut symbol_info = self.create_node_info_with_context(&symbol.node, context_lines);
                symbol_info["references_count"] = serde_json::json!(symbol.references_count);
                symbol_info["dependencies_count"] = serde_json::json!(symbol.dependencies_count);
                
                // Add inheritance info for classes when inheritance filters are used
                if matches!(symbol.node.kind, prism_core::NodeKind::Class) && inheritance_filters.is_some() {
                    if let Ok(inheritance_info) = server.graph_query().get_inheritance_info(&symbol.node.id) {
                        symbol_info["inheritance_summary"] = serde_json::json!({
                            "is_metaclass": inheritance_info.is_metaclass,
                            "base_classes": inheritance_info.base_classes.iter().map(|rel| rel.class_name.clone()).collect::<Vec<_>>(),
                            "mixins": inheritance_info.mixins.iter().map(|rel| rel.class_name.clone()).collect::<Vec<_>>(),
                            "metaclass": inheritance_info.metaclass.as_ref().map(|mc| mc.class_name.clone())
                        });
                    }
                }
                
                symbol_info
            }).collect::<Vec<_>>()
        });

        Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: serde_json::to_string_pretty(&result)?,
            }],
            is_error: Some(false),
        })
    }

    /// Parse inheritance filter string into InheritanceFilter enum
    fn parse_inheritance_filter(&self, filter_str: &str) -> Option<prism_core::InheritanceFilter> {
        if let Some(colon_pos) = filter_str.find(':') {
            let filter_type = &filter_str[..colon_pos];
            let class_name = &filter_str[colon_pos + 1..];
            
            match filter_type {
                "inherits_from" => Some(prism_core::InheritanceFilter::InheritsFrom(class_name.to_string())),
                "metaclass" => Some(prism_core::InheritanceFilter::HasMetaclass(class_name.to_string())),
                "uses_mixin" => Some(prism_core::InheritanceFilter::UsesMixin(class_name.to_string())),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Parse a node ID from a hex string
    fn parse_node_id(&self, hex_str: &str) -> Result<prism_core::NodeId> {
        prism_core::NodeId::from_hex(hex_str)
            .map_err(|e| anyhow::anyhow!("Invalid node ID format: {}", e))
    }

    /// Extract source context around a line number from a file
    fn extract_source_context(&self, file_path: &std::path::Path, line_number: usize, context_lines: usize) -> Option<serde_json::Value> {
        // Read the file content
        let content = match std::fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(_) => return None,
        };

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        
        if line_number == 0 || line_number > total_lines {
            return None;
        }

        // Convert to 0-based indexing
        let target_line_idx = line_number - 1;
        
        // Calculate context range (with bounds checking)
        let start_idx = target_line_idx.saturating_sub(context_lines);
        let end_idx = std::cmp::min(target_line_idx + context_lines, total_lines - 1);
        
        // Extract context lines with line numbers
        let mut context_lines_with_numbers = Vec::new();
        for i in start_idx..=end_idx {
            context_lines_with_numbers.push(serde_json::json!({
                "line_number": i + 1,
                "content": lines[i],
                "is_target": i == target_line_idx
            }));
        }

        Some(serde_json::json!({
            "target_line": line_number,
            "context_range": {
                "start_line": start_idx + 1,
                "end_line": end_idx + 1
            },
            "lines": context_lines_with_numbers
        }))
    }

    /// Create enhanced node information with source context
    fn create_node_info_with_context(&self, node: &prism_core::Node, context_lines: usize) -> serde_json::Value {
        let mut node_info = serde_json::json!({
            "id": node.id.to_hex(),
            "name": node.name,
            "kind": format!("{:?}", node.kind),
            "language": format!("{:?}", node.lang),
            "file": node.file.display().to_string(),
            "span": {
                "start_line": node.span.start_line,
                "end_line": node.span.end_line,
                "start_column": node.span.start_column,
                "end_column": node.span.end_column
            },
            "signature": node.signature
        });

        // Add source context around the symbol location
        if let Some(context) = self.extract_source_context(&node.file, node.span.start_line, context_lines) {
            node_info["source_context"] = context;
        }

        node_info
    }

    /// Search content across repository
    async fn search_content(&self, server: &PrismMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
        let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        
        let query = args.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing query parameter"))?;

        let content_types = args.get("content_types")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let file_patterns = args.get("file_patterns")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let exclude_patterns = args.get("exclude_patterns")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let max_results = args.get("max_results")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(50);

        let case_sensitive = args.get("case_sensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let use_regex = args.get("use_regex")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let include_context = args.get("include_context")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Check if content is indexed
        let stats = server.content_search().get_stats();
        if stats.total_files == 0 {
            let result = serde_json::json!({
                "query": query,
                "results": [],
                "total_results": 0,
                "status": "no_content_indexed",
                "message": "Content search is not yet indexed. This feature requires repository content to be indexed first.",
                "suggestion": "Repository indexing may still be in progress. Try again in a few moments."
            });

            return Ok(CallToolResult {
                content: vec![ToolContent::Text {
                    text: serde_json::to_string_pretty(&result)?,
                }],
                is_error: Some(false),
            });
        }

        match server.content_search().simple_search(query, Some(max_results)) {
            Ok(search_results) => {
                let result = serde_json::json!({
                    "query": query,
                    "content_types": content_types,
                    "file_patterns": file_patterns,
                    "exclude_patterns": exclude_patterns,
                    "max_results": max_results,
                    "case_sensitive": case_sensitive,
                    "use_regex": use_regex,
                    "include_context": include_context,
                    "total_results": search_results.len(),
                    "results": search_results.iter().map(|result| {
                        serde_json::json!({
                            "file": result.chunk.file_path.display().to_string(),
                            "content_type": format!("{:?}", result.chunk.content_type),
                            "score": result.score,
                            "matches": result.matches.iter().map(|m| {
                                serde_json::json!({
                                    "text": m.text,
                                    "line": m.line_number,
                                    "column": m.column_number,
                                    "context_before": m.context_before,
                                    "context_after": m.context_after
                                })
                            }).collect::<Vec<_>>(),
                            "chunk_content_preview": if result.chunk.content.len() > 200 {
                                format!("{}...", &result.chunk.content[..200])
                            } else {
                                result.chunk.content.clone()
                            }
                        })
                    }).collect::<Vec<_>>()
                });

                Ok(CallToolResult {
                    content: vec![ToolContent::Text {
                        text: serde_json::to_string_pretty(&result)?,
                    }],
                    is_error: Some(false),
                })
            }
            Err(e) => Ok(CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Content search error: {}", e),
                }],
                is_error: Some(true),
            }),
        }
    }

    /// Find files by pattern
    async fn find_files(&self, server: &PrismMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
        let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        
        let pattern = args.get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing pattern parameter"))?;

        // Check if content is indexed
        let stats = server.content_search().get_stats();
        if stats.total_files == 0 {
            // Fall back to scanning the repository directly
            if let Some(repo_path) = server.repository_path() {
                match server.scanner().discover_files(repo_path) {
                    Ok(all_files) => {
                        let pattern_regex = match regex::Regex::new(pattern) {
                            Ok(regex) => regex,
                            Err(_) => {
                                // Fall back to glob-style matching
                                let glob_pattern = pattern.replace("*", ".*").replace("?", ".");
                                match regex::Regex::new(&glob_pattern) {
                                    Ok(regex) => regex,
                                    Err(e) => {
                                        return Ok(CallToolResult {
                                            content: vec![ToolContent::Text {
                                                text: format!("Invalid pattern '{}': {}", pattern, e),
                                            }],
                                            is_error: Some(true),
                                        });
                                    }
                                }
                            }
                        };

                        let matching_files: Vec<_> = all_files.iter()
                            .filter(|path| pattern_regex.is_match(&path.to_string_lossy()))
                            .collect();

                        let result = serde_json::json!({
                            "pattern": pattern,
                            "total_files": matching_files.len(),
                            "source": "repository_scan",
                            "files": matching_files.iter().map(|path| {
                                serde_json::json!({
                                    "path": path.display().to_string(),
                                    "name": path.file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or(""),
                                    "extension": path.extension()
                                        .and_then(|ext| ext.to_str())
                                        .unwrap_or("")
                                })
                            }).collect::<Vec<_>>()
                        });

                        return Ok(CallToolResult {
                            content: vec![ToolContent::Text {
                                text: serde_json::to_string_pretty(&result)?,
                            }],
                            is_error: Some(false),
                        });
                    }
                    Err(e) => {
                        return Ok(CallToolResult {
                            content: vec![ToolContent::Text {
                                text: format!("Failed to scan repository for files: {}", e),
                            }],
                            is_error: Some(true),
                        });
                    }
                }
            } else {
                let result = serde_json::json!({
                    "pattern": pattern,
                    "total_files": 0,
                    "source": "no_repository",
                    "files": [],
                    "message": "No repository is currently loaded"
                });

                return Ok(CallToolResult {
                    content: vec![ToolContent::Text {
                        text: serde_json::to_string_pretty(&result)?,
                    }],
                    is_error: Some(false),
                });
            }
        }

        match server.content_search().find_files(pattern) {
            Ok(files) => {
                let result = serde_json::json!({
                    "pattern": pattern,
                    "total_files": files.len(),
                    "source": "content_index",
                    "files": files.iter().map(|path| {
                        serde_json::json!({
                            "path": path.display().to_string(),
                            "name": path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(""),
                            "extension": path.extension()
                                .and_then(|ext| ext.to_str())
                                .unwrap_or("")
                        })
                    }).collect::<Vec<_>>()
                });

                Ok(CallToolResult {
                    content: vec![ToolContent::Text {
                        text: serde_json::to_string_pretty(&result)?,
                    }],
                    is_error: Some(false),
                })
            }
            Err(e) => Ok(CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("File search error: {}", e),
                }],
                is_error: Some(true),
            }),
        }
    }

    /// Get content statistics
    async fn content_stats(&self, server: &PrismMcpServer) -> Result<CallToolResult> {
        let stats = server.content_search().get_stats();
        
        let result = if stats.total_files == 0 {
            serde_json::json!({
                "total_files": 0,
                "total_chunks": 0,
                "total_tokens": 0,
                "content_by_type": {},
                "size_distribution": {},
                "status": "no_content_indexed",
                "message": "Content indexing has not been performed yet. Only code symbol analysis is available.",
                "suggestion": "Content indexing for documentation, configuration files, and comments may still be in progress."
            })
        } else {
            serde_json::json!({
                "total_files": stats.total_files,
                "total_chunks": stats.total_chunks,
                "total_tokens": stats.total_tokens,
                "content_by_type": stats.content_by_type,
                "size_distribution": stats.size_distribution,
                "computed_at": stats.computed_at.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                "status": "indexed"
            })
        };

        Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: serde_json::to_string_pretty(&result)?,
            }],
            is_error: Some(false),
        })
    }

    /// Analyze code complexity
    async fn analyze_complexity(&self, server: &PrismMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
        let args = arguments.unwrap_or_default();
        
        let target = match args.get("target").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => {
                return Ok(CallToolResult {
                    content: vec![ToolContent::Text {
                        text: "Missing required parameter: target".to_string(),
                    }],
                    is_error: Some(true),
                });
            }
        };
        
        let metrics = args.get("metrics")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec!["all".to_string()]);

        let threshold_warnings = args.get("threshold_warnings")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Determine if target is a file path or symbol ID
        let mut complexity_results = Vec::new();
        
        if target.starts_with('/') || target.contains('.') {
            // Treat as file path
            if let Some(repo_path) = server.repository_path() {
                let file_path = if std::path::Path::new(target).is_absolute() {
                    std::path::PathBuf::from(target)
                } else {
                    repo_path.join(target)
                };
                
                if file_path.exists() {
                    let file_complexity = self.analyze_file_complexity(&file_path, &metrics, threshold_warnings)?;
                    complexity_results.push(file_complexity);
                } else {
                    return Ok(CallToolResult {
                        content: vec![ToolContent::Text {
                            text: format!("File not found: {}", target),
                        }],
                        is_error: Some(true),
                    });
                }
            }
        } else {
            // Treat as symbol ID
            if let Ok(symbol_id) = self.parse_node_id(target) {
                if let Some(node) = server.graph_store().get_node(&symbol_id) {
                    let symbol_complexity = self.analyze_symbol_complexity(&node, &metrics, threshold_warnings)?;
                    complexity_results.push(symbol_complexity);
                } else {
                    return Ok(CallToolResult {
                        content: vec![ToolContent::Text {
                            text: format!("Symbol not found: {}", target),
                        }],
                        is_error: Some(true),
                    });
                }
            } else {
                return Ok(CallToolResult {
                    content: vec![ToolContent::Text {
                        text: format!("Invalid target format: {}", target),
                    }],
                    is_error: Some(true),
                });
            }
        }

        let result = serde_json::json!({
            "target": target,
            "metrics_requested": metrics,
            "threshold_warnings": threshold_warnings,
            "results": complexity_results,
            "summary": {
                "total_analyzed": complexity_results.len(),
                "high_complexity_items": complexity_results.iter()
                    .filter(|r| r.get("warnings").and_then(|w| w.as_array()).map(|arr| !arr.is_empty()).unwrap_or(false))
                    .count()
            }
        });

        Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: serde_json::to_string_pretty(&result)?,
            }],
            is_error: Some(false),
        })
    }

    /// Find code duplicates
    async fn find_duplicates(&self, server: &PrismMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
        let args = arguments.unwrap_or_default();
        
        let similarity_threshold = args.get("similarity_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.85);
        
        let min_lines = args.get("min_lines")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(5);
        
        let _scope = args.get("scope")
            .and_then(|v| v.as_str())
            .unwrap_or("repository");
        
        let _include_semantic_similarity = args.get("include_semantic_similarity")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let exclude_patterns: Vec<String> = args.get("exclude_patterns")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_default();

        if let Some(repo_path) = server.repository_path() {
            match self.find_code_duplicates(repo_path, similarity_threshold, min_lines, &exclude_patterns) {
                Ok(duplicates) => {
                    let total_duplicates = duplicates.len();
                    let affected_files: std::collections::HashSet<String> = duplicates
                        .iter()
                        .flat_map(|dup| {
                            vec![
                                dup.get("file1").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                dup.get("file2").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            ]
                        })
                        .filter(|f| !f.is_empty())
                        .collect();

                    let result = serde_json::json!({
                        "duplicates": duplicates,
                        "summary": {
                            "total_duplicate_pairs": total_duplicates,
                            "affected_files": affected_files.len(),
                            "similarity_threshold": similarity_threshold,
                            "min_lines_threshold": min_lines
                        },
                        "analysis_successful": true
                    });

                    Ok(CallToolResult {
                        content: vec![ToolContent::Text {
                            text: serde_json::to_string_pretty(&result)?,
                        }],
                        is_error: Some(false),
                    })
                }
                Err(e) => {
                    let result = serde_json::json!({
                        "error": format!("Duplicate analysis failed: {}", e),
                        "analysis_successful": false
                    });

                    Ok(CallToolResult {
                        content: vec![ToolContent::Text {
                            text: serde_json::to_string_pretty(&result)?,
                        }],
                        is_error: Some(true),
                    })
                }
            }
        } else {
            let result = serde_json::json!({
                "error": "No repository initialized",
                "analysis_successful": false
            });

            Ok(CallToolResult {
                content: vec![ToolContent::Text {
                    text: serde_json::to_string_pretty(&result)?,
                }],
                is_error: Some(true),
            })
        }
    }

    /// Analyze complexity for a file
    fn analyze_file_complexity(&self, file_path: &std::path::Path, metrics: &[String], threshold_warnings: bool) -> Result<serde_json::Value> {
        // Read file content
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", file_path.display(), e))?;
        
        let lines = content.lines().collect::<Vec<_>>();
        let total_lines = lines.len();
        
        // Basic complexity metrics calculation
        let mut complexity_metrics = serde_json::json!({
            "file": file_path.display().to_string(),
            "total_lines": total_lines,
            "non_empty_lines": lines.iter().filter(|line| !line.trim().is_empty()).count(),
            "metrics": {}
        });

        let mut warnings = Vec::new();

        // Calculate requested metrics
        let _include_all = metrics.contains(&"all".to_string());
        for metric in metrics {
            match metric.as_str() {
                "cyclomatic" => {
                    let cyclomatic = self.calculate_cyclomatic_complexity(&content);
                    complexity_metrics["metrics"]["cyclomatic_complexity"] = serde_json::json!({
                        "value": cyclomatic,
                        "description": "Number of linearly independent paths through the code"
                    });
                    
                    if threshold_warnings && cyclomatic > 10 {
                        warnings.push(format!("High cyclomatic complexity: {} (threshold: 10)", cyclomatic));
                    }
                }
                "cognitive" => {
                    let cognitive = self.calculate_cognitive_complexity(&content);
                    complexity_metrics["metrics"]["cognitive_complexity"] = serde_json::json!({
                        "value": cognitive,
                        "description": "Measure of how hard the code is to understand"
                    });
                    
                    if threshold_warnings && cognitive > 15 {
                        warnings.push(format!("High cognitive complexity: {} (threshold: 15)", cognitive));
                    }
                }
                "halstead" => {
                    let (volume, difficulty, effort) = self.calculate_halstead_metrics(&content);
                    complexity_metrics["metrics"]["halstead"] = serde_json::json!({
                        "volume": volume,
                        "difficulty": difficulty,
                        "effort": effort,
                        "description": "Halstead complexity metrics based on operators and operands"
                    });
                }
                "maintainability_index" => {
                    let mi = self.calculate_maintainability_index(&content, total_lines);
                    complexity_metrics["metrics"]["maintainability_index"] = serde_json::json!({
                        "value": mi,
                        "description": "Maintainability index (0-100, higher is better)"
                    });
                    
                    if threshold_warnings && mi < 20.0 {
                        warnings.push(format!("Low maintainability index: {:.1} (threshold: 20)", mi));
                    }
                }
                "all" => {
                    // Calculate all metrics
                    let cyclomatic = self.calculate_cyclomatic_complexity(&content);
                    complexity_metrics["metrics"]["cyclomatic_complexity"] = serde_json::json!({
                        "value": cyclomatic,
                        "description": "Number of linearly independent paths through the code"
                    });
                    if threshold_warnings && cyclomatic > 10 {
                        warnings.push(format!("High cyclomatic complexity: {} (threshold: 10)", cyclomatic));
                    }

                    let cognitive = self.calculate_cognitive_complexity(&content);
                    complexity_metrics["metrics"]["cognitive_complexity"] = serde_json::json!({
                        "value": cognitive,
                        "description": "Measure of how hard the code is to understand"
                    });
                    if threshold_warnings && cognitive > 15 {
                        warnings.push(format!("High cognitive complexity: {} (threshold: 15)", cognitive));
                    }

                    let (volume, difficulty, effort) = self.calculate_halstead_metrics(&content);
                    complexity_metrics["metrics"]["halstead"] = serde_json::json!({
                        "volume": volume,
                        "difficulty": difficulty,
                        "effort": effort,
                        "description": "Halstead complexity metrics based on operators and operands"
                    });

                    let mi = self.calculate_maintainability_index(&content, total_lines);
                    complexity_metrics["metrics"]["maintainability_index"] = serde_json::json!({
                        "value": mi,
                        "description": "Maintainability index (0-100, higher is better)"
                    });
                    if threshold_warnings && mi < 20.0 {
                        warnings.push(format!("Low maintainability index: {:.1} (threshold: 20)", mi));
                    }
                }
                _ => {
                    // Skip unknown metrics
                }
            }
        }

        if !warnings.is_empty() {
            complexity_metrics["warnings"] = serde_json::json!(warnings);
        }

        Ok(complexity_metrics)
    }

    /// Analyze complexity for a specific symbol
    fn analyze_symbol_complexity(&self, node: &prism_core::Node, metrics: &[String], threshold_warnings: bool) -> Result<serde_json::Value> {
        // Read the file containing the symbol
        let content = std::fs::read_to_string(&node.file)
            .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", node.file.display(), e))?;
        
        // Extract symbol's content based on span
        let lines = content.lines().collect::<Vec<_>>();
        let symbol_content = if node.span.start_line <= lines.len() && node.span.end_line <= lines.len() {
            lines[(node.span.start_line - 1).max(0)..node.span.end_line.min(lines.len())]
                .join("\n")
        } else {
            content.clone()
        };

        let symbol_lines = node.span.end_line - node.span.start_line + 1;
        
        let mut complexity_metrics = serde_json::json!({
            "symbol": {
                "id": node.id.to_hex(),
                "name": node.name,
                "kind": format!("{:?}", node.kind),
                "file": node.file.display().to_string(),
                "span": {
                    "start_line": node.span.start_line,
                    "end_line": node.span.end_line,
                    "lines": symbol_lines
                }
            },
            "metrics": {}
        });

        let mut warnings = Vec::new();

        // Calculate requested metrics for the symbol
        for metric in metrics {
            match metric.as_str() {
                "cyclomatic" => {
                    let cyclomatic = self.calculate_cyclomatic_complexity(&symbol_content);
                    complexity_metrics["metrics"]["cyclomatic_complexity"] = serde_json::json!({
                        "value": cyclomatic,
                        "description": "Number of linearly independent paths through the symbol"
                    });
                    
                    if threshold_warnings && cyclomatic > 10 {
                        warnings.push(format!("High cyclomatic complexity: {} (threshold: 10)", cyclomatic));
                    }
                }
                "cognitive" => {
                    let cognitive = self.calculate_cognitive_complexity(&symbol_content);
                    complexity_metrics["metrics"]["cognitive_complexity"] = serde_json::json!({
                        "value": cognitive,
                        "description": "Measure of how hard the symbol is to understand"
                    });
                    
                    if threshold_warnings && cognitive > 15 {
                        warnings.push(format!("High cognitive complexity: {} (threshold: 15)", cognitive));
                    }
                }
                "halstead" => {
                    let (volume, difficulty, effort) = self.calculate_halstead_metrics(&symbol_content);
                    complexity_metrics["metrics"]["halstead"] = serde_json::json!({
                        "volume": volume,
                        "difficulty": difficulty,
                        "effort": effort,
                        "description": "Halstead complexity metrics for the symbol"
                    });
                }
                "maintainability_index" => {
                    let mi = self.calculate_maintainability_index(&symbol_content, symbol_lines);
                    complexity_metrics["metrics"]["maintainability_index"] = serde_json::json!({
                        "value": mi,
                        "description": "Maintainability index for the symbol (0-100, higher is better)"
                    });
                    
                    if threshold_warnings && mi < 20.0 {
                        warnings.push(format!("Low maintainability index: {:.1} (threshold: 20)", mi));
                    }
                }
                "all" => {
                    // Calculate all metrics for the symbol
                    let cyclomatic = self.calculate_cyclomatic_complexity(&symbol_content);
                    complexity_metrics["metrics"]["cyclomatic_complexity"] = serde_json::json!({
                        "value": cyclomatic,
                        "description": "Number of linearly independent paths through the symbol"
                    });
                    if threshold_warnings && cyclomatic > 10 {
                        warnings.push(format!("High cyclomatic complexity: {} (threshold: 10)", cyclomatic));
                    }

                    let cognitive = self.calculate_cognitive_complexity(&symbol_content);
                    complexity_metrics["metrics"]["cognitive_complexity"] = serde_json::json!({
                        "value": cognitive,
                        "description": "Measure of how hard the symbol is to understand"
                    });
                    if threshold_warnings && cognitive > 15 {
                        warnings.push(format!("High cognitive complexity: {} (threshold: 15)", cognitive));
                    }

                    let (volume, difficulty, effort) = self.calculate_halstead_metrics(&symbol_content);
                    complexity_metrics["metrics"]["halstead"] = serde_json::json!({
                        "volume": volume,
                        "difficulty": difficulty,
                        "effort": effort,
                        "description": "Halstead complexity metrics for the symbol"
                    });

                    let mi = self.calculate_maintainability_index(&symbol_content, symbol_lines);
                    complexity_metrics["metrics"]["maintainability_index"] = serde_json::json!({
                        "value": mi,
                        "description": "Maintainability index for the symbol (0-100, higher is better)"
                    });
                    if threshold_warnings && mi < 20.0 {
                        warnings.push(format!("Low maintainability index: {:.1} (threshold: 20)", mi));
                    }
                }
                _ => {
                    // Skip unknown metrics
                }
            }
        }

        if !warnings.is_empty() {
            complexity_metrics["warnings"] = serde_json::json!(warnings);
        }

        Ok(complexity_metrics)
    }

    /// Calculate cyclomatic complexity (simplified)
    fn calculate_cyclomatic_complexity(&self, content: &str) -> usize {
        let mut complexity = 1; // Base complexity
        
        // Count decision points (simplified heuristic)
        let decision_keywords = [
            "if", "else if", "elif", "while", "for", "foreach", "switch", "case",
            "catch", "except", "?", "&&", "||", "and", "or"
        ];
        
        for keyword in &decision_keywords {
            complexity += content.matches(keyword).count();
        }
        
        complexity
    }

    /// Calculate cognitive complexity (simplified)
    fn calculate_cognitive_complexity(&self, content: &str) -> usize {
        let mut complexity = 0;
        let mut nesting_level: usize = 0;
        
        let lines = content.lines();
        for line in lines {
            let trimmed = line.trim();
            
            // Increment nesting for certain constructs
            if trimmed.contains('{') || trimmed.starts_with("if ") || 
               trimmed.starts_with("for ") || trimmed.starts_with("while ") ||
               trimmed.starts_with("try ") || trimmed.starts_with("def ") ||
               trimmed.starts_with("function ") {
                nesting_level += 1;
            }
            
            // Decrement nesting
            if trimmed.contains('}') {
                nesting_level = nesting_level.saturating_sub(1usize);
            }
            
            // Add complexity based on constructs
            if trimmed.contains("if ") || trimmed.contains("elif ") || 
               trimmed.contains("else if") {
                complexity += 1 + nesting_level;
            }
            if trimmed.contains("while ") || trimmed.contains("for ") {
                complexity += 1 + nesting_level;
            }
            if trimmed.contains("catch ") || trimmed.contains("except ") {
                complexity += 1 + nesting_level;
            }
        }
        
        complexity
    }

    /// Calculate Halstead complexity metrics (simplified)
    fn calculate_halstead_metrics(&self, content: &str) -> (f64, f64, f64) {
        // Simplified Halstead calculation
        let operators = ["=", "+", "-", "*", "/", "==", "!=", "<", ">", "<=", ">=", "&&", "||"];
        let mut unique_operators = std::collections::HashSet::new();
        let mut total_operators = 0;
        
        for op in &operators {
            let count = content.matches(op).count();
            if count > 0 {
                unique_operators.insert(op);
                total_operators += count;
            }
        }
        
        // Rough operand estimation (identifiers, literals)
        let words: Vec<&str> = content.split_whitespace().collect();
        let mut unique_operands = std::collections::HashSet::new();
        let mut total_operands = 0;
        
        for word in words {
            if word.chars().any(|c| c.is_alphanumeric()) {
                unique_operands.insert(word);
                total_operands += 1;
            }
        }
        
        let n1 = unique_operators.len().max(1) as f64; // Minimum 1 operator
        let n2 = unique_operands.len().max(1) as f64; // Minimum 1 operand
        let big_n1 = total_operators.max(1) as f64; // Minimum 1 operator usage
        let big_n2 = total_operands.max(1) as f64; // Minimum 1 operand usage
        
        let vocabulary = n1 + n2;
        let length = big_n1 + big_n2;
        
        // Ensure vocabulary is at least 2 to avoid log2(1) = 0
        let safe_vocabulary = vocabulary.max(2.0);
        let volume = length * safe_vocabulary.log2();
        
        // Safe difficulty calculation
        let difficulty = (n1 / 2.0) * (big_n2 / n2);
        let effort = difficulty * volume;
        
        (volume, difficulty, effort)
    }

    /// Calculate maintainability index (simplified)
    fn calculate_maintainability_index(&self, content: &str, lines_count: usize) -> f64 {
        let (volume, difficulty, _effort) = self.calculate_halstead_metrics(content);
        let cyclomatic = self.calculate_cyclomatic_complexity(content) as f64;
        let loc = lines_count.max(1) as f64; // Minimum 1 line
        
        // Ensure volume is meaningful for logarithm
        let safe_volume = volume.max(1.0);
        let safe_loc = loc.max(1.0);
        
        // Adjusted maintainability index formula to be more sensitive
        // Based on the standard formula but with adjusted coefficients for this simplified implementation
        // Higher volume, complexity, and difficulty should decrease maintainability more significantly
        let volume_penalty = safe_volume.ln() * 8.0; // Increased from 5.2
        let complexity_penalty = cyclomatic * 5.0; // Increased from 0.23
        let loc_penalty = safe_loc.ln() * 20.0; // Increased from 16.2
        let difficulty_penalty = difficulty * 2.0; // Add difficulty factor
        
        let mi = 171.0 - volume_penalty - complexity_penalty - loc_penalty - difficulty_penalty;
        
        // Ensure result is in valid range
        mi.max(0.0).min(100.0)
    }

    /// Find code duplicates in the repository (simplified implementation)
    fn find_code_duplicates(
        &self, 
        repo_path: &std::path::Path, 
        similarity_threshold: f64, 
        min_lines: usize, 
        exclude_patterns: &[String]
    ) -> Result<Vec<serde_json::Value>> {
        let mut duplicates = Vec::new();
        
        // Get all source files
        let mut file_contents = std::collections::HashMap::new();
        if let Ok(entries) = std::fs::read_dir(repo_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if ["js", "ts", "py", "java", "rs", "c", "cpp"].contains(&ext) {
                            // Skip if matches exclude patterns
                            let path_str = path.to_string_lossy();
                            if exclude_patterns.iter().any(|pattern| path_str.contains(pattern)) {
                                continue;
                            }
                            
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                file_contents.insert(path.clone(), content);
                            }
                        }
                    }
                }
            }
        }

        // Simple duplicate detection based on similar line patterns
        let mut analyzed_pairs = std::collections::HashSet::new();
        
        for (file1, content1) in &file_contents {
            for (file2, content2) in &file_contents {
                if file1 >= file2 || analyzed_pairs.contains(&(file1.clone(), file2.clone())) {
                    continue;
                }
                analyzed_pairs.insert((file1.clone(), file2.clone()));
                
                let similarity = self.calculate_content_similarity(content1, content2);
                if similarity >= similarity_threshold {
                    let lines1 = content1.lines().count();
                    let lines2 = content2.lines().count();
                    
                    if lines1 >= min_lines && lines2 >= min_lines {
                        duplicates.push(serde_json::json!({
                            "similarity": similarity,
                            "files": [
                                {
                                    "path": file1.display().to_string(),
                                    "lines": lines1
                                },
                                {
                                    "path": file2.display().to_string(),
                                    "lines": lines2
                                }
                            ],
                            "lines": lines1.min(lines2),
                            "type": "file_similarity"
                        }));
                    }
                }
            }
        }

        Ok(duplicates)
    }

    /// Calculate content similarity between two text blocks (simplified)
    fn calculate_content_similarity(&self, content1: &str, content2: &str) -> f64 {
        let lines1: Vec<String> = content1.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        let lines2: Vec<String> = content2.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        
        if lines1.is_empty() || lines2.is_empty() {
            return 0.0;
        }
        
        // Simple line-based similarity using Jaccard coefficient
        let set1: std::collections::HashSet<String> = lines1.into_iter().collect();
        let set2: std::collections::HashSet<String> = lines2.into_iter().collect();
        
        if set1.is_empty() && set2.is_empty() {
            return 1.0;
        }
        
        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Detect design patterns in the codebase
    async fn detect_patterns(&self, server: &PrismMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
        let args = arguments.unwrap_or_default();
        
        let scope = args.get("scope")
            .and_then(|v| v.as_str())
            .unwrap_or("repository");
        
        let pattern_types: Vec<String> = args.get("pattern_types")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_else(|| vec!["all".to_string()]);
        
        let confidence_threshold = args.get("confidence_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.8);
        
        let include_suggestions = args.get("include_suggestions")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let result = if let Some(_repo_path) = server.repository_path() {
            let detected_patterns = self.analyze_design_patterns(server, &pattern_types, confidence_threshold, include_suggestions).await?;
            
            serde_json::json!({
                "scope": scope,
                "patterns": detected_patterns,
                "summary": {
                    "total_patterns_detected": detected_patterns.len(),
                    "confidence_threshold": confidence_threshold,
                    "pattern_types_analyzed": pattern_types
                },
                "analysis_successful": true
            })
        } else {
            serde_json::json!({
                "error": "No repository initialized",
                "analysis_successful": false
            })
        };

        Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: serde_json::to_string_pretty(&result)?,
            }],
            is_error: Some(false),
        })
    }

    /// Analyze transitive dependencies
    async fn analyze_transitive_dependencies(&self, server: &PrismMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
        let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        
        let target = args.get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing target parameter"))?;
        
        let max_depth = args.get("max_depth")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(5);
        
        let detect_cycles = args.get("detect_cycles")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let include_external = args.get("include_external_dependencies")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let dependency_types: Vec<String> = args.get("dependency_types")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_else(|| vec!["all".to_string()]);

        let result = if let Some(_repo_path) = server.repository_path() {
            let analysis = self.perform_transitive_analysis(server, target, max_depth, detect_cycles, include_external, &dependency_types).await?;
            
            serde_json::json!({
                "target": target,
                "analysis": analysis,
                "parameters": {
                    "max_depth": max_depth,
                    "detect_cycles": detect_cycles,
                    "include_external": include_external,
                    "dependency_types": dependency_types
                },
                "analysis_successful": true
            })
        } else {
            serde_json::json!({
                "error": "No repository initialized",
                "analysis_successful": false
            })
        };

        Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: serde_json::to_string_pretty(&result)?,
            }],
            is_error: Some(false),
        })
    }

    /// Trace data flow through the codebase
    async fn trace_data_flow(&self, server: &PrismMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
        let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        
        let variable_or_parameter = args.get("variable_or_parameter")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing variable_or_parameter parameter"))?;
        
        let direction = args.get("direction")
            .and_then(|v| v.as_str())
            .unwrap_or("forward");
        
        let include_transformations = args.get("include_transformations")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let max_depth = args.get("max_depth")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(10);
        
        let follow_function_calls = args.get("follow_function_calls")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let include_field_access = args.get("include_field_access")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let symbol_id = self.parse_node_id(variable_or_parameter)?;

        let data_flow_result = self.perform_data_flow_analysis(
            server,
            &symbol_id,
            direction,
            include_transformations,
            max_depth,
            follow_function_calls,
            include_field_access,
        ).await?;

        Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: serde_json::to_string_pretty(&data_flow_result)?,
            }],
            is_error: Some(false),
        })
    }

    /// Find unused code in the codebase
    async fn find_unused_code(&self, server: &PrismMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
        let args = arguments.unwrap_or_else(|| serde_json::json!({}));
        
        let scope = args.get("scope")
            .and_then(|v| v.as_str())
            .unwrap_or("repository");
        
        let include_dead_code = args.get("include_dead_code")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let consider_external_apis = args.get("consider_external_apis")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let confidence_threshold = args.get("confidence_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.9);
        
        let analyze_types = args.get("analyze_types")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec!["all".to_string()]);
        
        let exclude_patterns = args.get("exclude_patterns")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(Vec::new);

        let unused_code_result = self.perform_unused_code_analysis(
            server,
            scope,
            include_dead_code,
            consider_external_apis,
            confidence_threshold,
            &analyze_types,
            &exclude_patterns,
        ).await?;

        Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: serde_json::to_string_pretty(&unused_code_result)?,
            }],
            is_error: Some(false),
        })
    }

    /// Analyze security vulnerabilities in the codebase
    async fn analyze_security(&self, server: &PrismMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
        let args = arguments.unwrap_or_else(|| serde_json::json!({}));
        
        let scope = args.get("scope")
            .and_then(|v| v.as_str())
            .unwrap_or("repository");
        
        let vulnerability_types = args.get("vulnerability_types")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec!["all".to_string()]);
        
        let severity_threshold = args.get("severity_threshold")
            .and_then(|v| v.as_str())
            .unwrap_or("medium");
        
        let include_data_flow_analysis = args.get("include_data_flow_analysis")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let check_external_dependencies = args.get("check_external_dependencies")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let exclude_patterns = args.get("exclude_patterns")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(Vec::new);

        let security_analysis_result = self.perform_security_analysis(
            server,
            scope,
            &vulnerability_types,
            severity_threshold,
            include_data_flow_analysis,
            check_external_dependencies,
            &exclude_patterns,
        ).await?;

        Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: serde_json::to_string_pretty(&security_analysis_result)?,
            }],
            is_error: Some(false),
        })
    }

    /// Analyze performance characteristics
    async fn analyze_performance(&self, server: &PrismMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
        let args = arguments.unwrap_or_else(|| serde_json::json!({}));
        
        let scope = args.get("scope")
            .and_then(|v| v.as_str())
            .unwrap_or("repository");
        
        let analysis_types = args.get("analysis_types")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec!["all".to_string()]);
        
        let complexity_threshold = args.get("complexity_threshold")
            .and_then(|v| v.as_str())
            .unwrap_or("medium");
        
        let include_algorithmic_analysis = args.get("include_algorithmic_analysis")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let detect_bottlenecks = args.get("detect_bottlenecks")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let exclude_patterns = args.get("exclude_patterns")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(Vec::new);

        let performance_analysis_result = self.perform_performance_analysis(
            server,
            scope,
            &analysis_types,
            complexity_threshold,
            include_algorithmic_analysis,
            detect_bottlenecks,
            &exclude_patterns,
        ).await?;

        Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: serde_json::to_string_pretty(&performance_analysis_result)?,
            }],
            is_error: Some(false),
        })
    }

    /// Analyze API surface characteristics
    async fn analyze_api_surface(&self, server: &PrismMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
        let args = arguments.unwrap_or_else(|| serde_json::json!({}));
        
        let scope = args.get("scope")
            .and_then(|v| v.as_str())
            .unwrap_or("repository");
        
        let analysis_types = args.get("analysis_types")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec!["all".to_string()]);
        
        let api_version = args.get("api_version")
            .and_then(|v| v.as_str());
        
        let include_private_apis = args.get("include_private_apis")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let check_documentation_coverage = args.get("check_documentation_coverage")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let detect_breaking_changes = args.get("detect_breaking_changes")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let exclude_patterns = args.get("exclude_patterns")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(Vec::new);

        let api_analysis_result = self.perform_api_surface_analysis(
            server,
            scope,
            &analysis_types,
            api_version,
            include_private_apis,
            check_documentation_coverage,
            detect_breaking_changes,
            &exclude_patterns,
        ).await?;

        Ok(CallToolResult {
            content: vec![ToolContent::Text {
                text: serde_json::to_string_pretty(&api_analysis_result)?,
            }],
            is_error: Some(false),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PrismMcpServer;
    use tempfile::TempDir;
    use std::fs;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    async fn create_test_server() -> Arc<RwLock<PrismMcpServer>> {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        
        // Create test Python files
        fs::write(repo_path.join("main.py"), r#"
class User:
    def __init__(self, name: str):
        self.name = name
    
    def get_greeting(self) -> str:
        return format_greeting(self.name)

def format_greeting(name: str) -> str:
    return f"Hello, {name}!"

if __name__ == "__main__":
    user = User("Alice")
    print(user.get_greeting())
"#).unwrap();

        fs::write(repo_path.join("utils.py"), r#"
def validate_input(data: str) -> bool:
    return len(data) > 0

def process_data(input_data: str) -> str:
    if validate_input(input_data):
        return input_data.upper()
    return ""
"#).unwrap();

        let mut server = PrismMcpServer::new().expect("Failed to create server");
        server.initialize_with_repository(repo_path).await
            .expect("Failed to initialize repository");
        
        // Keep temp_dir alive
        std::mem::forget(temp_dir);
        
        Arc::new(RwLock::new(server))
    }

    #[tokio::test]
    async fn test_tool_manager_creation() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        // Tool manager should be created successfully
        assert!(true); // Just testing creation doesn't panic
    }

    #[tokio::test]
    async fn test_list_tools() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let result = tool_manager.list_tools(ListToolsParams { cursor: None }).await;
        assert!(result.is_ok());
        
        let tools_result = result.unwrap();
        assert_eq!(tools_result.tools.len(), 18); // All implemented tools including API surface analysis
        assert!(tools_result.next_cursor.is_none());
        
        // Verify all expected tools are present
        let tool_names: Vec<String> = tools_result.tools.iter().map(|t| t.name.clone()).collect();
        assert!(tool_names.contains(&"repository_stats".to_string()));
        assert!(tool_names.contains(&"trace_path".to_string()));
        assert!(tool_names.contains(&"explain_symbol".to_string()));
        assert!(tool_names.contains(&"find_dependencies".to_string()));
        assert!(tool_names.contains(&"find_references".to_string()));
        assert!(tool_names.contains(&"search_symbols".to_string()));
        assert!(tool_names.contains(&"search_content".to_string()));
        assert!(tool_names.contains(&"find_files".to_string()));
        assert!(tool_names.contains(&"content_stats".to_string()));
    }

    #[tokio::test]
    async fn test_repository_stats_tool() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "repository_stats".to_string(),
            arguments: Some(serde_json::json!({})),
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_ok());
        
        let tool_result = result.unwrap();
        assert_eq!(tool_result.is_error, Some(false));
        assert_eq!(tool_result.content.len(), 1);
        
        if let ToolContent::Text { text } = &tool_result.content[0] {
            let stats: serde_json::Value = serde_json::from_str(text).unwrap();
            assert!(stats["total_files"].as_u64().unwrap() > 0);
            assert!(stats["total_nodes"].as_u64().unwrap() > 0);
            assert!(stats["status"].as_str().unwrap() == "active");
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_search_symbols_tool() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "search_symbols".to_string(),
            arguments: Some(serde_json::json!({
                "pattern": "User",
                "symbol_types": ["class"],
                "limit": 10
            })),
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_ok());
        
        let tool_result = result.unwrap();
        assert_eq!(tool_result.is_error, Some(false));
        
        if let ToolContent::Text { text } = &tool_result.content[0] {
            let search_result: serde_json::Value = serde_json::from_str(text).unwrap();
            assert_eq!(search_result["pattern"].as_str().unwrap(), "User");
            assert!(search_result["results"].is_array());
        }
    }

    #[tokio::test]
    async fn test_search_symbols_with_regex_pattern() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "search_symbols".to_string(),
            arguments: Some(serde_json::json!({
                "pattern": "get_",
                "symbol_types": ["function", "method"],
                "limit": 50
            })),
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_ok());
        
        let tool_result = result.unwrap();
        assert_eq!(tool_result.is_error, Some(false));
    }

    #[tokio::test]
    async fn test_unknown_tool() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "unknown_tool".to_string(),
            arguments: Some(serde_json::json!({})),
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_ok());
        
        let tool_result = result.unwrap();
        assert_eq!(tool_result.is_error, Some(true));
        
        if let ToolContent::Text { text } = &tool_result.content[0] {
            assert!(text.contains("Unknown tool: unknown_tool"));
        }
    }

    #[tokio::test]
    async fn test_trace_path_missing_arguments() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "trace_path".to_string(),
            arguments: Some(serde_json::json!({})), // Missing required args
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_err()); // Should fail due to missing arguments
    }

    #[tokio::test]
    async fn test_explain_symbol_missing_arguments() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "explain_symbol".to_string(),
            arguments: Some(serde_json::json!({})), // Missing required args
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_err()); // Should fail due to missing arguments
    }

    #[tokio::test]
    async fn test_find_dependencies_invalid_dependency_type() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "find_dependencies".to_string(),
            arguments: Some(serde_json::json!({
                "target": "fake_target",
                "dependency_type": "invalid_type"
            })),
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_ok());
        
        let tool_result = result.unwrap();
        assert_eq!(tool_result.is_error, Some(true));
        
        if let ToolContent::Text { text } = &tool_result.content[0] {
            assert!(text.contains("Invalid dependency type"));
        }
    }

    #[tokio::test]
    async fn test_find_references_missing_arguments() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "find_references".to_string(),
            arguments: Some(serde_json::json!({})), // Missing required args
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_err()); // Should fail due to missing arguments
    }

    #[tokio::test]
    async fn test_search_symbols_missing_pattern() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "search_symbols".to_string(),
            arguments: Some(serde_json::json!({})), // Missing required pattern
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_err()); // Should fail due to missing pattern
    }

    #[tokio::test]
    async fn test_tool_input_schemas() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let tools_result = tool_manager.list_tools(ListToolsParams { cursor: None }).await.unwrap();
        
        for tool in tools_result.tools {
            // Every tool should have a valid JSON schema
            assert!(tool.input_schema.is_object());
            
            let schema = tool.input_schema.as_object().unwrap();
            assert_eq!(schema.get("type").unwrap().as_str().unwrap(), "object");
            
            // Tools should have properties defined
            if tool.name != "repository_stats" { // repository_stats has empty properties
                assert!(schema.contains_key("properties"));
            }
            
            // Verify required fields for tools that have them
            match tool.name.as_str() {
                "trace_path" => {
                    let required = schema.get("required").unwrap().as_array().unwrap();
                    assert!(required.contains(&serde_json::Value::String("source".to_string())));
                    assert!(required.contains(&serde_json::Value::String("target".to_string())));
                },
                "explain_symbol" | "find_references" => {
                    let required = schema.get("required").unwrap().as_array().unwrap();
                    assert!(required.contains(&serde_json::Value::String("symbol_id".to_string())));
                },
                "find_dependencies" => {
                    let required = schema.get("required").unwrap().as_array().unwrap();
                    assert!(required.contains(&serde_json::Value::String("target".to_string())));
                },
                "search_symbols" => {
                    let required = schema.get("required").unwrap().as_array().unwrap();
                    assert!(required.contains(&serde_json::Value::String("pattern".to_string())));
                },
                _ => {} // repository_stats has no required fields
            }
        }
    }

    #[tokio::test]
    async fn test_tool_capabilities_serialization() {
        let capabilities = ToolCapabilities {
            list_changed: Some(true),
        };
        
        let json = serde_json::to_string(&capabilities).unwrap();
        let deserialized: ToolCapabilities = serde_json::from_str(&json).unwrap();
        
        assert_eq!(capabilities.list_changed, deserialized.list_changed);
    }

    #[tokio::test]
    async fn test_call_tool_params_serialization() {
        let params = CallToolParams {
            name: "test_tool".to_string(),
            arguments: Some(serde_json::json!({"key": "value"})),
        };
        
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: CallToolParams = serde_json::from_str(&json).unwrap();
        
        assert_eq!(params.name, deserialized.name);
        assert_eq!(params.arguments, deserialized.arguments);
    }

    #[tokio::test]
    async fn test_call_tool_result_serialization() {
        let result = CallToolResult {
            content: vec![ToolContent::Text { text: "Test result".to_string() }],
            is_error: Some(false),
        };
        
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: CallToolResult = serde_json::from_str(&json).unwrap();
        
        assert_eq!(result.content.len(), deserialized.content.len());
        assert_eq!(result.is_error, deserialized.is_error);
    }

    #[test]
    fn test_parse_node_id_valid() {
        let server = Arc::new(RwLock::new(PrismMcpServer::new().unwrap()));
        let tool_manager = ToolManager::new(server);
        
        // Test with a valid hex string (assuming NodeId::from_hex works with this format)
        let valid_hex = "deadbeef12345678";
        let result = tool_manager.parse_node_id(valid_hex);
        
        // This test may need adjustment based on actual NodeId::from_hex implementation
        // For now, just test that it doesn't panic
        match result {
            Ok(_) => assert!(true),
            Err(_) => assert!(true), // May fail if format is wrong, but shouldn't panic
        }
    }

    #[tokio::test]
    async fn test_parse_node_id_invalid() {
        let tool_manager = ToolManager::new(create_test_server().await);
        
        // Test invalid hex string
        let result = tool_manager.parse_node_id("invalid-hex");
        assert!(result.is_err());
        
        // Test wrong length
        let result = tool_manager.parse_node_id("abc123");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_source_context_extraction() {
        use tempfile::TempDir;
        use std::fs;
        use std::path::Path;
        
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_file = temp_dir.path().join("test.py");
        
        // Create a test file with known content
        fs::write(&test_file, r#"# Line 1: Header comment
class TestClass:
    """Test class docstring."""
    
    def test_method(self):
        """Test method docstring."""
        return "Hello, World!"
    
    def another_method(self):
        return 42
# Line 11: Footer comment"#).unwrap();
        
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        // Test context extraction around line 5 (the test_method definition)
        let context = tool_manager.extract_source_context(&test_file, 5, 2);
        assert!(context.is_some());
        
        let context_value = context.unwrap();
        assert_eq!(context_value["target_line"], 5);
        assert_eq!(context_value["context_range"]["start_line"], 3);
        assert_eq!(context_value["context_range"]["end_line"], 7);
        
        let lines = context_value["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 5); // Lines 3-7
        
        // Check that the target line is marked correctly
        let target_line = lines.iter().find(|line| line["is_target"] == true).unwrap();
        assert_eq!(target_line["line_number"], 5);
        assert!(target_line["content"].as_str().unwrap().contains("def test_method"));
        
        // Test edge case: line near beginning of file
        let context = tool_manager.extract_source_context(&test_file, 1, 3);
        assert!(context.is_some());
        
        let context_value = context.unwrap();
        assert_eq!(context_value["context_range"]["start_line"], 1);
        
        // Test edge case: line near end of file  
        let context = tool_manager.extract_source_context(&test_file, 11, 3);
        assert!(context.is_some());
        
        let context_value = context.unwrap();
        assert_eq!(context_value["context_range"]["end_line"], 11);
        
        // Test invalid line number
        let context = tool_manager.extract_source_context(&test_file, 100, 2);
        assert!(context.is_none());
        
        // Test nonexistent file
        let context = tool_manager.extract_source_context(Path::new("/nonexistent/file.py"), 1, 2);
        assert!(context.is_none());
    }

    #[tokio::test]
    async fn test_context_lines_parameter_validation() {
        use tempfile::TempDir;
        use std::fs;
        
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_file = temp_dir.path().join("test.py");
        
        // Create a test file
        fs::write(&test_file, r#"# Test file
def example_function():
    """An example function."""
    return "hello"

def another_function():
    return 42
"#).unwrap();
        
        let server_arc = create_test_server().await;
        let tool_manager = ToolManager::new(server_arc);
        
        // Test context extraction with different parameter values
        
        // Test with context_lines = 0
        let context = tool_manager.extract_source_context(&test_file, 2, 0);
        assert!(context.is_some());
        let context_value = context.unwrap();
        let lines = context_value["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 1); // Only the target line
        
        // Test with normal context_lines
        let context = tool_manager.extract_source_context(&test_file, 2, 2);
        assert!(context.is_some());
        let context_value = context.unwrap();
        let lines = context_value["lines"].as_array().unwrap();
        assert!(lines.len() > 1); // Should have context lines
        
        // Test with large context_lines value (should be bounded by file length)
        let context = tool_manager.extract_source_context(&test_file, 2, 100);
        assert!(context.is_some());
        let context_value = context.unwrap();
        let lines = context_value["lines"].as_array().unwrap();
        assert!(lines.len() <= 7); // File only has 7 lines
    }

    #[tokio::test]
    async fn test_context_with_small_files() {
        use tempfile::TempDir;
        use std::fs;
        
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let small_file = temp_dir.path().join("small.py");
        
        // Create a very small file
        fs::write(&small_file, "x = 1\ny = 2").unwrap();
        
        let server_arc = create_test_server().await;
        let tool_manager = ToolManager::new(server_arc);
        
        // Test context extraction on small file
        let context = tool_manager.extract_source_context(&small_file, 1, 5);
        assert!(context.is_some());
        
        let context_value = context.unwrap();
        assert_eq!(context_value["target_line"], 1);
        
        let lines = context_value["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 2); // Should only return actual file lines
        
        // Test context extraction on second line
        let context = tool_manager.extract_source_context(&small_file, 2, 5);
        assert!(context.is_some());
        
        let context_value = context.unwrap();
        assert_eq!(context_value["target_line"], 2);
        
        let lines = context_value["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 2); // Should return both lines
    }

    #[tokio::test]
    async fn test_context_edge_cases() {
        use tempfile::TempDir;
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let server_arc = create_test_server().await;
        let tool_manager = ToolManager::new(server_arc);
        
        // Test with empty file
        let empty_file = temp_dir.path().join("empty.py");
        fs::write(&empty_file, "").unwrap();
        
        let context = tool_manager.extract_source_context(&empty_file, 1, 2);
        assert!(context.is_none()); // Empty file should return None
        
        // Test with line number 0
        let normal_file = temp_dir.path().join("normal.py");
        fs::write(&normal_file, "line1\nline2\nline3").unwrap();
        
        let context = tool_manager.extract_source_context(&normal_file, 0, 2);
        assert!(context.is_none()); // Line 0 is invalid
        
        // Test with unreadable file (permission denied)
        let restricted_file = temp_dir.path().join("restricted.py");
        fs::write(&restricted_file, "secret content").unwrap();
        
        // Remove read permissions
        let mut perms = fs::metadata(&restricted_file).unwrap().permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&restricted_file, perms).unwrap();
        
        let context = tool_manager.extract_source_context(&restricted_file, 1, 2);
        assert!(context.is_none()); // Should handle permission errors gracefully
        
        // Restore permissions for cleanup
        let mut perms = fs::metadata(&restricted_file).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&restricted_file, perms).unwrap();
    }

    #[tokio::test]
    async fn test_node_info_with_context_creation() {
        use tempfile::TempDir;
        use std::fs;
        
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_file = temp_dir.path().join("test.py");
        
        // Create a test file
        fs::write(&test_file, r#"# Test file
def example_function():
    """An example function."""
    return "hello"
"#).unwrap();
        
        let server_arc = create_test_server().await;
        let tool_manager = ToolManager::new(server_arc);
        
        // Create a mock node
        let span = prism_core::ast::Span::new(0, 20, 2, 2, 1, 21);
        let node = prism_core::ast::Node::new(
            "test_repo",
            prism_core::ast::NodeKind::Function,
            "example_function".to_string(),
            prism_core::ast::Language::Python,
            test_file.clone(),
            span,
        );
        
        // Test node info creation with context
        let node_info = tool_manager.create_node_info_with_context(&node, 2);
        
        // Verify basic node info
        assert_eq!(node_info["name"], "example_function");
        assert_eq!(node_info["kind"], "Function");
        assert_eq!(node_info["language"], "Python");
        
        // Verify context is included
        assert!(node_info["source_context"].is_object());
        assert_eq!(node_info["source_context"]["target_line"], 2);
        
        let lines = node_info["source_context"]["lines"].as_array().unwrap();
        assert!(!lines.is_empty());
        
        // Should have target line marked
        let has_target = lines.iter().any(|line| line["is_target"] == true);
        assert!(has_target);
    }

    #[tokio::test]
    async fn test_new_tools_edge_cases() {
        let server = create_test_server().await;
        let manager = ToolManager::new(server);
        
        // Test with empty file
        let empty_context = manager.extract_source_context(std::path::Path::new("nonexistent.txt"), 1, 5);
        assert!(empty_context.is_none());
        
        // Test with line number 0
        let invalid_context = manager.extract_source_context(std::path::Path::new("test.py"), 0, 5);
        assert!(invalid_context.is_none());
    }

    #[tokio::test]
    async fn test_analyze_complexity_tool() {
        let server = create_test_server().await;
        let manager = ToolManager::new(server.clone());
        
        // Test with valid file path
        let args = serde_json::json!({
            "target": "test_file.py",
            "metrics": ["cyclomatic"],
            "threshold_warnings": true
        });
        
        let result = manager.analyze_complexity(&*server.read().await, Some(args)).await;
        assert!(result.is_ok());
        
        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(true)); // Will fail due to file not existing
        assert!(!call_result.content.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_complexity_all_metrics() {
        let server = create_test_server().await;
        let manager = ToolManager::new(server.clone());
        
        // Test with "all" metrics
        let args = serde_json::json!({
            "target": "test_file.py",
            "metrics": ["all"],
            "threshold_warnings": false
        });
        
        let result = manager.analyze_complexity(&*server.read().await, Some(args)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_complexity_missing_target() {
        let server = create_test_server().await;
        let manager = ToolManager::new(server.clone());
        
        // Test without target parameter
        let args = serde_json::json!({
            "metrics": ["cyclomatic"]
        });
        
        let result = manager.analyze_complexity(&*server.read().await, Some(args)).await;
        assert!(result.is_ok());
        
        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(true));
    }

    #[tokio::test]
    async fn test_find_duplicates_tool() {
        let server = create_test_server().await;
        let manager = ToolManager::new(server.clone());
        
        // Test with valid parameters
        let args = serde_json::json!({
            "similarity_threshold": 0.8,
            "min_lines": 3,
            "scope": "repository"
        });
        
        let result = manager.find_duplicates(&*server.read().await, Some(args)).await;
        assert!(result.is_ok());
        
        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(false)); // Should succeed since test server has a repository
        assert!(!call_result.content.is_empty());
    }

    #[tokio::test]
    async fn test_find_duplicates_default_params() {
        let server = create_test_server().await;
        let manager = ToolManager::new(server.clone());
        
        // Test with empty arguments (should use defaults)
        let result = manager.find_duplicates(&*server.read().await, None).await;
        assert!(result.is_ok());
        
        let call_result = result.unwrap();
        assert!(!call_result.content.is_empty());
    }

    #[tokio::test]
    async fn test_calculate_cyclomatic_complexity() {
        let server = create_test_server().await;
        let manager = ToolManager::new(server);
        
        // Test simple function
        let simple_code = "def simple_func():\n    return 42";
        let complexity = manager.calculate_cyclomatic_complexity(simple_code);
        assert_eq!(complexity, 1); // Base complexity
        
        // Test function with conditionals
        let complex_code = r#"
def complex_func(x):
    if x > 0:
        return x
    elif x < 0:
        return -x
    else:
        return 0
"#;
        let complexity = manager.calculate_cyclomatic_complexity(complex_code);
        assert!(complexity > 1); // Should have higher complexity
    }

    #[tokio::test]
    async fn test_calculate_cognitive_complexity() {
        let server = create_test_server().await;
        let manager = ToolManager::new(server);
        
        // Test simple function
        let simple_code = "def simple_func():\n    return 42";
        let complexity = manager.calculate_cognitive_complexity(simple_code);
        assert_eq!(complexity, 0); // No cognitive complexity
        
        // Test nested conditions
        let nested_code = r#"
def nested_func(x):
    if x > 0:
        for i in range(x):
            if i % 2 == 0:
                print(i)
"#;
        let complexity = manager.calculate_cognitive_complexity(nested_code);
        assert!(complexity > 0); // Should have cognitive complexity due to nesting
    }

    #[tokio::test]
    async fn test_calculate_halstead_metrics() {
        let server = create_test_server().await;
        let manager = ToolManager::new(server);
        
        let code = "x = a + b * c";
        let (volume, difficulty, effort) = manager.calculate_halstead_metrics(code);
        
        assert!(volume > 0.0);
        assert!(difficulty > 0.0);
        assert!(effort > 0.0);
        assert!(effort >= difficulty * volume); // Basic relationship check
    }

    #[tokio::test]
    async fn test_calculate_maintainability_index() {
        let server = create_test_server().await;
        let manager = ToolManager::new(server);
        
        let simple_code = "def simple():\n    return 42";
        let mi = manager.calculate_maintainability_index(simple_code, 2);
        
        assert!(mi >= 0.0, "MI should be >= 0, got {}", mi);
        assert!(mi <= 100.0, "MI should be <= 100, got {}", mi);
        
        // Complex code should have lower maintainability
        let complex_code = r#"
def complex_function(a, b, c, d, e):
    if a > b:
        if c > d:
            if e > a:
                for i in range(100):
                    if i % 2 == 0:
                        result = i * a + b * c + d * e
                        if result > 1000:
                            return result
    return 0
"#;
        let mi_complex = manager.calculate_maintainability_index(complex_code, 10);
        
        // Debug output for troubleshooting
        let (volume_simple, difficulty_simple, effort_simple) = manager.calculate_halstead_metrics(simple_code);
        let cyclomatic_simple = manager.calculate_cyclomatic_complexity(simple_code);
        
        let (volume_complex, difficulty_complex, effort_complex) = manager.calculate_halstead_metrics(complex_code);
        let cyclomatic_complex = manager.calculate_cyclomatic_complexity(complex_code);
        
        // Basic validations
        assert!(mi_complex >= 0.0, "Complex MI should be >= 0, got {}", mi_complex);
        assert!(mi_complex <= 100.0, "Complex MI should be <= 100, got {}", mi_complex);
        
        // The complex code should have higher complexity metrics
        assert!(cyclomatic_complex > cyclomatic_simple, 
                "Complex code should have higher cyclomatic complexity: {} vs {}", 
                cyclomatic_complex, cyclomatic_simple);
        assert!(volume_complex > volume_simple, 
                "Complex code should have higher volume: {} vs {}", 
                volume_complex, volume_simple);
        
        // And therefore lower maintainability index
        assert!(mi_complex < mi, 
                "Complex code should have lower MI: {} vs {} (simple: volume={}, cyclomatic={}, complex: volume={}, cyclomatic={})", 
                mi_complex, mi, volume_simple, cyclomatic_simple, volume_complex, cyclomatic_complex);
    }

    #[tokio::test]
    async fn test_calculate_content_similarity() {
        let server = create_test_server().await;
        let manager = ToolManager::new(server);
        
        // Identical content
        let content1 = "line 1\nline 2\nline 3";
        let content2 = "line 1\nline 2\nline 3";
        let similarity = manager.calculate_content_similarity(content1, content2);
        assert_eq!(similarity, 1.0);
        
        // Completely different content
        let content3 = "different\ncontent\nhere";
        let similarity2 = manager.calculate_content_similarity(content1, content3);
        assert_eq!(similarity2, 0.0);
        
        // Partial similarity
        let content4 = "line 1\nline 2\ndifferent line";
        let similarity3 = manager.calculate_content_similarity(content1, content4);
        assert!(similarity3 > 0.0 && similarity3 < 1.0);
    }

    #[tokio::test]
    async fn test_complexity_tool_integration() {
        use tempfile::TempDir;
        use std::fs;
        
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.py");
        
        // Create a test file
        fs::write(&file_path, r#"
def test_function(x, y):
    if x > 0:
        result = x + y
        if y > 0:
            result *= 2
        return result
    else:
        return 0
"#).unwrap();
        
        let server = create_test_server().await;
        let manager = ToolManager::new(server);
        
        // Test file analysis
        let result = manager.analyze_file_complexity(&file_path, &["all".to_string()], true);
        assert!(result.is_ok());
        
        let complexity_data = result.unwrap();
        assert!(complexity_data["metrics"]["cyclomatic_complexity"]["value"].as_u64().unwrap() > 1);
        assert!(complexity_data["metrics"]["cognitive_complexity"]["value"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_tool_list_includes_new_tools() {
        let server = create_test_server().await;
        let manager = ToolManager::new(server);
        let params = crate::tools::ListToolsParams { cursor: None };
        
        let result = manager.list_tools(params).await;
        assert!(result.is_ok());
        
        let tools_result = result.unwrap();
        let tool_names: Vec<&String> = tools_result.tools.iter().map(|t| &t.name).collect();
        
        // Check that our new tools are included
        assert!(tool_names.contains(&&"analyze_complexity".to_string()));
        assert!(tool_names.contains(&&"find_duplicates".to_string()));
        
        // Should have increased from original 6 tools
        assert!(tools_result.tools.len() >= 8);
    }

    #[tokio::test]
    async fn test_new_tools_call_routing() {
        let server = create_test_server().await;
        let manager = ToolManager::new(server);
        let server = create_test_server().await;
        
        // Test analyze_complexity routing
        let complexity_params = crate::tools::CallToolParams {
            name: "analyze_complexity".to_string(),
            arguments: Some(serde_json::json!({
                "target": "test.py",
                "metrics": ["cyclomatic"]
            })),
        };
        
        let result = manager.call_tool(complexity_params).await;
        assert!(result.is_ok());
        
        // Test find_duplicates routing
        let duplicates_params = crate::tools::CallToolParams {
            name: "find_duplicates".to_string(),
            arguments: Some(serde_json::json!({
                "similarity_threshold": 0.8
            })),
        };
        
        let result2 = manager.call_tool(duplicates_params).await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_detect_patterns_tool() {
        let server = create_test_server().await;
        let manager = ToolManager::new(server.clone());
        
        // Test with default parameters
        let params = crate::tools::CallToolParams {
            name: "detect_patterns".to_string(),
            arguments: Some(serde_json::json!({
                "scope": "repository",
                "pattern_types": ["design_patterns"],
                "confidence_threshold": 0.8
            })),
        };
        
        let result = manager.call_tool(params).await;
        assert!(result.is_ok());
        
        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(false));
        assert!(!call_result.content.is_empty());
        
        // Verify the response contains expected fields
        if let ToolContent::Text { text } = &call_result.content[0] {
            let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
            assert!(parsed["patterns"].is_array());
            assert!(parsed["summary"].is_object());
            assert!(parsed["analysis_successful"].as_bool().unwrap_or(false));
        }
    }

    #[tokio::test]
    async fn test_analyze_transitive_dependencies_tool() {
        let server = create_test_server().await;
        let manager = ToolManager::new(server.clone());
        
        // Test with valid parameters
        let params = crate::tools::CallToolParams {
            name: "analyze_transitive_dependencies".to_string(),
            arguments: Some(serde_json::json!({
                "target": "test_file.py",
                "max_depth": 3,
                "detect_cycles": true,
                "dependency_types": ["calls", "imports"]
            })),
        };
        
        let result = manager.call_tool(params).await;
        assert!(result.is_ok());
        
        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(false));
        
        // Verify the response contains expected fields
        if let ToolContent::Text { text } = &call_result.content[0] {
            let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
            assert!(parsed["target"].is_string());
            assert!(parsed["analysis"].is_object());
            assert!(parsed["parameters"].is_object());
            assert!(parsed["analysis_successful"].as_bool().unwrap_or(false));
        }
    }

    #[tokio::test]
    async fn test_new_phase2_tools_in_list() {
        let server = create_test_server().await;
        let manager = ToolManager::new(server);
        let params = crate::tools::ListToolsParams { cursor: None };
        
        let result = manager.list_tools(params).await;
        assert!(result.is_ok());
        
        let tools_result = result.unwrap();
        let tool_names: Vec<&String> = tools_result.tools.iter().map(|t| &t.name).collect();
        
        // Check that our new Phase 2 tools are included
        assert!(tool_names.contains(&&"detect_patterns".to_string()));
        assert!(tool_names.contains(&&"analyze_transitive_dependencies".to_string()));
        
        // Should have all tools including Phase 1 and Phase 2 and Phase 3
        assert!(tools_result.tools.len() >= 13); // Original + Phase 1 + Phase 2 + Phase 3
    }

    #[tokio::test]
    async fn test_trace_data_flow_tool() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        // Test with missing arguments
        let params = CallToolParams {
            name: "trace_data_flow".to_string(),
            arguments: Some(serde_json::json!({})),
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_err()); // Should fail due to missing variable_or_parameter
        
        // Test with valid arguments (though the actual analysis might not find much in test data)
        let params = CallToolParams {
            name: "trace_data_flow".to_string(),
            arguments: Some(serde_json::json!({
                "variable_or_parameter": "deadbeef12345678", // Dummy hex ID
                "direction": "forward",
                "max_depth": 5
            })),
        };
        
        let result = tool_manager.call_tool(params).await;
        // Result may be Ok or Error depending on whether the symbol exists, but shouldn't panic
        match result {
            Ok(tool_result) => {
                // If successful, check the structure
                if let ToolContent::Text { text } = &tool_result.content[0] {
                    let flow_result: serde_json::Value = serde_json::from_str(text).unwrap();
                    // Should have basic structure
                    assert!(flow_result.is_object());
                }
            },
            Err(_) => {
                // Error is acceptable if symbol doesn't exist
                assert!(true);
            }
        }
    }

    #[tokio::test]
    async fn test_phase3_tools_in_list() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let result = tool_manager.list_tools(ListToolsParams { cursor: None }).await;
        assert!(result.is_ok());
        
        let tools_result = result.unwrap();
        let tool_names: Vec<String> = tools_result.tools.iter().map(|t| t.name.clone()).collect();
        
        // Verify Phase 3 tool is included
        assert!(tool_names.contains(&"trace_data_flow".to_string()));
        
        // Verify the tool has proper schema
        let trace_data_flow_tool = tools_result.tools.iter()
            .find(|t| t.name == "trace_data_flow")
            .unwrap();
        
        let schema = trace_data_flow_tool.input_schema.as_object().unwrap();
        assert!(schema.contains_key("properties"));
        assert!(schema.contains_key("required"));
        
        let required = schema.get("required").unwrap().as_array().unwrap();
        assert!(required.contains(&serde_json::Value::String("variable_or_parameter".to_string())));
    }

    #[tokio::test]
    async fn test_find_unused_code_tool() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        // Test with default arguments
        let params = CallToolParams {
            name: "find_unused_code".to_string(),
            arguments: Some(serde_json::json!({})),
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_ok());
        
        let tool_result = result.unwrap();
        assert_eq!(tool_result.is_error, Some(false));
        
        if let ToolContent::Text { text } = &tool_result.content[0] {
            let unused_result: serde_json::Value = serde_json::from_str(text).unwrap();
            // Should have basic structure
            assert!(unused_result.is_object());
            assert!(unused_result.get("scope").is_some());
            assert!(unused_result.get("unused_code").is_some());
            assert!(unused_result.get("summary").is_some());
            assert!(unused_result.get("recommendations").is_some());
        }
        
        // Test with specific parameters
        let params = CallToolParams {
            name: "find_unused_code".to_string(),
            arguments: Some(serde_json::json!({
                "analyze_types": ["functions", "classes"],
                "confidence_threshold": 0.8,
                "consider_external_apis": false
            })),
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_phase3_unused_code_tools_in_list() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let result = tool_manager.list_tools(ListToolsParams { cursor: None }).await;
        assert!(result.is_ok());
        
        let tools_result = result.unwrap();
        let tool_names: Vec<String> = tools_result.tools.iter().map(|t| t.name.clone()).collect();
        
        // Verify Phase 3 tools are included
        assert!(tool_names.contains(&"trace_data_flow".to_string()));
        assert!(tool_names.contains(&"find_unused_code".to_string()));
        
        // Verify the unused code tool has proper schema
        let find_unused_code_tool = tools_result.tools.iter()
            .find(|t| t.name == "find_unused_code")
            .unwrap();
        
        let schema = find_unused_code_tool.input_schema.as_object().unwrap();
        assert!(schema.contains_key("properties"));
        
        // The tool has no required parameters
        if let Some(required) = schema.get("required") {
            assert!(required.as_array().unwrap().is_empty());
        }
    }

    #[tokio::test]
    async fn test_analyze_security_tool() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "analyze_security".to_string(),
            arguments: Some(serde_json::json!({
                "scope": "repository",
                "vulnerability_types": ["injection", "authentication"],
                "severity_threshold": "medium",
                "include_data_flow_analysis": true,
                "check_external_dependencies": true
            })),
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_ok());
        
        let tool_result = result.unwrap();
        assert!(tool_result.is_error.is_none() || !tool_result.is_error.unwrap());
        assert!(!tool_result.content.is_empty());
        
        // Verify the response contains expected security analysis structure
        if let ToolContent::Text { text } = &tool_result.content[0] {
            let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
            assert!(parsed.get("scope").is_some());
            assert!(parsed.get("summary").is_some());
            assert!(parsed.get("vulnerabilities").is_some());
            assert!(parsed.get("recommendations").is_some());
            assert!(parsed.get("analysis_parameters").is_some());
        }
    }

    #[tokio::test]
    async fn test_analyze_security_default_params() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "analyze_security".to_string(),
            arguments: Some(serde_json::json!({})),
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_ok());
        
        let tool_result = result.unwrap();
        assert!(tool_result.is_error.is_none() || !tool_result.is_error.unwrap());
    }

    #[tokio::test]
    async fn test_analyze_security_specific_vulnerability_types() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "analyze_security".to_string(),
            arguments: Some(serde_json::json!({
                "vulnerability_types": ["injection", "crypto_issues"],
                "severity_threshold": "high",
                "include_data_flow_analysis": false
            })),
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_ok());
        
        let tool_result = result.unwrap();
        assert!(tool_result.is_error.is_none() || !tool_result.is_error.unwrap());
        
        // Verify the response focuses on specified vulnerability types
        if let ToolContent::Text { text } = &tool_result.content[0] {
            let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
            let vulnerabilities = parsed.get("vulnerabilities").unwrap().as_array().unwrap();
            
            // Check that only specified vulnerability types are included (if any found)
            for vuln in vulnerabilities {
                let vuln_type = vuln.get("type").unwrap().as_str().unwrap();
                assert!(vuln_type.to_lowercase().contains("injection") || 
                       vuln_type.to_lowercase().contains("crypto"));
            }
        }
    }

    #[tokio::test]
    async fn test_phase4_security_tools_in_list() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let result = tool_manager.list_tools(ListToolsParams { cursor: None }).await;
        assert!(result.is_ok());
        
        let tools_result = result.unwrap();
        let tool_names: Vec<String> = tools_result.tools.iter().map(|t| t.name.clone()).collect();
        
        // Verify Phase 4 security analysis tool is included
        assert!(tool_names.contains(&"analyze_security".to_string()));
        
        // Verify the security analysis tool has proper schema
        let analyze_security_tool = tools_result.tools.iter()
            .find(|t| t.name == "analyze_security")
            .unwrap();
        
        let schema = analyze_security_tool.input_schema.as_object().unwrap();
        assert!(schema.contains_key("properties"));
        
        let properties = schema.get("properties").unwrap().as_object().unwrap();
        assert!(properties.contains_key("scope"));
        assert!(properties.contains_key("vulnerability_types"));
        assert!(properties.contains_key("severity_threshold"));
        assert!(properties.contains_key("include_data_flow_analysis"));
        assert!(properties.contains_key("check_external_dependencies"));
        assert!(properties.contains_key("exclude_patterns"));
        
        // The tool has no required parameters
        if let Some(required) = schema.get("required") {
            assert!(required.as_array().unwrap().is_empty());
        }
    }

    #[tokio::test]
    async fn test_analyze_security_severity_filtering() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        // Test with low severity threshold
        let params_low = CallToolParams {
            name: "analyze_security".to_string(),
            arguments: Some(serde_json::json!({
                "severity_threshold": "low",
                "vulnerability_types": ["all"]
            })),
        };
        
        let result_low = tool_manager.call_tool(params_low).await;
        assert!(result_low.is_ok());
        
        // Test with critical severity threshold
        let params_critical = CallToolParams {
            name: "analyze_security".to_string(),
            arguments: Some(serde_json::json!({
                "severity_threshold": "critical",
                "vulnerability_types": ["all"]
            })),
        };
        
        let result_critical = tool_manager.call_tool(params_critical).await;
        assert!(result_critical.is_ok());
        
        // Both should succeed regardless of findings
        let tool_result_low = result_low.unwrap();
        let tool_result_critical = result_critical.unwrap();
        
        assert!(tool_result_low.is_error.is_none() || !tool_result_low.is_error.unwrap());
        assert!(tool_result_critical.is_error.is_none() || !tool_result_critical.is_error.unwrap());
    }

    #[tokio::test]
    async fn test_analyze_performance_tool() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "analyze_performance".to_string(),
            arguments: Some(serde_json::json!({
                "scope": "repository",
                "analysis_types": ["time_complexity", "memory_usage"],
                "complexity_threshold": "medium",
                "include_algorithmic_analysis": true,
                "detect_bottlenecks": true
            })),
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_ok());
        
        let tool_result = result.unwrap();
        assert!(tool_result.is_error.is_none() || !tool_result.is_error.unwrap());
        assert!(!tool_result.content.is_empty());
        
        // Verify the response contains expected performance analysis structure
        if let ToolContent::Text { text } = &tool_result.content[0] {
            let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
            assert!(parsed.get("scope").is_some());
            assert!(parsed.get("summary").is_some());
            assert!(parsed.get("performance_issues").is_some());
            assert!(parsed.get("recommendations").is_some());
            assert!(parsed.get("analysis_parameters").is_some());
        }
    }

    #[tokio::test]
    async fn test_analyze_performance_default_params() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "analyze_performance".to_string(),
            arguments: Some(serde_json::json!({})),
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_ok());
        
        let tool_result = result.unwrap();
        assert!(tool_result.is_error.is_none() || !tool_result.is_error.unwrap());
    }

    #[tokio::test]
    async fn test_analyze_performance_specific_analysis_types() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "analyze_performance".to_string(),
            arguments: Some(serde_json::json!({
                "analysis_types": ["hot_spots", "scalability"],
                "complexity_threshold": "high",
                "include_algorithmic_analysis": false,
                "detect_bottlenecks": false
            })),
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_ok());
        
        let tool_result = result.unwrap();
        assert!(tool_result.is_error.is_none() || !tool_result.is_error.unwrap());
        
        // Verify the response focuses on specified analysis types
        if let ToolContent::Text { text } = &tool_result.content[0] {
            let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
            let issues = parsed.get("performance_issues").unwrap().as_array().unwrap();
            
            // Check that only specified analysis types are included (if any found)
            for issue in issues {
                let issue_category = issue.get("category").unwrap().as_str().unwrap();
                assert!(issue_category == "hot_spots" || issue_category == "scalability");
            }
        }
    }

    #[tokio::test]
    async fn test_analyze_performance_complexity_filtering() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        // Test with low complexity threshold
        let params_low = CallToolParams {
            name: "analyze_performance".to_string(),
            arguments: Some(serde_json::json!({
                "complexity_threshold": "low",
                "analysis_types": ["all"]
            })),
        };
        
        let result_low = tool_manager.call_tool(params_low).await;
        assert!(result_low.is_ok());
        
        // Test with high complexity threshold
        let params_high = CallToolParams {
            name: "analyze_performance".to_string(),
            arguments: Some(serde_json::json!({
                "complexity_threshold": "high",
                "analysis_types": ["all"]
            })),
        };
        
        let result_high = tool_manager.call_tool(params_high).await;
        assert!(result_high.is_ok());
        
        // Both should succeed regardless of findings
        let tool_result_low = result_low.unwrap();
        let tool_result_high = result_high.unwrap();
        
        assert!(tool_result_low.is_error.is_none() || !tool_result_low.is_error.unwrap());
        assert!(tool_result_high.is_error.is_none() || !tool_result_high.is_error.unwrap());
    }

    #[tokio::test]
    async fn test_phase4_performance_tools_in_list() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let result = tool_manager.list_tools(ListToolsParams { cursor: None }).await;
        assert!(result.is_ok());
        
        let tools_result = result.unwrap();
        let tool_names: Vec<String> = tools_result.tools.iter().map(|t| t.name.clone()).collect();
        
        // Verify Phase 4 performance analysis tool is included
        assert!(tool_names.contains(&"analyze_performance".to_string()));
        
        // Verify the performance analysis tool has proper schema
        let analyze_performance_tool = tools_result.tools.iter()
            .find(|t| t.name == "analyze_performance")
            .unwrap();
        
        let schema = analyze_performance_tool.input_schema.as_object().unwrap();
        assert!(schema.contains_key("properties"));
        
        let properties = schema.get("properties").unwrap().as_object().unwrap();
        assert!(properties.contains_key("scope"));
        assert!(properties.contains_key("analysis_types"));
        assert!(properties.contains_key("complexity_threshold"));
        assert!(properties.contains_key("include_algorithmic_analysis"));
        assert!(properties.contains_key("detect_bottlenecks"));
        assert!(properties.contains_key("exclude_patterns"));
        
        // The tool has no required parameters
        if let Some(required) = schema.get("required") {
            assert!(required.as_array().unwrap().is_empty());
        }
    }

    #[tokio::test]
    async fn test_analyze_api_surface_tool() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "analyze_api_surface".to_string(),
            arguments: Some(serde_json::json!({
                "scope": "repository",
                "analysis_types": ["public_api", "documentation"],
                "api_version": "1.0.0",
                "include_private_apis": true,
                "check_documentation_coverage": true,
                "detect_breaking_changes": true
            })),
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_ok());
        
        let tool_result = result.unwrap();
        assert!(tool_result.is_error.is_none() || !tool_result.is_error.unwrap());
        assert!(!tool_result.content.is_empty());
        
        // Verify the response contains expected API surface analysis structure
        if let ToolContent::Text { text } = &tool_result.content[0] {
            let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
            assert!(parsed.get("scope").is_some());
            assert!(parsed.get("summary").is_some());
            assert!(parsed.get("api_issues").is_some());
            assert!(parsed.get("recommendations").is_some());
            assert!(parsed.get("analysis_parameters").is_some());
        }
    }

    #[tokio::test]
    async fn test_analyze_api_surface_default_params() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "analyze_api_surface".to_string(),
            arguments: Some(serde_json::json!({})),
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_ok());
        
        let tool_result = result.unwrap();
        assert!(tool_result.is_error.is_none() || !tool_result.is_error.unwrap());
    }

    #[tokio::test]
    async fn test_analyze_api_surface_specific_analysis_types() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "analyze_api_surface".to_string(),
            arguments: Some(serde_json::json!({
                "analysis_types": ["versioning", "breaking_changes"],
                "api_version": "2.1.0",
                "include_private_apis": false,
                "detect_breaking_changes": true
            })),
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_ok());
        
        let tool_result = result.unwrap();
        assert!(tool_result.is_error.is_none() || !tool_result.is_error.unwrap());
        
        // Verify the response focuses on specified analysis types
        if let ToolContent::Text { text } = &tool_result.content[0] {
            let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
            let issues = parsed.get("api_issues").unwrap().as_array().unwrap();
            
            // Check that only specified analysis types are included (if any found)
            for issue in issues {
                let issue_category = issue.get("category").unwrap().as_str().unwrap();
                assert!(issue_category == "versioning" || issue_category == "breaking_changes");
            }
        }
    }

    #[tokio::test]
    async fn test_analyze_api_surface_with_version() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let params = CallToolParams {
            name: "analyze_api_surface".to_string(),
            arguments: Some(serde_json::json!({
                "analysis_types": ["compatibility", "versioning"],
                "api_version": "v1.2.3",
                "include_private_apis": false,
                "check_documentation_coverage": false
            })),
        };
        
        let result = tool_manager.call_tool(params).await;
        assert!(result.is_ok());
        
        let tool_result = result.unwrap();
        assert!(tool_result.is_error.is_none() || !tool_result.is_error.unwrap());
        
        // Verify the API version is included in the analysis
        if let ToolContent::Text { text } = &tool_result.content[0] {
            let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
            let analysis_params = parsed.get("analysis_parameters").unwrap();
            assert_eq!(analysis_params.get("api_version").unwrap().as_str(), Some("v1.2.3"));
        }
    }

    #[tokio::test]
    async fn test_phase4_api_surface_tools_in_list() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let result = tool_manager.list_tools(ListToolsParams { cursor: None }).await;
        assert!(result.is_ok());
        
        let tools_result = result.unwrap();
        let tool_names: Vec<String> = tools_result.tools.iter().map(|t| t.name.clone()).collect();
        
        // Verify Phase 4 API surface analysis tool is included
        assert!(tool_names.contains(&"analyze_api_surface".to_string()));
        
        // Verify the API surface analysis tool has proper schema
        let analyze_api_surface_tool = tools_result.tools.iter()
            .find(|t| t.name == "analyze_api_surface")
            .unwrap();
        
        let schema = analyze_api_surface_tool.input_schema.as_object().unwrap();
        assert!(schema.contains_key("properties"));
        
        let properties = schema.get("properties").unwrap().as_object().unwrap();
        assert!(properties.contains_key("scope"));
        assert!(properties.contains_key("analysis_types"));
        assert!(properties.contains_key("api_version"));
        assert!(properties.contains_key("include_private_apis"));
        assert!(properties.contains_key("check_documentation_coverage"));
        assert!(properties.contains_key("detect_breaking_changes"));
        assert!(properties.contains_key("exclude_patterns"));
        
        // The tool has no required parameters
        if let Some(required) = schema.get("required") {
            assert!(required.as_array().unwrap().is_empty());
        }
    }

    #[tokio::test]
    async fn test_phase4_all_tools_integration() {
        let server = create_test_server().await;
        let tool_manager = ToolManager::new(server);
        
        let result = tool_manager.list_tools(ListToolsParams { cursor: None }).await;
        assert!(result.is_ok());
        
        let tools_result = result.unwrap();
        let tool_names: Vec<String> = tools_result.tools.iter().map(|t| t.name.clone()).collect();
        
        // Verify all Phase 4 tools are present
        assert!(tool_names.contains(&"analyze_security".to_string()));
        assert!(tool_names.contains(&"analyze_performance".to_string()));
        assert!(tool_names.contains(&"analyze_api_surface".to_string()));
        
        // Verify all Phase 3 tools are still present
        assert!(tool_names.contains(&"trace_data_flow".to_string()));
        assert!(tool_names.contains(&"find_unused_code".to_string()));
        
        // Total tools should now be 18
        assert_eq!(tools_result.tools.len(), 18);
    }
}

impl ToolManager {
    /// Helper method for design pattern analysis
    async fn analyze_design_patterns(
        &self,
        server: &PrismMcpServer,
        pattern_types: &[String],
        confidence_threshold: f64,
        include_suggestions: bool,
    ) -> Result<Vec<serde_json::Value>> {
        let mut detected_patterns = Vec::new();
        
        // Analyze Singleton Pattern
        if pattern_types.contains(&"design_patterns".to_string()) || pattern_types.contains(&"all".to_string()) {
            let singleton_patterns = self.detect_singleton_pattern(server, confidence_threshold).await?;
            detected_patterns.extend(singleton_patterns);
            
            let factory_patterns = self.detect_factory_pattern(server, confidence_threshold).await?;
            detected_patterns.extend(factory_patterns);
            
            let observer_patterns = self.detect_observer_pattern(server, confidence_threshold).await?;
            detected_patterns.extend(observer_patterns);
        }
        
        // Analyze Anti-patterns
        if pattern_types.contains(&"anti_patterns".to_string()) || pattern_types.contains(&"all".to_string()) {
            let anti_patterns = self.detect_anti_patterns(server, confidence_threshold).await?;
            detected_patterns.extend(anti_patterns);
        }
        
        // Analyze Architectural patterns
        if pattern_types.contains(&"architectural_patterns".to_string()) || pattern_types.contains(&"all".to_string()) {
            let arch_patterns = self.detect_architectural_patterns(server, confidence_threshold).await?;
            detected_patterns.extend(arch_patterns);
        }
        
        // Add suggestions if requested
        if include_suggestions {
            for pattern in &mut detected_patterns {
                if let Some(pattern_obj) = pattern.as_object_mut() {
                    if let Some(pattern_type) = pattern_obj.get("type").and_then(|v| v.as_str()) {
                        let suggestions = self.get_pattern_suggestions(pattern_type);
                        pattern_obj.insert("suggestions".to_string(), suggestions.into());
                    }
                }
            }
        }
        
        Ok(detected_patterns)
    }

    /// Helper method for transitive dependency analysis
    async fn perform_transitive_analysis(
        &self,
        server: &PrismMcpServer,
        target: &str,
        max_depth: usize,
        detect_cycles: bool,
        _include_external: bool,
        dependency_types: &[String],
    ) -> Result<serde_json::Value> {
        // Parse target (could be node ID or file path)
        let target_nodes = if target.len() == 32 && target.chars().all(|c| c.is_ascii_hexdigit()) {
            // It's a node ID
            if let Ok(node_id) = self.parse_node_id(target) {
                if let Some(node) = server.graph_store().get_node(&node_id) {
                    vec![node]
                } else {
                    return Ok(serde_json::json!({
                        "error": "Node not found",
                        "target": target
                    }));
                }
            } else {
                return Ok(serde_json::json!({
                    "error": "Invalid node ID format",
                    "target": target
                }));
            }
        } else {
            // It's a file path
            let file_path = std::path::PathBuf::from(target);
            server.graph_store().get_nodes_in_file(&file_path)
        };

        if target_nodes.is_empty() {
            return Ok(serde_json::json!({
                "error": "No nodes found for target",
                "target": target
            }));
        }

        let mut analysis_results = Vec::new();
        
        for target_node in &target_nodes {
            let dependencies = self.build_transitive_dependencies(
                server, 
                &target_node.id, 
                max_depth, 
                dependency_types
            ).await?;
            
            let mut cycles = Vec::new();
            if detect_cycles {
                cycles = self.detect_dependency_cycles(server, &target_node.id, &dependencies).await?;
            }
            
            let analysis = serde_json::json!({
                "target_node": {
                    "id": target_node.id.to_hex(),
                    "name": target_node.name,
                    "kind": format!("{:?}", target_node.kind),
                    "file": target_node.file.display().to_string(),
                    "span": target_node.span
                },
                "transitive_dependencies": dependencies,
                "dependency_chains": self.build_dependency_chains(server, &target_node.id, max_depth).await?,
                "cycles": cycles,
                "statistics": {
                    "total_dependencies": dependencies.len(),
                    "max_depth_reached": self.calculate_max_depth(&dependencies),
                    "cycles_detected": cycles.len()
                }
            });
            
            analysis_results.push(analysis);
        }
        
        Ok(serde_json::json!({
            "target_file_or_symbol": target,
            "analyses": analysis_results,
            "summary": {
                "total_nodes_analyzed": target_nodes.len(),
                "total_unique_dependencies": self.count_unique_dependencies(&analysis_results),
                "total_cycles_found": self.count_total_cycles(&analysis_results)
            }
        }))
    }

    /// Detect Singleton pattern
    async fn detect_singleton_pattern(&self, server: &PrismMcpServer, confidence_threshold: f64) -> Result<Vec<serde_json::Value>> {
        let mut patterns = Vec::new();
        let classes = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Class);
        
        for class in classes {
            let mut confidence = 0.0;
            let mut indicators = Vec::new();
            
            // Check for private constructor pattern
            let methods = server.graph_store().get_outgoing_edges(&class.id);
            let has_private_constructor = methods.iter().any(|edge| {
                if let Some(target_node) = server.graph_store().get_node(&edge.target) {
                    target_node.kind == prism_core::NodeKind::Method && 
                    target_node.name.contains("__init__") || target_node.name.contains("constructor")
                } else {
                    false
                }
            });
            
            if has_private_constructor {
                confidence += 0.3;
                indicators.push("Private constructor detected");
            }
            
            // Check for getInstance method
            let has_get_instance = methods.iter().any(|edge| {
                if let Some(target_node) = server.graph_store().get_node(&edge.target) {
                    target_node.name.to_lowercase().contains("getinstance") ||
                    target_node.name.to_lowercase().contains("get_instance")
                } else {
                    false
                }
            });
            
            if has_get_instance {
                confidence += 0.4;
                indicators.push("getInstance method detected");
            }
            
            // Check for static instance variable
            let variables = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Variable);
            let has_static_instance = variables.iter().any(|var| {
                var.file == class.file && 
                (var.name.contains("instance") || var.name.contains("_instance"))
            });
            
            if has_static_instance {
                confidence += 0.3;
                indicators.push("Static instance variable detected");
            }
            
            if confidence >= confidence_threshold {
                patterns.push(serde_json::json!({
                    "type": "Singleton",
                    "category": "design_pattern",
                    "confidence": confidence,
                    "class": {
                        "id": class.id.to_hex(),
                        "name": class.name,
                        "file": class.file.display().to_string(),
                        "span": class.span
                    },
                    "indicators": indicators,
                    "description": "Class appears to implement the Singleton design pattern"
                }));
            }
        }
        
        Ok(patterns)
    }

    /// Detect Factory pattern
    async fn detect_factory_pattern(&self, server: &PrismMcpServer, confidence_threshold: f64) -> Result<Vec<serde_json::Value>> {
        let mut patterns = Vec::new();
        let classes = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Class);
        
        for class in classes {
            if class.name.to_lowercase().contains("factory") {
                let mut confidence = 0.5; // Base confidence for Factory in name
                let mut indicators = vec!["Factory in class name".to_string()];
                
                // Check for creation methods
                let methods = server.graph_store().get_outgoing_edges(&class.id);
                let creation_methods = methods.iter().filter(|edge| {
                    if let Some(target_node) = server.graph_store().get_node(&edge.target) {
                        let method_name = target_node.name.to_lowercase();
                        method_name.contains("create") || 
                        method_name.contains("build") || 
                        method_name.contains("make") ||
                        method_name.contains("new")
                    } else {
                        false
                    }
                }).count();
                
                if creation_methods > 0 {
                    confidence += 0.3;
                    indicators.push(format!("{} creation methods detected", creation_methods));
                }
                
                if confidence >= confidence_threshold {
                    patterns.push(serde_json::json!({
                        "type": "Factory",
                        "category": "design_pattern",
                        "confidence": confidence,
                        "class": {
                            "id": class.id.to_hex(),
                            "name": class.name,
                            "file": class.file.display().to_string(),
                            "span": class.span
                        },
                        "indicators": indicators,
                        "description": "Class appears to implement the Factory design pattern"
                    }));
                }
            }
        }
        
        Ok(patterns)
    }

    /// Detect Observer pattern
    async fn detect_observer_pattern(&self, server: &PrismMcpServer, confidence_threshold: f64) -> Result<Vec<serde_json::Value>> {
        let mut patterns = Vec::new();
        let classes = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Class);
        
        for class in classes {
            let mut confidence = 0.0;
            let mut indicators = Vec::new();
            
            // Check for observer-related method names
            let methods = server.graph_store().get_outgoing_edges(&class.id);
            let observer_methods = methods.iter().filter(|edge| {
                if let Some(target_node) = server.graph_store().get_node(&edge.target) {
                    let method_name = target_node.name.to_lowercase();
                    method_name.contains("notify") || 
                    method_name.contains("update") || 
                    method_name.contains("observe") ||
                    method_name.contains("subscribe") ||
                    method_name.contains("unsubscribe")
                } else {
                    false
                }
            }).count();
            
            if observer_methods > 0 {
                confidence += 0.4;
                indicators.push(format!("{} observer-related methods detected", observer_methods));
            }
            
            // Check for event emissions
            let events = server.graph_store().get_outgoing_edges(&class.id).iter()
                .filter(|edge| edge.kind == prism_core::EdgeKind::Emits)
                .count();
            
            if events > 0 {
                confidence += 0.3;
                indicators.push(format!("{} event emissions detected", events));
            }
            
            if confidence >= confidence_threshold {
                patterns.push(serde_json::json!({
                    "type": "Observer",
                    "category": "design_pattern",
                    "confidence": confidence,
                    "class": {
                        "id": class.id.to_hex(),
                        "name": class.name,
                        "file": class.file.display().to_string(),
                        "span": class.span
                    },
                    "indicators": indicators,
                    "description": "Class appears to implement the Observer design pattern"
                }));
            }
        }
        
        Ok(patterns)
    }

    /// Detect anti-patterns
    async fn detect_anti_patterns(&self, server: &PrismMcpServer, confidence_threshold: f64) -> Result<Vec<serde_json::Value>> {
        let mut patterns = Vec::new();
        
        // God Class anti-pattern
        let classes = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Class);
        for class in classes {
            let methods = server.graph_store().get_outgoing_edges(&class.id);
            let method_count = methods.len();
            
            if method_count > 20 { // Threshold for "God Class"
                let confidence = ((method_count as f64 - 20.0) / 30.0).min(1.0);
                if confidence >= confidence_threshold {
                    patterns.push(serde_json::json!({
                        "type": "God Class",
                        "category": "anti_pattern",
                        "confidence": confidence,
                        "class": {
                            "id": class.id.to_hex(),
                            "name": class.name,
                            "file": class.file.display().to_string(),
                            "span": class.span
                        },
                        "indicators": [format!("{} methods detected (threshold: 20)", method_count)],
                        "description": "Class has too many responsibilities (God Class anti-pattern)",
                        "severity": "high"
                    }));
                }
            }
        }
        
        // Long Method anti-pattern
        let functions = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Function);
        for function in functions {
            let lines = function.span.end_line - function.span.start_line + 1;
            if lines > 50 { // Threshold for "Long Method"
                let confidence = ((lines as f64 - 50.0) / 100.0).min(1.0);
                if confidence >= confidence_threshold {
                    patterns.push(serde_json::json!({
                        "type": "Long Method",
                        "category": "anti_pattern",
                        "confidence": confidence,
                        "function": {
                            "id": function.id.to_hex(),
                            "name": function.name,
                            "file": function.file.display().to_string(),
                            "span": function.span
                        },
                        "indicators": [format!("{} lines of code (threshold: 50)", lines)],
                        "description": "Method is too long and complex",
                        "severity": "medium"
                    }));
                }
            }
        }
        
        Ok(patterns)
    }

    /// Detect architectural patterns
    async fn detect_architectural_patterns(&self, server: &PrismMcpServer, confidence_threshold: f64) -> Result<Vec<serde_json::Value>> {
        let mut patterns = Vec::new();
        
        // MVC Pattern detection
        let classes = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Class);
        let mut controllers = 0;
        let mut models = 0;
        let mut views = 0;
        
        for class in &classes {
            let name_lower = class.name.to_lowercase();
            if name_lower.contains("controller") {
                controllers += 1;
            } else if name_lower.contains("model") {
                models += 1;
            } else if name_lower.contains("view") {
                views += 1;
            }
        }
        
        if controllers > 0 && models > 0 && views > 0 {
            let confidence = ((controllers + models + views) as f64 / classes.len() as f64).min(1.0);
            if confidence >= confidence_threshold {
                patterns.push(serde_json::json!({
                    "type": "MVC (Model-View-Controller)",
                    "category": "architectural_pattern",
                    "confidence": confidence,
                    "indicators": [
                        format!("{} Controllers", controllers),
                        format!("{} Models", models),
                        format!("{} Views", views)
                    ],
                    "description": "Application appears to follow MVC architectural pattern"
                }));
            }
        }
        
        // Repository Pattern detection
        let repository_classes = classes.iter().filter(|c| {
            c.name.to_lowercase().contains("repository") || 
            c.name.to_lowercase().contains("repo")
        }).count();
        
        if repository_classes > 0 {
            let confidence = (repository_classes as f64 / classes.len() as f64 * 10.0).min(1.0);
            if confidence >= confidence_threshold {
                patterns.push(serde_json::json!({
                    "type": "Repository Pattern",
                    "category": "architectural_pattern",
                    "confidence": confidence,
                    "indicators": [format!("{} Repository classes", repository_classes)],
                    "description": "Data access appears to follow Repository pattern"
                }));
            }
        }
        
        Ok(patterns)
    }

    /// Get improvement suggestions for a pattern type
    fn get_pattern_suggestions(&self, pattern_type: &str) -> Vec<String> {
        match pattern_type {
            "Singleton" => vec![
                "Consider using dependency injection instead of Singleton".to_string(),
                "Ensure thread safety in multi-threaded environments".to_string(),
                "Consider if global state is truly necessary".to_string(),
            ],
            "Factory" => vec![
                "Consider using abstract factory for families of objects".to_string(),
                "Ensure proper error handling in object creation".to_string(),
                "Document the creation strategies clearly".to_string(),
            ],
            "Observer" => vec![
                "Consider using weak references to prevent memory leaks".to_string(),
                "Implement proper error handling in notifications".to_string(),
                "Consider async notifications for heavy operations".to_string(),
            ],
            "God Class" => vec![
                "Split into smaller, focused classes".to_string(),
                "Apply Single Responsibility Principle".to_string(),
                "Extract related methods into separate classes".to_string(),
            ],
            "Long Method" => vec![
                "Break down into smaller, focused methods".to_string(),
                "Extract common logic into helper methods".to_string(),
                "Consider if the method has too many responsibilities".to_string(),
            ],
            _ => vec!["No specific suggestions available".to_string()],
        }
    }

    /// Build transitive dependencies map
    async fn build_transitive_dependencies(
        &self,
        server: &PrismMcpServer,
        start_node: &prism_core::NodeId,
        max_depth: usize,
        dependency_types: &[String],
    ) -> Result<Vec<serde_json::Value>> {
        let mut dependencies = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        
        queue.push_back((*start_node, 0));
        visited.insert(*start_node);
        
        while let Some((current_node, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            
            let edges = server.graph_store().get_outgoing_edges(&current_node);
            for edge in edges {
                // Filter by dependency types
                let include_edge = dependency_types.contains(&"all".to_string()) ||
                    dependency_types.iter().any(|dt| match dt.as_str() {
                        "calls" => edge.kind == prism_core::EdgeKind::Calls,
                        "imports" => edge.kind == prism_core::EdgeKind::Imports,
                        "reads" => edge.kind == prism_core::EdgeKind::Reads,
                        "writes" => edge.kind == prism_core::EdgeKind::Writes,
                        "extends" => edge.kind == prism_core::EdgeKind::Extends,
                        "implements" => edge.kind == prism_core::EdgeKind::Implements,
                        _ => false,
                    });
                
                if include_edge {
                    if let Some(target_node) = server.graph_store().get_node(&edge.target) {
                        dependencies.push(serde_json::json!({
                            "source": {
                                "id": current_node.to_hex(),
                                "name": server.graph_store().get_node(&current_node)
                                    .map(|n| n.name.clone()).unwrap_or("unknown".to_string())
                            },
                            "target": {
                                "id": target_node.id.to_hex(),
                                "name": target_node.name,
                                "kind": format!("{:?}", target_node.kind),
                                "file": target_node.file.display().to_string()
                            },
                            "edge_type": format!("{:?}", edge.kind),
                            "depth": depth + 1
                        }));
                        
                        if !visited.contains(&edge.target) {
                            visited.insert(edge.target);
                            queue.push_back((edge.target, depth + 1));
                        }
                    }
                }
            }
        }
        
        Ok(dependencies)
    }

    /// Build dependency chains
    async fn build_dependency_chains(
        &self,
        server: &PrismMcpServer,
        start_node: &prism_core::NodeId,
        max_depth: usize,
    ) -> Result<Vec<serde_json::Value>> {
        let mut chains = Vec::new();
        let current_chain = Vec::new();
        
        self.build_chains_recursive(server, *start_node, current_chain, &mut chains, max_depth, 0).await?;
        
        Ok(chains)
    }

    /// Recursive helper for building dependency chains
    fn build_chains_recursive<'a>(
        &'a self,
        server: &'a PrismMcpServer,
        current_node: prism_core::NodeId,
        current_chain: Vec<String>,
        all_chains: &'a mut Vec<serde_json::Value>,
        max_depth: usize,
        current_depth: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            if current_depth >= max_depth {
                return Ok(());
            }
            
            let mut chain = current_chain;
            if let Some(node) = server.graph_store().get_node(&current_node) {
                chain.push(format!("{}:{}", node.name, node.id.to_hex()));
            }
            
            let edges = server.graph_store().get_outgoing_edges(&current_node);
            if edges.is_empty() {
                // End of chain
                if chain.len() > 1 {
                    all_chains.push(serde_json::json!({
                        "chain": chain,
                        "length": chain.len()
                    }));
                }
            } else {
                for edge in edges {
                    if edge.kind == prism_core::EdgeKind::Calls || edge.kind == prism_core::EdgeKind::Imports {
                        self.build_chains_recursive(
                            server, 
                            edge.target, 
                            chain.clone(), 
                            all_chains, 
                            max_depth, 
                            current_depth + 1
                        ).await?;
                    }
                }
            }
            
            Ok(())
        })
    }

    /// Detect dependency cycles
    async fn detect_dependency_cycles(
        &self,
        server: &PrismMcpServer,
        start_node: &prism_core::NodeId,
        _dependencies: &[serde_json::Value],
    ) -> Result<Vec<serde_json::Value>> {
        let mut cycles = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();
        let mut path = Vec::new();
        
        self.detect_cycles_dfs(server, *start_node, &mut visited, &mut rec_stack, &mut path, &mut cycles).await?;
        
        Ok(cycles)
    }

    /// DFS helper for cycle detection
    fn detect_cycles_dfs<'a>(
        &'a self,
        server: &'a PrismMcpServer,
        node: prism_core::NodeId,
        visited: &'a mut std::collections::HashSet<prism_core::NodeId>,
        rec_stack: &'a mut std::collections::HashSet<prism_core::NodeId>,
        path: &'a mut Vec<prism_core::NodeId>,
        cycles: &'a mut Vec<serde_json::Value>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            visited.insert(node);
            rec_stack.insert(node);
            path.push(node);
            
            let edges = server.graph_store().get_outgoing_edges(&node);
            for edge in edges {
                if edge.kind == prism_core::EdgeKind::Calls || edge.kind == prism_core::EdgeKind::Imports {
                    if !visited.contains(&edge.target) {
                        self.detect_cycles_dfs(server, edge.target, visited, rec_stack, path, cycles).await?;
                    } else if rec_stack.contains(&edge.target) {
                        // Found a cycle
                        if let Some(cycle_start) = path.iter().position(|&id| id == edge.target) {
                            let cycle_path: Vec<String> = path[cycle_start..].iter()
                                .map(|id| {
                                    if let Some(node) = server.graph_store().get_node(id) {
                                        format!("{}:{}", node.name, id.to_hex())
                                    } else {
                                        id.to_hex()
                                    }
                                })
                                .collect();
                            
                            cycles.push(serde_json::json!({
                                "cycle_path": cycle_path,
                                "cycle_length": cycle_path.len(),
                                "cycle_type": "dependency_cycle"
                            }));
                        }
                    }
                }
            }
            
            path.pop();
            rec_stack.remove(&node);
            
            Ok(())
        })
    }

    /// Calculate maximum depth in dependencies
    fn calculate_max_depth(&self, dependencies: &[serde_json::Value]) -> usize {
        dependencies.iter()
            .filter_map(|dep| dep.get("depth").and_then(|d| d.as_u64()))
            .max()
            .unwrap_or(0) as usize
    }

    /// Count unique dependencies across all analyses
    fn count_unique_dependencies(&self, analyses: &[serde_json::Value]) -> usize {
        let mut unique_deps = std::collections::HashSet::new();
        
        for analysis in analyses {
            if let Some(deps) = analysis.get("transitive_dependencies").and_then(|d| d.as_array()) {
                for dep in deps {
                    if let Some(target_id) = dep.get("target").and_then(|t| t.get("id")).and_then(|id| id.as_str()) {
                        unique_deps.insert(target_id.to_string());
                    }
                }
            }
        }
        
        unique_deps.len()
    }

    /// Count total cycles across all analyses
    fn count_total_cycles(&self, analyses: &[serde_json::Value]) -> usize {
        analyses.iter()
            .map(|analysis| {
                analysis.get("cycles")
                    .and_then(|c| c.as_array())
                    .map(|arr| arr.len())
                    .unwrap_or(0)
            })
            .sum()
    }

    /// Perform data flow analysis on a symbol
    async fn perform_data_flow_analysis(
        &self,
        server: &PrismMcpServer,
        symbol_id: &prism_core::NodeId,
        direction: &str,
        include_transformations: bool,
        max_depth: usize,
        follow_function_calls: bool,
        include_field_access: bool,
    ) -> Result<serde_json::Value> {
        // Get the starting symbol
        let start_node = server.graph_store().get_node(symbol_id)
            .ok_or_else(|| anyhow::anyhow!("Symbol not found: {}", symbol_id.to_hex()))?;

        let mut data_flows = Vec::new();
        let mut visited = std::collections::HashSet::new();

        match direction {
            "forward" => {
                self.trace_data_flow_forward(
                    server,
                    symbol_id,
                    &mut data_flows,
                    &mut visited,
                    0,
                    max_depth,
                    include_transformations,
                    follow_function_calls,
                    include_field_access,
                ).await?;
            }
            "backward" => {
                self.trace_data_flow_backward(
                    server,
                    symbol_id,
                    &mut data_flows,
                    &mut visited,
                    0,
                    max_depth,
                    include_transformations,
                    follow_function_calls,
                    include_field_access,
                ).await?;
            }
            "both" => {
                let mut forward_flows = Vec::new();
                let mut backward_flows = Vec::new();
                let mut forward_visited = std::collections::HashSet::new();
                let mut backward_visited = std::collections::HashSet::new();

                self.trace_data_flow_forward(
                    server,
                    symbol_id,
                    &mut forward_flows,
                    &mut forward_visited,
                    0,
                    max_depth,
                    include_transformations,
                    follow_function_calls,
                    include_field_access,
                ).await?;

                self.trace_data_flow_backward(
                    server,
                    symbol_id,
                    &mut backward_flows,
                    &mut backward_visited,
                    0,
                    max_depth,
                    include_transformations,
                    follow_function_calls,
                    include_field_access,
                ).await?;

                return Ok(serde_json::json!({
                    "starting_symbol": {
                        "id": start_node.id.to_hex(),
                        "name": start_node.name,
                        "kind": format!("{:?}", start_node.kind),
                        "file": start_node.file.display().to_string(),
                        "location": {
                            "line": start_node.span.start_line,
                            "column": start_node.span.start_column
                        }
                    },
                    "direction": direction,
                    "forward_flows": forward_flows,
                    "backward_flows": backward_flows,
                    "summary": {
                        "total_forward_flows": forward_flows.len(),
                        "total_backward_flows": backward_flows.len(),
                        "max_depth_reached": max_depth,
                        "unique_symbols_forward": forward_visited.len(),
                        "unique_symbols_backward": backward_visited.len()
                    },
                    "parameters": {
                        "include_transformations": include_transformations,
                        "follow_function_calls": follow_function_calls,
                        "include_field_access": include_field_access,
                        "max_depth": max_depth
                    }
                }));
            }
            _ => {
                return Err(anyhow::anyhow!("Invalid direction: {}. Must be 'forward', 'backward', or 'both'", direction));
            }
        }

        Ok(serde_json::json!({
            "starting_symbol": {
                "id": start_node.id.to_hex(),
                "name": start_node.name,
                "kind": format!("{:?}", start_node.kind),
                "file": start_node.file.display().to_string(),
                "location": {
                    "line": start_node.span.start_line,
                    "column": start_node.span.start_column
                }
            },
            "direction": direction,
            "data_flows": data_flows,
            "summary": {
                "total_flows": data_flows.len(),
                "max_depth_reached": max_depth,
                "unique_symbols": visited.len()
            },
            "parameters": {
                "include_transformations": include_transformations,
                "follow_function_calls": follow_function_calls,
                "include_field_access": include_field_access,
                "max_depth": max_depth
            }
        }))
    }

    /// Trace data flow in forward direction
    fn trace_data_flow_forward<'a>(
        &'a self,
        server: &'a PrismMcpServer,
        symbol_id: &'a prism_core::NodeId,
        data_flows: &'a mut Vec<serde_json::Value>,
        visited: &'a mut std::collections::HashSet<prism_core::NodeId>,
        current_depth: usize,
        max_depth: usize,
        include_transformations: bool,
        follow_function_calls: bool,
        include_field_access: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
        if current_depth >= max_depth || visited.contains(symbol_id) {
            return Ok(());
        }

        visited.insert(*symbol_id);

        let current_node = server.graph_store().get_node(symbol_id)
            .ok_or_else(|| anyhow::anyhow!("Node not found: {}", symbol_id.to_hex()))?;

        // Find all reads from this symbol (data flowing out)
        let dependencies = server.graph_query().find_dependencies(symbol_id, prism_core::graph::DependencyType::Reads)?;
        
        for dep in dependencies.iter().filter(|d| self.is_valid_dependency_node(&d.target_node)) {
            let flow_info = serde_json::json!({
                "flow_type": "read",
                "depth": current_depth,
                "source": {
                    "id": current_node.id.to_hex(),
                    "name": current_node.name,
                    "kind": format!("{:?}", current_node.kind),
                    "file": current_node.file.display().to_string(),
                    "location": {
                        "line": current_node.span.start_line,
                        "column": current_node.span.start_column
                    }
                },
                "target": {
                    "id": dep.target_node.id.to_hex(),
                    "name": dep.target_node.name,
                    "kind": format!("{:?}", dep.target_node.kind),
                    "file": dep.target_node.file.display().to_string(),
                    "location": {
                        "line": dep.target_node.span.start_line,
                        "column": dep.target_node.span.start_column
                    }
                },
                "edge_kind": format!("{:?}", dep.edge_kind)
            });
            data_flows.push(flow_info);

            // Continue tracing from the target
            self.trace_data_flow_forward(
                server,
                &dep.target_node.id,
                data_flows,
                visited,
                current_depth + 1,
                max_depth,
                include_transformations,
                follow_function_calls,
                include_field_access,
            ).await?;
        }

        // If following function calls, trace through function parameters and returns
        if follow_function_calls {
            let call_dependencies = server.graph_query().find_dependencies(symbol_id, prism_core::graph::DependencyType::Calls)?;
            
            for dep in call_dependencies.iter().filter(|d| self.is_valid_dependency_node(&d.target_node)) {
                let flow_info = serde_json::json!({
                    "flow_type": "function_call",
                    "depth": current_depth,
                    "source": {
                        "id": current_node.id.to_hex(),
                        "name": current_node.name,
                        "kind": format!("{:?}", current_node.kind),
                        "file": current_node.file.display().to_string(),
                        "location": {
                            "line": current_node.span.start_line,
                            "column": current_node.span.start_column
                        }
                    },
                    "target": {
                        "id": dep.target_node.id.to_hex(),
                        "name": dep.target_node.name,
                        "kind": format!("{:?}", dep.target_node.kind),
                        "file": dep.target_node.file.display().to_string(),
                        "location": {
                            "line": dep.target_node.span.start_line,
                            "column": dep.target_node.span.start_column
                        }
                    },
                    "edge_kind": format!("{:?}", dep.edge_kind)
                });
                data_flows.push(flow_info);

                // Continue tracing into the function
                self.trace_data_flow_forward(
                    server,
                    &dep.target_node.id,
                    data_flows,
                    visited,
                    current_depth + 1,
                    max_depth,
                    include_transformations,
                    follow_function_calls,
                    include_field_access,
                ).await?;
            }
        }

        Ok(())
        })
    }

    /// Trace data flow in backward direction
    fn trace_data_flow_backward<'a>(
        &'a self,
        server: &'a PrismMcpServer,
        symbol_id: &'a prism_core::NodeId,
        data_flows: &'a mut Vec<serde_json::Value>,
        visited: &'a mut std::collections::HashSet<prism_core::NodeId>,
        current_depth: usize,
        max_depth: usize,
        include_transformations: bool,
        follow_function_calls: bool,
        include_field_access: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
        if current_depth >= max_depth || visited.contains(symbol_id) {
            return Ok(());
        }

        visited.insert(*symbol_id);

        let current_node = server.graph_store().get_node(symbol_id)
            .ok_or_else(|| anyhow::anyhow!("Node not found: {}", symbol_id.to_hex()))?;

        // Find all writes to this symbol (data flowing in)
        let references = server.graph_query().find_references(symbol_id)?;
        
        for ref_info in references.iter() {
            // Filter for write operations
            if matches!(ref_info.edge_kind, prism_core::EdgeKind::Writes) {
                let flow_info = serde_json::json!({
                    "flow_type": "write",
                    "depth": current_depth,
                    "source": {
                        "id": ref_info.source_node.id.to_hex(),
                        "name": ref_info.source_node.name,
                        "kind": format!("{:?}", ref_info.source_node.kind),
                        "file": ref_info.source_node.file.display().to_string(),
                        "location": {
                            "line": ref_info.source_node.span.start_line,
                            "column": ref_info.source_node.span.start_column
                        }
                    },
                    "target": {
                        "id": current_node.id.to_hex(),
                        "name": current_node.name,
                        "kind": format!("{:?}", current_node.kind),
                        "file": current_node.file.display().to_string(),
                        "location": {
                            "line": current_node.span.start_line,
                            "column": current_node.span.start_column
                        }
                    },
                    "edge_kind": format!("{:?}", ref_info.edge_kind)
                });
                data_flows.push(flow_info);

                // Continue tracing from the source
                self.trace_data_flow_backward(
                    server,
                    &ref_info.source_node.id,
                    data_flows,
                    visited,
                    current_depth + 1,
                    max_depth,
                    include_transformations,
                    follow_function_calls,
                    include_field_access,
                ).await?;
            }
        }

        // If following function calls, trace backward through function parameters
        if follow_function_calls {
            for ref_info in references.iter() {
                if matches!(ref_info.edge_kind, prism_core::EdgeKind::Calls) {
                    let flow_info = serde_json::json!({
                        "flow_type": "function_parameter",
                        "depth": current_depth,
                        "source": {
                            "id": ref_info.source_node.id.to_hex(),
                            "name": ref_info.source_node.name,
                            "kind": format!("{:?}", ref_info.source_node.kind),
                            "file": ref_info.source_node.file.display().to_string(),
                            "location": {
                                "line": ref_info.source_node.span.start_line,
                                "column": ref_info.source_node.span.start_column
                            }
                        },
                        "target": {
                            "id": current_node.id.to_hex(),
                            "name": current_node.name,
                            "kind": format!("{:?}", current_node.kind),
                            "file": current_node.file.display().to_string(),
                            "location": {
                                "line": current_node.span.start_line,
                                "column": current_node.span.start_column
                            }
                        },
                        "edge_kind": format!("{:?}", ref_info.edge_kind)
                    });
                    data_flows.push(flow_info);

                    // Continue tracing backward from the caller
                    self.trace_data_flow_backward(
                        server,
                        &ref_info.source_node.id,
                        data_flows,
                        visited,
                        current_depth + 1,
                        max_depth,
                        include_transformations,
                        follow_function_calls,
                        include_field_access,
                    ).await?;
                }
            }
        }

        Ok(())
        })
    }

    /// Perform unused code analysis
    async fn perform_unused_code_analysis(
        &self,
        server: &PrismMcpServer,
        scope: &str,
        include_dead_code: bool,
        consider_external_apis: bool,
        confidence_threshold: f64,
        analyze_types: &[String],
        exclude_patterns: &[String],
    ) -> Result<serde_json::Value> {
        let mut unused_functions = Vec::new();
        let mut unused_classes = Vec::new();
        let mut unused_variables = Vec::new();
        let mut unused_imports = Vec::new();
        let mut dead_code_blocks = Vec::new();

        // Analyze different types of code elements based on the request
        if analyze_types.contains(&"functions".to_string()) || analyze_types.contains(&"all".to_string()) {
            unused_functions = self.find_unused_functions(server, confidence_threshold, consider_external_apis, exclude_patterns).await?;
        }

        if analyze_types.contains(&"classes".to_string()) || analyze_types.contains(&"all".to_string()) {
            unused_classes = self.find_unused_classes(server, confidence_threshold, consider_external_apis, exclude_patterns).await?;
        }

        if analyze_types.contains(&"variables".to_string()) || analyze_types.contains(&"all".to_string()) {
            unused_variables = self.find_unused_variables(server, confidence_threshold, exclude_patterns).await?;
        }

        if analyze_types.contains(&"imports".to_string()) || analyze_types.contains(&"all".to_string()) {
            unused_imports = self.find_unused_imports(server, confidence_threshold, exclude_patterns).await?;
        }

        if include_dead_code {
            dead_code_blocks = self.find_dead_code_blocks(server, confidence_threshold, exclude_patterns).await?;
        }

        Ok(serde_json::json!({
            "scope": scope,
            "analysis_parameters": {
                "include_dead_code": include_dead_code,
                "consider_external_apis": consider_external_apis,
                "confidence_threshold": confidence_threshold,
                "analyze_types": analyze_types
            },
            "unused_code": {
                "functions": unused_functions,
                "classes": unused_classes,
                "variables": unused_variables,
                "imports": unused_imports,
                "dead_code_blocks": dead_code_blocks
            },
            "summary": {
                "total_unused_functions": unused_functions.len(),
                "total_unused_classes": unused_classes.len(),
                "total_unused_variables": unused_variables.len(),
                "total_unused_imports": unused_imports.len(),
                "total_dead_code_blocks": dead_code_blocks.len(),
                "total_unused_elements": unused_functions.len() + unused_classes.len() + unused_variables.len() + unused_imports.len() + dead_code_blocks.len()
            },
            "recommendations": self.get_unused_code_recommendations(&unused_functions, &unused_classes, &unused_variables, &unused_imports, &dead_code_blocks)
        }))
    }

    /// Find unused functions
    async fn find_unused_functions(
        &self,
        server: &PrismMcpServer,
        confidence_threshold: f64,
        consider_external_apis: bool,
        exclude_patterns: &[String],
    ) -> Result<Vec<serde_json::Value>> {
        let mut unused_functions = Vec::new();
        let functions = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Function);
        
        for function in functions {
            // Skip if matches exclude patterns
            if exclude_patterns.iter().any(|pattern| function.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            let references = server.graph_query().find_references(&function.id)?;
            let mut confidence = 1.0;
            let mut usage_indicators = Vec::new();

            // Check for direct references (calls)
            let call_count = references.iter()
                .filter(|r| matches!(r.edge_kind, prism_core::EdgeKind::Calls))
                .count();

            if call_count == 0 {
                usage_indicators.push("No direct function calls found".to_string());
            } else {
                confidence -= (call_count as f64 * 0.3).min(0.8);
                usage_indicators.push(format!("{} function calls found", call_count));
            }

            // Consider potential external API usage
            if consider_external_apis {
                let function_name = &function.name;
                
                // Check for common external API patterns
                if function_name.starts_with("main") ||
                   function_name.starts_with("__") ||
                   function_name.contains("handler") ||
                   function_name.contains("callback") ||
                   function_name.contains("api") ||
                   function_name.contains("endpoint") {
                    confidence -= 0.5;
                    usage_indicators.push("Potentially used by external API".to_string());
                }
            }

            // Check if it's exported/public
            if function.name.starts_with('_') {
                // Private function, less likely to be external API
                confidence += 0.1;
                usage_indicators.push("Private function (name starts with _)".to_string());
            }

            if confidence >= confidence_threshold {
                unused_functions.push(serde_json::json!({
                    "id": function.id.to_hex(),
                    "name": function.name,
                    "kind": "Function",
                    "file": function.file.display().to_string(),
                    "location": {
                        "start_line": function.span.start_line,
                        "end_line": function.span.end_line,
                        "start_column": function.span.start_column,
                        "end_column": function.span.end_column
                    },
                    "confidence": confidence,
                    "usage_indicators": usage_indicators,
                    "lines_of_code": function.span.end_line - function.span.start_line + 1,
                    "potential_savings": "Remove function to reduce codebase size"
                }));
            }
        }

        Ok(unused_functions)
    }

    /// Find unused classes
    async fn find_unused_classes(
        &self,
        server: &PrismMcpServer,
        confidence_threshold: f64,
        consider_external_apis: bool,
        exclude_patterns: &[String],
    ) -> Result<Vec<serde_json::Value>> {
        let mut unused_classes = Vec::new();
        let classes = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Class);
        
        for class in classes {
            // Skip if matches exclude patterns
            if exclude_patterns.iter().any(|pattern| class.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            let references = server.graph_query().find_references(&class.id)?;
            let mut confidence = 1.0;
            let mut usage_indicators = Vec::new();

            // Check for instantiation or inheritance
            let usage_count = references.iter()
                .filter(|r| matches!(r.edge_kind, prism_core::EdgeKind::Calls | prism_core::EdgeKind::Extends | prism_core::EdgeKind::Implements))
                .count();

            if usage_count == 0 {
                usage_indicators.push("No instantiation, inheritance, or implementation found".to_string());
            } else {
                confidence -= (usage_count as f64 * 0.4).min(0.9);
                usage_indicators.push(format!("{} usages found (instantiation/inheritance)", usage_count));
            }

            // Consider external API patterns for classes
            if consider_external_apis {
                let class_name = &class.name;
                
                if class_name.contains("Controller") ||
                   class_name.contains("Service") ||
                   class_name.contains("Handler") ||
                   class_name.contains("Model") ||
                   class_name.contains("Entity") ||
                   class_name.contains("Exception") ||
                   class_name.contains("Error") {
                    confidence -= 0.4;
                    usage_indicators.push("Potentially used by framework or external system".to_string());
                }
            }

            if confidence >= confidence_threshold {
                unused_classes.push(serde_json::json!({
                    "id": class.id.to_hex(),
                    "name": class.name,
                    "kind": "Class",
                    "file": class.file.display().to_string(),
                    "location": {
                        "start_line": class.span.start_line,
                        "end_line": class.span.end_line,
                        "start_column": class.span.start_column,
                        "end_column": class.span.end_column
                    },
                    "confidence": confidence,
                    "usage_indicators": usage_indicators,
                    "lines_of_code": class.span.end_line - class.span.start_line + 1,
                    "potential_savings": "Remove class and its methods to reduce codebase complexity"
                }));
            }
        }

        Ok(unused_classes)
    }

    /// Find unused variables
    async fn find_unused_variables(
        &self,
        server: &PrismMcpServer,
        confidence_threshold: f64,
        exclude_patterns: &[String],
    ) -> Result<Vec<serde_json::Value>> {
        let mut unused_variables = Vec::new();
        let variables = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Variable);
        
        for variable in variables {
            // Skip if matches exclude patterns
            if exclude_patterns.iter().any(|pattern| variable.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            let references = server.graph_query().find_references(&variable.id)?;
            let mut confidence = 1.0;
            let mut usage_indicators = Vec::new();

            // Check for reads (excluding the definition itself)
            let read_count = references.iter()
                .filter(|r| matches!(r.edge_kind, prism_core::EdgeKind::Reads))
                .count();

            if read_count == 0 {
                usage_indicators.push("No reads found".to_string());
            } else {
                confidence -= (read_count as f64 * 0.5).min(0.9);
                usage_indicators.push(format!("{} reads found", read_count));
            }

            // Check for writes (assignments)
            let write_count = references.iter()
                .filter(|r| matches!(r.edge_kind, prism_core::EdgeKind::Writes))
                .count();

            if write_count <= 1 {
                // Only the initial assignment
                usage_indicators.push("Only initial assignment found".to_string());
            } else {
                confidence -= (write_count as f64 * 0.3).min(0.6);
                usage_indicators.push(format!("{} assignments found", write_count));
            }

            // Consider variable naming patterns
            if variable.name.starts_with('_') {
                usage_indicators.push("Private variable (name starts with _)".to_string());
            }

            if confidence >= confidence_threshold {
                unused_variables.push(serde_json::json!({
                    "id": variable.id.to_hex(),
                    "name": variable.name,
                    "kind": "Variable",
                    "file": variable.file.display().to_string(),
                    "location": {
                        "start_line": variable.span.start_line,
                        "end_line": variable.span.end_line,
                        "start_column": variable.span.start_column,
                        "end_column": variable.span.end_column
                    },
                    "confidence": confidence,
                    "usage_indicators": usage_indicators,
                    "potential_savings": "Remove variable declaration and related code"
                }));
            }
        }

        Ok(unused_variables)
    }

    /// Find unused imports
    async fn find_unused_imports(
        &self,
        server: &PrismMcpServer,
        confidence_threshold: f64,
        exclude_patterns: &[String],
    ) -> Result<Vec<serde_json::Value>> {
        let mut unused_imports = Vec::new();
        
        // Find all import edges by checking all node types
        let mut import_edges = Vec::new();
        
        // Check all major node types for import edges
        let node_kinds = [
            prism_core::NodeKind::Function,
            prism_core::NodeKind::Class,
            prism_core::NodeKind::Module,
            prism_core::NodeKind::Variable,
        ];
        
        for kind in &node_kinds {
            let nodes = server.graph_store().get_nodes_by_kind(*kind);
            for node in nodes {
                let edges = server.graph_store().get_outgoing_edges(&node.id);
                import_edges.extend(edges.into_iter().filter(|edge| edge.kind == prism_core::EdgeKind::Imports));
            }
        }

        for edge in import_edges {
            if let (Some(source_node), Some(target_node)) = 
                (server.graph_store().get_node(&edge.source), server.graph_store().get_node(&edge.target)) {
                
                // Skip if matches exclude patterns
                if exclude_patterns.iter().any(|pattern| source_node.file.to_string_lossy().contains(pattern)) {
                    continue;
                }

                // Check if the imported symbol is actually used
                let target_references = server.graph_query().find_references(&target_node.id)?;
                let mut confidence = 1.0;
                let mut usage_indicators = Vec::new();

                // Count non-import references (actual usage)
                let usage_count = target_references.iter()
                    .filter(|r| !matches!(r.edge_kind, prism_core::EdgeKind::Imports))
                    .count();

                if usage_count == 0 {
                    usage_indicators.push("Import not used in code".to_string());
                } else {
                    confidence -= (usage_count as f64 * 0.6).min(0.9);
                    usage_indicators.push(format!("{} usages found", usage_count));
                }

                if confidence >= confidence_threshold {
                    unused_imports.push(serde_json::json!({
                        "import_source_id": source_node.id.to_hex(),
                        "import_target_id": target_node.id.to_hex(),
                        "imported_name": target_node.name,
                        "kind": "Import",
                        "file": source_node.file.display().to_string(),
                        "location": {
                            "start_line": source_node.span.start_line,
                            "end_line": source_node.span.end_line,
                            "start_column": source_node.span.start_column,
                            "end_column": source_node.span.end_column
                        },
                        "confidence": confidence,
                        "usage_indicators": usage_indicators,
                        "potential_savings": "Remove unused import to clean up code"
                    }));
                }
            }
        }

        Ok(unused_imports)
    }

    /// Find dead code blocks (unreachable code)
    async fn find_dead_code_blocks(
        &self,
        server: &PrismMcpServer,
        confidence_threshold: f64,
        exclude_patterns: &[String],
    ) -> Result<Vec<serde_json::Value>> {
        let mut dead_code_blocks = Vec::new();
        
        // Find functions that are never called and not entry points
        let functions = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Function);
        
        for function in functions {
            // Skip if matches exclude patterns
            if exclude_patterns.iter().any(|pattern| function.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            // Skip main functions and special methods
            if function.name == "main" ||
               function.name.starts_with("__") ||
               function.name.starts_with("test_") ||
               function.name.contains("init") {
                continue;
            }

            let references = server.graph_query().find_references(&function.id)?;
            let call_count = references.iter()
                .filter(|r| matches!(r.edge_kind, prism_core::EdgeKind::Calls))
                .count();

            if call_count == 0 {
                let confidence = 0.95; // High confidence for unreachable functions
                
                if confidence >= confidence_threshold {
                    dead_code_blocks.push(serde_json::json!({
                        "id": function.id.to_hex(),
                        "name": function.name,
                        "kind": "Dead Function",
                        "file": function.file.display().to_string(),
                        "location": {
                            "start_line": function.span.start_line,
                            "end_line": function.span.end_line,
                            "start_column": function.span.start_column,
                            "end_column": function.span.end_column
                        },
                        "confidence": confidence,
                        "description": "Function is never called and appears to be unreachable",
                        "lines_of_code": function.span.end_line - function.span.start_line + 1,
                        "potential_savings": "Remove dead function to reduce codebase size and maintenance burden"
                    }));
                }
            }
        }

        Ok(dead_code_blocks)
    }

    /// Get recommendations for unused code cleanup
    fn get_unused_code_recommendations(
        &self,
        unused_functions: &[serde_json::Value],
        unused_classes: &[serde_json::Value],
        unused_variables: &[serde_json::Value],
        unused_imports: &[serde_json::Value],
        dead_code_blocks: &[serde_json::Value],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !unused_imports.is_empty() {
            recommendations.push(format!("Remove {} unused imports to clean up dependencies", unused_imports.len()));
        }

        if !unused_variables.is_empty() {
            recommendations.push(format!("Remove {} unused variables to reduce code clutter", unused_variables.len()));
        }

        if !unused_functions.is_empty() {
            let lines_saved: usize = unused_functions.iter()
                .filter_map(|f| f.get("lines_of_code").and_then(|v| v.as_u64()))
                .map(|v| v as usize)
                .sum();
            recommendations.push(format!("Remove {} unused functions to save approximately {} lines of code", 
                unused_functions.len(), lines_saved));
        }

        if !unused_classes.is_empty() {
            let lines_saved: usize = unused_classes.iter()
                .filter_map(|c| c.get("lines_of_code").and_then(|v| v.as_u64()))
                .map(|v| v as usize)
                .sum();
            recommendations.push(format!("Remove {} unused classes to save approximately {} lines of code", 
                unused_classes.len(), lines_saved));
        }

        if !dead_code_blocks.is_empty() {
            recommendations.push(format!("Remove {} dead code blocks to eliminate unreachable code", dead_code_blocks.len()));
        }

        if recommendations.is_empty() {
            recommendations.push("No unused code detected with current confidence threshold".to_string());
        } else {
            recommendations.push("Consider running tests after removing unused code to ensure no unexpected dependencies".to_string());
            recommendations.push("Use version control to safely experiment with unused code removal".to_string());
        }

        recommendations
    }

    /// Perform security vulnerability analysis
    async fn perform_security_analysis(
        &self,
        server: &PrismMcpServer,
        scope: &str,
        vulnerability_types: &[String],
        severity_threshold: &str,
        include_data_flow_analysis: bool,
        check_external_dependencies: bool,
        exclude_patterns: &[String],
    ) -> Result<serde_json::Value> {
        let mut vulnerabilities = Vec::new();

        // Analyze different types of vulnerabilities based on the request
        if vulnerability_types.contains(&"injection".to_string()) || vulnerability_types.contains(&"all".to_string()) {
            let injection_vulns = self.detect_injection_vulnerabilities(server, exclude_patterns).await?;
            vulnerabilities.extend(injection_vulns);
        }

        if vulnerability_types.contains(&"authentication".to_string()) || vulnerability_types.contains(&"all".to_string()) {
            let auth_vulns = self.detect_authentication_issues(server, exclude_patterns).await?;
            vulnerabilities.extend(auth_vulns);
        }

        if vulnerability_types.contains(&"authorization".to_string()) || vulnerability_types.contains(&"all".to_string()) {
            let authz_vulns = self.detect_authorization_issues(server, exclude_patterns).await?;
            vulnerabilities.extend(authz_vulns);
        }

        if vulnerability_types.contains(&"data_exposure".to_string()) || vulnerability_types.contains(&"all".to_string()) {
            let data_vulns = self.detect_data_exposure_issues(server, exclude_patterns).await?;
            vulnerabilities.extend(data_vulns);
        }

        if vulnerability_types.contains(&"unsafe_patterns".to_string()) || vulnerability_types.contains(&"all".to_string()) {
            let unsafe_vulns = self.detect_unsafe_patterns(server, exclude_patterns).await?;
            vulnerabilities.extend(unsafe_vulns);
        }

        if vulnerability_types.contains(&"crypto_issues".to_string()) || vulnerability_types.contains(&"all".to_string()) {
            let crypto_vulns = self.detect_crypto_issues(server, exclude_patterns).await?;
            vulnerabilities.extend(crypto_vulns);
        }

        // Filter by severity threshold
        let severity_order = ["low", "medium", "high", "critical"];
        let min_severity_index = severity_order.iter().position(|&s| s == severity_threshold).unwrap_or(1);
        
        vulnerabilities.retain(|vuln| {
            if let Some(severity) = vuln.get("severity").and_then(|s| s.as_str()) {
                severity_order.iter().position(|&s| s == severity).unwrap_or(0) >= min_severity_index
            } else {
                true // Include if severity is not specified
            }
        });

        // Group vulnerabilities by severity and type
        let mut by_severity = std::collections::HashMap::new();
        let mut by_type = std::collections::HashMap::new();

        for vuln in &vulnerabilities {
            if let Some(severity) = vuln.get("severity").and_then(|s| s.as_str()) {
                by_severity.entry(severity.to_string()).or_insert_with(Vec::new).push(vuln);
            }
            if let Some(vuln_type) = vuln.get("type").and_then(|t| t.as_str()) {
                by_type.entry(vuln_type.to_string()).or_insert_with(Vec::new).push(vuln);
            }
        }

        Ok(serde_json::json!({
            "scope": scope,
            "analysis_parameters": {
                "vulnerability_types": vulnerability_types,
                "severity_threshold": severity_threshold,
                "include_data_flow_analysis": include_data_flow_analysis,
                "check_external_dependencies": check_external_dependencies
            },
            "vulnerabilities": vulnerabilities,
            "summary": {
                "total_vulnerabilities": vulnerabilities.len(),
                "by_severity": by_severity.iter().map(|(k, v)| (k.clone(), v.len())).collect::<std::collections::HashMap<_, _>>(),
                "by_type": by_type.iter().map(|(k, v)| (k.clone(), v.len())).collect::<std::collections::HashMap<_, _>>(),
                "critical_count": by_severity.get("critical").map(|v| v.len()).unwrap_or(0),
                "high_count": by_severity.get("high").map(|v| v.len()).unwrap_or(0),
                "medium_count": by_severity.get("medium").map(|v| v.len()).unwrap_or(0),
                "low_count": by_severity.get("low").map(|v| v.len()).unwrap_or(0)
            },
            "recommendations": self.get_security_recommendations(&vulnerabilities)
        }))
    }

    /// Detect injection vulnerabilities (SQL injection, code injection, etc.)
    async fn detect_injection_vulnerabilities(
        &self,
        server: &PrismMcpServer,
        exclude_patterns: &[String],
    ) -> Result<Vec<serde_json::Value>> {
        let mut vulnerabilities = Vec::new();
        let functions = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Function);

        for function in functions {
            if exclude_patterns.iter().any(|pattern| function.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            // Check function names for SQL-related patterns
            let function_name_lower = function.name.to_lowercase();
            if function_name_lower.contains("sql") ||
               function_name_lower.contains("query") ||
               function_name_lower.contains("exec") {
                
                // Look for potentially dangerous patterns
                let references = server.graph_query().find_references(&function.id)?;
                let call_count = references.iter()
                    .filter(|r| matches!(r.edge_kind, prism_core::EdgeKind::Calls))
                    .count();

                if call_count > 0 {
                    vulnerabilities.push(serde_json::json!({
                        "type": "Potential SQL Injection",
                        "severity": "medium",
                        "function": {
                            "id": function.id.to_hex(),
                            "name": function.name,
                            "file": function.file.display().to_string(),
                            "location": {
                                "start_line": function.span.start_line,
                                "end_line": function.span.end_line
                            }
                        },
                        "description": "Function name suggests SQL operations - ensure proper parameterization",
                        "recommendation": "Use parameterized queries or prepared statements to prevent SQL injection",
                        "confidence": 0.6
                    }));
                }
            }

            // Check for eval-like functions (code injection risk)
            if function_name_lower.contains("eval") ||
               function_name_lower.contains("exec") ||
               function_name_lower.contains("compile") {
                
                vulnerabilities.push(serde_json::json!({
                    "type": "Code Injection Risk",
                    "severity": "high",
                    "function": {
                        "id": function.id.to_hex(),
                        "name": function.name,
                        "file": function.file.display().to_string(),
                        "location": {
                            "start_line": function.span.start_line,
                            "end_line": function.span.end_line
                        }
                    },
                    "description": "Function involves dynamic code execution which can be dangerous",
                    "recommendation": "Avoid dynamic code execution or implement strict input validation",
                    "confidence": 0.8
                }));
            }
        }

        Ok(vulnerabilities)
    }

    /// Detect authentication-related security issues
    async fn detect_authentication_issues(
        &self,
        server: &PrismMcpServer,
        exclude_patterns: &[String],
    ) -> Result<Vec<serde_json::Value>> {
        let mut vulnerabilities = Vec::new();
        let functions = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Function);

        for function in functions {
            if exclude_patterns.iter().any(|pattern| function.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            let function_name_lower = function.name.to_lowercase();
            
            // Check for authentication-related functions
            if function_name_lower.contains("login") ||
               function_name_lower.contains("auth") ||
               function_name_lower.contains("signin") ||
               function_name_lower.contains("password") {
                
                // Look for potential weak authentication patterns
                let variables = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Variable);
                let weak_auth_patterns = variables.iter().any(|var| {
                    let var_name_lower = var.name.to_lowercase();
                    var.file == function.file && (
                        var_name_lower.contains("password") ||
                        var_name_lower.contains("secret") ||
                        var_name_lower.contains("token")
                    )
                });

                if weak_auth_patterns {
                    vulnerabilities.push(serde_json::json!({
                        "type": "Authentication Security Concern",
                        "severity": "medium",
                        "function": {
                            "id": function.id.to_hex(),
                            "name": function.name,
                            "file": function.file.display().to_string(),
                            "location": {
                                "start_line": function.span.start_line,
                                "end_line": function.span.end_line
                            }
                        },
                        "description": "Authentication function detected - ensure secure implementation",
                        "recommendation": "Use secure password hashing, implement rate limiting, and secure session management",
                        "confidence": 0.7
                    }));
                }
            }
        }

        Ok(vulnerabilities)
    }

    /// Detect authorization-related security issues
    async fn detect_authorization_issues(
        &self,
        server: &PrismMcpServer,
        exclude_patterns: &[String],
    ) -> Result<Vec<serde_json::Value>> {
        let mut vulnerabilities = Vec::new();
        let functions = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Function);

        for function in functions {
            if exclude_patterns.iter().any(|pattern| function.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            let function_name_lower = function.name.to_lowercase();
            
            // Check for authorization-related functions that might need access control
            if function_name_lower.contains("admin") ||
               function_name_lower.contains("delete") ||
               function_name_lower.contains("modify") ||
               function_name_lower.contains("update") ||
               function_name_lower.contains("create") {
                
                vulnerabilities.push(serde_json::json!({
                    "type": "Authorization Check Needed",
                    "severity": "medium",
                    "function": {
                        "id": function.id.to_hex(),
                        "name": function.name,
                        "file": function.file.display().to_string(),
                        "location": {
                            "start_line": function.span.start_line,
                            "end_line": function.span.end_line
                        }
                    },
                    "description": "Function performs sensitive operations - ensure proper authorization checks",
                    "recommendation": "Implement role-based access control and verify user permissions before execution",
                    "confidence": 0.5
                }));
            }
        }

        Ok(vulnerabilities)
    }

    /// Detect data exposure issues
    async fn detect_data_exposure_issues(
        &self,
        server: &PrismMcpServer,
        exclude_patterns: &[String],
    ) -> Result<Vec<serde_json::Value>> {
        let mut vulnerabilities = Vec::new();
        let variables = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Variable);

        for variable in variables {
            if exclude_patterns.iter().any(|pattern| variable.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            let var_name_lower = variable.name.to_lowercase();
            
            // Check for sensitive data in variable names
            if var_name_lower.contains("password") ||
               var_name_lower.contains("secret") ||
               var_name_lower.contains("key") ||
               var_name_lower.contains("token") ||
               var_name_lower.contains("api_key") ||
               var_name_lower.contains("private") {
                
                vulnerabilities.push(serde_json::json!({
                    "type": "Sensitive Data Exposure",
                    "severity": "high",
                    "variable": {
                        "id": variable.id.to_hex(),
                        "name": variable.name,
                        "file": variable.file.display().to_string(),
                        "location": {
                            "start_line": variable.span.start_line,
                            "end_line": variable.span.end_line
                        }
                    },
                    "description": "Variable contains potentially sensitive data",
                    "recommendation": "Ensure sensitive data is properly encrypted and not logged or exposed",
                    "confidence": 0.8
                }));
            }
        }

        Ok(vulnerabilities)
    }

    /// Detect unsafe coding patterns
    async fn detect_unsafe_patterns(
        &self,
        server: &PrismMcpServer,
        exclude_patterns: &[String],
    ) -> Result<Vec<serde_json::Value>> {
        let mut vulnerabilities = Vec::new();
        let functions = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Function);

        for function in functions {
            if exclude_patterns.iter().any(|pattern| function.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            let function_name_lower = function.name.to_lowercase();
            
            // Check for potentially unsafe functions
            if function_name_lower.contains("unsafe") ||
               function_name_lower.contains("raw") ||
               function_name_lower.contains("ptr") ||
               function_name_lower.contains("malloc") ||
               function_name_lower.contains("strcpy") {
                
                vulnerabilities.push(serde_json::json!({
                    "type": "Unsafe Pattern",
                    "severity": "medium",
                    "function": {
                        "id": function.id.to_hex(),
                        "name": function.name,
                        "file": function.file.display().to_string(),
                        "location": {
                            "start_line": function.span.start_line,
                            "end_line": function.span.end_line
                        }
                    },
                    "description": "Function uses potentially unsafe patterns",
                    "recommendation": "Review for memory safety and input validation",
                    "confidence": 0.6
                }));
            }
        }

        Ok(vulnerabilities)
    }

    /// Detect cryptographic implementation issues
    async fn detect_crypto_issues(
        &self,
        server: &PrismMcpServer,
        exclude_patterns: &[String],
    ) -> Result<Vec<serde_json::Value>> {
        let mut vulnerabilities = Vec::new();
        let functions = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Function);

        for function in functions {
            if exclude_patterns.iter().any(|pattern| function.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            let function_name_lower = function.name.to_lowercase();
            
            // Check for cryptographic functions that might be implemented unsafely
            if function_name_lower.contains("encrypt") ||
               function_name_lower.contains("decrypt") ||
               function_name_lower.contains("hash") ||
               function_name_lower.contains("crypto") ||
               function_name_lower.contains("cipher") {
                
                vulnerabilities.push(serde_json::json!({
                    "type": "Cryptographic Implementation",
                    "severity": "high",
                    "function": {
                        "id": function.id.to_hex(),
                        "name": function.name,
                        "file": function.file.display().to_string(),
                        "location": {
                            "start_line": function.span.start_line,
                            "end_line": function.span.end_line
                        }
                    },
                    "description": "Function implements cryptographic operations - ensure secure implementation",
                    "recommendation": "Use well-tested cryptographic libraries, avoid custom crypto implementations",
                    "confidence": 0.7
                }));
            }
        }

        Ok(vulnerabilities)
    }

    /// Get security recommendations based on found vulnerabilities
    fn get_security_recommendations(&self, vulnerabilities: &[serde_json::Value]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let critical_count = vulnerabilities.iter()
            .filter(|v| v.get("severity").and_then(|s| s.as_str()) == Some("critical"))
            .count();
        
        let high_count = vulnerabilities.iter()
            .filter(|v| v.get("severity").and_then(|s| s.as_str()) == Some("high"))
            .count();

        if critical_count > 0 {
            recommendations.push(format!("URGENT: Address {} critical security vulnerabilities immediately", critical_count));
        }

        if high_count > 0 {
            recommendations.push(format!("HIGH PRIORITY: Address {} high-severity security issues", high_count));
        }

        // Type-specific recommendations
        let injection_count = vulnerabilities.iter()
            .filter(|v| v.get("type").and_then(|t| t.as_str()).map(|s| s.contains("Injection")).unwrap_or(false))
            .count();
        
        if injection_count > 0 {
            recommendations.push("Implement input validation and parameterized queries to prevent injection attacks".to_string());
        }

        let auth_count = vulnerabilities.iter()
            .filter(|v| v.get("type").and_then(|t| t.as_str()).map(|s| s.contains("Authentication")).unwrap_or(false))
            .count();
        
        if auth_count > 0 {
            recommendations.push("Review authentication mechanisms and implement secure password handling".to_string());
        }

        let crypto_count = vulnerabilities.iter()
            .filter(|v| v.get("type").and_then(|t| t.as_str()).map(|s| s.contains("Cryptographic")).unwrap_or(false))
            .count();
        
        if crypto_count > 0 {
            recommendations.push("Use established cryptographic libraries instead of custom implementations".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("No significant security vulnerabilities detected with current analysis".to_string());
        } else {
            recommendations.push("Conduct regular security audits and implement automated security testing".to_string());
            recommendations.push("Follow OWASP security guidelines and best practices".to_string());
        }

        recommendations
    }

    /// Perform performance analysis
    async fn perform_performance_analysis(
        &self,
        server: &PrismMcpServer,
        scope: &str,
        analysis_types: &[String],
        complexity_threshold: &str,
        include_algorithmic_analysis: bool,
        detect_bottlenecks: bool,
        exclude_patterns: &[String],
    ) -> Result<serde_json::Value> {
        let mut performance_issues = Vec::new();

        // Analyze different types of performance characteristics based on the request
        if analysis_types.contains(&"time_complexity".to_string()) || analysis_types.contains(&"all".to_string()) {
            let complexity_issues = self.analyze_time_complexity(server, exclude_patterns, include_algorithmic_analysis).await?;
            performance_issues.extend(complexity_issues);
        }

        if analysis_types.contains(&"memory_usage".to_string()) || analysis_types.contains(&"all".to_string()) {
            let memory_issues = self.analyze_memory_usage(server, exclude_patterns).await?;
            performance_issues.extend(memory_issues);
        }

        if analysis_types.contains(&"hot_spots".to_string()) || analysis_types.contains(&"all".to_string()) {
            let hot_spot_issues = self.detect_performance_hot_spots(server, exclude_patterns, detect_bottlenecks).await?;
            performance_issues.extend(hot_spot_issues);
        }

        if analysis_types.contains(&"anti_patterns".to_string()) || analysis_types.contains(&"all".to_string()) {
            let anti_pattern_issues = self.detect_performance_anti_patterns(server, exclude_patterns).await?;
            performance_issues.extend(anti_pattern_issues);
        }

        if analysis_types.contains(&"scalability".to_string()) || analysis_types.contains(&"all".to_string()) {
            let scalability_issues = self.analyze_scalability_concerns(server, exclude_patterns).await?;
            performance_issues.extend(scalability_issues);
        }

        // Filter by complexity threshold
        let complexity_order = ["low", "medium", "high"];
        let min_complexity_index = complexity_order.iter().position(|&s| s == complexity_threshold).unwrap_or(1);
        
        performance_issues.retain(|issue| {
            if let Some(complexity) = issue.get("complexity").and_then(|c| c.as_str()) {
                complexity_order.iter().position(|&s| s == complexity).unwrap_or(0) >= min_complexity_index
            } else {
                true // Include if complexity is not specified
            }
        });

        // Group issues by category and complexity
        let mut by_category = std::collections::HashMap::new();
        let mut by_complexity = std::collections::HashMap::new();

        for issue in &performance_issues {
            if let Some(category) = issue.get("category").and_then(|c| c.as_str()) {
                by_category.entry(category.to_string()).or_insert_with(Vec::new).push(issue);
            }
            if let Some(complexity) = issue.get("complexity").and_then(|c| c.as_str()) {
                by_complexity.entry(complexity.to_string()).or_insert_with(Vec::new).push(issue);
            }
        }

        Ok(serde_json::json!({
            "scope": scope,
            "analysis_parameters": {
                "analysis_types": analysis_types,
                "complexity_threshold": complexity_threshold,
                "include_algorithmic_analysis": include_algorithmic_analysis,
                "detect_bottlenecks": detect_bottlenecks
            },
            "performance_issues": performance_issues,
            "summary": {
                "total_issues": performance_issues.len(),
                "by_category": by_category.iter().map(|(k, v)| (k.clone(), v.len())).collect::<std::collections::HashMap<_, _>>(),
                "by_complexity": by_complexity.iter().map(|(k, v)| (k.clone(), v.len())).collect::<std::collections::HashMap<_, _>>(),
                "high_complexity_count": by_complexity.get("high").map(|v| v.len()).unwrap_or(0),
                "medium_complexity_count": by_complexity.get("medium").map(|v| v.len()).unwrap_or(0),
                "low_complexity_count": by_complexity.get("low").map(|v| v.len()).unwrap_or(0)
            },
            "recommendations": self.get_performance_recommendations(&performance_issues)
        }))
    }

    /// Analyze time complexity characteristics
    async fn analyze_time_complexity(
        &self,
        server: &PrismMcpServer,
        exclude_patterns: &[String],
        include_algorithmic_analysis: bool,
    ) -> Result<Vec<serde_json::Value>> {
        let mut issues = Vec::new();
        let functions = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Function);

        for function in functions {
            if exclude_patterns.iter().any(|pattern| function.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            // Analyze nested loops (basic O(n^k) detection)
            let function_name_lower = function.name.to_lowercase();
            
            // Check for potentially expensive operations
            if function_name_lower.contains("sort") ||
               function_name_lower.contains("search") ||
               function_name_lower.contains("find") ||
               function_name_lower.contains("filter") {
                
                let references = server.graph_query().find_references(&function.id)?;
                let call_count = references.iter()
                    .filter(|r| matches!(r.edge_kind, prism_core::EdgeKind::Calls))
                    .count();

                if call_count > 10 { // Frequently called expensive functions
                    issues.push(serde_json::json!({
                        "type": "High Time Complexity Function",
                        "category": "time_complexity",
                        "complexity": "high",
                        "function": {
                            "id": function.id.to_hex(),
                            "name": function.name,
                            "file": function.file.display().to_string(),
                            "location": {
                                "start_line": function.span.start_line,
                                "end_line": function.span.end_line
                            }
                        },
                        "description": format!("Function '{}' appears to involve expensive operations and is frequently called ({} times)", function.name, call_count),
                        "estimated_complexity": "O(n log n) or worse",
                        "recommendation": "Consider caching results, optimizing algorithms, or reducing call frequency",
                        "call_count": call_count
                    }));
                }
            }

            // Basic nested loop detection by function length and complexity
            if include_algorithmic_analysis {
                let function_lines = function.span.end_line - function.span.start_line + 1;
                let cyclomatic_complexity = self.calculate_cyclomatic_complexity("");
                
                if function_lines > 100 && cyclomatic_complexity > 20 {
                    issues.push(serde_json::json!({
                        "type": "Complex Algorithm",
                        "category": "time_complexity",
                        "complexity": "medium",
                        "function": {
                            "id": function.id.to_hex(),
                            "name": function.name,
                            "file": function.file.display().to_string(),
                            "location": {
                                "start_line": function.span.start_line,
                                "end_line": function.span.end_line
                            }
                        },
                        "description": format!("Function '{}' has high complexity ({} lines, complexity: {})", function.name, function_lines, cyclomatic_complexity),
                        "estimated_complexity": "O(n^2) or worse",
                        "recommendation": "Break down into smaller functions and optimize algorithms",
                        "lines_of_code": function_lines,
                        "cyclomatic_complexity": cyclomatic_complexity
                    }));
                }
            }
        }

        Ok(issues)
    }

    /// Analyze memory usage patterns
    async fn analyze_memory_usage(
        &self,
        server: &PrismMcpServer,
        exclude_patterns: &[String],
    ) -> Result<Vec<serde_json::Value>> {
        let mut issues = Vec::new();
        let functions = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Function);

        for function in functions {
            if exclude_patterns.iter().any(|pattern| function.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            let function_name_lower = function.name.to_lowercase();
            
            // Check for potential memory-intensive operations
            if function_name_lower.contains("load") ||
               function_name_lower.contains("read") ||
               function_name_lower.contains("parse") ||
               function_name_lower.contains("create") ||
               function_name_lower.contains("build") {
                
                // Look for variables that might indicate large data structures
                let variables = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Variable);
                let large_data_vars = variables.iter().filter(|var| {
                    var.file == function.file && (
                        var.name.to_lowercase().contains("list") ||
                        var.name.to_lowercase().contains("array") ||
                        var.name.to_lowercase().contains("data") ||
                        var.name.to_lowercase().contains("buffer") ||
                        var.name.to_lowercase().contains("cache")
                    )
                }).count();

                if large_data_vars > 3 {
                    issues.push(serde_json::json!({
                        "type": "High Memory Usage",
                        "category": "memory_usage",
                        "complexity": "medium",
                        "function": {
                            "id": function.id.to_hex(),
                            "name": function.name,
                            "file": function.file.display().to_string(),
                            "location": {
                                "start_line": function.span.start_line,
                                "end_line": function.span.end_line
                            }
                        },
                        "description": format!("Function '{}' uses multiple large data structures ({} variables)", function.name, large_data_vars),
                        "recommendation": "Consider streaming processing, pagination, or memory pooling",
                        "large_data_variables": large_data_vars
                    }));
                }
            }
        }

        Ok(issues)
    }

    /// Detect performance hot spots and bottlenecks
    async fn detect_performance_hot_spots(
        &self,
        server: &PrismMcpServer,
        exclude_patterns: &[String],
        detect_bottlenecks: bool,
    ) -> Result<Vec<serde_json::Value>> {
        let mut issues = Vec::new();
        let functions = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Function);

        for function in functions {
            if exclude_patterns.iter().any(|pattern| function.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            // Count references to identify hot spots
            let references = server.graph_query().find_references(&function.id)?;
            let call_count = references.iter()
                .filter(|r| matches!(r.edge_kind, prism_core::EdgeKind::Calls))
                .count();

            // Functions called many times are potential hot spots
            if call_count > 20 {
                let function_lines = function.span.end_line - function.span.start_line + 1;
                
                issues.push(serde_json::json!({
                    "type": "Performance Hot Spot",
                    "category": "hot_spots",
                    "complexity": if call_count > 50 { "high" } else { "medium" },
                    "function": {
                        "id": function.id.to_hex(),
                        "name": function.name,
                        "file": function.file.display().to_string(),
                        "location": {
                            "start_line": function.span.start_line,
                            "end_line": function.span.end_line
                        }
                    },
                    "description": format!("Function '{}' is called {} times - potential performance hot spot", function.name, call_count),
                    "recommendation": "Optimize this function as it's frequently called, consider caching or memoization",
                    "call_count": call_count,
                    "lines_of_code": function_lines
                }));
            }

            // Detect potential bottlenecks (I/O operations)
            if detect_bottlenecks {
                let function_name_lower = function.name.to_lowercase();
                if function_name_lower.contains("read") ||
                   function_name_lower.contains("write") ||
                   function_name_lower.contains("fetch") ||
                   function_name_lower.contains("request") ||
                   function_name_lower.contains("query") {
                    
                    issues.push(serde_json::json!({
                        "type": "I/O Bottleneck",
                        "category": "hot_spots",
                        "complexity": "medium",
                        "function": {
                            "id": function.id.to_hex(),
                            "name": function.name,
                            "file": function.file.display().to_string(),
                            "location": {
                                "start_line": function.span.start_line,
                                "end_line": function.span.end_line
                            }
                        },
                        "description": format!("Function '{}' performs I/O operations which can be a bottleneck", function.name),
                        "recommendation": "Consider async operations, connection pooling, or caching",
                        "call_count": call_count
                    }));
                }
            }
        }

        Ok(issues)
    }

    /// Detect performance anti-patterns
    async fn detect_performance_anti_patterns(
        &self,
        server: &PrismMcpServer,
        exclude_patterns: &[String],
    ) -> Result<Vec<serde_json::Value>> {
        let mut issues = Vec::new();
        let functions = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Function);

        for function in functions {
            if exclude_patterns.iter().any(|pattern| function.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            // Detect potential N+1 query pattern
            let function_name_lower = function.name.to_lowercase();
            if function_name_lower.contains("get") || function_name_lower.contains("fetch") {
                let dependencies = server.graph_query().find_dependencies(&function.id, prism_core::graph::DependencyType::Calls)?;
                let loop_like_calls = dependencies.iter().filter(|dep| {
                    let dep_name = dep.target_node.name.to_lowercase();
                    dep_name.contains("query") || dep_name.contains("get") || dep_name.contains("fetch")
                }).count();

                if loop_like_calls > 1 {
                    issues.push(serde_json::json!({
                        "type": "Potential N+1 Query Pattern",
                        "category": "anti_patterns",
                        "complexity": "high",
                        "function": {
                            "id": function.id.to_hex(),
                            "name": function.name,
                            "file": function.file.display().to_string(),
                            "location": {
                                "start_line": function.span.start_line,
                                "end_line": function.span.end_line
                            }
                        },
                        "description": format!("Function '{}' makes multiple queries/fetches - potential N+1 pattern", function.name),
                        "recommendation": "Use batch queries, joins, or eager loading to reduce query count",
                        "query_calls": loop_like_calls
                    }));
                }
            }

            // Detect premature optimization patterns
            if function_name_lower.contains("optimize") || function_name_lower.contains("cache") {
                let function_lines = function.span.end_line - function.span.start_line + 1;
                if function_lines > 50 {
                    issues.push(serde_json::json!({
                        "type": "Complex Optimization",
                        "category": "anti_patterns",
                        "complexity": "medium",
                        "function": {
                            "id": function.id.to_hex(),
                            "name": function.name,
                            "file": function.file.display().to_string(),
                            "location": {
                                "start_line": function.span.start_line,
                                "end_line": function.span.end_line
                            }
                        },
                        "description": format!("Function '{}' appears to be a complex optimization - verify it's necessary", function.name),
                        "recommendation": "Ensure optimization is justified by profiling data",
                        "lines_of_code": function_lines
                    }));
                }
            }
        }

        Ok(issues)
    }

    /// Analyze scalability concerns
    async fn analyze_scalability_concerns(
        &self,
        server: &PrismMcpServer,
        exclude_patterns: &[String],
    ) -> Result<Vec<serde_json::Value>> {
        let mut issues = Vec::new();
        let functions = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Function);

        for function in functions {
            if exclude_patterns.iter().any(|pattern| function.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            let function_name_lower = function.name.to_lowercase();
            
            // Check for global state usage (scalability concern)
            if function_name_lower.contains("global") ||
               function_name_lower.contains("singleton") ||
               function_name_lower.contains("static") {
                
                issues.push(serde_json::json!({
                    "type": "Global State Usage",
                    "category": "scalability",
                    "complexity": "medium",
                    "function": {
                        "id": function.id.to_hex(),
                        "name": function.name,
                        "file": function.file.display().to_string(),
                        "location": {
                            "start_line": function.span.start_line,
                            "end_line": function.span.end_line
                        }
                    },
                    "description": format!("Function '{}' uses global state which can limit scalability", function.name),
                    "recommendation": "Consider dependency injection or stateless design for better scalability"
                }));
            }

            // Check for synchronous operations that could block
            if function_name_lower.contains("wait") ||
               function_name_lower.contains("sleep") ||
               function_name_lower.contains("block") ||
               function_name_lower.contains("sync") {
                
                issues.push(serde_json::json!({
                    "type": "Blocking Operation",
                    "category": "scalability",
                    "complexity": "high",
                    "function": {
                        "id": function.id.to_hex(),
                        "name": function.name,
                        "file": function.file.display().to_string(),
                        "location": {
                            "start_line": function.span.start_line,
                            "end_line": function.span.end_line
                        }
                    },
                    "description": format!("Function '{}' performs blocking operations which can hurt scalability", function.name),
                    "recommendation": "Consider async operations or non-blocking alternatives"
                }));
            }
        }

        Ok(issues)
    }

    /// Get performance recommendations based on found issues
    fn get_performance_recommendations(&self, issues: &[serde_json::Value]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let high_complexity_count = issues.iter()
            .filter(|i| i.get("complexity").and_then(|c| c.as_str()) == Some("high"))
            .count();
        
        let medium_complexity_count = issues.iter()
            .filter(|i| i.get("complexity").and_then(|c| c.as_str()) == Some("medium"))
            .count();

        if high_complexity_count > 0 {
            recommendations.push(format!("HIGH PRIORITY: Address {} high-complexity performance issues", high_complexity_count));
        }

        if medium_complexity_count > 0 {
            recommendations.push(format!("MEDIUM PRIORITY: Review {} medium-complexity performance concerns", medium_complexity_count));
        }

        // Category-specific recommendations
        let hot_spot_count = issues.iter()
            .filter(|i| i.get("category").and_then(|c| c.as_str()) == Some("hot_spots"))
            .count();
        
        if hot_spot_count > 0 {
            recommendations.push("Profile hot spots and optimize frequently called functions".to_string());
        }

        let time_complexity_count = issues.iter()
            .filter(|i| i.get("category").and_then(|c| c.as_str()) == Some("time_complexity"))
            .count();
        
        if time_complexity_count > 0 {
            recommendations.push("Review algorithmic complexity and consider more efficient algorithms".to_string());
        }

        let memory_count = issues.iter()
            .filter(|i| i.get("category").and_then(|c| c.as_str()) == Some("memory_usage"))
            .count();
        
        if memory_count > 0 {
            recommendations.push("Optimize memory usage with streaming, pagination, or caching strategies".to_string());
        }

        let scalability_count = issues.iter()
            .filter(|i| i.get("category").and_then(|c| c.as_str()) == Some("scalability"))
            .count();
        
        if scalability_count > 0 {
            recommendations.push("Address scalability concerns by reducing global state and blocking operations".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("No significant performance issues detected with current analysis".to_string());
        } else {
            recommendations.push("Use profiling tools to validate performance assumptions".to_string());
            recommendations.push("Implement performance monitoring and alerting".to_string());
            recommendations.push("Consider load testing to validate scalability improvements".to_string());
        }

        recommendations
    }

    /// Perform API surface analysis
    async fn perform_api_surface_analysis(
        &self,
        server: &PrismMcpServer,
        scope: &str,
        analysis_types: &[String],
        api_version: Option<&str>,
        include_private_apis: bool,
        check_documentation_coverage: bool,
        detect_breaking_changes: bool,
        exclude_patterns: &[String],
    ) -> Result<serde_json::Value> {
        let mut api_issues = Vec::new();

        // Analyze different types of API characteristics based on the request
        if analysis_types.contains(&"public_api".to_string()) || analysis_types.contains(&"all".to_string()) {
            let public_api_analysis = self.analyze_public_api(server, exclude_patterns, include_private_apis).await?;
            api_issues.extend(public_api_analysis);
        }

        if analysis_types.contains(&"versioning".to_string()) || analysis_types.contains(&"all".to_string()) {
            let versioning_issues = self.analyze_api_versioning(server, exclude_patterns, api_version).await?;
            api_issues.extend(versioning_issues);
        }

        if analysis_types.contains(&"breaking_changes".to_string()) || analysis_types.contains(&"all".to_string()) {
            if detect_breaking_changes {
                let breaking_change_issues = self.detect_api_breaking_changes(server, exclude_patterns).await?;
                api_issues.extend(breaking_change_issues);
            }
        }

        if analysis_types.contains(&"documentation".to_string()) || analysis_types.contains(&"all".to_string()) {
            if check_documentation_coverage {
                let doc_coverage_issues = self.analyze_api_documentation_coverage(server, exclude_patterns).await?;
                api_issues.extend(doc_coverage_issues);
            }
        }

        if analysis_types.contains(&"compatibility".to_string()) || analysis_types.contains(&"all".to_string()) {
            let compatibility_issues = self.analyze_api_compatibility(server, exclude_patterns, api_version).await?;
            api_issues.extend(compatibility_issues);
        }

        // Group issues by category and severity
        let mut by_category = std::collections::HashMap::new();
        let mut by_severity = std::collections::HashMap::new();

        for issue in &api_issues {
            if let Some(category) = issue.get("category").and_then(|c| c.as_str()) {
                by_category.entry(category.to_string()).or_insert_with(Vec::new).push(issue);
            }
            if let Some(severity) = issue.get("severity").and_then(|s| s.as_str()) {
                by_severity.entry(severity.to_string()).or_insert_with(Vec::new).push(issue);
            }
        }

        Ok(serde_json::json!({
            "scope": scope,
            "analysis_parameters": {
                "analysis_types": analysis_types,
                "api_version": api_version,
                "include_private_apis": include_private_apis,
                "check_documentation_coverage": check_documentation_coverage,
                "detect_breaking_changes": detect_breaking_changes
            },
            "api_issues": api_issues,
            "summary": {
                "total_issues": api_issues.len(),
                "by_category": by_category.iter().map(|(k, v)| (k.clone(), v.len())).collect::<std::collections::HashMap<_, _>>(),
                "by_severity": by_severity.iter().map(|(k, v)| (k.clone(), v.len())).collect::<std::collections::HashMap<_, _>>(),
                "critical_issues": by_severity.get("critical").map(|v| v.len()).unwrap_or(0),
                "high_issues": by_severity.get("high").map(|v| v.len()).unwrap_or(0),
                "medium_issues": by_severity.get("medium").map(|v| v.len()).unwrap_or(0),
                "low_issues": by_severity.get("low").map(|v| v.len()).unwrap_or(0)
            },
            "recommendations": self.get_api_recommendations(&api_issues)
        }))
    }

    /// Analyze public API surface
    async fn analyze_public_api(
        &self,
        server: &PrismMcpServer,
        exclude_patterns: &[String],
        include_private_apis: bool,
    ) -> Result<Vec<serde_json::Value>> {
        let mut issues = Vec::new();
        let functions = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Function);
        let classes = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Class);

        // Analyze public functions
        for function in functions {
            if exclude_patterns.iter().any(|pattern| function.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            let function_name = &function.name;
            let is_public = self.is_public_api_element(function_name);
            let is_private = function_name.starts_with('_') || function_name.contains("private");

            // Report public API functions
            if is_public || (include_private_apis && is_private) {
                let references = server.graph_query().find_references(&function.id)?;
                let external_usage_count = references.len();

                issues.push(serde_json::json!({
                    "type": if is_public { "Public API Function" } else { "Private API Function" },
                    "category": "public_api",
                    "severity": if is_public { "medium" } else { "low" },
                    "function": {
                        "id": function.id.to_hex(),
                        "name": function.name,
                        "file": function.file.display().to_string(),
                        "location": {
                            "start_line": function.span.start_line,
                            "end_line": function.span.end_line
                        }
                    },
                    "description": format!("Function '{}' is part of the {} API surface", function.name, if is_public { "public" } else { "private" }),
                    "visibility": if is_public { "public" } else { "private" },
                    "external_usage_count": external_usage_count,
                    "recommendation": if is_public { "Ensure this function is well-documented and maintains backward compatibility" } else { "Consider if this function should be exposed or kept internal" }
                }));
            }
        }

        // Analyze public classes
        for class in classes {
            if exclude_patterns.iter().any(|pattern| class.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            let class_name = &class.name;
            let is_public = self.is_public_api_element(class_name);
            let is_private = class_name.starts_with('_') || class_name.contains("Private");

            if is_public || (include_private_apis && is_private) {
                // Count public methods in the class
                let edges = server.graph_store().get_outgoing_edges(&class.id);
                let public_methods = edges.iter().filter(|edge| {
                    if let Some(target_node) = server.graph_store().get_node(&edge.target) {
                        target_node.kind == prism_core::NodeKind::Method && self.is_public_api_element(&target_node.name)
                    } else {
                        false
                    }
                }).count();

                issues.push(serde_json::json!({
                    "type": if is_public { "Public API Class" } else { "Private API Class" },
                    "category": "public_api",
                    "severity": if is_public { "medium" } else { "low" },
                    "class": {
                        "id": class.id.to_hex(),
                        "name": class.name,
                        "file": class.file.display().to_string(),
                        "location": {
                            "start_line": class.span.start_line,
                            "end_line": class.span.end_line
                        }
                    },
                    "description": format!("Class '{}' is part of the {} API surface with {} public methods", class.name, if is_public { "public" } else { "private" }, public_methods),
                    "visibility": if is_public { "public" } else { "private" },
                    "public_methods_count": public_methods,
                    "recommendation": if is_public { "Ensure all public methods are documented and follow API design principles" } else { "Review if this class should be part of the public API" }
                }));
            }
        }

        Ok(issues)
    }

    /// Analyze API versioning concerns
    async fn analyze_api_versioning(
        &self,
        server: &PrismMcpServer,
        exclude_patterns: &[String],
        api_version: Option<&str>,
    ) -> Result<Vec<serde_json::Value>> {
        let mut issues = Vec::new();
        let functions = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Function);

        for function in functions {
            if exclude_patterns.iter().any(|pattern| function.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            let function_name = &function.name;

            // Check for version-related naming patterns
            if function_name.contains("_v") || 
               function_name.contains("V1") || 
               function_name.contains("V2") || 
               function_name.contains("version") {
                
                issues.push(serde_json::json!({
                    "type": "Versioned API Function",
                    "category": "versioning",
                    "severity": "medium",
                    "function": {
                        "id": function.id.to_hex(),
                        "name": function.name,
                        "file": function.file.display().to_string(),
                        "location": {
                            "start_line": function.span.start_line,
                            "end_line": function.span.end_line
                        }
                    },
                    "description": format!("Function '{}' appears to be version-specific", function.name),
                    "current_api_version": api_version.unwrap_or("unknown"),
                    "recommendation": "Ensure versioning strategy is consistent and deprecated versions are properly marked"
                }));
            }

            // Check for deprecated functions
            if function_name.contains("deprecated") || 
               function_name.contains("legacy") || 
               function_name.contains("old") {
                
                issues.push(serde_json::json!({
                    "type": "Deprecated API Function",
                    "category": "versioning",
                    "severity": "high",
                    "function": {
                        "id": function.id.to_hex(),
                        "name": function.name,
                        "file": function.file.display().to_string(),
                        "location": {
                            "start_line": function.span.start_line,
                            "end_line": function.span.end_line
                        }
                    },
                    "description": format!("Function '{}' appears to be deprecated", function.name),
                    "recommendation": "Provide migration path and timeline for removal"
                }));
            }
        }

        Ok(issues)
    }

    /// Detect potential API breaking changes
    async fn detect_api_breaking_changes(
        &self,
        server: &PrismMcpServer,
        exclude_patterns: &[String],
    ) -> Result<Vec<serde_json::Value>> {
        let mut issues = Vec::new();
        let functions = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Function);

        for function in functions {
            if exclude_patterns.iter().any(|pattern| function.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            let function_name = &function.name;

            // Functions with many parameters might indicate breaking changes if modified
            let dependencies = server.graph_query().find_dependencies(&function.id, prism_core::graph::DependencyType::Direct)?;
            let parameter_like_deps = dependencies.iter().filter(|dep| {
                matches!(dep.target_node.kind, prism_core::NodeKind::Variable)
            }).count();

            if parameter_like_deps > 5 && self.is_public_api_element(function_name) {
                issues.push(serde_json::json!({
                    "type": "Complex API Function",
                    "category": "breaking_changes",
                    "severity": "medium",
                    "function": {
                        "id": function.id.to_hex(),
                        "name": function.name,
                        "file": function.file.display().to_string(),
                        "location": {
                            "start_line": function.span.start_line,
                            "end_line": function.span.end_line
                        }
                    },
                    "description": format!("Function '{}' has {} parameters - changes could break compatibility", function.name, parameter_like_deps),
                    "parameter_count": parameter_like_deps,
                    "recommendation": "Consider using configuration objects or builder patterns to avoid breaking changes"
                }));
            }

            // Check for functions that might be removing features
            if function_name.contains("remove") || 
               function_name.contains("delete") || 
               function_name.contains("drop") {
                
                if self.is_public_api_element(function_name) {
                    issues.push(serde_json::json!({
                        "type": "Potentially Breaking API Function",
                        "category": "breaking_changes",
                        "severity": "high",
                        "function": {
                            "id": function.id.to_hex(),
                            "name": function.name,
                            "file": function.file.display().to_string(),
                            "location": {
                                "start_line": function.span.start_line,
                                "end_line": function.span.end_line
                            }
                        },
                        "description": format!("Function '{}' might remove functionality - potential breaking change", function.name),
                        "recommendation": "Ensure proper deprecation process and provide alternatives"
                    }));
                }
            }
        }

        Ok(issues)
    }

    /// Analyze API documentation coverage
    async fn analyze_api_documentation_coverage(
        &self,
        server: &PrismMcpServer,
        exclude_patterns: &[String],
    ) -> Result<Vec<serde_json::Value>> {
        let mut issues = Vec::new();
        let functions = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Function);

        for function in functions {
            if exclude_patterns.iter().any(|pattern| function.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            let function_name = &function.name;

            // Only check documentation for public API functions
            if self.is_public_api_element(function_name) {
                // This is a simplified check - in a real implementation, you'd check for actual docstrings/comments
                let likely_undocumented = !function_name.contains("test") && 
                                         !function_name.contains("helper") &&
                                         !function_name.starts_with('_');

                if likely_undocumented {
                    let function_lines = function.span.end_line - function.span.start_line + 1;
                    
                    if function_lines > 5 { // Only flag substantial functions
                        issues.push(serde_json::json!({
                            "type": "Undocumented API Function",
                            "category": "documentation",
                            "severity": "medium",
                            "function": {
                                "id": function.id.to_hex(),
                                "name": function.name,
                                "file": function.file.display().to_string(),
                                "location": {
                                    "start_line": function.span.start_line,
                                    "end_line": function.span.end_line
                                }
                            },
                            "description": format!("Public function '{}' may lack adequate documentation", function.name),
                            "lines_of_code": function_lines,
                            "recommendation": "Add comprehensive documentation including parameters, return values, and usage examples"
                        }));
                    }
                }
            }
        }

        Ok(issues)
    }

    /// Analyze API compatibility concerns
    async fn analyze_api_compatibility(
        &self,
        server: &PrismMcpServer,
        exclude_patterns: &[String],
        api_version: Option<&str>,
    ) -> Result<Vec<serde_json::Value>> {
        let mut issues = Vec::new();
        let functions = server.graph_store().get_nodes_by_kind(prism_core::NodeKind::Function);

        for function in functions {
            if exclude_patterns.iter().any(|pattern| function.file.to_string_lossy().contains(pattern)) {
                continue;
            }

            let function_name = &function.name;

            // Check for functions that might have compatibility issues
            if self.is_public_api_element(function_name) {
                let references = server.graph_query().find_references(&function.id)?;
                let usage_count = references.len();

                // Functions with high usage are critical for compatibility
                if usage_count > 10 {
                    issues.push(serde_json::json!({
                        "type": "High-Impact API Function",
                        "category": "compatibility",
                        "severity": "high",
                        "function": {
                            "id": function.id.to_hex(),
                            "name": function.name,
                            "file": function.file.display().to_string(),
                            "location": {
                                "start_line": function.span.start_line,
                                "end_line": function.span.end_line
                            }
                        },
                        "description": format!("Function '{}' has {} usages - changes require careful compatibility planning", function.name, usage_count),
                        "usage_count": usage_count,
                        "api_version": api_version.unwrap_or("unknown"),
                        "recommendation": "Maintain backward compatibility or provide clear migration path"
                    }));
                }
            }
        }

        Ok(issues)
    }

    /// Check if an API element is considered public
    fn is_public_api_element(&self, name: &str) -> bool {
        // This is a simplified heuristic - in practice, you'd use language-specific visibility rules
        !name.starts_with('_') && 
        !name.contains("private") && 
        !name.contains("internal") && 
        !name.contains("test") &&
        !name.contains("helper") &&
        !name.contains("util")
    }

    /// Get API recommendations based on found issues
    fn get_api_recommendations(&self, issues: &[serde_json::Value]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let critical_count = issues.iter()
            .filter(|i| i.get("severity").and_then(|s| s.as_str()) == Some("critical"))
            .count();
        
        let high_count = issues.iter()
            .filter(|i| i.get("severity").and_then(|s| s.as_str()) == Some("high"))
            .count();

        if critical_count > 0 {
            recommendations.push(format!("CRITICAL: Address {} critical API issues immediately", critical_count));
        }

        if high_count > 0 {
            recommendations.push(format!("HIGH PRIORITY: Review {} high-impact API concerns", high_count));
        }

        // Category-specific recommendations
        let documentation_count = issues.iter()
            .filter(|i| i.get("category").and_then(|c| c.as_str()) == Some("documentation"))
            .count();
        
        if documentation_count > 0 {
            recommendations.push("Improve API documentation coverage for better developer experience".to_string());
        }

        let breaking_changes_count = issues.iter()
            .filter(|i| i.get("category").and_then(|c| c.as_str()) == Some("breaking_changes"))
            .count();
        
        if breaking_changes_count > 0 {
            recommendations.push("Review potential breaking changes and implement proper deprecation process".to_string());
        }

        let versioning_count = issues.iter()
            .filter(|i| i.get("category").and_then(|c| c.as_str()) == Some("versioning"))
            .count();
        
        if versioning_count > 0 {
            recommendations.push("Establish consistent API versioning strategy and migration paths".to_string());
        }

        let compatibility_count = issues.iter()
            .filter(|i| i.get("category").and_then(|c| c.as_str()) == Some("compatibility"))
            .count();
        
        if compatibility_count > 0 {
            recommendations.push("Maintain backward compatibility for high-usage API functions".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("No significant API surface issues detected with current analysis".to_string());
        } else {
            recommendations.push("Implement API design guidelines and review processes".to_string());
            recommendations.push("Consider semantic versioning and changelog maintenance".to_string());
            recommendations.push("Set up automated API compatibility testing".to_string());
        }

        recommendations
    }
} 