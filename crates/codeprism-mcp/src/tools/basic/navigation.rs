//! Code navigation and path tracing tools.
//!
//! This module provides tools for navigating code relationships,
//! tracing paths between symbols, and understanding code flow.

use crate::{tools_legacy::*, CodePrismMcpServer};
use anyhow::Result;
use serde_json::Value;

/// List navigation tools
pub fn list_tools() -> Vec<Tool> {
    // FUTURE: Implement modular navigation tools
    // PLANNED: return empty list as these tools are still in legacy
    Vec::new()
}

/// Handle navigation tool calls
pub async fn call_tool(
    tool_name: &str,
    _server: &CodePrismMcpServer,
    _arguments: Option<Value>,
) -> Result<CallToolResult> {
    // FUTURE: Implement modular navigation tool routing
    Err(anyhow::anyhow!(
        "Navigation tool '{}' not yet implemented in modular architecture in modular architecture. Use legacy tools.",
        tool_name
    ))
}

// FUTURE: Implement these navigation tools:
// - trace_path
// Each should be extracted from tools_legacy.rs and modernized
