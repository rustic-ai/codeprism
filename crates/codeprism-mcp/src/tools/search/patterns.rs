//! Pattern detection tools

use crate::tools_legacy::{CallToolParams, CallToolResult, Tool, ToolContent};
use crate::CodePrismMcpServer;
use anyhow::Result;
use serde_json::Value;

/// List pattern detection tools
pub fn list_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "detect_patterns".to_string(),
            title: Some("Detect Design Patterns".to_string()),
            description: "Identify design patterns, architectural structures, and metaprogramming patterns in the codebase".to_string(),
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
                            "enum": ["design_patterns", "anti_patterns", "architectural_patterns", "metaprogramming_patterns", "all"]
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
        }
    ]
}

/// Route pattern detection tool calls
pub async fn call_tool(
    server: &CodePrismMcpServer,
    params: &CallToolParams,
) -> Result<CallToolResult> {
    match params.name.as_str() {
        "detect_patterns" => detect_patterns(server, params.arguments.as_ref()).await,
        _ => Err(anyhow::anyhow!(
            "Unknown pattern detection tool: {}",
            params.name
        )),
    }
}

/// Detect design patterns in the codebase
async fn detect_patterns(
    server: &CodePrismMcpServer,
    arguments: Option<&Value>,
) -> Result<CallToolResult> {
    let default_args = serde_json::Value::Object(serde_json::Map::new());
    let args = arguments.unwrap_or(&default_args);

    let _scope = args
        .get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("repository");

    let pattern_types: Vec<String> = args
        .get("pattern_types")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_else(|| vec!["all".to_string()]);

    let confidence_threshold = args
        .get("confidence_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.8);

    let include_suggestions = args
        .get("include_suggestions")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // Basic pattern analysis implementation
    // For Phase 1, we provide a simplified version that will be enhanced in later phases
    let mut detected_patterns = Vec::new();

    // Simple pattern detection based on graph analysis
    let graph_stats = server.graph_store().get_stats();

    // Check for common patterns based on node counts and relationships
    if graph_stats.total_nodes > 0 {
        // Singleton pattern detection (simplified)
        if let Some(class_count) = graph_stats
            .nodes_by_kind
            .get(&codeprism_core::ast::NodeKind::Class)
        {
            if *class_count > 0 {
                detected_patterns.push(serde_json::json!({
                    "pattern_type": "structural",
                    "pattern_name": "Class-based Architecture",
                    "confidence": 0.9,
                    "description": format!("Detected {} classes in the codebase", class_count),
                    "locations": ["repository-wide"],
                    "impact": "positive",
                    "suggestions": if include_suggestions {
                        vec!["Ensure proper encapsulation", "Consider inheritance hierarchies"]
                    } else {
                        vec![]
                    }
                }));
            }
        }

        // Module pattern detection
        if let Some(module_count) = graph_stats
            .nodes_by_kind
            .get(&codeprism_core::ast::NodeKind::Module)
        {
            if *module_count > 1 {
                detected_patterns.push(serde_json::json!({
                    "pattern_type": "architectural",
                    "pattern_name": "Modular Architecture",
                    "confidence": 0.85,
                    "description": format!("Detected {} modules providing separation of concerns", module_count),
                    "locations": ["repository-wide"],
                    "impact": "positive",
                    "suggestions": if include_suggestions {
                        vec!["Maintain clear module boundaries", "Consider module dependencies"]
                    } else {
                        vec![]
                    }
                }));
            }
        }
    }

    // Filter patterns based on requested types
    if !pattern_types.contains(&"all".to_string()) {
        detected_patterns.retain(|pattern| {
            if let Some(pattern_type) = pattern.get("pattern_type").and_then(|v| v.as_str()) {
                pattern_types
                    .iter()
                    .any(|requested| match requested.as_str() {
                        "design_patterns" => pattern_type == "design",
                        "architectural_patterns" => pattern_type == "architectural",
                        "anti_patterns" => pattern_type == "anti",
                        "metaprogramming_patterns" => pattern_type == "metaprogramming",
                        _ => false,
                    })
            } else {
                false
            }
        });
    }

    // Filter by confidence threshold
    detected_patterns.retain(|pattern| {
        pattern
            .get("confidence")
            .and_then(|v| v.as_f64())
            .map(|conf| conf >= confidence_threshold)
            .unwrap_or(false)
    });

    let result = serde_json::json!({
        "analysis": {
            "total_patterns_detected": detected_patterns.len(),
            "confidence_threshold": confidence_threshold,
            "pattern_types_analyzed": pattern_types,
            "include_suggestions": include_suggestions
        },
        "patterns": detected_patterns,
        "summary": {
            "architectural_health": if !detected_patterns.is_empty() { "good" } else { "needs_analysis" },
            "complexity_indicators": {
                "total_nodes": graph_stats.total_nodes,
                "total_edges": graph_stats.total_edges,
                "density": if graph_stats.total_nodes > 0 {
                    graph_stats.total_edges as f64 / graph_stats.total_nodes as f64
                } else { 0.0 }
            }
        },
        "analysis_metadata": {
            "version": "1.0.0-phase1",
            "note": "Pattern detection will be significantly enhanced in Phase 2 with advanced metaprogramming analysis"
        }
    });

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
}
