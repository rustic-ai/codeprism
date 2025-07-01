//! Dependency analysis tools.
//!
//! This module provides tools for analyzing transitive dependencies,
//! detecting cycles, and mapping dependency relationships.

use crate::{tools_legacy::*, CodePrismMcpServer};
use anyhow::Result;
use serde_json::Value;

/// List dependency analysis tools
pub fn list_tools() -> Vec<Tool> {
    vec![Tool {
        name: "analyze_transitive_dependencies".to_string(),
        title: Some("Analyze Transitive Dependencies".to_string()),
        description: "Analyze complete dependency chains with cycle detection and depth analysis"
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Target symbol or file to analyze dependencies for"
                },
                "max_depth": {
                    "type": "integer",
                    "default": 5,
                    "description": "Maximum depth for dependency traversal"
                },
                "detect_cycles": {
                    "type": "boolean",
                    "default": true,
                    "description": "Whether to detect circular dependencies"
                },
                "include_external_dependencies": {
                    "type": "boolean",
                    "default": false,
                    "description": "Include external library dependencies"
                },
                "dependency_types": {
                    "type": "array",
                    "items": {"type": "string"},
                    "default": ["all"],
                    "description": "Types of dependencies to analyze"
                }
            },
            "required": ["target"]
        }),
    }]
}

/// Call dependency analysis tool
pub async fn call_tool(
    tool_name: &str,
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    match tool_name {
        "analyze_transitive_dependencies" => {
            analyze_transitive_dependencies(server, arguments).await
        }
        _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
    }
}

/// Analyze transitive dependencies
async fn analyze_transitive_dependencies(
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    let target = args
        .get("target")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing target parameter"))?;

    let max_depth = args
        .get("max_depth")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(5);

    let detect_cycles = args
        .get("detect_cycles")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_external = args
        .get("include_external_dependencies")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let dependency_types: Vec<String> = args
        .get("dependency_types")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_else(|| vec!["all".to_string()]);

    let result = if server.repository_path().is_some() {
        let analysis = perform_transitive_analysis(
            server,
            target,
            max_depth,
            detect_cycles,
            include_external,
            &dependency_types,
        )
        .await?;

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

async fn perform_transitive_analysis(
    server: &CodePrismMcpServer,
    target: &str,
    max_depth: usize,
    detect_cycles: bool,
    _include_external: bool,
    dependency_types: &[String],
) -> Result<serde_json::Value> {
    // Parse target as node ID
    let target_id = parse_node_id(target)?;

    // Build transitive dependency tree
    let transitive_deps =
        build_transitive_dependencies(server, &target_id, max_depth, dependency_types).await?;

    // Detect cycles if requested
    let cycles = if detect_cycles {
        detect_dependency_cycles(server, &target_id, &transitive_deps).await?
    } else {
        Vec::new()
    };

    Ok(serde_json::json!({
        "transitive_dependencies": transitive_deps,
        "cycles": cycles,
        "summary": {
            "total_dependencies": transitive_deps.len(),
            "cycles_detected": cycles.len(),
            "max_depth_reached": calculate_max_depth(&transitive_deps),
        }
    }))
}

async fn build_transitive_dependencies(
    server: &CodePrismMcpServer,
    start_node: &codeprism_core::NodeId,
    max_depth: usize,
    _dependency_types: &[String],
) -> Result<Vec<serde_json::Value>> {
    let mut dependencies = Vec::new();
    let mut visited = std::collections::HashSet::new();

    build_deps_recursive(
        server,
        start_node,
        &mut dependencies,
        &mut visited,
        0,
        max_depth,
    )
    .await?;

    Ok(dependencies)
}

#[async_recursion::async_recursion]
async fn build_deps_recursive(
    server: &CodePrismMcpServer,
    node_id: &codeprism_core::NodeId,
    dependencies: &mut Vec<serde_json::Value>,
    visited: &mut std::collections::HashSet<codeprism_core::NodeId>,
    current_depth: usize,
    max_depth: usize,
) -> Result<()> {
    if current_depth >= max_depth || visited.contains(node_id) {
        return Ok(());
    }

    visited.insert(*node_id);

    if let Ok(deps) = server
        .graph_query()
        .find_dependencies(node_id, codeprism_core::graph::DependencyType::Direct)
    {
        for dep in deps {
            dependencies.push(serde_json::json!({
                "source": node_id.to_hex(),
                "target": dep.target_node.id.to_hex(),
                "target_name": dep.target_node.name,
                "dependency_type": format!("{:?}", dep.edge_kind),
                "depth": current_depth + 1
            }));

            // Recurse into dependencies
            build_deps_recursive(
                server,
                &dep.target_node.id,
                dependencies,
                visited,
                current_depth + 1,
                max_depth,
            )
            .await?;
        }
    }

    Ok(())
}

async fn detect_dependency_cycles(
    server: &CodePrismMcpServer,
    start_node: &codeprism_core::NodeId,
    _dependencies: &[serde_json::Value],
) -> Result<Vec<serde_json::Value>> {
    let mut cycles = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut rec_stack = std::collections::HashSet::new();
    let mut path = Vec::new();

    detect_cycles_dfs(
        server,
        *start_node,
        &mut visited,
        &mut rec_stack,
        &mut path,
        &mut cycles,
    )
    .await?;

    Ok(cycles)
}

#[async_recursion::async_recursion]
async fn detect_cycles_dfs(
    server: &CodePrismMcpServer,
    node: codeprism_core::NodeId,
    visited: &mut std::collections::HashSet<codeprism_core::NodeId>,
    rec_stack: &mut std::collections::HashSet<codeprism_core::NodeId>,
    path: &mut Vec<codeprism_core::NodeId>,
    cycles: &mut Vec<serde_json::Value>,
) -> Result<()> {
    visited.insert(node);
    rec_stack.insert(node);
    path.push(node);

    if let Ok(deps) = server
        .graph_query()
        .find_dependencies(&node, codeprism_core::graph::DependencyType::Direct)
    {
        for dep in deps {
            let target_id = dep.target_node.id;

            if rec_stack.contains(&target_id) {
                // Found a cycle
                let cycle_start = path.iter().position(|&n| n == target_id).unwrap_or(0);
                let cycle_path: Vec<String> =
                    path[cycle_start..].iter().map(|id| id.to_hex()).collect();

                cycles.push(serde_json::json!({
                    "cycle_path": cycle_path,
                    "cycle_length": cycle_path.len(),
                    "severity": if cycle_path.len() <= 3 { "High" } else { "Medium" }
                }));
            } else if !visited.contains(&target_id) {
                detect_cycles_dfs(server, target_id, visited, rec_stack, path, cycles).await?;
            }
        }
    }

    rec_stack.remove(&node);
    path.pop();

    Ok(())
}

fn calculate_max_depth(dependencies: &[serde_json::Value]) -> usize {
    dependencies
        .iter()
        .filter_map(|dep| dep.get("depth").and_then(|d| d.as_u64()))
        .max()
        .unwrap_or(0) as usize
}

fn parse_node_id(hex_str: &str) -> Result<codeprism_core::NodeId> {
    codeprism_core::NodeId::from_hex(hex_str)
        .map_err(|_| anyhow::anyhow!("Invalid node ID format: {}", hex_str))
}
