//! Search and discovery tools.
//!
//! This module provides tools for searching content, finding files,
//! and discovering code elements in the repository.

use crate::{tools_legacy::*, CodePrismMcpServer};
use anyhow::Result;
use serde_json::Value;

/// List search tools
///
/// Returns a list of search-related tools. Currently delegating to legacy
/// implementation while gradual migration is in progress.
pub fn list_tools() -> Vec<Tool> {
    // FUTURE: Implement modular search tools (search_symbols, search_content, find_files)
    // PLANNED: Gradual migration from legacy tools_legacy.rs implementation
    Vec::new()
}

/// Handle search tool calls
///
/// Routes search tool calls to appropriate functions.
/// Currently delegating to legacy implementation.
pub async fn call_tool(
    tool_name: &str,
    _server: &CodePrismMcpServer,
    _arguments: Option<Value>,
) -> Result<CallToolResult> {
    // PLANNED: Implement modular search tool routing in next migration phase
    Err(anyhow::anyhow!(
        "Search tool '{}' not yet implemented in modular architecture in modular architecture. Use legacy tools.",
        tool_name
    ))
}

// FUTURE: Implement these search tools:
// - search_content: Search file contents with regex support
// - search_symbols: Find symbols matching patterns
// - find_files: Locate files by name or pattern
// NOTE: Each should be extracted from tools_legacy.rs and modernized
