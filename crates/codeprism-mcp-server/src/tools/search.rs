//! Search tools parameter types

use serde::{Deserialize, Serialize};

// PLANNED(#170): Add search parameter types when implementing search tools
// NOTE: Foundation module for search tool parameter definitions

/// Search tool parameters foundation type
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchParams {
    /// Search query
    pub query: String,
}
