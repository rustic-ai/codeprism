//! Symbol explanation and search tools

use anyhow::Result;
use serde_json::Value;
use crate::tools::{Tool, CallToolParams, CallToolResult, ToolContent};
use crate::context::{WorkflowContext, ToolSuggestion, SessionManager, SessionState};
use crate::context::workflow::ConfidenceLevel;
use crate::context::session::SessionId;
use crate::PrismMcpServer;

/// List symbol tools
pub fn list_tools() -> Vec<Tool> {
    vec![
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
        }
    ]
}

/// Route symbol tool calls
pub async fn call_tool(server: &PrismMcpServer, params: &CallToolParams) -> Result<CallToolResult> {
    match params.name.as_str() {
        "explain_symbol" => explain_symbol(server, params.arguments.as_ref()).await,
        "search_symbols" => search_symbols(server, params.arguments.as_ref()).await,
        _ => Err(anyhow::anyhow!("Unknown symbol tool: {}", params.name)),
    }
}

/// Parse node ID from hex string
fn parse_node_id(hex_str: &str) -> Result<prism_core::NodeId> {
    prism_core::NodeId::from_hex(hex_str)
        .map_err(|e| anyhow::anyhow!("Invalid node ID '{}': {}", hex_str, e))
}

/// Resolve symbol name to node ID using search
async fn resolve_symbol_name(server: &PrismMcpServer, symbol_name: &str) -> Result<Option<prism_core::NodeId>> {
    // Try to search for the symbol by name
    let results = server.graph_query().search_symbols(symbol_name, None, Some(10))?;
    
    // Look for exact match first
    for result in &results {
        if result.node.name == symbol_name {
            return Ok(Some(result.node.id));
        }
    }
    
    // If no exact match, return the first result if any
    if let Some(first) = results.first() {
        Ok(Some(first.node.id))
    } else {
        Ok(None)
    }
}

/// Resolve symbol identifier - try as node ID first, then as symbol name
async fn resolve_symbol_identifier(server: &PrismMcpServer, identifier: &str) -> Result<prism_core::NodeId> {
    // First try to parse as node ID
    if let Ok(node_id) = parse_node_id(identifier) {
        return Ok(node_id);
    }
    
    // Then try to resolve as symbol name
    if let Some(node_id) = resolve_symbol_name(server, identifier).await? {
        return Ok(node_id);
    }
    
    Err(anyhow::anyhow!("Could not resolve symbol identifier '{}'. Please provide either a valid node ID (hex string) or symbol name that exists in the codebase.", identifier))
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

/// Parse inheritance filter string into InheritanceFilter enum
fn parse_inheritance_filter(filter_str: &str) -> Option<prism_core::InheritanceFilter> {
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

/// Generate workflow suggestions for a symbol
fn generate_symbol_workflow_suggestions(symbol_id_str: &str, node: &prism_core::Node) -> Vec<ToolSuggestion> {
    let mut suggestions = Vec::new();

    // Suggest finding references if not already done
    suggestions.push(
        ToolSuggestion::new(
            "find_references".to_string(),
            ConfidenceLevel::High,
            format!("Find all references to symbol '{}'", node.name),
            "Understanding of how and where the symbol is used throughout the codebase".to_string(),
            1,
        ).with_parameter("symbol_id".to_string(), serde_json::Value::String(symbol_id_str.to_string()))
    );

    // Suggest dependency analysis
    suggestions.push(
        ToolSuggestion::new(
            "find_dependencies".to_string(),
            ConfidenceLevel::Medium,
            format!("Analyze dependencies for symbol '{}'", node.name),
            "Understanding of what this symbol depends on".to_string(),
            2,
        ).with_parameter("target".to_string(), serde_json::Value::String(symbol_id_str.to_string()))
    );

    // For classes, suggest inheritance analysis
    if matches!(node.kind, prism_core::NodeKind::Class) {
        suggestions.push(
            ToolSuggestion::new(
                "trace_inheritance".to_string(),
                ConfidenceLevel::Medium,
                format!("Analyze inheritance hierarchy for class '{}'", node.name),
                "Complete understanding of inheritance relationships and method resolution".to_string(),
                3,
            ).with_parameter("class_name".to_string(), serde_json::Value::String(node.name.clone()))
        );
    }

    // For functions, suggest data flow analysis
    if matches!(node.kind, prism_core::NodeKind::Function | prism_core::NodeKind::Method) {
        suggestions.push(
            ToolSuggestion::new(
                "trace_data_flow".to_string(),
                ConfidenceLevel::Medium,
                format!("Trace data flow through function '{}'", node.name),
                "Understanding of how data flows through this function".to_string(),
                3,
            ).with_parameter("function_id".to_string(), serde_json::Value::String(symbol_id_str.to_string()))
        );
    }

    suggestions
}

/// Explain a symbol with context and workflow guidance
async fn explain_symbol(server: &PrismMcpServer, arguments: Option<&Value>) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    
    // Support both "symbol_id" and "symbol" parameter names for backward compatibility
    let symbol_id_str = args.get("symbol_id")
        .or_else(|| args.get("symbol"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing symbol_id parameter (or symbol)"))?;
    
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

    // Session context for workflow guidance
    let session_id = args.get("session_id")
        .and_then(|v| v.as_str())
        .map(|s| SessionId(s.to_string()));

    let symbol_id = resolve_symbol_identifier(server, symbol_id_str).await?;

    if let Some(node) = server.graph_store().get_node(&symbol_id) {
        let mut result = serde_json::json!({
            "symbol": create_node_info_with_context(&node, context_lines)
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
                .filter(|dep| is_valid_dependency_node(&dep.target_node))
                .collect();
            
            result["dependencies"] = serde_json::json!(
                valid_dependencies.iter().map(|dep| {
                    let mut dep_info = create_node_info_with_context(&dep.target_node, context_lines);
                    dep_info["edge_kind"] = serde_json::json!(format!("{:?}", dep.edge_kind));
                    dep_info
                    }).collect::<Vec<_>>()
            );
        }

        if include_usages {
            let references = server.graph_query().find_references(&symbol_id)?;
            result["usages"] = serde_json::json!(
                references.iter().map(|ref_| {
                    let mut usage_info = create_node_info_with_context(&ref_.source_node, context_lines);
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

        // Add workflow guidance
        let workflow_suggestions = generate_symbol_workflow_suggestions(symbol_id_str, &node);
        result["workflow_guidance"] = serde_json::json!({
            "next_steps": workflow_suggestions.iter().map(|suggestion| {
                serde_json::json!({
                    "tool": suggestion.tool_name,
                    "parameters": suggestion.suggested_parameters,
                    "confidence": format!("{:?}", suggestion.confidence),
                    "reasoning": suggestion.reasoning,
                    "expected_outcome": suggestion.expected_outcome,
                    "priority": suggestion.priority
                })
            }).collect::<Vec<_>>(),
            "analysis_context": {
                "symbol_type": format!("{:?}", node.kind),
                "file": node.file.display().to_string(),
                "complexity_hint": match node.kind {
                    prism_core::NodeKind::Class => "Consider analyzing inheritance relationships and decorator patterns",
                    prism_core::NodeKind::Function | prism_core::NodeKind::Method => "Consider tracing data flow and analyzing complexity",
                    prism_core::NodeKind::Module => "Consider exploring contained symbols and dependencies",
                    _ => "Consider analyzing dependencies and references"
                }
            }
        });

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

/// Search symbols by pattern
async fn search_symbols(server: &PrismMcpServer, arguments: Option<&Value>) -> Result<CallToolResult> {
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
                .filter_map(|s| parse_inheritance_filter(s))
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
            let mut symbol_info = create_node_info_with_context(&symbol.node, context_lines);
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