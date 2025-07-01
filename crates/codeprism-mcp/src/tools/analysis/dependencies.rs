//! Dependency analysis tools.
//!
//! This module provides tools for analyzing transitive dependencies,
//! detecting cycles, and mapping dependency relationships.

use crate::{tools_legacy::*, CodePrismMcpServer};
use anyhow::Result;
use serde_json::Value;

/// List dependency analysis tools
pub fn list_tools() -> Vec<Tool> {
    // FUTURE: Implement modular dependency tools
    Vec::new()
}

/// Handle dependency tool calls
pub async fn call_tool(
    tool_name: &str,
    _server: &CodePrismMcpServer,
    _arguments: Option<Value>,
) -> Result<CallToolResult> {
    // FUTURE: Implement modular dependency tool routing
    Err(anyhow::anyhow!(
        "Dependency tool '{}' not yet implemented in modular architecture in modular architecture. Use legacy tools.",
        tool_name
    ))
}

// FUTURE: Implement these dependency tools:
// - analyze_transitive_dependencies
