//! Navigation tools for tracing paths and dependencies

use anyhow::Result;
use serde_json::Value;
use crate::tools_legacy::{Tool, CallToolParams, CallToolResult, ToolContent};
use crate::PrismMcpServer;

/// List navigation tools
pub fn list_tools() -> Vec<Tool> {
    vec![
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
        }
    ]
}

/// Route navigation tool calls
pub async fn call_tool(server: &PrismMcpServer, params: &CallToolParams) -> Result<CallToolResult> {
    match params.name.as_str() {
        "trace_path" => trace_path(server, params.arguments.as_ref()).await,
        "find_dependencies" => find_dependencies(server, params.arguments.as_ref()).await,
        "find_references" => find_references(server, params.arguments.as_ref()).await,
        _ => Err(anyhow::anyhow!("Unknown navigation tool: {}", params.name)),
    }
}

/// Parse node ID from hex string
fn parse_node_id(hex_str: &str) -> Result<prism_core::NodeId> {
    prism_core::NodeId::from_hex(hex_str)
        .map_err(|e| anyhow::anyhow!("Invalid node ID '{}': {}", hex_str, e))
}

/// Extract source context around a specific line
fn extract_source_context(file_path: &std::path::Path, line_number: usize, context_lines: usize) -> Option<serde_json::Value> {
    if let Ok(content) = std::fs::read_to_string(file_path) {
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        
        if line_number == 0 || line_number > total_lines {
            return None;
        }
        
        // Convert to 0-based indexing
        let target_line_idx = line_number - 1;
        
        // Calculate context range
        let start_idx = target_line_idx.saturating_sub(context_lines);
        let end_idx = std::cmp::min(target_line_idx + context_lines + 1, total_lines);
        
        let context_lines_data: Vec<serde_json::Value> = (start_idx..end_idx)
            .map(|idx| {
                serde_json::json!({
                    "line_number": idx + 1,
                    "content": lines[idx],
                    "is_target": idx == target_line_idx
                })
            })
            .collect();
        
        Some(serde_json::json!({
            "file": file_path.display().to_string(),
            "target_line": line_number,
            "context_start": start_idx + 1,
            "context_end": end_idx,
            "lines": context_lines_data
        }))
    } else {
        None
    }
}

/// Create node info with source context
fn create_node_info_with_context(node: &prism_core::Node, context_lines: usize) -> serde_json::Value {
    let mut info = serde_json::json!({
        "id": node.id.to_hex(),
        "name": node.name,
        "kind": format!("{:?}", node.kind),
        "file": node.file.display().to_string(),
        "span": {
            "start_line": node.span.start_line,
            "end_line": node.span.end_line,
            "start_column": node.span.start_column,
            "end_column": node.span.end_column
        }
    });
    
    if let Some(context) = extract_source_context(&node.file, node.span.start_line, context_lines) {
        info["source_context"] = context;
    }
    
    info
}

/// Validate that a dependency node has a valid name
fn is_valid_dependency_node(node: &prism_core::Node) -> bool {
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

/// Trace path between two symbols
async fn trace_path(server: &PrismMcpServer, arguments: Option<&Value>) -> Result<CallToolResult> {
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
    let source_id = parse_node_id(source_str)?;
    let target_id = parse_node_id(target_str)?;

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

/// Find dependencies of a symbol
async fn find_dependencies(server: &PrismMcpServer, arguments: Option<&Value>) -> Result<CallToolResult> {
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
    let dependencies = if let Ok(node_id) = parse_node_id(target) {
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
        .filter(|dep| is_valid_dependency_node(&dep.target_node))
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
async fn find_references(server: &PrismMcpServer, arguments: Option<&Value>) -> Result<CallToolResult> {
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

    let symbol_id = parse_node_id(symbol_id_str)?;
    let references = server.graph_query().find_references(&symbol_id)?;

    let result = serde_json::json!({
        "symbol_id": symbol_id_str,
        "references": references.iter().map(|ref_| {
            let mut ref_info = create_node_info_with_context(&ref_.source_node, context_lines);
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