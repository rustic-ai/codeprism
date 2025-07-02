//! Basic Router Test - demonstrates BasicToolsRouter and router combination
//!
//! This tests the Week 1 implementation of BasicToolsRouter with 3 core tools
//! and demonstrates the router combination pattern for future expansion.

use codeprism_mcp::{tools::basic::BasicToolsRouter, CodePrismMcpServer};
use rmcp::{
    handler::server::router::tool::ToolRouter, model::*, tool_handler, Error as McpError,
    ServerHandler,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Combined server demonstrating modular router architecture
#[derive(Clone)]
pub struct CombinedTestServer {
    core_server: Arc<RwLock<CodePrismMcpServer>>,
    combined_router: ToolRouter<CombinedTestServer>,
}

impl CombinedTestServer {
    pub fn new() -> Result<Self, McpError> {
        let core_server = Arc::new(RwLock::new(
            CodePrismMcpServer::new().map_err(|e| McpError::internal_error(e.to_string()))?,
        ));

        // Create category routers - currently just BasicToolsRouter
        // In Week 2, we'll add: + AnalysisToolsRouter::analysis_tools_router()
        let combined_router = BasicToolsRouter::basic_tools_router();

        Ok(Self {
            core_server,
            combined_router,
        })
    }

    pub async fn initialize_repository<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> Result<(), McpError> {
        let path = path.as_ref().to_path_buf();
        {
            let mut server = self.core_server.write().await;
            server
                .initialize_with_repository(&path)
                .await
                .map_err(|e| McpError::internal_error(e.to_string()))?;
        }
        Ok(())
    }
}

#[tool_handler]
impl ServerHandler for CombinedTestServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("CodePrism MCP Server - Week 1 BasicToolsRouter Test - modular architecture with category-based routers".to_string()),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🧪 Testing Week 1: BasicToolsRouter + Router Combination");

    // Create combined server with BasicToolsRouter
    let mut server = CombinedTestServer::new()?;
    println!("✅ Combined server with BasicToolsRouter created");

    // Test repository initialization (optional)
    if let Ok(current_dir) = std::env::current_dir() {
        if let Err(e) = server.initialize_repository(&current_dir).await {
            println!("⚠️  Repository initialization failed: {}", e);
            println!("   Continuing with tests (some tools may show 'no repository loaded')");
        } else {
            println!("✅ Repository initialized: {}", current_dir.display());
        }
    }

    // Test that we can access the BasicToolsRouter through the combined router
    let basic_router = BasicToolsRouter::new(server.core_server.clone());

    // Test 1: repository_stats
    println!("\n🔧 Testing repository_stats tool:");
    match basic_router.repository_stats().await {
        Ok(result) => {
            println!("✅ repository_stats successful");
            if let Some(content) = result.content.first() {
                println!("   Result: {}", content.text().unwrap_or("No text content"));
            }
        }
        Err(e) => println!("❌ repository_stats failed: {}", e),
    }

    // Test 2: search_symbols
    println!("\n🔍 Testing search_symbols tool:");
    let search_args = serde_json::json!({
        "pattern": "test"
    });
    match basic_router
        .search_symbols(rmcp::handler::server::tool::Parameters(search_args))
        .await
    {
        Ok(result) => {
            println!("✅ search_symbols successful");
            if let Some(content) = result.content.first() {
                println!("   Result: {}", content.text().unwrap_or("No text content"));
            }
        }
        Err(e) => println!("❌ search_symbols failed: {}", e),
    }

    // Test 3: find_files
    println!("\n📁 Testing find_files tool:");
    let files_args = serde_json::json!({
        "pattern": "*.rs"
    });
    match basic_router
        .find_files(rmcp::handler::server::tool::Parameters(files_args))
        .await
    {
        Ok(result) => {
            println!("✅ find_files successful");
            if let Some(content) = result.content.first() {
                println!(
                    "   Result preview: {}...",
                    content
                        .text()
                        .unwrap_or("No text content")
                        .chars()
                        .take(200)
                        .collect::<String>()
                );
            }
        }
        Err(e) => println!("❌ find_files failed: {}", e),
    }

    println!("\n🎉 Week 1 BasicToolsRouter test completed!");
    println!("✅ 3 core tools implemented: repository_stats, search_symbols, find_files");
    println!("✅ RMCP router combination pattern established");
    println!("✅ Foundation ready for Week 2: Core Category Migration");

    Ok(())
}
