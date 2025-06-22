//! Workflow orchestration and guidance tools
//!
//! Provides intelligent workflow guidance, batch analysis execution,
//! and systematic analysis orchestration for efficient code analysis.

pub mod batch;
pub mod guidance;
pub mod optimization;

use anyhow::Result;
use serde_json::Value;

use crate::tools::{CallToolResult, Tool};
use crate::CodePrismMcpServer;

/// Register all workflow orchestration tools
pub fn register_workflow_tools() -> Vec<Tool> {
    vec![
        guidance::create_suggest_analysis_workflow_tool(),
        batch::create_batch_analysis_tool(),
        optimization::create_optimize_workflow_tool(),
    ]
}

/// Route workflow tool calls to appropriate handlers
pub async fn handle_workflow_tool(
    tool_name: &str,
    server: &CodePrismMcpServer,
    arguments: Option<&Value>,
) -> Result<CallToolResult> {
    match tool_name {
        "suggest_analysis_workflow" => guidance::suggest_analysis_workflow(server, arguments).await,
        "batch_analysis" => batch::batch_analysis(server, arguments).await,
        "optimize_workflow" => optimization::optimize_workflow(server, arguments).await,
        _ => Err(anyhow::anyhow!("Unknown workflow tool: {}", tool_name)),
    }
}
