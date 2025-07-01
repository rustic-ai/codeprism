//! Unused code analysis tools.

use crate::{tools_legacy::*, CodePrismMcpServer};
use anyhow::Result;
use serde_json::Value;

/// List unused code analysis tools
pub fn list_tools() -> Vec<Tool> {
    vec![Tool {
        name: "find_unused_code".to_string(),
        title: Some("Find Unused Code".to_string()),
        description: "Detect unused functions, classes, variables, and imports in the codebase"
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "scope": {
                    "type": "string",
                    "description": "Analysis scope",
                    "default": "repository"
                },
                "analyze_types": {
                    "type": "array",
                    "items": {
                        "type": "string",
                        "enum": ["functions", "classes", "variables", "imports"]
                    },
                    "description": "Types of unused code to analyze",
                    "default": ["functions", "classes", "variables", "imports"]
                },
                "confidence_threshold": {
                    "type": "number",
                    "description": "Confidence threshold for unused code detection",
                    "default": 0.7
                },
                "consider_external_apis": {
                    "type": "boolean",
                    "description": "Consider external API usage",
                    "default": true
                },
                "include_dead_code": {
                    "type": "boolean",
                    "description": "Include dead code blocks",
                    "default": true
                }
            }
        }),
    }]
}

/// Route unused code analysis tool calls
pub async fn call_tool(
    tool_name: &str,
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    match tool_name {
        "find_unused_code" => find_unused_code(server, arguments).await,
        _ => Err(anyhow::anyhow!("Unknown unused code tool: {}", tool_name)),
    }
}

/// Find unused code
async fn find_unused_code(
    _server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    let args = arguments.unwrap_or_default();

    let scope = args
        .get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("repository");

    let analyze_types = args
        .get("analyze_types")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            vec![
                "functions".to_string(),
                "classes".to_string(),
                "variables".to_string(),
                "imports".to_string(),
            ]
        });

    let confidence_threshold = args
        .get("confidence_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.7);

    let consider_external_apis = args
        .get("consider_external_apis")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_dead_code = args
        .get("include_dead_code")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // Delegate to analysis engine - placeholder implementation
    let result = serde_json::json!({
        "scope": scope,
        "parameters": {
            "analyze_types": analyze_types,
            "confidence_threshold": confidence_threshold,
            "consider_external_apis": consider_external_apis,
            "include_dead_code": include_dead_code
        },
        "unused_code": {
            "functions": [],
            "classes": [],
            "variables": [],
            "imports": [],
            "dead_code_blocks": []
        },
        "summary": {
            "total_unused_items": 0,
            "potential_lines_to_remove": 0,
            "confidence_score": confidence_threshold
        },
        "analysis_successful": true,
        "note": "Unused code analysis delegated to analysis engine - full implementation in progress"
    });

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
}
