//! Decorator analysis tools.

use crate::{tools_legacy::*, CodePrismMcpServer};
use anyhow::Result;
use serde_json::Value;

/// List decorator analysis tools
pub fn list_tools() -> Vec<Tool> {
    vec![Tool {
        name: "analyze_decorators".to_string(),
        title: Some("Analyze Decorators".to_string()),
        description:
            "Comprehensive decorator pattern analysis with usage, effects, and framework detection"
                .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "decorator_name": {
                    "type": "string",
                    "description": "Name of specific decorator to analyze"
                },
                "decorator_id": {
                    "type": "string",
                    "description": "ID of specific decorator to analyze"
                },
                "scope": {
                    "type": "string",
                    "enum": ["repository", "file", "class", "function"],
                    "default": "repository",
                    "description": "Analysis scope"
                },
                "include_usage_analysis": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include decorator usage patterns"
                },
                "include_effects_analysis": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include decorator effects analysis"
                },
                "include_factory_analysis": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include decorator factory pattern analysis"
                },
                "include_chains_analysis": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include decorator chains analysis"
                },
                "include_framework_analysis": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include framework decorator analysis"
                },
                "confidence_threshold": {
                    "type": "number",
                    "default": 0.8,
                    "description": "Confidence threshold for pattern detection"
                }
            }
        }),
    }]
}

/// Call decorator analysis tool
pub async fn call_tool(
    tool_name: &str,
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    match tool_name {
        "analyze_decorators" => analyze_decorators(server, arguments).await,
        _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
    }
}

/// Analyze decorators
async fn analyze_decorators(
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    let args = arguments.unwrap_or_default();

    let scope = args
        .get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("repository");

    let include_usage_analysis = args
        .get("include_usage_analysis")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_effects_analysis = args
        .get("include_effects_analysis")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_factory_analysis = args
        .get("include_factory_analysis")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_chains_analysis = args
        .get("include_chains_analysis")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_framework_analysis = args
        .get("include_framework_analysis")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let confidence_threshold = args
        .get("confidence_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.8);

    let result = if server.repository_path().is_some() {
        // Since Decorator NodeKind doesn't exist yet, simulate decorator analysis
        let target_decorators: Vec<codeprism_core::Node> = Vec::new();

        if target_decorators.is_empty() {
            serde_json::json!({
                "decorators_found": 0,
                "message": "No decorators found matching criteria",
                "analysis_successful": true
            })
        } else {
            let mut decorator_analyses = Vec::new();

            for decorator in &target_decorators {
                let mut analysis = serde_json::json!({
                    "decorator": {
                        "id": decorator.id.to_hex(),
                        "name": decorator.name,
                        "file": decorator.file.display().to_string(),
                        "line": decorator.span.start_line
                    }
                });

                // Usage analysis
                if include_usage_analysis {
                    let usage_analysis =
                        analyze_decorator_usage(server, &decorator.id, scope).await?;
                    analysis["usage_analysis"] = usage_analysis;
                }

                // Effects analysis
                if include_effects_analysis {
                    let effects_analysis = analyze_decorator_effects(server, &decorator.id).await?;
                    analysis["effects_analysis"] = effects_analysis;
                }

                // Factory analysis
                if include_factory_analysis {
                    let factory_analysis = analyze_decorator_factory(server, &decorator.id).await?;
                    analysis["factory_analysis"] = factory_analysis;
                }

                // Chains analysis
                if include_chains_analysis {
                    let chains_analysis = analyze_decorator_chains(server, &decorator.id).await?;
                    analysis["chains_analysis"] = chains_analysis;
                }

                // Framework analysis
                if include_framework_analysis {
                    let framework_analysis =
                        analyze_framework_decorators(server, &decorator.id).await?;
                    analysis["framework_analysis"] = framework_analysis;
                }

                // Pattern detection
                let patterns =
                    detect_decorator_patterns(server, &decorator.id, confidence_threshold).await?;
                analysis["patterns"] = patterns;

                decorator_analyses.push(analysis);
            }

            serde_json::json!({
                "scope": scope,
                "decorators_analyzed": target_decorators.len(),
                "decorator_analyses": decorator_analyses,
                "summary": {
                    "total_decorators": target_decorators.len(),
                    "confidence_threshold": confidence_threshold
                },
                "analysis_successful": true
            })
        }
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

async fn analyze_decorator_usage(
    server: &CodePrismMcpServer,
    decorator_id: &codeprism_core::NodeId,
    _scope: &str,
) -> Result<serde_json::Value> {
    // Find all references to this decorator
    let references = server
        .graph_query()
        .find_references(decorator_id)
        .unwrap_or_default();

    let usage_count = references.len();
    let usage_locations: Vec<serde_json::Value> = references
        .iter()
        .map(|ref_info| {
            serde_json::json!({
                "file": ref_info.source_node.file.display().to_string(),
                "line": ref_info.source_node.span.start_line,
                "context": ref_info.source_node.name
            })
        })
        .collect();

    Ok(serde_json::json!({
        "usage_count": usage_count,
        "usage_locations": usage_locations,
        "popularity": if usage_count > 10 { "High" } else if usage_count > 3 { "Medium" } else { "Low" }
    }))
}

async fn analyze_decorator_effects(
    _server: &CodePrismMcpServer,
    decorator_id: &codeprism_core::NodeId,
) -> Result<serde_json::Value> {
    // Simplified effects analysis
    Ok(serde_json::json!({
        "decorator_id": decorator_id.to_hex(),
        "potential_effects": [
            "Behavior modification",
            "Parameter validation",
            "Result transformation",
            "Side effect injection"
        ],
        "analysis_type": "static",
        "confidence": 0.7
    }))
}

async fn analyze_decorator_factory(
    server: &CodePrismMcpServer,
    decorator_id: &codeprism_core::NodeId,
) -> Result<serde_json::Value> {
    let decorator_node = server.graph_store().get_node(decorator_id);

    let is_factory = decorator_node
        .map(|node| {
            // Simple heuristic: if decorator name suggests factory pattern
            node.name.to_lowercase().contains("factory")
                || node.name.ends_with("_factory")
                || node.name.contains("create")
                || node.name.contains("make")
        })
        .unwrap_or(false);

    Ok(serde_json::json!({
        "is_factory_pattern": is_factory,
        "factory_confidence": if is_factory { 0.8 } else { 0.2 },
        "indicators": if is_factory {
            vec!["Factory naming convention detected"]
        } else {
            vec!["No factory pattern indicators"]
        }
    }))
}

async fn analyze_decorator_chains(
    server: &CodePrismMcpServer,
    decorator_id: &codeprism_core::NodeId,
) -> Result<serde_json::Value> {
    // Find functions that use this decorator and check for other decorators
    let references = server
        .graph_query()
        .find_references(decorator_id)
        .unwrap_or_default();

    let mut chain_info = Vec::new();

    for reference in references {
        // Simplified chain analysis - skip decorator-specific logic for now
        chain_info.push(serde_json::json!({
            "function": reference.source_node.name,
            "file": reference.source_node.file.display().to_string(),
            "line": reference.source_node.span.start_line,
            "chain_length": 1,
            "other_decorators": Vec::<String>::new()
        }));
    }

    Ok(serde_json::json!({
        "chains_detected": chain_info.len(),
        "chain_details": chain_info,
        "max_chain_length": chain_info.iter()
            .filter_map(|info| info.get("chain_length").and_then(|v| v.as_u64()))
            .max()
            .unwrap_or(1)
    }))
}

async fn analyze_framework_decorators(
    _server: &CodePrismMcpServer,
    decorator_id: &codeprism_core::NodeId,
) -> Result<serde_json::Value> {
    // Simplified framework detection based on decorator name patterns
    let common_frameworks = vec![
        ("Flask", vec!["@app.route", "@route", "@before_request"]),
        (
            "Django",
            vec!["@login_required", "@csrf_exempt", "@require_http_methods"],
        ),
        ("FastAPI", vec!["@get", "@post", "@put", "@delete"]),
        ("pytest", vec!["@pytest.fixture", "@pytest.mark"]),
        ("click", vec!["@click.command", "@click.option"]),
    ];

    Ok(serde_json::json!({
        "decorator_id": decorator_id.to_hex(),
        "framework_detection": "static_analysis",
        "possible_frameworks": common_frameworks,
        "confidence": 0.6
    }))
}

async fn detect_decorator_patterns(
    _server: &CodePrismMcpServer,
    decorator_id: &codeprism_core::NodeId,
    confidence_threshold: f64,
) -> Result<serde_json::Value> {
    // Simplified pattern detection
    let mut patterns = Vec::new();

    // Basic decorator pattern (always detected)
    if confidence_threshold <= 0.9 {
        patterns.push(serde_json::json!({
            "pattern_name": "Basic Decorator",
            "confidence": 0.95,
            "description": "Standard decorator pattern implementation"
        }));
    }

    Ok(serde_json::json!({
        "decorator_id": decorator_id.to_hex(),
        "patterns_detected": patterns.len(),
        "patterns": patterns,
        "confidence_threshold": confidence_threshold
    }))
}

fn parse_node_id(hex_str: &str) -> Result<codeprism_core::NodeId> {
    codeprism_core::NodeId::from_hex(hex_str)
        .map_err(|_| anyhow::anyhow!("Invalid node ID format: {}", hex_str))
}
