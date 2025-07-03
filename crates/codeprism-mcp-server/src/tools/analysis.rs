//! Analysis tools parameter types

use serde::{Deserialize, Serialize};

// PLANNED(#169): Add analysis parameter types when implementing analysis tools
// NOTE: Foundation module for analysis tool parameter definitions

/// Analysis tool parameters foundation type
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisParams {
    /// Base parameter for analysis operations
    pub operation: String,
}
