//! Data flow analysis tools.
//!
//! This module provides tools for tracing data flow through the codebase,
//! following variable assignments, function parameters, and transformations.

use crate::{tools_legacy::*, CodePrismMcpServer};
use anyhow::Result;
use serde_json::Value;

/// List data flow analysis tools
pub fn list_tools() -> Vec<Tool> {
    Vec::new()
}

/// Handle data flow tool calls
pub async fn call_tool(
    tool_name: &str,
    _server: &CodePrismMcpServer,
    _arguments: Option<Value>,
) -> Result<CallToolResult> {
    Err(anyhow::anyhow!(
        "Data flow tool '{}' not yet implemented in modular architecture in modular architecture. Use legacy tools.",
        tool_name
    ))
}

// FUTURE: Implement trace_data_flow
