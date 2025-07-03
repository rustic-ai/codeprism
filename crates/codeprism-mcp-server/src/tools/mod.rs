//! MCP tool parameter types for CodePrism
//!
//! This module contains parameter type definitions for all MCP tools organized by category:
//! - Core tools: Navigation, symbol analysis, and repository information
//! - Search tools: Content search and pattern matching  
//! - Analysis tools: Code complexity and quality analysis
//! - Workflow tools: Code optimization and batch processing
//!
//! The actual tool implementations are methods on the CodePrismMcpServer struct.

pub mod analysis;
pub mod core;
pub mod search;
pub mod workflow;

// Re-export parameter types for convenience
pub use core::{
    ExplainSymbolParams, FindDependenciesParams, FindReferencesParams, RepositoryStatsParams,
    SearchSymbolsParams, TracePathParams,
};

/// Tool execution result metadata
#[derive(Debug, Clone)]
pub struct ToolMetadata {
    /// Execution duration in milliseconds
    pub duration_ms: u64,

    /// Number of files processed (if applicable)
    pub files_processed: Option<usize>,

    /// Memory usage in bytes (if available)
    pub memory_used_bytes: Option<usize>,
}
