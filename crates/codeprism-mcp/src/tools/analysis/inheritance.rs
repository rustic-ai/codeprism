//! Inheritance analysis tools.

use crate::{tools_legacy::*, CodePrismMcpServer};
use anyhow::Result;
use serde_json::Value;

pub fn list_tools() -> Vec<Tool> {
    Vec::new()
}

pub async fn call_tool(
    tool_name: &str,
    _server: &CodePrismMcpServer,
    _arguments: Option<Value>,
) -> Result<CallToolResult> {
    Err(anyhow::anyhow!(
        "Inheritance tool '{}' not yet implemented in modular architecture.",
        tool_name
    ))
}

// FUTURE: Implement trace_inheritance
