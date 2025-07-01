//! Symbol analysis and explanation tools.
//!
//! This module provides tools for analyzing code symbols, finding references,
//! dependencies, and providing detailed explanations.

use crate::{tools_legacy::*, CodePrismMcpServer};
use anyhow::Result;
use serde_json::Value;

/// List symbol tools
pub fn list_tools() -> Vec<Tool> {
    // FUTURE: Implement modular symbol tools
    // PLANNED: return empty list as these tools are still in legacy
    Vec::new()
}

/// Handle symbol tool calls
pub async fn call_tool(
    tool_name: &str,
    _server: &CodePrismMcpServer,
    _arguments: Option<Value>,
) -> Result<CallToolResult> {
    // FUTURE: Implement modular symbol tool routing
    Err(anyhow::anyhow!(
        "Symbol tool '{}' not yet implemented in modular architecture in modular architecture. Use legacy tools.",
        tool_name
    ))
}

// FUTURE: Implement these symbol tools:
// - explain_symbol
// - find_references
// - find_dependencies
// Each should be extracted from tools_legacy.rs and modernized
