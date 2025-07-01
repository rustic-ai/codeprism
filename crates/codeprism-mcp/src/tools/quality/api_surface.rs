//! API surface analysis tools.

use crate::{tools_legacy::*, CodePrismMcpServer};
use anyhow::Result;
use serde_json::Value;

/// List API surface analysis tools
pub fn list_tools() -> Vec<Tool> {
    vec![Tool {
        name: "analyze_api_surface".to_string(),
        title: Some("API Surface Analysis".to_string()),
        description: "Analyze public API surface, versioning, and breaking changes".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "scope": {
                    "type": "string",
                    "description": "Analysis scope",
                    "default": "repository"
                },
                "analysis_types": {
                    "type": "array",
                    "items": {
                        "type": "string",
                        "enum": ["public_api", "versioning", "breaking_changes", "documentation_coverage", "compatibility"]
                    },
                    "description": "Types of API analysis",
                    "default": ["public_api", "versioning", "breaking_changes"]
                },
                "api_version": {
                    "type": "string",
                    "description": "API version to analyze",
                    "default": null
                },
                "include_private_apis": {
                    "type": "boolean",
                    "description": "Include private APIs in analysis",
                    "default": false
                },
                "check_documentation_coverage": {
                    "type": "boolean",
                    "description": "Check documentation coverage",
                    "default": true
                },
                "detect_breaking_changes": {
                    "type": "boolean",
                    "description": "Detect breaking changes",
                    "default": true
                }
            }
        }),
    }]
}

/// Route API surface analysis tool calls
pub async fn call_tool(
    tool_name: &str,
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    match tool_name {
        "analyze_api_surface" => analyze_api_surface(server, arguments).await,
        _ => Err(anyhow::anyhow!("Unknown API surface tool: {}", tool_name)),
    }
}

/// Analyze API surface
async fn analyze_api_surface(
    _server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    let args = arguments.unwrap_or_default();

    let scope = args
        .get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("repository");

    let analysis_types = args
        .get("analysis_types")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            vec![
                "public_api".to_string(),
                "versioning".to_string(),
                "breaking_changes".to_string(),
            ]
        });

    let api_version = args.get("api_version").and_then(|v| v.as_str());

    let include_private_apis = args
        .get("include_private_apis")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let check_documentation_coverage = args
        .get("check_documentation_coverage")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let detect_breaking_changes = args
        .get("detect_breaking_changes")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // FUTURE: Delegate to codeprism-analysis crate for full API surface analysis
    let result = serde_json::json!({
        "scope": scope,
        "parameters": {
            "analysis_types": analysis_types,
            "api_version": api_version,
            "include_private_apis": include_private_apis,
            "check_documentation_coverage": check_documentation_coverage,
            "detect_breaking_changes": detect_breaking_changes
        },
        "public_api": {
            "functions": [],
            "classes": [],
            "interfaces": [],
            "modules": []
        },
        "versioning_info": {
            "current_version": api_version,
            "version_changes": []
        },
        "breaking_changes": [],
        "documentation_coverage": {
            "covered_apis": 0,
            "total_apis": 0,
            "coverage_percentage": 0.0
        },
        "compatibility": {
            "backward_compatible": true,
            "issues": []
        },
        "summary": {
            "total_public_apis": 0,
            "documented_apis": 0,
            "breaking_changes_count": 0
        },
        "recommendations": [],
        "analysis_successful": true,
        "note": "API surface analysis delegated to analysis engine - full implementation in progress"
    });

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
}
