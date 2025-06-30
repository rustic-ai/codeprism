//! Semantic search tools
//!
//! This module will contain semantic search capabilities in Phase 2 of the enhancement plan.
//! Semantic search tools for enhanced code understanding.

use crate::tools_legacy::{CallToolParams, CallToolResult, Tool};
use crate::CodePrismMcpServer;
use anyhow::Result;

/// List semantic search tools (available in Phase 2)
pub fn list_tools() -> Vec<Tool> {
    // Phase 2 will implement semantic_search tool
    vec![]
}

/// Route semantic search tool calls (available in Phase 2)
pub async fn call_tool(
    _server: &CodePrismMcpServer,
    params: &CallToolParams,
) -> Result<CallToolResult> {
    Err(anyhow::anyhow!(
        "Semantic search tool '{}' not yet implemented (Phase 2 feature)",
        params.name
    ))
}
