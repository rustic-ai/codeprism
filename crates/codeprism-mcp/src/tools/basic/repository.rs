//! Repository information and statistics tools.
//!
//! This module provides tools for getting information about the repository,
//! including file counts, graph statistics, and content indexing status.

use crate::{tools_legacy::*, CodePrismMcpServer};
use anyhow::Result;
use serde_json::Value;

/// Repository statistics tool definition
///
/// Returns comprehensive statistics about the repository including file counts,
/// graph nodes/edges, and indexing status.
pub fn repository_stats_tool() -> Tool {
    Tool {
        name: "repository_stats".to_string(),
        title: Some("Repository Statistics".to_string()),
        description: "Get comprehensive statistics about the repository".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }),
    }
}

/// Content statistics tool definition
///
/// Returns statistics about indexed content including files, chunks, tokens,
/// and content type distribution.
pub fn content_stats_tool() -> Tool {
    Tool {
        name: "content_stats".to_string(),
        title: Some("Content Statistics".to_string()),
        description: "Get statistics about indexed content".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }),
    }
}

/// Get repository statistics
///
/// Provides comprehensive information about the repository including:
/// - Repository path
/// - Total files discovered
/// - Graph statistics (nodes, edges, node types)
/// - Repository status
///
/// # Examples
///
/// ```
/// # use codeprism_mcp::tools::basic::repository::repository_stats;
/// # use codeprism_mcp::CodePrismMcpServer;
/// # tokio_test::block_on(async {
/// # let server = CodePrismMcpServer::new()?;
/// let result = repository_stats(&server).await?;
/// assert!(!result.content.is_empty());
/// # Ok::<(), anyhow::Error>(())
/// # });
/// ```
pub async fn repository_stats(server: &CodePrismMcpServer) -> Result<CallToolResult> {
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
            "error": "No repository initialized",
            "status": "not_initialized"
        })
    };

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
}

/// Get content indexing statistics  
///
/// Provides information about content indexing including:
/// - Total files indexed
/// - Content chunks and tokens
/// - Content distribution by type
/// - Size distribution
/// - Indexing status and timestamps
///
/// # Examples
///
/// ```
/// # use codeprism_mcp::tools::basic::repository::content_stats;
/// # use codeprism_mcp::CodePrismMcpServer;
/// # tokio_test::block_on(async {
/// # let server = CodePrismMcpServer::new()?;
/// let result = content_stats(&server).await?;
/// assert!(!result.content.is_empty());
/// # Ok::<(), anyhow::Error>(())
/// # });
/// ```
pub async fn content_stats(server: &CodePrismMcpServer) -> Result<CallToolResult> {
    let stats = server.content_search().get_stats();

    let result = if stats.total_files == 0 {
        serde_json::json!({
            "total_files": 0,
            "total_chunks": 0,
            "total_tokens": 0,
            "content_by_type": {},
            "size_distribution": {},
            "status": "no_content_indexed",
            "message": "Content indexing has not been performed yet. Only code symbol analysis is available.",
            "suggestion": "Content indexing for documentation, configuration files, and comments may still be in progress."
        })
    } else {
        serde_json::json!({
            "total_files": stats.total_files,
            "total_chunks": stats.total_chunks,
            "total_tokens": stats.total_tokens,
            "content_by_type": stats.content_by_type,
            "size_distribution": stats.size_distribution,
            "computed_at": stats.computed_at.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "status": "indexed"
        })
    };

    Ok(CallToolResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: Some(false),
    })
}

/// Handle repository tool calls
///
/// Routes repository tool calls to the appropriate function based on tool name.
///
/// # Arguments
///
/// * `tool_name` - Name of the tool to call
/// * `server` - CodePrism server instance
/// * `_arguments` - Tool arguments (not used for repository tools)
///
/// # Returns
///
/// Returns the result of the tool call or an error for unknown tools.
pub async fn call_tool(
    tool_name: &str,
    server: &CodePrismMcpServer,
    _arguments: Option<Value>,
) -> Result<CallToolResult> {
    match tool_name {
        "repository_stats" => repository_stats(server).await,
        "content_stats" => content_stats(server).await,
        _ => Err(anyhow::anyhow!("Unknown repository tool: {}", tool_name)),
    }
}

/// List all repository tools
///
/// Returns a list of all available repository tools.
pub fn list_tools() -> Vec<Tool> {
    vec![repository_stats_tool(), content_stats_tool()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_repository_stats_tool_definition() {
        let tool = repository_stats_tool();
        assert_eq!(tool.name, "repository_stats");
        assert!(tool.title.is_some());
        assert!(!tool.description.is_empty());
    }

    #[tokio::test]
    async fn test_content_stats_tool_definition() {
        let tool = content_stats_tool();
        assert_eq!(tool.name, "content_stats");
        assert!(tool.title.is_some());
        assert!(!tool.description.is_empty());
    }

    #[tokio::test]
    async fn test_list_tools_returns_repository_tools() {
        let tools = list_tools();
        assert_eq!(tools.len(), 2);

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"repository_stats"));
        assert!(tool_names.contains(&"content_stats"));
    }

    #[tokio::test]
    async fn test_call_tool_handles_repository_stats() {
        let server = CodePrismMcpServer::new().unwrap();
        let result = call_tool("repository_stats", &server, None).await.unwrap();

        // Should return a result (even if repository is not initialized)
        assert!(!result.content.is_empty());
        assert_eq!(result.is_error, Some(false));
    }

    #[tokio::test]
    async fn test_call_tool_handles_content_stats() {
        let server = CodePrismMcpServer::new().unwrap();
        let result = call_tool("content_stats", &server, None).await.unwrap();

        // Should return a result (even if no content is indexed)
        assert!(!result.content.is_empty());
        assert_eq!(result.is_error, Some(false));
    }

    #[tokio::test]
    async fn test_call_tool_rejects_unknown_tool() {
        let server = CodePrismMcpServer::new().unwrap();
        let result = call_tool("unknown_tool", &server, None).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown repository tool"));
    }
}
