//! Advanced code analysis tools.
//!
//! This module contains sophisticated analysis tools for complexity,
//! patterns, dependencies, data flow, inheritance, and decorators.

pub mod complexity;
pub mod data_flow;
pub mod decorators;
pub mod dependencies;
pub mod inheritance;
pub mod patterns;

// Re-export specific functions to avoid naming conflicts
// PLANNED: All analysis modules will be migrated from tools_legacy.rs in future phases
