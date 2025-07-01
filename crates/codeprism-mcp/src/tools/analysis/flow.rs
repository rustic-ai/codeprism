//! Data flow analysis tools

#![allow(clippy::too_many_arguments)]

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

/// List data flow analysis tools
pub fn list_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "trace_data_flow".to_string(),
            title: Some("Trace Data Flow".to_string()),
            description: "Trace data flow through the codebase with variable and parameter tracking".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "variable_or_parameter": {
                        "type": "string",
                        "description": "Variable or parameter to trace"
                    },
                    "direction": {
                        "type": "string",
                        "enum": ["forward", "backward", "both"],
                        "default": "forward",
                        "description": "Direction to trace data flow"
                    },
                    "include_transformations": {
                        "type": "boolean",
                        "default": true,
                        "description": "Include data transformations in trace"
                    },
                    "max_depth": {
                        "type": "integer",
                        "default": 10,
                        "description": "Maximum depth for flow traversal"
                    },
                    "follow_function_calls": {
                        "type": "boolean",
                        "default": true,
                        "description": "Follow data flow through function calls"
                    },
                    "include_field_access": {
                        "type": "boolean",
                        "default": true,
                        "description": "Include field access in data flow"
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

/// Call data flow analysis tool
pub async fn call_tool(
    tool_name: &str,
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    match tool_name {
        "trace_data_flow" => trace_data_flow(server, arguments).await,
        // NOTE: analyze_transitive_dependencies moved to dependencies module
        _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
    }
}

/// Trace data flow through the codebase
async fn trace_data_flow(
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    let variable_or_parameter = args
        .get("variable_or_parameter")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing variable_or_parameter parameter"))?;

    let direction = args
        .get("direction")
        .and_then(|v| v.as_str())
        .unwrap_or("forward");

    let include_transformations = args
        .get("include_transformations")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let max_depth = args
        .get("max_depth")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(10);

    let follow_function_calls = args
        .get("follow_function_calls")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_field_access = args
        .get("include_field_access")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let symbol_id = parse_node_id(variable_or_parameter)?;

    let data_flow_result = perform_data_flow_analysis(
        server,
        &symbol_id,
        direction,
        include_transformations,
        max_depth,
        follow_function_calls,
        include_field_access,
    )
    .await?;

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&data_flow_result)?,
        }],
        is_error: Some(false),
    })
}

async fn perform_data_flow_analysis(
    server: &CodePrismMcpServer,
    symbol_id: &codeprism_core::NodeId,
    direction: &str,
    include_transformations: bool,
    max_depth: usize,
    follow_function_calls: bool,
    include_field_access: bool,
) -> Result<serde_json::Value> {
    let mut data_flows = Vec::new();
    let mut visited = std::collections::HashSet::new();

    // Get the starting symbol information
    let start_symbol = server
        .graph_store()
        .get_node(symbol_id)
        .ok_or_else(|| anyhow::anyhow!("Symbol not found: {}", symbol_id.to_hex()))?;

    match direction {
        "forward" => {
            trace_data_flow_forward(
                server,
                symbol_id,
                &mut data_flows,
                &mut visited,
                0,
                max_depth,
                include_transformations,
                follow_function_calls,
                include_field_access,
            )
            .await?;
        }
        "backward" => {
            trace_data_flow_backward(
                server,
                symbol_id,
                &mut data_flows,
                &mut visited,
                0,
                max_depth,
                include_transformations,
                follow_function_calls,
                include_field_access,
            )
            .await?;
        }
        "both" => {
            trace_data_flow_forward(
                server,
                symbol_id,
                &mut data_flows,
                &mut visited,
                0,
                max_depth,
                include_transformations,
                follow_function_calls,
                include_field_access,
            )
            .await?;

            visited.clear();

            trace_data_flow_backward(
                server,
                symbol_id,
                &mut data_flows,
                &mut visited,
                0,
                max_depth,
                include_transformations,
                follow_function_calls,
                include_field_access,
            )
            .await?;
        }
        _ => return Err(anyhow::anyhow!("Invalid direction: {}", direction)),
    }

    // Build summary statistics
    let flow_types: std::collections::HashMap<String, usize> =
        data_flows
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, flow| {
                if let Some(flow_type) = flow.get("flow_type").and_then(|v| v.as_str()) {
                    *acc.entry(flow_type.to_string()).or_insert(0) += 1;
                }
                acc
            });

    let transformations_count = data_flows
        .iter()
        .filter(|flow| {
            flow.get("has_transformation")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        })
        .count();

    Ok(serde_json::json!({
        "start_symbol": {
            "id": symbol_id.to_hex(),
            "name": start_symbol.name,
            "kind": format!("{:?}", start_symbol.kind),
            "file": start_symbol.file.display().to_string(),
            "line": start_symbol.span.start_line
        },
        "data_flows": data_flows,
        "summary": {
            "total_flows": data_flows.len(),
            "flow_types": flow_types,
            "transformations_found": transformations_count,
            "direction": direction,
            "max_depth": max_depth
        },
        "parameters": {
            "include_transformations": include_transformations,
            "follow_function_calls": follow_function_calls,
            "include_field_access": include_field_access
        }
    }))
}

#[async_recursion::async_recursion]
async fn trace_data_flow_forward(
    server: &CodePrismMcpServer,
    symbol_id: &codeprism_core::NodeId,
    data_flows: &mut Vec<serde_json::Value>,
    visited: &mut std::collections::HashSet<codeprism_core::NodeId>,
    current_depth: usize,
    max_depth: usize,
    include_transformations: bool,
    follow_function_calls: bool,
    include_field_access: bool,
) -> Result<()> {
    if current_depth >= max_depth || visited.contains(symbol_id) {
        return Ok(());
    }

    visited.insert(*symbol_id);

    // Find all references to this symbol (forward flow)
    if let Ok(references) = server.graph_query().find_references(symbol_id) {
        for reference in references {
            let ref_node = &reference.source_node;

            // Determine flow type based on context
            let flow_type = determine_flow_type(ref_node, include_field_access);

            // Check if this is a transformation
            let has_transformation = include_transformations && is_transformation_context(ref_node);

            // Create flow entry
            let flow_entry = serde_json::json!({
                "source_id": symbol_id.to_hex(),
                "target_id": ref_node.id.to_hex(),
                "target_name": ref_node.name,
                "target_kind": format!("{:?}", ref_node.kind),
                "flow_type": flow_type,
                "direction": "forward",
                "depth": current_depth + 1,
                "has_transformation": has_transformation,
                "location": {
                    "file": ref_node.file.display().to_string(),
                    "line": ref_node.span.start_line,
                    "column": ref_node.span.start_column
                }
            });

            data_flows.push(flow_entry);

            // Follow function calls if enabled
            if follow_function_calls
                && matches!(
                    ref_node.kind,
                    codeprism_core::NodeKind::Function | codeprism_core::NodeKind::Method
                )
            {
                trace_data_flow_forward(
                    server,
                    &ref_node.id,
                    data_flows,
                    visited,
                    current_depth + 1,
                    max_depth,
                    include_transformations,
                    follow_function_calls,
                    include_field_access,
                )
                .await?;
            }
        }
    }

    Ok(())
}

#[async_recursion::async_recursion]
async fn trace_data_flow_backward(
    server: &CodePrismMcpServer,
    symbol_id: &codeprism_core::NodeId,
    data_flows: &mut Vec<serde_json::Value>,
    visited: &mut std::collections::HashSet<codeprism_core::NodeId>,
    current_depth: usize,
    max_depth: usize,
    include_transformations: bool,
    follow_function_calls: bool,
    include_field_access: bool,
) -> Result<()> {
    if current_depth >= max_depth || visited.contains(symbol_id) {
        return Ok(());
    }

    visited.insert(*symbol_id);

    // Find dependencies that flow into this symbol (backward flow)
    if let Ok(dependencies) = server
        .graph_query()
        .find_dependencies(symbol_id, codeprism_core::graph::DependencyType::Direct)
    {
        for dependency in dependencies {
            let dep_node = &dependency.target_node;

            // Determine flow type based on context
            let flow_type = determine_flow_type(dep_node, include_field_access);

            // Check if this is a transformation
            let has_transformation = include_transformations && is_transformation_context(dep_node);

            // Create flow entry
            let flow_entry = serde_json::json!({
                "source_id": dep_node.id.to_hex(),
                "target_id": symbol_id.to_hex(),
                "source_name": dep_node.name,
                "source_kind": format!("{:?}", dep_node.kind),
                "flow_type": flow_type,
                "direction": "backward",
                "depth": current_depth + 1,
                "has_transformation": has_transformation,
                "location": {
                    "file": dep_node.file.display().to_string(),
                    "line": dep_node.span.start_line,
                    "column": dep_node.span.start_column
                }
            });

            data_flows.push(flow_entry);

            // Follow function calls if enabled
            if follow_function_calls
                && matches!(
                    dep_node.kind,
                    codeprism_core::NodeKind::Function | codeprism_core::NodeKind::Method
                )
            {
                trace_data_flow_backward(
                    server,
                    &dep_node.id,
                    data_flows,
                    visited,
                    current_depth + 1,
                    max_depth,
                    include_transformations,
                    follow_function_calls,
                    include_field_access,
                )
                .await?;
            }
        }
    }

    Ok(())
}

fn determine_flow_type(node: &codeprism_core::Node, include_field_access: bool) -> &'static str {
    match node.kind {
        codeprism_core::NodeKind::Variable => "variable_reference",
        codeprism_core::NodeKind::Parameter => "parameter_passing",
        codeprism_core::NodeKind::Function | codeprism_core::NodeKind::Method => "function_call",
        // codeprism_core::NodeKind::Field if include_field_access => "field_access",
        codeprism_core::NodeKind::Class => "object_instantiation",
        _ => "generic_reference",
    }
}

fn is_transformation_context(node: &codeprism_core::Node) -> bool {
    // Simple heuristic to detect transformations
    let name_lower = node.name.to_lowercase();
    name_lower.contains("transform")
        || name_lower.contains("convert")
        || name_lower.contains("map")
        || name_lower.contains("filter")
        || name_lower.contains("reduce")
        || name_lower.contains("process")
}

fn parse_node_id(hex_str: &str) -> Result<codeprism_core::NodeId> {
    codeprism_core::NodeId::from_hex(hex_str)
        .map_err(|_| anyhow::anyhow!("Invalid node ID format: {}", hex_str))
}
