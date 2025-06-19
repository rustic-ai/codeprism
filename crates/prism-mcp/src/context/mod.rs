//! Context management for MCP tools
//!
//! This module provides session management, workflow tracking, and analysis caching
//! to enable intelligent tool guidance and reduce redundant operations.

pub mod cache;
pub mod session;
pub mod workflow;

// Re-export main types for easy access
pub use cache::{AnalysisCache, CacheEntry, CacheKey};
pub use session::{AnalysisHistory, SessionManager, SessionState, WorkflowStage};
pub use workflow::{ToolSuggestion, WorkflowContext, WorkflowSuggestion};
