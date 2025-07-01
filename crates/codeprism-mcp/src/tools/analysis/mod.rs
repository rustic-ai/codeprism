//! Analysis tools module.
//!
//! This module contains advanced analysis tools for code complexity,
//! pattern detection, dependency analysis, data flow tracing, and
//! inheritance analysis.

pub mod complexity;
pub mod decorators;
pub mod dependencies;
pub mod flow;
pub mod inheritance;
pub mod patterns;

// Re-export specific functions to avoid naming conflicts
// PLANNED: All analysis modules will be migrated from tools_legacy.rs in future phases
