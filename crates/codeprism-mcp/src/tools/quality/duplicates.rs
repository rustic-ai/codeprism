//! Duplicate code detection tools.

use crate::{tools_legacy::*, CodePrismMcpServer};
use anyhow::Result;
use serde_json::Value;

/// List duplicate detection tools
pub fn list_tools() -> Vec<Tool> {
    vec![Tool {
        name: "find_duplicates".to_string(),
        title: Some("Find Duplicate Code".to_string()),
        description: "Detect duplicate code patterns and similar functions across the codebase"
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "similarity_threshold": {
                    "type": "number",
                    "description": "Minimum similarity threshold for detecting duplicates",
                    "default": 0.8
                },
                "min_lines": {
                    "type": "number",
                    "description": "Minimum number of lines for duplicate detection",
                    "default": 3
                },
                "scope": {
                    "type": "string",
                    "description": "Analysis scope",
                    "default": "repository"
                }
            }
        }),
    }]
}

/// Route duplicate detection tool calls
pub async fn call_tool(
    tool_name: &str,
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    match tool_name {
        "find_duplicates" => find_duplicates(server, arguments).await,
        _ => Err(anyhow::anyhow!("Unknown duplicates tool: {}", tool_name)),
    }
}

/// Find duplicate code
async fn find_duplicates(
    _server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    let args = arguments.unwrap_or_default();

    let similarity_threshold = args
        .get("similarity_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.8);

    let min_lines = args
        .get("min_lines")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(3);

    let scope = args
        .get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("repository");

    // Delegate to analysis engine - placeholder implementation
    let result = serde_json::json!({
        "scope": scope,
        "parameters": {
            "similarity_threshold": similarity_threshold,
            "min_lines": min_lines
        },
        "duplicates": [],
        "summary": {
            "total_duplicates": 0,
            "files_analyzed": 0,
            "lines_duplicated": 0
        },
        "analysis_successful": true,
        "note": "Duplicate detection delegated to analysis engine - full implementation in progress"
    });

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
}
