//! Workflow tools parameter types

use serde::{Deserialize, Serialize};

// PLANNED(#171): Add workflow parameter types when implementing workflow tools
// NOTE: Foundation module for workflow tool parameter definitions

/// Workflow tool parameters foundation type
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowParams {
    /// Workflow operation type
    pub operation: String,
}
