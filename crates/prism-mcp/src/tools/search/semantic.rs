//! Semantic search tools
//!
//! This module will contain semantic search capabilities in Phase 2 of the enhancement plan.
//! Currently a placeholder for future implementation.

use crate::tools_legacy::{CallToolParams, CallToolResult, Tool};
use crate::PrismMcpServer;
use anyhow::Result;

/// List semantic search tools (placeholder for Phase 2)
pub fn list_tools() -> Vec<Tool> {
    // Phase 2 will implement semantic_search tool
    vec![]
}

/// Route semantic search tool calls (placeholder for Phase 2)
pub async fn call_tool(
    _server: &PrismMcpServer,
    params: &CallToolParams,
) -> Result<CallToolResult> {
    Err(anyhow::anyhow!(
        "Semantic search tool '{}' not yet implemented (Phase 2 feature)",
        params.name
    ))
}
