//! API surface analysis tools

use anyhow::Result;
use serde_json::Value;
use crate::tools_legacy::{Tool, CallToolParams, CallToolResult, ToolContent};
use crate::PrismMcpServer;

/// List API surface analysis tools
pub fn list_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "analyze_api_surface".to_string(),
            title: Some("Analyze API Surface".to_string()),
            description: "Comprehensive API surface analysis including public interface mapping, versioning compatibility, and documentation coverage".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "scope": {
                        "type": "string",
                        "description": "Analysis scope (repository, package, or module)",
                        "default": "repository"
                    },
                    "analysis_types": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["public_api", "versioning", "breaking_changes", "documentation_coverage", "compatibility", "all"]
                        },
                        "description": "Types of API analysis to conduct",
                        "default": ["all"]
                    },
                    "api_version": {
                        "type": "string",
                        "description": "API version to analyze (optional)"
                    },
                    "include_private_apis": {
                        "type": "boolean",
                        "description": "Include private/internal APIs in analysis",
                        "default": false
                    },
                    "check_documentation_coverage": {
                        "type": "boolean",
                        "description": "Check documentation coverage for public APIs",
                        "default": true
                    },
                    "detect_breaking_changes": {
                        "type": "boolean",
                        "description": "Detect potential breaking changes",
                        "default": true
                    },
                    "exclude_patterns": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "File patterns to exclude from analysis"
                    }
                },
                "required": []
            }),
        }
    ]
}

/// Route API surface analysis tool calls
pub async fn call_tool(server: &PrismMcpServer, params: &CallToolParams) -> Result<CallToolResult> {
    match params.name.as_str() {
        "analyze_api_surface" => analyze_api_surface(server, params.arguments.as_ref()).await,
        _ => Err(anyhow::anyhow!("Unknown API surface analysis tool: {}", params.name)),
    }
}

/// Analyze API surface (placeholder implementation)
async fn analyze_api_surface(_server: &PrismMcpServer, _arguments: Option<&Value>) -> Result<CallToolResult> {
    let result = serde_json::json!({
        "analysis": {
            "status": "phase1_implementation",
            "message": "Full API surface analysis implementation will be completed in Phase 1 continuation"
        },
        "api_surface": {
            "public_apis": [],
            "private_apis": [],
            "versioning_info": {},
            "documentation_coverage": {}
        },
        "issues": [],
        "recommendations": [],
        "note": "This tool is being modularized as part of Phase 1 enhancement. Full implementation coming soon."
    });

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
} 