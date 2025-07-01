//! Minimal RMCP Server Example
//!
//! This demonstrates the RMCP SDK integration with stdio transport
//! and the CodePrism tool bridge for Phase 1 of the migration.

use anyhow::Result;
use codeprism_mcp::{CodePrismMcpServer, CodePrismRmcpBridge};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber::fmt::init;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    init();

    println!("🚀 Starting minimal RMCP server example...");

    // Create CodePrism MCP server instance
    let codeprism_server = CodePrismMcpServer::new()?;
    let server_arc = Arc::new(RwLock::new(codeprism_server));

    // Create RMCP bridge
    let bridge = CodePrismRmcpBridge::new(server_arc.clone());

    println!("📋 Available tools through RMCP bridge:");
    for tool in bridge.get_available_tools() {
        println!("  - {}", tool);
    }

    // Test a simple tool call through the bridge
    println!("\n🔧 Testing repository_stats tool...");
    match bridge.call_tool("repository_stats", Value::Null).await {
        Ok(result) => {
            println!("✅ Tool call successful!");
            println!("📊 Result: {}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            println!("⚠️  Tool call failed: {}", e);
            println!("ℹ️  This is expected if no repository is loaded");
        }
    }

    // FUTURE(Phase2): Integrate with actual RMCP SDK Server when available
    // This foundation is ready for Phase 2 custom code elimination
    println!("\n📝 Next steps:");
    println!("  1. ✅ RMCP dependency added");
    println!("  2. ✅ Tool adapter bridge created");
    println!("  3. ⏳ Integrate with RMCP Server (Phase 2)");
    println!("  4. ⏳ Test stdio transport through RMCP");
    println!("  5. ⏳ Performance benchmark comparison");

    println!("\n🎯 Phase 1 foundation ready for RMCP Server integration!");

    Ok(())
}
