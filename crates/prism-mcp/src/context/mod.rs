//! Context management for MCP tools
//! 
//! This module provides session management, workflow tracking, and analysis caching
//! to enable intelligent tool guidance and reduce redundant operations.

pub mod session;
pub mod workflow;
pub mod cache;

// Re-export main types for easy access
pub use session::{SessionManager, SessionState, AnalysisHistory, WorkflowStage};
pub use workflow::{WorkflowContext, WorkflowSuggestion, ToolSuggestion};
pub use cache::{AnalysisCache, CacheKey, CacheEntry}; 