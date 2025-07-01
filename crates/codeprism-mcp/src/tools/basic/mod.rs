//! Basic MCP tools for repository navigation and search.
//!
//! This module contains fundamental tools for exploring and understanding
//! code repositories, including repository statistics, file search, and
//! basic symbol navigation.

pub mod navigation;
pub mod repository;
pub mod search;
// FUTURE: Re-enable symbols module after API compatibility updates
// pub mod symbols;

// Re-export specific functions to avoid naming conflicts
pub use repository::{content_stats, content_stats_tool, repository_stats, repository_stats_tool};
// PLANNED: Other modules (search, symbols, navigation) will be migrated in future phases
