//! Integration tests for tool router functionality

#[cfg(test)]
mod tests {
    use crate::{CodePrismMcpServer, Config};

    #[tokio::test]
    async fn test_server_with_tools_creation() {
        // Test that we can create a server with tool router
        let config = Config::default();
        let server = CodePrismMcpServer::new(config).await;
        assert!(
            server.is_ok(),
            "Server with tools should be created successfully"
        );

        let server = server.unwrap();
        // Server created successfully with tool router

        // Verify server info includes tool capabilities
        use rmcp::ServerHandler;
        let info = server.get_info();
        assert!(
            info.capabilities.tools.is_some(),
            "Server should have tool capabilities"
        );
    }

    #[test]
    fn test_tool_router_compilation() {
        // This test primarily verifies that the #[tool_router] macro compiles correctly
        // and the tools are properly registered at compile time
        // The fact that this test compiles and runs means the tool router macro worked

        // Verify that the CodePrismMcpServer type exists and has the expected structure
        use std::mem;
        let size = mem::size_of::<crate::CodePrismMcpServer>();
        assert!(size > 0, "CodePrismMcpServer should have non-zero size");
    }
}
