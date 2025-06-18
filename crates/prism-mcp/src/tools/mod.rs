//! MCP Tools modular implementation
//! 
//! This module organizes tools into logical categories for better maintainability

pub mod core;
pub mod search;
pub mod analysis;
pub mod workflow;

// Re-export all types from tools_legacy for backward compatibility
pub use crate::tools_legacy::{
    Tool, CallToolParams, CallToolResult, ListToolsParams, ListToolsResult, 
    ToolContent, ToolManager, ToolCapabilities
};

use anyhow::Result;
use serde_json::Value;
use crate::PrismMcpServer;

/// Tool registry that coordinates all modular tools
pub struct ToolRegistry {
    server: std::sync::Arc<tokio::sync::RwLock<PrismMcpServer>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new(server: std::sync::Arc<tokio::sync::RwLock<PrismMcpServer>>) -> Self {
        Self { server }
    }

    /// List all available tools from all modules
    pub async fn list_tools(&self, _params: ListToolsParams) -> Result<ListToolsResult> {
        let mut tools = Vec::new();
        
        // Core navigation tools
        tools.extend(core::navigation::list_tools());
        tools.extend(core::symbols::list_tools());
        tools.extend(core::repository::list_tools());
        
        // Search and discovery tools
        tools.extend(search::content::list_tools());
        tools.extend(search::patterns::list_tools());
        
        // Analysis tools
        tools.extend(analysis::complexity::list_tools());
        tools.extend(analysis::flow::list_tools());
        tools.extend(analysis::specialized::list_tools());
        tools.extend(analysis::quality::list_tools());

        // Workflow orchestration tools
        tools.extend(workflow::register_workflow_tools());

        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    /// Route tool calls to appropriate modules
    pub async fn call_tool(&self, params: CallToolParams) -> Result<CallToolResult> {
        let server = self.server.read().await;
        
        match params.name.as_str() {
            // Core navigation tools
            "repository_stats" => core::repository::call_tool(&*server, &params).await,
            "trace_path" | "find_dependencies" | "find_references" => {
                core::navigation::call_tool(&*server, &params).await
            },
            "explain_symbol" | "search_symbols" => {
                core::symbols::call_tool(&*server, &params).await
            },
            
            // Search and discovery tools
            "search_content" | "find_files" | "content_stats" => {
                search::content::call_tool(&*server, &params).await
            },
            "detect_patterns" => {
                search::patterns::call_tool(&*server, &params).await
            },
            
            // Analysis tools
            "analyze_complexity" => {
                analysis::complexity::call_tool(&*server, &params).await
            },
            "trace_data_flow" | "analyze_transitive_dependencies" => {
                analysis::flow::call_tool(&*server, &params).await
            },
            "trace_inheritance" | "analyze_decorators" => {
                analysis::specialized::call_tool(&*server, &params).await
            },
            "find_duplicates" | "find_unused_code" | "analyze_security" | "analyze_performance" | "analyze_api_surface" => {
                analysis::quality::call_tool(&*server, &params).await
            },
            
            // Workflow orchestration tools
            "suggest_analysis_workflow" | "batch_analysis" | "optimize_workflow" => {
                workflow::handle_workflow_tool(&params.name, &*server, params.arguments.as_ref()).await
            },
            
            _ => {
                Err(anyhow::anyhow!("Unknown tool: {}", params.name))
            }
        }
    }
} 