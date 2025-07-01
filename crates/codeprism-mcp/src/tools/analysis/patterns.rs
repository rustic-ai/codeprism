//! Design pattern detection tools.
//!
//! This module provides tools for detecting design patterns, anti-patterns,
//! architectural patterns, and metaprogramming patterns.

use crate::{tools_legacy::*, CodePrismMcpServer};
use anyhow::Result;
use serde_json::Value;

/// List pattern detection tools
pub fn list_tools() -> Vec<Tool> {
    // FUTURE: Implement modular pattern tools
    Vec::new()
}

/// Handle pattern tool calls
pub async fn call_tool(
    tool_name: &str,
    _server: &CodePrismMcpServer,
    _arguments: Option<Value>,
) -> Result<CallToolResult> {
    // FUTURE: Implement modular pattern tool routing
    Err(anyhow::anyhow!(
        "Pattern tool '{}' not yet implemented in modular architecture in modular architecture. Use legacy tools.",
        tool_name
    ))
}

// FUTURE: Implement these pattern tools:
// - detect_patterns
