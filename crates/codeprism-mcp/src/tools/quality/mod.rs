//! Code quality analysis tools.
//!
//! This module contains tools for analyzing code quality, finding duplicates,
//! unused code, security issues, performance problems, and API surface analysis.

pub mod api_surface;
pub mod duplicates;
pub mod performance;
pub mod security;
pub mod unused;

// Re-export specific functions to avoid naming conflicts
use crate::{tools_legacy::*, CodePrismMcpServer};
use anyhow::Result;
use serde_json::Value;

/// List all quality analysis tools
pub fn list_tools() -> Vec<Tool> {
    let mut tools = Vec::new();
    tools.extend(duplicates::list_tools());
    tools.extend(unused::list_tools());
    tools.extend(security::list_tools());
    tools.extend(performance::list_tools());
    tools.extend(api_surface::list_tools());
    tools
}

/// Route quality tool calls to appropriate modules
pub async fn call_tool(
    tool_name: &str,
    server: &CodePrismMcpServer,
    arguments: Option<Value>,
) -> Result<CallToolResult> {
    match tool_name {
        "find_duplicates" => duplicates::call_tool(tool_name, server, arguments).await,
        "find_unused_code" => unused::call_tool(tool_name, server, arguments).await,
        "analyze_security" => security::call_tool(tool_name, server, arguments).await,
        "analyze_performance" => performance::call_tool(tool_name, server, arguments).await,
        "analyze_api_surface" => api_surface::call_tool(tool_name, server, arguments).await,
        _ => Err(anyhow::anyhow!("Unknown quality tool: {}", tool_name)),
    }
}
