//! Code quality analysis tools.
//!
//! This module contains tools for analyzing code quality, finding duplicates,
//! unused code, security issues, performance problems, and API surface analysis.

pub mod api_surface;
pub mod duplicates;
pub mod performance;
pub mod security;
pub mod unused;

// Re-export specific functions to avoid naming conflicts
// PLANNED: All quality modules will be migrated from tools_legacy.rs in future phases
