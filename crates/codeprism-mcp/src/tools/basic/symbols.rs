//! Symbol navigation and analysis tools.
//!
//! This module provides tools for navigating between symbols, understanding symbol
//! context, and analyzing symbol relationships in the codebase.

use crate::{tools_legacy::*, CodePrismMcpServer};
use anyhow::Result;
use serde_json::Value;

/// List symbol navigation tools
///
/// Returns a list of symbol-related navigation tools available in the modular architecture.
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
    ]
}

/// Handle symbol navigation tool calls
///
/// Routes symbol tool calls to appropriate functions.
pub async fn call_tool(
    tool_name: &str,
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    match tool_name {
        "trace_path" => trace_path(server, arguments).await,
        "explain_symbol" => explain_symbol(server, arguments).await,
        "find_dependencies" => find_dependencies(server, arguments).await,
        "find_references" => find_references(server, arguments).await,
        _ => Err(anyhow::anyhow!("Unknown symbol tool: {}", tool_name)),
    }
}

/// Trace path between two symbols
async fn trace_path(
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    let source_str = args
        .get("source")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing source parameter"))?;

    let target_str = args
        .get("target")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing target parameter"))?;

    let max_depth = args
        .get("max_depth")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);

    // Parse node IDs from hex strings
    let source_id = parse_node_id(source_str)?;
    let target_id = parse_node_id(target_str)?;

    match server
        .graph_query()
        .find_path(&source_id, &target_id, max_depth)?
    {
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

/// Explain a symbol with detailed context
async fn explain_symbol(
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    let symbol_id_str = args
        .get("symbol_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing symbol_id parameter"))?;

    let include_dependencies = args
        .get("include_dependencies")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let include_usages = args
        .get("include_usages")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let context_lines = args
        .get("context_lines")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(4);

    let symbol_id = parse_node_id(symbol_id_str)?;

    if let Some(node) = server.graph_store().get_node(&symbol_id) {
        let mut result = serde_json::json!({
            "symbol": create_node_info_with_context(&node, context_lines)
        });

        // Enhanced inheritance information for classes
        if matches!(node.kind, codeprism_core::NodeKind::Class) {
            if let Ok(inheritance_info) = server.graph_query().get_inheritance_info(&symbol_id) {
                let mut inheritance_data = serde_json::Map::new();

                // Basic inheritance information
                inheritance_data.insert(
                    "class_name".to_string(),
                    serde_json::Value::String(inheritance_info.class_name),
                );
                inheritance_data.insert(
                    "is_metaclass".to_string(),
                    serde_json::Value::Bool(inheritance_info.is_metaclass),
                );

                // Base classes
                if !inheritance_info.base_classes.is_empty() {
                    let base_classes: Vec<_> = inheritance_info
                        .base_classes
                        .iter()
                        .map(|rel| {
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
                        })
                        .collect();
                    inheritance_data.insert(
                        "base_classes".to_string(),
                        serde_json::Value::Array(base_classes),
                    );
                }

                // Subclasses
                if !inheritance_info.subclasses.is_empty() {
                    let subclasses: Vec<_> = inheritance_info
                        .subclasses
                        .iter()
                        .map(|rel| {
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
                        })
                        .collect();
                    inheritance_data.insert(
                        "subclasses".to_string(),
                        serde_json::Value::Array(subclasses),
                    );
                }

                // Metaclass information
                if let Some(metaclass) = inheritance_info.metaclass {
                    inheritance_data.insert(
                        "metaclass".to_string(),
                        serde_json::json!({
                            "name": metaclass.class_name,
                            "file": metaclass.file.display().to_string(),
                            "span": {
                                "start_line": metaclass.span.start_line,
                                "end_line": metaclass.span.end_line,
                                "start_column": metaclass.span.start_column,
                                "end_column": metaclass.span.end_column
                            }
                        }),
                    );
                }

                // Mixins
                if !inheritance_info.mixins.is_empty() {
                    let mixins: Vec<_> = inheritance_info
                        .mixins
                        .iter()
                        .map(|rel| {
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
                        })
                        .collect();
                    inheritance_data.insert("mixins".to_string(), serde_json::Value::Array(mixins));
                }

                result["inheritance"] = serde_json::Value::Object(inheritance_data);
            }
        }

        // Add dependency information if requested
        if include_dependencies {
            if let Ok(dependencies) = server
                .graph_query()
                .find_dependencies(&symbol_id, codeprism_core::graph::DependencyType::Direct)
            {
                let deps: Vec<_> = dependencies
                    .iter()
                    .map(|dep| {
                        serde_json::json!({
                            "target_id": dep.target_node.id.to_hex(),
                            "kind": format!("{:?}", dep.edge_kind),
                            "target_info": create_node_info_with_context(&dep.target_node, 0)
                        })
                    })
                    .collect();
                result["dependencies"] = serde_json::Value::Array(deps);
            }
        }

        // Add usage information if requested
        if include_usages {
            if let Ok(references) = server.graph_query().find_references(&symbol_id) {
                let refs: Vec<_> = references
                    .iter()
                    .map(|ref_info| {
                        serde_json::json!({
                            "source_id": ref_info.source_node.id.to_hex(),
                            "kind": format!("{:?}", ref_info.edge_kind),
                            "source_info": create_node_info_with_context(&ref_info.source_node, 0)
                        })
                    })
                    .collect();
                result["usages"] = serde_json::Value::Array(refs);
            }
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
                text: format!("Symbol with ID '{}' not found", symbol_id_str),
            }],
            is_error: Some(true),
        })
    }
}

/// Find dependencies for a symbol or file
async fn find_dependencies(
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    let target = args
        .get("target")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing target parameter"))?;

    let dependency_type = args
        .get("dependency_type")
        .and_then(|v| v.as_str())
        .unwrap_or("direct");

    // Determine if target is a node ID or file path
    let dependencies = if let Ok(node_id) = parse_node_id(target) {
        // Target is a node ID
        match dependency_type {
            "direct" => {
                let deps = server
                    .graph_query()
                    .find_dependencies(&node_id, codeprism_core::graph::DependencyType::Direct)?;
                deps.into_iter()
                    .filter(|dep| is_valid_dependency_node(&dep.target_node))
                    .map(|dep| {
                        serde_json::json!({
                            "target_id": dep.target_node.id.to_hex(),
                            "target_name": dep.target_node.name,
                            "target_kind": format!("{:?}", dep.target_node.kind),
                            "dependency_kind": format!("{:?}", dep.edge_kind),
                            "target_file": dep.target_node.file.display().to_string(),
                            "target_span": {
                                "start_line": dep.target_node.span.start_line,
                                "end_line": dep.target_node.span.end_line,
                                "start_column": dep.target_node.span.start_column,
                                "end_column": dep.target_node.span.end_column
                            }
                        })
                    })
                    .collect()
            }
            "calls" => {
                let deps = server
                    .graph_query()
                    .find_dependencies(&node_id, codeprism_core::graph::DependencyType::Calls)?;
                deps.into_iter()
                    .filter(|dep| is_valid_dependency_node(&dep.target_node))
                    .map(|dep| {
                        serde_json::json!({
                            "target_id": dep.target_node.id.to_hex(),
                            "target_name": dep.target_node.name,
                            "target_kind": format!("{:?}", dep.target_node.kind),
                            "dependency_kind": "calls",
                            "target_file": dep.target_node.file.display().to_string(),
                            "target_span": {
                                "start_line": dep.target_node.span.start_line,
                                "end_line": dep.target_node.span.end_line,
                                "start_column": dep.target_node.span.start_column,
                                "end_column": dep.target_node.span.end_column
                            }
                        })
                    })
                    .collect()
            }
            "imports" => {
                let deps = server
                    .graph_query()
                    .find_dependencies(&node_id, codeprism_core::graph::DependencyType::Imports)?;
                deps.into_iter()
                    .filter(|dep| is_valid_dependency_node(&dep.target_node))
                    .map(|dep| {
                        serde_json::json!({
                            "target_id": dep.target_node.id.to_hex(),
                            "target_name": dep.target_node.name,
                            "target_kind": format!("{:?}", dep.target_node.kind),
                            "dependency_kind": "imports",
                            "target_file": dep.target_node.file.display().to_string(),
                            "target_span": {
                                "start_line": dep.target_node.span.start_line,
                                "end_line": dep.target_node.span.end_line,
                                "start_column": dep.target_node.span.start_column,
                                "end_column": dep.target_node.span.end_column
                            }
                        })
                    })
                    .collect()
            }
            "reads" => {
                let deps = server
                    .graph_query()
                    .find_dependencies(&node_id, codeprism_core::graph::DependencyType::Reads)?;
                deps.into_iter()
                    .filter(|dep| is_valid_dependency_node(&dep.target_node))
                    .map(|dep| {
                        serde_json::json!({
                            "target_id": dep.target_node.id.to_hex(),
                            "target_name": dep.target_node.name,
                            "target_kind": format!("{:?}", dep.target_node.kind),
                            "dependency_kind": "reads",
                            "target_file": dep.target_node.file.display().to_string(),
                            "target_span": {
                                "start_line": dep.target_node.span.start_line,
                                "end_line": dep.target_node.span.end_line,
                                "start_column": dep.target_node.span.start_column,
                                "end_column": dep.target_node.span.end_column
                            }
                        })
                    })
                    .collect()
            }
            "writes" => {
                let deps = server
                    .graph_query()
                    .find_dependencies(&node_id, codeprism_core::graph::DependencyType::Writes)?;
                deps.into_iter()
                    .filter(|dep| is_valid_dependency_node(&dep.target_node))
                    .map(|dep| {
                        serde_json::json!({
                            "target_id": dep.target_node.id.to_hex(),
                            "target_name": dep.target_node.name,
                            "target_kind": format!("{:?}", dep.target_node.kind),
                            "dependency_kind": "writes",
                            "target_file": dep.target_node.file.display().to_string(),
                            "target_span": {
                                "start_line": dep.target_node.span.start_line,
                                "end_line": dep.target_node.span.end_line,
                                "start_column": dep.target_node.span.start_column,
                                "end_column": dep.target_node.span.end_column
                            }
                        })
                    })
                    .collect()
            }
            _ => {
                return Ok(CallToolResult {
                    content: vec![ToolContent::Text {
                        text: format!("Invalid dependency type: {}", dependency_type),
                    }],
                    is_error: Some(true),
                });
            }
        }
    } else {
        // Target is a file path - analyze file dependencies
        Vec::new() // Placeholder for file-level dependency analysis
    };

    let result = serde_json::json!({
        "target": target,
        "dependency_type": dependency_type,
        "dependencies": dependencies
    });

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
}

/// Find references to a symbol
async fn find_references(
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    let symbol_id_str = args
        .get("symbol_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing symbol_id parameter"))?;

    let include_definitions = args
        .get("include_definitions")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let context_lines = args
        .get("context_lines")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(4);

    let symbol_id = parse_node_id(symbol_id_str)?;

    let mut result = serde_json::json!({
        "symbol_id": symbol_id_str,
        "include_definitions": include_definitions,
        "references": []
    });

    // Find references to the symbol
    if let Ok(references) = server.graph_query().find_references(&symbol_id) {
        let refs: Vec<_> = references
            .iter()
            .map(|ref_info| {
                serde_json::json!({
                    "source_id": ref_info.source_node.id.to_hex(),
                    "source_name": ref_info.source_node.name,
                    "source_kind": format!("{:?}", ref_info.source_node.kind),
                    "reference_kind": format!("{:?}", ref_info.edge_kind),
                    "file": ref_info.source_node.file.display().to_string(),
                    "span": {
                        "start_line": ref_info.source_node.span.start_line,
                        "end_line": ref_info.source_node.span.end_line,
                        "start_column": ref_info.source_node.span.start_column,
                        "end_column": ref_info.source_node.span.end_column
                    },
                    "source_context": extract_source_context(
                        &ref_info.source_node.file,
                        ref_info.source_node.span.start_line,
                        context_lines
                    )
                })
            })
            .collect();

        result["references"] = serde_json::Value::Array(refs);
    }

    // Include the definition if requested
    if include_definitions {
        if let Some(definition_node) = server.graph_store().get_node(&symbol_id) {
            result["definition"] = serde_json::json!({
                "id": symbol_id.to_hex(),
                "name": definition_node.name,
                "kind": format!("{:?}", definition_node.kind),
                "file": definition_node.file.display().to_string(),
                "span": {
                    "start_line": definition_node.span.start_line,
                    "end_line": definition_node.span.end_line,
                    "start_column": definition_node.span.start_column,
                    "end_column": definition_node.span.end_column
                },
                "source_context": extract_source_context(
                    &definition_node.file,
                    definition_node.span.start_line,
                    context_lines
                )
            });
        }
    }

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
}

/// Helper functions extracted from legacy implementation

/// Parse a node ID from a hex string
fn parse_node_id(hex_str: &str) -> Result<codeprism_core::NodeId> {
    codeprism_core::NodeId::from_hex(hex_str)
        .map_err(|e| anyhow::anyhow!("Invalid node ID format: {}", e))
}

/// Check if a node is valid for dependency analysis
fn is_valid_dependency_node(node: &codeprism_core::Node) -> bool {
    matches!(
        node.kind,
        codeprism_core::NodeKind::Function
            | codeprism_core::NodeKind::Class
            | codeprism_core::NodeKind::Variable
            | codeprism_core::NodeKind::Module
            | codeprism_core::NodeKind::Method
    )
}

/// Extract source context around a line number from a file
fn extract_source_context(
    file_path: &std::path::Path,
    line_number: usize,
    context_lines: usize,
) -> Option<serde_json::Value> {
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
    for (i, _) in lines.iter().enumerate().take(end_idx + 1).skip(start_idx) {
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
fn create_node_info_with_context(
    node: &codeprism_core::Node,
    context_lines: usize,
) -> serde_json::Value {
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
    if let Some(context) = extract_source_context(&node.file, node.span.start_line, context_lines) {
        node_info["source_context"] = context;
    }

    node_info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_node_id_valid() {
        let hex_str = "1234567890abcdef1234567890abcdef"; // 32 characters = 16 bytes
        let result = parse_node_id(hex_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_node_id_invalid() {
        let hex_str = "invalid-hex";
        let result = parse_node_id(hex_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_valid_dependency_node() {
        let node = codeprism_core::Node::new(
            "test_repo",
            codeprism_core::NodeKind::Function,
            "test_function".to_string(),
            codeprism_core::Language::Python,
            std::path::PathBuf::from("test.py"),
            codeprism_core::Span::new(0, 10, 1, 1, 1, 10),
        )
        .with_signature("def test_function():".to_string());

        assert!(is_valid_dependency_node(&node));
    }

    #[test]
    fn test_symbol_tools_list() {
        let tools = list_tools();
        assert_eq!(tools.len(), 4);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"trace_path"));
        assert!(tool_names.contains(&"explain_symbol"));
        assert!(tool_names.contains(&"find_dependencies"));
        assert!(tool_names.contains(&"find_references"));
    }
}
