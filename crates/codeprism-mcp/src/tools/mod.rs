//! Modular MCP Tools Implementation
//!
//! This module contains a modern, modular implementation of CodePrism MCP tools.
//! The tools are organized into logical groups for better maintainability and testing.
//!
//! # Architecture
//!
//! ```text
//! tools/
//! ├── types.rs          # Core MCP types and shared structures
//! ├── basic/            # Basic repository and search tools  
//! ├── analysis/         # Advanced code analysis tools
//! ├── quality/          # Code quality and security tools
//! └── legacy.rs         # Backward compatibility wrapper
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use codeprism_mcp::tools::{ToolManager, CallToolParams};
//! use std::sync::Arc;
//! use tokio::sync::RwLock;
//! use serde_json::json;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let server = Arc::new(RwLock::new(codeprism_mcp::CodePrismMcpServer::new()?));
//! let manager = ToolManager::new(server);
//! let result = manager.call_tool(CallToolParams {
//!     name: "search_symbols".to_string(),
//!     arguments: Some(json!({"pattern": "User"})),
//! }).await?;
//! # Ok(())
//! # }
//! ```

pub mod analysis;
pub mod basic;
pub mod dynamic_enablement;
pub mod quality;
pub mod types;

// Re-export core types for convenience - using legacy types for compatibility
pub use crate::tools_legacy::{
    CallToolParams, CallToolResult, ListToolsParams, ListToolsResult, Tool, ToolCapabilities,
    ToolContent, ToolManager as LegacyToolManager,
};

// Export ToolManager as ToolRegistry for backward compatibility
pub use ToolManager as ToolRegistry;

// Also re-export the new types with different names for gradual migration
pub use types::{
    CallToolParams as NewCallToolParams, CallToolResult as NewCallToolResult,
    ListToolsParams as NewListToolsParams, ListToolsResult as NewListToolsResult, Tool as NewTool,
    ToolCapabilities as NewToolCapabilities, ToolContent as NewToolContent,
};

use crate::CodePrismMcpServer;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Modern tool manager that coordinates all tool modules
///
/// This is the main entry point for tool functionality, providing a clean
/// interface while delegating to specialized tool modules.
///
/// # Examples
///
/// Creating a tool manager:
/// ```rust,no_run
/// use std::sync::Arc;
/// use tokio::sync::RwLock;
/// use codeprism_mcp::{CodePrismMcpServer, tools::ToolManager};
///
/// # async fn example() -> anyhow::Result<()> {
/// let server = Arc::new(RwLock::new(CodePrismMcpServer::new()?));
/// let manager = ToolManager::new(server);
/// # Ok(())
/// # }
/// ```
pub struct ToolManager {
    server: Arc<RwLock<CodePrismMcpServer>>,
}

impl ToolManager {
    /// Create a new tool manager
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use tokio::sync::RwLock;
    /// use codeprism_mcp::{CodePrismMcpServer, tools::ToolManager};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let server = Arc::new(RwLock::new(CodePrismMcpServer::new()?));
    /// let manager = ToolManager::new(server);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(server: Arc<RwLock<CodePrismMcpServer>>) -> Self {
        Self { server }
    }

    /// List all available tools
    ///
    /// Returns a comprehensive list of all tools provided by CodePrism MCP server,
    /// including basic repository tools, advanced analysis tools, and code quality tools.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use codeprism_mcp::tools::{ToolManager, ListToolsParams};
    /// use std::sync::Arc;
    /// use tokio::sync::RwLock;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let server = Arc::new(RwLock::new(codeprism_mcp::CodePrismMcpServer::new()?));
    /// let manager = ToolManager::new(server);
    /// let tools = manager.list_tools(ListToolsParams { cursor: None }).await?;
    /// assert!(!tools.tools.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_tools(&self, _params: ListToolsParams) -> Result<ListToolsResult> {
        let mut tools = Vec::new();

        // Add tools from modular structure
        tools.extend(basic::repository::list_tools());
        tools.extend(basic::search::list_tools());
        tools.extend(basic::symbols::list_tools());
        tools.extend(analysis::complexity::list_tools());
        tools.extend(analysis::patterns::list_tools());
        tools.extend(analysis::dependencies::list_tools());
        tools.extend(analysis::flow::list_tools());
        tools.extend(analysis::inheritance::list_tools());
        tools.extend(analysis::decorators::list_tools());
        tools.extend(quality::list_tools());

        // Add remaining tools from legacy implementation
        let legacy_manager = crate::tools_legacy::ToolManager::new(self.server.clone());
        let legacy_result = legacy_manager.list_tools(_params).await?;

        // Filter out tools that are now implemented in modular structure
        let modular_tool_names: std::collections::HashSet<String> =
            tools.iter().map(|t| t.name.clone()).collect();

        for tool in legacy_result.tools {
            if !modular_tool_names.contains(&tool.name) {
                tools.push(tool);
            }
        }

        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    /// Call a specific tool with parameters
    ///
    /// Routes tool calls to the appropriate module based on the tool name.
    /// Provides error handling and validation for all tool calls.
    ///
    /// # Arguments
    ///
    /// * `params` - Tool call parameters including name and arguments
    ///
    /// # Returns
    ///
    /// Returns a `CallToolResult` containing the tool output or error information.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use codeprism_mcp::tools::{ToolManager, CallToolParams};
    /// use std::sync::Arc;
    /// use tokio::sync::RwLock;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let server = Arc::new(RwLock::new(codeprism_mcp::CodePrismMcpServer::new()?));
    /// let manager = ToolManager::new(server);
    /// let result = manager.call_tool(CallToolParams {
    ///     name: "repository_stats".to_string(),
    ///     arguments: None,
    /// }).await?;
    /// assert!(!result.content.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn call_tool(&self, params: CallToolParams) -> Result<CallToolResult> {
        let server = self.server.read().await;

        // Route to modular tools first
        match params.name.as_str() {
            // Basic repository tools
            "repository_stats" | "content_stats" => {
                basic::repository::call_tool(&params.name, &server, params.arguments).await
            }

            // Search tools
            "search_symbols" | "search_content" | "find_files" => {
                basic::search::call_tool(&params.name, &server, params.arguments).await
            }

            // Symbol navigation tools
            "trace_path" | "explain_symbol" | "find_dependencies" | "find_references" => {
                basic::symbols::call_tool(&params.name, &server, params.arguments).await
            }

            // Analysis tools
            "analyze_complexity" => {
                analysis::complexity::call_tool(&params.name, &server, params.arguments).await
            }
            "detect_patterns" => {
                analysis::patterns::call_tool(&params.name, &server, params.arguments).await
            }
            "analyze_transitive_dependencies" => {
                analysis::dependencies::call_tool(&params.name, &server, params.arguments).await
            }
            "trace_data_flow" => {
                analysis::flow::call_tool(&params.name, &server, params.arguments).await
            }
            "trace_inheritance" => {
                analysis::inheritance::call_tool(&params.name, &server, params.arguments).await
            }
            "analyze_decorators" => {
                analysis::decorators::call_tool(&params.name, &server, params.arguments).await
            }

            // Quality tools
            "find_duplicates"
            | "find_unused_code"
            | "analyze_security"
            | "analyze_performance"
            | "analyze_api_surface" => {
                quality::call_tool(&params.name, &server, params.arguments).await
            }

            // All other tools still use legacy implementation
            _ => {
                drop(server); // Release the read lock before calling legacy
                let legacy_manager = crate::tools_legacy::ToolManager::new(self.server.clone());
                legacy_manager.call_tool(params).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    async fn create_test_manager() -> Result<ToolManager> {
        let server = Arc::new(RwLock::new(CodePrismMcpServer::new()?));
        Ok(ToolManager::new(server))
    }

    #[tokio::test]
    async fn test_tool_manager_creation() {
        let manager = create_test_manager().await.unwrap();
        // Tool manager should be created successfully
        assert!(std::ptr::addr_of!(manager.server).is_aligned());
    }

    #[tokio::test]
    async fn test_list_tools_returns_tools() {
        let manager = create_test_manager().await.unwrap();
        let result = manager
            .list_tools(ListToolsParams { cursor: None })
            .await
            .unwrap();

        // Should return some tools
        assert!(!result.tools.is_empty());

        // Should include basic tools
        let tool_names: Vec<&str> = result.tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"repository_stats"));
        assert!(tool_names.contains(&"search_symbols"));
    }

    #[tokio::test]
    async fn test_call_tool_repository_stats() {
        let manager = create_test_manager().await.unwrap();
        let result = manager
            .call_tool(CallToolParams {
                name: "repository_stats".to_string(),
                arguments: None,
            })
            .await
            .unwrap();

        // Should return successful result
        assert_eq!(result.is_error, Some(false));
        assert!(!result.content.is_empty());
    }
}
