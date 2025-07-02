//! Debug tool sources for migration analysis

use codeprism_mcp::{tools::ToolManager, CodePrismMcpServer};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = Arc::new(RwLock::new(CodePrismMcpServer::new()?));
    let manager = ToolManager::new(server.clone());

    // Get modular tools
    let mut modular_tools = Vec::new();
    modular_tools.extend(codeprism_mcp::tools::basic::repository::list_tools());
    modular_tools.extend(codeprism_mcp::tools::basic::search::list_tools());
    modular_tools.extend(codeprism_mcp::tools::basic::symbols::list_tools());
    modular_tools.extend(codeprism_mcp::tools::analysis::complexity::list_tools());
    modular_tools.extend(codeprism_mcp::tools::analysis::patterns::list_tools());
    modular_tools.extend(codeprism_mcp::tools::analysis::dependencies::list_tools());
    modular_tools.extend(codeprism_mcp::tools::analysis::flow::list_tools());
    modular_tools.extend(codeprism_mcp::tools::analysis::inheritance::list_tools());
    modular_tools.extend(codeprism_mcp::tools::analysis::decorators::list_tools());
    modular_tools.extend(codeprism_mcp::tools::quality::list_tools());

    // Get legacy tools
    let legacy_manager = codeprism_mcp::tools_legacy::ToolManager::new(server);
    let legacy_result = legacy_manager
        .list_tools(codeprism_mcp::tools::ListToolsParams { cursor: None })
        .await?;

    println!("=== MODULAR TOOLS ({}) ===", modular_tools.len());
    for (i, tool) in modular_tools.iter().enumerate() {
        println!("{}. {} - {}", i + 1, tool.name, tool.description);
    }

    println!("\n=== LEGACY TOOLS ({}) ===", legacy_result.tools.len());
    for (i, tool) in legacy_result.tools.iter().enumerate() {
        println!("{}. {} - {}", i + 1, tool.name, tool.description);
    }

    // Check for duplicates
    let modular_names: std::collections::HashSet<String> =
        modular_tools.iter().map(|t| t.name.clone()).collect();
    let legacy_names: std::collections::HashSet<String> =
        legacy_result.tools.iter().map(|t| t.name.clone()).collect();

    let duplicates: Vec<_> = modular_names.intersection(&legacy_names).collect();

    println!("\n=== DUPLICATES ({}) ===", duplicates.len());
    for dup in &duplicates {
        println!("- {}", dup);
    }

    // Final combined list (how ToolManager does it)
    let combined_result = manager
        .list_tools(codeprism_mcp::tools::ListToolsParams { cursor: None })
        .await?;

    println!(
        "\n=== FINAL COMBINED TOOLS ({}) ===",
        combined_result.tools.len()
    );
    for (i, tool) in combined_result.tools.iter().enumerate() {
        println!("{}. {} - {}", i + 1, tool.name, tool.description);
    }

    Ok(())
}
