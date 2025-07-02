//! List all available tools for migration analysis

use codeprism_mcp::{tools::ToolManager, CodePrismMcpServer};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = Arc::new(RwLock::new(CodePrismMcpServer::new()?));
    let manager = ToolManager::new(server);

    let tools_result = manager
        .list_tools(codeprism_mcp::tools::ListToolsParams { cursor: None })
        .await?;

    println!("Total tools available: {}", tools_result.tools.len());
    println!("\nAll available tools:");

    for (i, tool) in tools_result.tools.iter().enumerate() {
        println!("{}. {} - {}", i + 1, tool.name, tool.description);
    }

    Ok(())
}
