//! Repository statistics and information tools

use crate::tools_legacy::{CallToolParams, CallToolResult, Tool, ToolContent};
use crate::PrismMcpServer;
use anyhow::Result;
use serde_json::Value;

/// List repository tools
pub fn list_tools() -> Vec<Tool> {
    vec![Tool {
        name: "repository_stats".to_string(),
        title: Some("Repository Statistics".to_string()),
        description: "Get comprehensive statistics about the repository".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {}
        }),
    }]
}

/// Route repository tool calls
pub async fn call_tool(server: &PrismMcpServer, params: &CallToolParams) -> Result<CallToolResult> {
    match params.name.as_str() {
        "repository_stats" => repository_stats(server).await,
        _ => Err(anyhow::anyhow!("Unknown repository tool: {}", params.name)),
    }
}

/// Get repository statistics
async fn repository_stats(server: &PrismMcpServer) -> Result<CallToolResult> {
    let result = if let Some(repo_path) = server.repository_path() {
        let file_count = server
            .scanner()
            .discover_files(repo_path)
            .map(|files| files.len())
            .unwrap_or(0);

        let graph_stats = server.graph_store().get_stats();

        serde_json::json!({
            "repository_path": repo_path.display().to_string(),
            "total_files": file_count,
            "total_nodes": graph_stats.total_nodes,
            "total_edges": graph_stats.total_edges,
            "nodes_by_kind": graph_stats.nodes_by_kind,
            "status": "active"
        })
    } else {
        serde_json::json!({
            "error": "No repository initialized"
        })
    };

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
}
