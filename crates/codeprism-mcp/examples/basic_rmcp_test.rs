//! Basic RMCP Test Example
//!
//! This tests that our RMCP SDK dependency works correctly
//! and demonstrates the patterns we'll use in the migration.

use rmcp::{
    handler::server::router::tool::ToolRouter, model::*, schemars, service::RequestContext, tool,
    tool_handler, tool_router, Error as McpError, RoleServer, ServerHandler,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct TestServer {
    data: Arc<RwLock<String>>,
    tool_router: ToolRouter<TestServer>,
}

#[tool_router]
impl TestServer {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new("Hello from RMCP!".to_string())),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Get test data to verify RMCP is working")]
    pub async fn get_test_data(&self) -> Result<CallToolResult, McpError> {
        let data = self.data.read().await;
        Ok(CallToolResult::success(vec![Content::text(data.clone())]))
    }

    #[tool(description = "Set test data with a new value")]
    pub async fn set_test_data(
        &self,
        #[tool(param)]
        #[schemars(description = "New value to set")]
        value: String,
    ) -> Result<CallToolResult, McpError> {
        let mut data = self.data.write().await;
        *data = value.clone();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Set data to: {}",
            value
        ))]))
    }
}

#[tool_handler]
impl ServerHandler for TestServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Test server to verify RMCP SDK integration".to_string()),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🧪 Testing RMCP SDK integration...");

    let server = TestServer::new();
    println!("✅ RMCP server created successfully");

    // Test tool execution
    let result = server.get_test_data().await?;
    println!("✅ Tool execution successful: {:?}", result.is_success());

    let result = server.set_test_data("RMCP is working!".to_string()).await?;
    println!(
        "✅ Parameterized tool execution successful: {:?}",
        result.is_success()
    );

    println!("🎉 RMCP SDK integration test completed successfully!");

    Ok(())
}
