//! Core navigation tools parameter types
//!
//! This module contains parameter type definitions for core navigation tools.
//! The actual tool implementations are in the server module as methods.

use serde::{Deserialize, Serialize};

/// Repository statistics parameters (no parameters needed)
#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryStatsParams {}

/// Symbol explanation parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct ExplainSymbolParams {
    /// Symbol identifier (node ID or symbol name)
    pub symbol_id: String,
    /// Include dependency information
    #[serde(default)]
    pub include_dependencies: bool,
    /// Include usage/reference information  
    #[serde(default)]
    pub include_usages: bool,
    /// Number of context lines around the symbol
    #[serde(default = "default_context_lines")]
    pub context_lines: usize,
}

/// Symbol search parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchSymbolsParams {
    /// Search pattern (supports regex)
    pub pattern: String,
    /// Filter by symbol types
    #[serde(default)]
    pub symbol_types: Vec<String>,
    /// Maximum number of results
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Number of context lines
    #[serde(default = "default_context_lines")]
    pub context_lines: usize,
}

/// Trace path parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct TracePathParams {
    /// Source symbol identifier
    pub source: String,
    /// Target symbol identifier
    pub target: String,
    /// Maximum search depth
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
}

/// Find dependencies parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct FindDependenciesParams {
    /// Target to analyze (symbol ID or file path)
    pub target: String,
    /// Type of dependencies to find
    #[serde(default = "default_dependency_type")]
    pub dependency_type: String,
}

/// Find references parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct FindReferencesParams {
    /// Symbol identifier to find references for
    pub symbol_id: String,
    /// Include symbol definitions
    #[serde(default = "default_true")]
    pub include_definitions: bool,
    /// Number of context lines
    #[serde(default = "default_context_lines")]
    pub context_lines: usize,
}

// Default value functions
fn default_context_lines() -> usize {
    4
}

fn default_limit() -> usize {
    50
}

fn default_max_depth() -> usize {
    10
}

fn default_dependency_type() -> String {
    "direct".to_string()
}

fn default_true() -> bool {
    true
}
