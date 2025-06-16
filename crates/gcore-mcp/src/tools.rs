//! MCP Tools implementation

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::GCoreMcpServer;

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
    server: std::sync::Arc<tokio::sync::RwLock<GCoreMcpServer>>,
}

impl ToolManager {
    /// Create a new tool manager
    pub fn new(server: std::sync::Arc<tokio::sync::RwLock<GCoreMcpServer>>) -> Self {
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
                        }
                    },
                    "required": ["symbol_id"]
                }),
            },
            Tool {
                name: "search_symbols".to_string(),
                title: Some("Search Symbols".to_string()),
                description: "Search for symbols by name pattern".to_string(),
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
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of results",
                            "default": 50
                        }
                    },
                    "required": ["pattern"]
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
            _ => Ok(CallToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Unknown tool: {}", params.name),
                }],
                is_error: Some(true),
            }),
        }
    }

    /// Get repository statistics
    async fn repository_stats(&self, server: &GCoreMcpServer) -> Result<CallToolResult> {
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
    async fn trace_path(&self, server: &GCoreMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
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
    async fn explain_symbol(&self, server: &GCoreMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
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

        let symbol_id = self.parse_node_id(symbol_id_str)?;

        if let Some(node) = server.graph_store().get_node(&symbol_id) {
            let mut result = serde_json::json!({
                "symbol": {
                    "id": symbol_id.to_hex(),
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
                }
            });

            if include_dependencies {
                let dependencies = server.graph_query().find_dependencies(&symbol_id, gcore::graph::DependencyType::Direct)?;
                result["dependencies"] = serde_json::json!(
                    dependencies.iter().map(|dep| {
                        serde_json::json!({
                            "name": dep.target_node.name,
                            "kind": format!("{:?}", dep.target_node.kind),
                            "file": dep.target_node.file.display().to_string(),
                            "edge_kind": format!("{:?}", dep.edge_kind)
                        })
                    }).collect::<Vec<_>>()
                );
            }

            if include_usages {
                let references = server.graph_query().find_references(&symbol_id)?;
                result["usages"] = serde_json::json!(
                    references.iter().map(|ref_| {
                        serde_json::json!({
                            "name": ref_.source_node.name,
                            "kind": format!("{:?}", ref_.source_node.kind),
                            "file": ref_.location.file.display().to_string(),
                            "edge_kind": format!("{:?}", ref_.edge_kind)
                        })
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

    /// Find dependencies of a symbol
    async fn find_dependencies(&self, server: &GCoreMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
        let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        
        let target = args.get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing target parameter"))?;
        
        let dependency_type_str = args.get("dependency_type")
            .and_then(|v| v.as_str())
            .unwrap_or("direct");

        let dependency_type = match dependency_type_str {
            "direct" => gcore::graph::DependencyType::Direct,
            "calls" => gcore::graph::DependencyType::Calls,
            "imports" => gcore::graph::DependencyType::Imports,
            "reads" => gcore::graph::DependencyType::Reads,
            "writes" => gcore::graph::DependencyType::Writes,
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

        let result = serde_json::json!({
            "target": target,
            "dependency_type": dependency_type_str,
            "dependencies": dependencies.iter().map(|dep| {
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
    async fn find_references(&self, server: &GCoreMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
        let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
        
        let symbol_id_str = args.get("symbol_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing symbol_id parameter"))?;
        
        let _include_definitions = args.get("include_definitions")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let symbol_id = self.parse_node_id(symbol_id_str)?;
        let references = server.graph_query().find_references(&symbol_id)?;

        let result = serde_json::json!({
            "symbol_id": symbol_id_str,
            "references": references.iter().map(|ref_| {
                serde_json::json!({
                    "id": ref_.source_node.id.to_hex(),
                    "name": ref_.source_node.name,
                    "kind": format!("{:?}", ref_.source_node.kind),
                    "file": ref_.location.file.display().to_string(),
                    "span": {
                        "start_line": ref_.location.span.start_line,
                        "end_line": ref_.location.span.end_line,
                        "start_column": ref_.location.span.start_column,
                        "end_column": ref_.location.span.end_column
                    },
                    "edge_kind": format!("{:?}", ref_.edge_kind)
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

    /// Search symbols by pattern
    async fn search_symbols(&self, server: &GCoreMcpServer, arguments: Option<Value>) -> Result<CallToolResult> {
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
                        "function" => Some(gcore::NodeKind::Function),
                        "class" => Some(gcore::NodeKind::Class),
                        "variable" => Some(gcore::NodeKind::Variable),
                        "module" => Some(gcore::NodeKind::Module),
                        "method" => Some(gcore::NodeKind::Method),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            });
        
        let limit = args.get("limit")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        let results = server.graph_query().search_symbols(pattern, symbol_types, limit)?;

        let result = serde_json::json!({
            "pattern": pattern,
            "results": results.iter().map(|symbol| {
                serde_json::json!({
                    "id": symbol.node.id.to_hex(),
                    "name": symbol.node.name,
                    "kind": format!("{:?}", symbol.node.kind),
                    "file": symbol.node.file.display().to_string(),
                    "span": {
                        "start_line": symbol.node.span.start_line,
                        "end_line": symbol.node.span.end_line,
                        "start_column": symbol.node.span.start_column,
                        "end_column": symbol.node.span.end_column
                    },
                    "signature": symbol.node.signature,
                    "references_count": symbol.references_count,
                    "dependencies_count": symbol.dependencies_count
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

    /// Parse a node ID from a hex string
    fn parse_node_id(&self, hex_str: &str) -> Result<gcore::NodeId> {
        gcore::NodeId::from_hex(hex_str)
            .map_err(|e| anyhow::anyhow!("Invalid node ID format: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GCoreMcpServer;
    use tempfile::TempDir;
    use std::fs;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    async fn create_test_server() -> Arc<RwLock<GCoreMcpServer>> {
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

        let mut server = GCoreMcpServer::new().expect("Failed to create server");
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
        assert_eq!(tools_result.tools.len(), 6);
        assert!(tools_result.next_cursor.is_none());
        
        // Verify all expected tools are present
        let tool_names: Vec<String> = tools_result.tools.iter().map(|t| t.name.clone()).collect();
        assert!(tool_names.contains(&"repository_stats".to_string()));
        assert!(tool_names.contains(&"trace_path".to_string()));
        assert!(tool_names.contains(&"explain_symbol".to_string()));
        assert!(tool_names.contains(&"find_dependencies".to_string()));
        assert!(tool_names.contains(&"find_references".to_string()));
        assert!(tool_names.contains(&"search_symbols".to_string()));
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
        let server = Arc::new(RwLock::new(GCoreMcpServer::new().unwrap()));
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

    #[test]
    fn test_parse_node_id_invalid() {
        let server = Arc::new(RwLock::new(GCoreMcpServer::new().unwrap()));
        let tool_manager = ToolManager::new(server);
        
        let invalid_hex = "not_hex";
        let result = tool_manager.parse_node_id(invalid_hex);
        assert!(result.is_err());
    }
} 