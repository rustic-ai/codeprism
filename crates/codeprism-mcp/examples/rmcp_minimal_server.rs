//! Native RMCP Server Example
//!
//! This demonstrates the native RMCP SDK implementation with CodePrism tools.
//! Shows how to use the official RMCP server with stdio transport.

use anyhow::Result;
use codeprism_mcp::server::CodePrismRmcpServer;
use rmcp::ServiceExt;
use tracing_subscriber::fmt::init;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    init();

    println!("🚀 Starting native RMCP server example...");

    // Create native RMCP server instance
    let mut server = CodePrismRmcpServer::new()?;

    // Initialize with current directory as repository
    let current_dir = std::env::current_dir()?;
    server.initialize_with_repository(&current_dir).await?;

    println!("📋 Native RMCP server created with tools:");
    println!("  - repository_stats: Get comprehensive repository statistics");
    println!("  - content_stats: Get detailed content statistics");
    println!("  - analyze_complexity: Analyze code complexity metrics");

    println!("\n🔧 Starting server with stdio transport...");
    println!("ℹ️  Use Ctrl+C to stop the server");

    // Start the server with stdio transport (like the main binary)
    let service = server
        .serve((tokio::io::stdin(), tokio::io::stdout()))
        .await?;

    service.waiting().await?;

    Ok(())
}
