//! MCP tool implementations for CodePrism
//!
//! This module contains all the MCP tool implementations organized by category:
//! - Core tools: Basic file operations and content search
//! - Search tools: Semantic search and dependency analysis
//! - Analysis tools: Code complexity and pattern analysis
//! - Workflow tools: Validation and code generation

pub mod analysis;
pub mod core;
pub mod search;
pub mod workflow;

pub use analysis::AnalysisTools;
/// Re-export tool modules for convenience
pub use core::CoreTools;
pub use search::SearchTools;
pub use workflow::WorkflowTools;

/// Common tool trait that all MCP tools must implement
pub trait McpTool {
    /// Get the tool name
    fn name(&self) -> &str;

    /// Get the tool description
    fn description(&self) -> &str;

    /// Get the tool schema for parameter validation
    fn schema(&self) -> serde_json::Value;
}

/// Tool execution result
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// Whether the tool execution was successful
    pub success: bool,

    /// The result content (JSON value)
    pub content: serde_json::Value,

    /// Optional error message if execution failed
    pub error: Option<String>,

    /// Execution metadata
    pub metadata: ToolMetadata,
}

/// Metadata about tool execution
#[derive(Debug, Clone)]
pub struct ToolMetadata {
    /// Execution duration in milliseconds
    pub duration_ms: u64,

    /// Number of files processed (if applicable)
    pub files_processed: Option<usize>,

    /// Memory usage in bytes (if available)
    pub memory_used_bytes: Option<usize>,
}

impl ToolResult {
    /// Create a successful tool result
    pub fn success(content: serde_json::Value, metadata: ToolMetadata) -> Self {
        Self {
            success: true,
            content,
            error: None,
            metadata,
        }
    }

    /// Create a failed tool result
    pub fn error(error: impl Into<String>, metadata: ToolMetadata) -> Self {
        Self {
            success: false,
            content: serde_json::Value::Null,
            error: Some(error.into()),
            metadata,
        }
    }
}
