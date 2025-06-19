//! Data flow analysis tools

use crate::tools_legacy::{CallToolParams, CallToolResult, Tool, ToolContent};
use crate::CodePrismMcpServer;
use anyhow::Result;
use serde_json::Value;

/// Trace data flow for a specific symbol
async fn trace_symbol_data_flow(
    server: &CodePrismMcpServer,
    node: &codeprism_core::Node,
    direction: &str,
    max_depth: usize,
    include_transformations: bool,
) -> Value {
    let mut flows = Vec::new();
    let mut transformations = Vec::new();

    // Get dependencies and references for the symbol
    match direction {
        "forward" | "both" => {
            if let Ok(dependencies) = server
                .graph_query()
                .find_dependencies(&node.id, codeprism_core::graph::DependencyType::Direct)
            {
                for dep in dependencies.iter().take(max_depth) {
                    flows.push(serde_json::json!({
                        "direction": "forward",
                        "from": {
                            "id": node.id.to_hex(),
                            "name": node.name,
                            "kind": format!("{:?}", node.kind)
                        },
                        "to": {
                            "id": dep.target_node.id.to_hex(),
                            "name": dep.target_node.name,
                            "kind": format!("{:?}", dep.target_node.kind),
                            "file": dep.target_node.file.display().to_string()
                        },
                        "edge_type": format!("{:?}", dep.edge_kind)
                    }));

                    if include_transformations
                        && matches!(dep.edge_kind, codeprism_core::EdgeKind::Calls)
                    {
                        transformations.push(serde_json::json!({
                            "type": "function_call",
                            "source": node.name,
                            "target": dep.target_node.name,
                            "transformation": "call"
                        }));
                    }
                }
            }
        }
        _ => {}
    }

    match direction {
        "backward" | "both" => {
            if let Ok(references) = server.graph_query().find_references(&node.id) {
                for ref_info in references.iter().take(max_depth) {
                    flows.push(serde_json::json!({
                        "direction": "backward",
                        "from": {
                            "id": ref_info.source_node.id.to_hex(),
                            "name": ref_info.source_node.name,
                            "kind": format!("{:?}", ref_info.source_node.kind),
                            "file": ref_info.source_node.file.display().to_string()
                        },
                        "to": {
                            "id": node.id.to_hex(),
                            "name": node.name,
                            "kind": format!("{:?}", node.kind)
                        },
                        "edge_type": format!("{:?}", ref_info.edge_kind)
                    }));
                }
            }
        }
        _ => {}
    }

    serde_json::json!({
        "target": node.name,
        "analysis": {
            "direction": direction,
            "max_depth": max_depth,
            "include_transformations": include_transformations,
            "symbol_info": {
                "id": node.id.to_hex(),
                "name": node.name,
                "kind": format!("{:?}", node.kind),
                "file": node.file.display().to_string()
            }
        },
        "data_flow": {
            "flows": flows,
            "transformations": transformations
        },
        "summary": {
            "total_flow_steps": flows.len(),
            "transformation_count": transformations.len(),
            "directions_analyzed": match direction {
                "both" => vec!["forward", "backward"],
                dir => vec![dir]
            }
        }
    })
}

/// Analyze transitive dependencies for a specific symbol
async fn analyze_symbol_transitive_dependencies(
    server: &CodePrismMcpServer,
    node: &codeprism_core::Node,
    max_depth: usize,
    detect_cycles: bool,
) -> Value {
    let mut direct_deps = Vec::new();
    let mut transitive_deps = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut cycles = Vec::new();

    // Get direct dependencies
    if let Ok(dependencies) = server
        .graph_query()
        .find_dependencies(&node.id, codeprism_core::graph::DependencyType::Direct)
    {
        for dep in &dependencies {
            direct_deps.push(serde_json::json!({
                "id": dep.target_node.id.to_hex(),
                "name": dep.target_node.name,
                "kind": format!("{:?}", dep.target_node.kind),
                "file": dep.target_node.file.display().to_string(),
                "edge_type": format!("{:?}", dep.edge_kind)
            }));
        }

        // Collect transitive dependencies
        for dep in &dependencies {
            collect_transitive_deps(
                server,
                &dep.target_node.id,
                &mut transitive_deps,
                &mut visited,
                &mut cycles,
                max_depth,
                1,
                detect_cycles,
                &node.id,
            )
            .await;
        }
    }

    serde_json::json!({
        "target": node.name,
        "analysis": {
            "max_depth": max_depth,
            "detect_cycles": detect_cycles,
            "symbol_info": {
                "id": node.id.to_hex(),
                "name": node.name,
                "kind": format!("{:?}", node.kind),
                "file": node.file.display().to_string()
            }
        },
        "dependencies": {
            "direct": direct_deps,
            "transitive": transitive_deps,
            "cycles": cycles
        },
        "summary": {
            "total_direct": direct_deps.len(),
            "total_transitive": transitive_deps.len(),
            "max_depth_reached": visited.len(),
            "cycles_detected": cycles.len()
        }
    })
}

/// Recursively collect transitive dependencies
async fn collect_transitive_deps(
    server: &CodePrismMcpServer,
    current_id: &codeprism_core::NodeId,
    transitive_deps: &mut Vec<Value>,
    visited: &mut std::collections::HashSet<codeprism_core::NodeId>,
    cycles: &mut Vec<Value>,
    max_depth: usize,
    current_depth: usize,
    detect_cycles: bool,
    root_id: &codeprism_core::NodeId,
) {
    if current_depth >= max_depth {
        return;
    }

    if visited.contains(current_id) {
        if detect_cycles && current_id == root_id {
            cycles.push(serde_json::json!({
                "cycle_detected": true,
                "depth": current_depth,
                "node_id": current_id.to_hex()
            }));
        }
        return;
    }

    visited.insert(*current_id);

    if let Ok(dependencies) = server
        .graph_query()
        .find_dependencies(current_id, codeprism_core::graph::DependencyType::Direct)
    {
        for dep in dependencies {
            transitive_deps.push(serde_json::json!({
                "id": dep.target_node.id.to_hex(),
                "name": dep.target_node.name,
                "kind": format!("{:?}", dep.target_node.kind),
                "file": dep.target_node.file.display().to_string(),
                "edge_type": format!("{:?}", dep.edge_kind),
                "depth": current_depth
            }));

            // Recurse
            Box::pin(collect_transitive_deps(
                server,
                &dep.target_node.id,
                transitive_deps,
                visited,
                cycles,
                max_depth,
                current_depth + 1,
                detect_cycles,
                root_id,
            ))
            .await;
        }
    }
}

/// List flow analysis tools
pub fn list_tools() -> Vec<Tool> {
    vec![
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
                        "description": "Follow data flow across function calls",
                        "default": true
                    },
                    "include_field_access": {
                        "type": "boolean",
                        "description": "Include field/attribute access patterns",
                        "default": true
                    }
                },
                "required": ["variable_or_parameter"]
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
        }
    ]
}

/// Route flow analysis tool calls
pub async fn call_tool(
    server: &CodePrismMcpServer,
    params: &CallToolParams,
) -> Result<CallToolResult> {
    match params.name.as_str() {
        "trace_data_flow" => trace_data_flow(server, params.arguments.as_ref()).await,
        "analyze_transitive_dependencies" => {
            analyze_transitive_dependencies(server, params.arguments.as_ref()).await
        }
        _ => Err(anyhow::anyhow!(
            "Unknown flow analysis tool: {}",
            params.name
        )),
    }
}

/// Trace data flow
async fn trace_data_flow(
    server: &CodePrismMcpServer,
    arguments: Option<&Value>,
) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    // Support multiple parameter names for backward compatibility
    let variable_or_parameter = args
        .get("variable_or_parameter")
        .or_else(|| args.get("start_symbol"))
        .or_else(|| args.get("symbol"))
        .or_else(|| args.get("target"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Missing variable_or_parameter parameter (or start_symbol, symbol, target)"
            )
        })?;

    let direction = args
        .get("direction")
        .and_then(|v| v.as_str())
        .unwrap_or("forward");

    let max_depth = args
        .get("max_depth")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(10);

    let include_transformations = args
        .get("include_transformations")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // Try to resolve the symbol
    let result = if let Ok(symbol_results) =
        server
            .graph_query()
            .search_symbols(variable_or_parameter, None, Some(1))
    {
        if let Some(symbol_result) = symbol_results.first() {
            // Found the symbol, now trace its data flow
            trace_symbol_data_flow(
                server,
                &symbol_result.node,
                direction,
                max_depth,
                include_transformations,
            )
            .await
        } else {
            serde_json::json!({
                "target": variable_or_parameter,
                "error": "Symbol not found",
                "suggestion": "Check if the symbol name is correct or try using a different identifier"
            })
        }
    } else {
        serde_json::json!({
            "target": variable_or_parameter,
            "error": "Failed to search for symbol",
            "suggestion": "Ensure the repository is properly indexed"
        })
    };

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(result.get("error").is_some()),
    })
}

/// Analyze transitive dependencies
async fn analyze_transitive_dependencies(
    server: &CodePrismMcpServer,
    arguments: Option<&Value>,
) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    // Support multiple parameter names for backward compatibility
    let target = args
        .get("target")
        .or_else(|| args.get("symbol"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing target parameter (or symbol)"))?;

    let max_depth = args
        .get("max_depth")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(5);

    let detect_cycles = args
        .get("detect_cycles")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // Try to resolve the symbol
    let result = if let Ok(symbol_results) =
        server.graph_query().search_symbols(target, None, Some(1))
    {
        if let Some(symbol_result) = symbol_results.first() {
            // Found the symbol, now analyze its transitive dependencies
            analyze_symbol_transitive_dependencies(
                server,
                &symbol_result.node,
                max_depth,
                detect_cycles,
            )
            .await
        } else {
            serde_json::json!({
                "target": target,
                "error": "Symbol not found",
                "suggestion": "Check if the symbol name is correct or try using a different identifier"
            })
        }
    } else {
        serde_json::json!({
            "target": target,
            "error": "Failed to search for symbol",
            "suggestion": "Ensure the repository is properly indexed"
        })
    };

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(result.get("error").is_some()),
    })
}
