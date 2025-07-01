//! Dynamic tool enablement module.
//!
//! This module provides backward compatibility for the dynamic tool enablement
//! functionality during the modularization process.

use serde::{Deserialize, Serialize};

/// Dynamic tool manager for backward compatibility
#[derive(Debug, Clone)]
pub struct DynamicToolManager {
    // Placeholder implementation
}

impl DynamicToolManager {
    /// Create a new dynamic tool manager
    pub fn new() -> Self {
        Self {}
    }

    /// Get configuration summary for backward compatibility
    pub fn get_summary(&self) -> ToolEnablementSummary {
        ToolEnablementSummary {
            enabled_tools: 0,
            disabled_tools: 0,
            total_tools: 0,
        }
    }
}

impl Default for DynamicToolManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Repository analysis for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryAnalysis {
    /// Analysis results
    pub results: String,
}

/// Tool enablement summary for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEnablementSummary {
    /// Number of enabled tools
    pub enabled_tools: usize,
    /// Number of disabled tools
    pub disabled_tools: usize,
    /// Total tools
    pub total_tools: usize,
}
