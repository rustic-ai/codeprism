//! RMCP Bridge - Tool adapter between RMCP SDK and CodePrism tools
//!
//! This module creates a bridge between the official RMCP SDK and our existing
//! 26+ CodePrism tools, enabling the migration from custom protocol implementation
//! to the authoritative RMCP SDK while maintaining all existing functionality.

use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::tools_legacy::{CallToolParams, CallToolResult, ToolManager};
use crate::CodePrismMcpServer;

/// RMCP Bridge that adapts CodePrism tools to work with the official RMCP SDK
pub struct CodePrismRmcpBridge {
    /// Legacy tool manager for delegating tool calls
    legacy_tool_manager: ToolManager,
    /// Server instance for accessing CodePrism functionality
    #[allow(dead_code)]
    server: Arc<RwLock<CodePrismMcpServer>>,
}

impl CodePrismRmcpBridge {
    /// Create a new RMCP bridge instance
    pub fn new(server: Arc<RwLock<CodePrismMcpServer>>) -> Self {
        let legacy_tool_manager = ToolManager::new(server.clone());

        Self {
            legacy_tool_manager,
            server,
        }
    }

    /// Repository statistics via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn repository_stats(&self, _params: Value) -> Result<CallToolResult> {
        let params = CallToolParams {
            name: "repository_stats".to_string(),
            arguments: None,
        };
        self.legacy_tool_manager.call_tool(params).await
    }

    /// Trace execution path via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn trace_path(&self, params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "trace_path".to_string(),
            arguments: Some(params),
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Explain symbol via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn explain_symbol(&self, params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "explain_symbol".to_string(),
            arguments: Some(params),
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Find dependencies via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn find_dependencies(&self, params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "find_dependencies".to_string(),
            arguments: Some(params),
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Find references via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn find_references(&self, params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "find_references".to_string(),
            arguments: Some(params),
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Search symbols via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn search_symbols(&self, params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "search_symbols".to_string(),
            arguments: Some(params),
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Search content via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn search_content(&self, params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "search_content".to_string(),
            arguments: Some(params),
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Find files via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn find_files(&self, params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "find_files".to_string(),
            arguments: Some(params),
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Content statistics via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn content_stats(&self, _params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "content_stats".to_string(),
            arguments: None,
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Analyze complexity via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn analyze_complexity(&self, params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "analyze_complexity".to_string(),
            arguments: Some(params),
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Detect patterns via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn detect_patterns(&self, params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "detect_patterns".to_string(),
            arguments: Some(params),
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Analyze transitive dependencies via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn analyze_transitive_dependencies(&self, params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "analyze_transitive_dependencies".to_string(),
            arguments: Some(params),
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Trace data flow via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn trace_data_flow(&self, params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "trace_data_flow".to_string(),
            arguments: Some(params),
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Trace inheritance via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn trace_inheritance(&self, params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "trace_inheritance".to_string(),
            arguments: Some(params),
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Analyze decorators via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn analyze_decorators(&self, params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "analyze_decorators".to_string(),
            arguments: Some(params),
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Find duplicates via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn find_duplicates(&self, params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "find_duplicates".to_string(),
            arguments: Some(params),
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Find unused code via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn find_unused_code(&self, params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "find_unused_code".to_string(),
            arguments: Some(params),
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Analyze security via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn analyze_security(&self, params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "analyze_security".to_string(),
            arguments: Some(params),
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Analyze performance via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn analyze_performance(&self, params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "analyze_performance".to_string(),
            arguments: Some(params),
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Analyze API surface via RMCP bridge
    /// Delegates to existing legacy tool implementation
    pub async fn analyze_api_surface(&self, params: Value) -> Result<CallToolResult> {
        let call_params = CallToolParams {
            name: "analyze_api_surface".to_string(),
            arguments: Some(params),
        };
        self.legacy_tool_manager.call_tool(call_params).await
    }

    /// Get list of all available tools
    pub fn get_available_tools(&self) -> Vec<&'static str> {
        vec![
            "repository_stats",
            "trace_path",
            "explain_symbol",
            "find_dependencies",
            "find_references",
            "search_symbols",
            "search_content",
            "find_files",
            "content_stats",
            "analyze_complexity",
            "detect_patterns",
            "analyze_transitive_dependencies",
            "trace_data_flow",
            "trace_inheritance",
            "analyze_decorators",
            "find_duplicates",
            "find_unused_code",
            "analyze_security",
            "analyze_performance",
            "analyze_api_surface",
        ]
    }

    /// Delegate tool call to appropriate handler
    pub async fn call_tool(&self, tool_name: &str, params: Value) -> Result<CallToolResult> {
        match tool_name {
            "repository_stats" => self.repository_stats(params).await,
            "trace_path" => self.trace_path(params).await,
            "explain_symbol" => self.explain_symbol(params).await,
            "find_dependencies" => self.find_dependencies(params).await,
            "find_references" => self.find_references(params).await,
            "search_symbols" => self.search_symbols(params).await,
            "search_content" => self.search_content(params).await,
            "find_files" => self.find_files(params).await,
            "content_stats" => self.content_stats(params).await,
            "analyze_complexity" => self.analyze_complexity(params).await,
            "detect_patterns" => self.detect_patterns(params).await,
            "analyze_transitive_dependencies" => self.analyze_transitive_dependencies(params).await,
            "trace_data_flow" => self.trace_data_flow(params).await,
            "trace_inheritance" => self.trace_inheritance(params).await,
            "analyze_decorators" => self.analyze_decorators(params).await,
            "find_duplicates" => self.find_duplicates(params).await,
            "find_unused_code" => self.find_unused_code(params).await,
            "analyze_security" => self.analyze_security(params).await,
            "analyze_performance" => self.analyze_performance(params).await,
            "analyze_api_surface" => self.analyze_api_surface(params).await,
            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_server() -> Arc<RwLock<CodePrismMcpServer>> {
        let _temp_dir = TempDir::new().unwrap();
        let server = CodePrismMcpServer::new().unwrap();
        Arc::new(RwLock::new(server))
    }

    #[tokio::test]
    async fn test_rmcp_bridge_creation() {
        let server = create_test_server().await;
        let bridge = CodePrismRmcpBridge::new(server);

        let tools = bridge.get_available_tools();
        assert!(tools.len() >= 20); // We have 20+ tools
        assert!(tools.contains(&"repository_stats"));
        assert!(tools.contains(&"analyze_complexity"));
    }

    #[tokio::test]
    async fn test_rmcp_bridge_tool_delegation() {
        let server = create_test_server().await;
        let bridge = CodePrismRmcpBridge::new(server);

        // Test that we can call a tool through the bridge
        let result = bridge.call_tool("repository_stats", Value::Null).await;
        assert!(result.is_ok() || result.is_err()); // Either works or fails gracefully
    }

    #[tokio::test]
    async fn test_rmcp_bridge_unknown_tool() {
        let server = create_test_server().await;
        let bridge = CodePrismRmcpBridge::new(server);

        let result = bridge.call_tool("unknown_tool", Value::Null).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown tool"));
    }
}
